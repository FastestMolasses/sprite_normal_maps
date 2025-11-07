use bevy::{
    color::palettes::css,
    math::primitives::Rectangle,
    prelude::*,
    reflect::TypePath,
    render::render_resource::{AsBindGroup, ShaderRef, ShaderType},
    sprite::{AlphaMode2d, Material2d, Material2dPlugin},
};

mod volume;
use volume::*;

mod gpu_volume;
use gpu_volume::*;

mod scenes;
use scenes::*;

mod lighting;
use lighting::*;

mod ui;
use ui::*;

#[derive(ShaderType, Debug, Clone, Default)]
pub struct LightUniformData {
    light_pos_world_3d: Vec3, // XY = ground position, Z = virtual height
    sprite_world_pos: Vec2,   // Sprite's position on the ground (XY)
    light_color: LinearRgba,
    ambient_light_color: LinearRgba,
    light_radius: f32,
    light_falloff: f32,
    position_scale: f32,
    debug_mode: u32,
}

#[derive(AsBindGroup, Debug, Clone, Asset, TypePath)]
pub struct PositionMappedMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub diffuse_texture: Handle<Image>,

    #[texture(2)]
    #[sampler(3)]
    pub position_texture: Handle<Image>,

    #[texture(4)]
    #[sampler(5)]
    pub normal_texture: Handle<Image>,

    #[uniform(6)]
    pub uniform_data: LightUniformData,
}

impl Material2d for PositionMappedMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/position_lighting_2d.wgsl".into()
    }

    fn vertex_shader() -> ShaderRef {
        "shaders/position_lighting_2d.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(Material2dPlugin::<PositionMappedMaterial>::default())
        .add_plugins(GpuVolumeRenderPlugin)
        .init_resource::<CurrentScene>()
        .init_resource::<VolumeRenderMode>()
        .add_systems(Startup, (setup_texture_mapped_scene, setup_camera))
        .add_systems(
            Update,
            (
                handle_input,
                handle_scene_switching,
                control_light_properties,
                control_volume_rotation,
                toggle_render_mode,
                update_procedural_volume,
                update_gpu_volume,
                update_material_light_info,
                update_debug_mode_display,
            ),
        )
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

/// Component to store the procedural volume and rotation state
#[derive(Component)]
struct ProceduralVolume {
    pub volume: Volume,
    pub rotation: Vec3, // Euler angles in radians
    pub target_rotation: Vec3, // Target rotation for smooth interpolation
    pub params: RockGenerationParams,
    pub needs_update: bool,
    pub update_timer: f32, // Debounce timer to prevent constant updates
}

fn setup_initial_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    meshes: ResMut<Assets<Mesh>>,
    custom_materials: ResMut<Assets<PositionMappedMaterial>>,
    images: ResMut<Assets<Image>>,
    current_scene: Res<CurrentScene>,
    render_mode: Res<VolumeRenderMode>,
) {
    // Spawn camera (shared between all scenes)
    commands.spawn(Camera2d);

    // Setup the initial scene based on current scene resource
    match *current_scene {
        CurrentScene::TextureMapped => {
            setup_texture_mapped_scene(commands, asset_server, meshes, custom_materials);
        }
        CurrentScene::Procedural => {
            setup_procedural_scene(commands, asset_server, meshes, custom_materials, images, *render_mode);
        }
    }
}

fn setup_texture_mapped_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut custom_materials: ResMut<Assets<PositionMappedMaterial>>,
) {
    let diffuse_handle: Handle<Image> = asset_server.load("tree_diffuse_color.png");
    let position_handle: Handle<Image> = asset_server.load("tree_position2.png");
    let normal_handle: Handle<Image> = asset_server.load("tree_normal.png");

    // Define initial light properties
    let initial_light_props = MovableLightMarker {
        color: css::WHITE.into(),
        intensity: 1.0,
        ambient_color: css::DARK_SLATE_GRAY.into(),
        ambient_intensity: 0.2,
        radius: 300.0,
        falloff: 1.5,
        // This scale converts Blender units to Bevy world units
        // Adjust based on your Blender scene scale (typically 0.01 to 1.0)
        position_scale: 1.0,
        debug_mode: 0, // 0=normal, 1=show position map, 2=show normals, 3=show distance, 4=show ground level, 5=show 3D positions
        virtual_height: 0.0, // Start at ground level (virtual Z = 0)
    };
    // Light starts at same XY as sprite (center), at ground level (virtual height = 0)
    let initial_light_pos_xy = Vec2::new(0.0, 100.0);

    // Create the material instance
    let tree_material = custom_materials.add(PositionMappedMaterial {
        diffuse_texture: diffuse_handle,
        position_texture: position_handle,
        normal_texture: normal_handle,
        uniform_data: LightUniformData {
            light_pos_world_3d: Vec3::new(
                initial_light_pos_xy.x,
                initial_light_pos_xy.y,
                initial_light_props.virtual_height,
            ),
            sprite_world_pos: Vec2::new(0.0, 100.0), // Sprite is at this XY position
            light_color: LinearRgba::from(initial_light_props.color)
                * initial_light_props.intensity,
            ambient_light_color: LinearRgba::from(initial_light_props.ambient_color)
                * initial_light_props.ambient_intensity,
            light_radius: initial_light_props.radius,
            light_falloff: initial_light_props.falloff,
            position_scale: initial_light_props.position_scale,
            debug_mode: initial_light_props.debug_mode,
        },
    });

    let sprite_width = 1024.0;
    let sprite_height = 1024.0;

    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(sprite_width, sprite_height))),
        MeshMaterial2d(tree_material),
        Transform::from_xyz(0.0, 100.0, 0.0),
        PositionMappedSprite,
        TextureMappedSceneEntity,
    ));

    // Spawn a visible marker for the light source
    commands.spawn((
        initial_light_props,
        Sprite {
            color: css::LIME.into(),
            custom_size: Some(Vec2::splat(16.0)),
            ..default()
        },
        Transform::from_xyz(initial_light_pos_xy.x, initial_light_pos_xy.y, 10.0),
        TextureMappedSceneEntity,
    ));

    // Spawn UI for this scene
    spawn_texture_mapped_ui(&mut commands);
}

fn setup_procedural_scene(
    mut commands: Commands,
    _asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut custom_materials: ResMut<Assets<PositionMappedMaterial>>,
    mut images: ResMut<Assets<Image>>,
    render_mode: VolumeRenderMode,
) {
    // Generate the rock volume
    let params = RockGenerationParams {
        size: 64,
        scale: 3.0,
        octaves: 4,
        lacunarity: 2.0,
        persistence: 0.5,
        threshold: 0.0,
        seed: 42,
    };
    
    let volume = generate_rock_volume(&params);
    
    // Initial rotation
    let initial_rotation = Vec3::ZERO;
    
    // Create textures based on render mode
    let (position_handle, normal_handle, diffuse_handle, volume_texture_handle) = match render_mode {
        VolumeRenderMode::Cpu => {
            // CPU path: Render volume to 2D maps using software raymarching
            let output_size = 256;
            let render_result = render_volume_to_maps(&volume, output_size, initial_rotation);
            
            // Create Bevy Image assets from the generated data
            let position_image = Image::new(
                bevy::render::render_resource::Extent3d {
                    width: render_result.width,
                    height: render_result.height,
                    depth_or_array_layers: 1,
                },
                bevy::render::render_resource::TextureDimension::D2,
                render_result.position_map,
                bevy::render::render_resource::TextureFormat::Rgba8Unorm,
                bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD,
            );
            
            let normal_image = Image::new(
                bevy::render::render_resource::Extent3d {
                    width: render_result.width,
                    height: render_result.height,
                    depth_or_array_layers: 1,
                },
                bevy::render::render_resource::TextureDimension::D2,
                render_result.normal_map,
                bevy::render::render_resource::TextureFormat::Rgba8Unorm,
                bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD,
            );
            
            let diffuse_image = Image::new(
                bevy::render::render_resource::Extent3d {
                    width: render_result.width,
                    height: render_result.height,
                    depth_or_array_layers: 1,
                },
                bevy::render::render_resource::TextureDimension::D2,
                render_result.diffuse_map,
                bevy::render::render_resource::TextureFormat::Rgba8Unorm,
                bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD,
            );
            
            let position_handle = images.add(position_image);
            let normal_handle = images.add(normal_image);
            let diffuse_handle = images.add(diffuse_image);
            
            (position_handle, normal_handle, diffuse_handle, None)
        }
        VolumeRenderMode::Gpu => {
            // GPU path: Create empty output textures and upload volume to GPU
            let output_size = 256;
            
            // Create volume texture for GPU
            let volume_handle = create_volume_texture(&volume, &mut images);
            
            // Create output textures
            let (pos_handle, norm_handle, diff_handle) = create_output_textures(output_size, &mut images);
            
            (pos_handle, norm_handle, diff_handle, Some(volume_handle))
        }
    };
    
    let output_size = 256; // Fixed size for sprite display
    
    // Setup lighting
    let initial_light_props = MovableLightMarker {
        color: css::AQUA.into(),
        intensity: 1.0,
        ambient_color: Color::srgb(0.1, 0.1, 0.15),
        ambient_intensity: 0.3,
        radius: 400.0,
        falloff: 2.0,
        position_scale: 1.0,
        debug_mode: 0,
        virtual_height: 50.0,
    };

    let initial_light_pos_xy = Vec2::new(0.0, 0.0);
    let sprite_pos = Vec2::new(0.0, 0.0);

    // Create the material with procedurally generated textures
    let rock_material = custom_materials.add(PositionMappedMaterial {
        diffuse_texture: diffuse_handle.clone(),
        position_texture: position_handle.clone(),
        normal_texture: normal_handle.clone(),
        uniform_data: LightUniformData {
            light_pos_world_3d: Vec3::new(
                initial_light_pos_xy.x,
                initial_light_pos_xy.y,
                initial_light_props.virtual_height,
            ),
            sprite_world_pos: sprite_pos,
            light_color: LinearRgba::from(initial_light_props.color)
                * initial_light_props.intensity,
            ambient_light_color: LinearRgba::from(initial_light_props.ambient_color)
                * initial_light_props.ambient_intensity,
            light_radius: initial_light_props.radius,
            light_falloff: initial_light_props.falloff,
            position_scale: initial_light_props.position_scale,
            debug_mode: initial_light_props.debug_mode,
        },
    });

    // Spawn the procedural rock sprite with appropriate components based on render mode
    let sprite_size = output_size as f32;
    
    match render_mode {
        VolumeRenderMode::Cpu => {
            // CPU mode: Use ProceduralVolume component for manual updates
            commands.spawn((
                Mesh2d(meshes.add(Rectangle::new(sprite_size, sprite_size))),
                MeshMaterial2d(rock_material),
                Transform::from_xyz(sprite_pos.x, sprite_pos.y, 0.0),
                PositionMappedSprite,
                ProceduralSceneEntity,
                ProceduralVolume {
                    volume: volume.clone(),
                    rotation: initial_rotation,
                    target_rotation: initial_rotation,
                    params: params.clone(),
                    needs_update: false,
                    update_timer: 0.0,
                },
            ));
        }
        VolumeRenderMode::Gpu => {
            // GPU mode: Use GpuVolumeRenderer component for automatic GPU rendering
            commands.spawn((
                Mesh2d(meshes.add(Rectangle::new(sprite_size, sprite_size))),
                MeshMaterial2d(rock_material),
                Transform::from_xyz(sprite_pos.x, sprite_pos.y, 0.0),
                PositionMappedSprite,
                ProceduralSceneEntity,
                GpuVolumeRenderer {
                    volume_texture: volume_texture_handle.unwrap(),
                    position_output: position_handle.clone(),
                    normal_output: normal_handle.clone(),
                    diffuse_output: diffuse_handle.clone(),
                    rotation: initial_rotation,
                    volume_size: params.size as f32,
                    output_size,
                },
            ));
        }
    }

    // Spawn a visible marker for the light source
    commands.spawn((
        initial_light_props,
        Sprite {
            color: css::ORANGE.into(),
            custom_size: Some(Vec2::splat(16.0)),
            ..default()
        },
        Transform::from_xyz(initial_light_pos_xy.x, initial_light_pos_xy.y, 10.0),
        ProceduralSceneEntity,
    ));

    // Spawn UI for this scene
    spawn_procedural_ui(&mut commands, render_mode.as_str());
}

fn handle_scene_switching(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut current_scene: ResMut<CurrentScene>,
    mut commands: Commands,
    texture_scene_query: Query<Entity, With<TextureMappedSceneEntity>>,
    procedural_scene_query: Query<Entity, With<ProceduralSceneEntity>>,
    ui_query: Query<(Entity, &SceneUi)>,
    asset_server: Res<AssetServer>,
    meshes: ResMut<Assets<Mesh>>,
    custom_materials: ResMut<Assets<PositionMappedMaterial>>,
    images: ResMut<Assets<Image>>,
    render_mode: Res<VolumeRenderMode>,
) {
    if keyboard_input.just_pressed(KeyCode::F1) {
        // Despawn UI for the CURRENT scene before switching
        let old_scene_type = match *current_scene {
            CurrentScene::TextureMapped => SceneType::TextureMapped,
            CurrentScene::Procedural => SceneType::Procedural,
        };
        despawn_scene_ui(commands.reborrow(), ui_query, old_scene_type);
        
        // Toggle scene
        *current_scene = match *current_scene {
            CurrentScene::TextureMapped => CurrentScene::Procedural,
            CurrentScene::Procedural => CurrentScene::TextureMapped,
        };

        // Despawn all entities from both scenes
        for entity in texture_scene_query.iter() {
            commands.entity(entity).despawn();
        }
        for entity in procedural_scene_query.iter() {
            commands.entity(entity).despawn();
        }

        // Setup the new scene (will spawn new UI)
        match *current_scene {
            CurrentScene::TextureMapped => {
                setup_texture_mapped_scene(commands, asset_server, meshes, custom_materials);
            }
            CurrentScene::Procedural => {
                setup_procedural_scene(commands, asset_server, meshes, custom_materials, images, *render_mode);
            }
        }
    }
}

fn control_light_properties(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut light_query: Query<(&mut MovableLightMarker, &Transform)>,
) {
    if let Ok((mut light_props, _light_transform)) = light_query.single_mut() {
        let dt = time.delta_secs();

        // Control light intensity
        if keyboard_input.pressed(KeyCode::KeyI) {
            light_props.intensity += 0.5 * dt;
        }
        if keyboard_input.pressed(KeyCode::KeyK) {
            light_props.intensity = (light_props.intensity - 0.5 * dt).max(0.0);
        }

        // Control ambient intensity
        if keyboard_input.pressed(KeyCode::KeyO) {
            light_props.ambient_intensity += 0.3 * dt;
        }
        if keyboard_input.pressed(KeyCode::KeyL) {
            light_props.ambient_intensity = (light_props.ambient_intensity - 0.3 * dt).max(0.0);
        }

        // Control light radius
        if keyboard_input.pressed(KeyCode::BracketRight) {
            light_props.radius += 100.0 * dt;
        }
        if keyboard_input.pressed(KeyCode::BracketLeft) {
            light_props.radius = (light_props.radius - 100.0 * dt).max(10.0);
        }

        // Control light falloff
        if keyboard_input.pressed(KeyCode::Equal) {
            light_props.falloff += 0.5 * dt;
        }
        if keyboard_input.pressed(KeyCode::Minus) {
            light_props.falloff = (light_props.falloff - 0.5 * dt).max(0.1);
        }

        // Control virtual height (U/J keys for up/down in game world)
        if keyboard_input.pressed(KeyCode::KeyU) {
            light_props.virtual_height += 50.0 * dt;
        }
        if keyboard_input.pressed(KeyCode::KeyJ) {
            light_props.virtual_height -= 50.0 * dt;
        }

        // Control position scale
        if keyboard_input.pressed(KeyCode::KeyP) {
            light_props.position_scale += 0.1 * dt;
        }
        if keyboard_input.pressed(KeyCode::Semicolon) {
            light_props.position_scale = (light_props.position_scale - 0.1 * dt).max(0.01);
        }

        // Cycle debug modes
        if keyboard_input.just_pressed(KeyCode::KeyV) {
            light_props.debug_mode = (light_props.debug_mode + 1) % 6;
        }
    }
}

fn handle_input(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<MovableLightMarker>>,
) {
    const SPEED: f32 = 200.0;
    if let Ok(mut light_transform) = query.single_mut() {
        let dt = time.delta_secs();

        if keyboard_input.pressed(KeyCode::KeyW) {
            light_transform.translation.y += SPEED * dt;
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            light_transform.translation.y -= SPEED * dt;
        }
        if keyboard_input.pressed(KeyCode::KeyA) {
            light_transform.translation.x -= SPEED * dt;
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            light_transform.translation.x += SPEED * dt;
        }
    }
}

/// System to control procedural volume rotation with keyboard
fn control_volume_rotation(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut cpu_volume_query: Query<&mut ProceduralVolume>,
    mut gpu_volume_query: Query<&mut GpuVolumeRenderer>,
) {
    let dt = time.delta_secs();
    let rotation_speed = 0.5; // radians per second
    
    let mut rotation_delta = Vec3::ZERO;
    let mut reset = false;
    
    // Calculate rotation changes based on input
    if keyboard_input.pressed(KeyCode::KeyQ) {
        rotation_delta.y += rotation_speed * dt;
    }
    if keyboard_input.pressed(KeyCode::KeyE) {
        rotation_delta.y -= rotation_speed * dt;
    }
    if keyboard_input.pressed(KeyCode::KeyR) {
        rotation_delta.x += rotation_speed * dt;
    }
    if keyboard_input.pressed(KeyCode::KeyF) {
        rotation_delta.x -= rotation_speed * dt;
    }
    if keyboard_input.pressed(KeyCode::KeyT) {
        rotation_delta.z += rotation_speed * dt;
    }
    if keyboard_input.pressed(KeyCode::KeyY) {
        rotation_delta.z -= rotation_speed * dt;
    }
    if keyboard_input.just_pressed(KeyCode::KeyX) {
        reset = true;
    }
    
    // Apply to CPU volume if present
    if let Ok(mut proc_volume) = cpu_volume_query.single_mut() {
        let rotation_changed = rotation_delta != Vec3::ZERO || reset;
        
        if reset {
            proc_volume.target_rotation = Vec3::ZERO;
        } else {
            proc_volume.target_rotation += rotation_delta;
        }

        // Update timer (debouncing for CPU rendering)
        if rotation_changed {
            proc_volume.update_timer = 0.3; // Wait 300ms after rotation stops
        } else if proc_volume.update_timer > 0.0 {
            proc_volume.update_timer -= dt;
            
            // When timer expires, trigger update
            if proc_volume.update_timer <= 0.0 && proc_volume.rotation != proc_volume.target_rotation {
                proc_volume.rotation = proc_volume.target_rotation;
                proc_volume.needs_update = true;
            }
        }
    }
    
    // Apply to GPU volume if present (no debouncing needed, updates every frame)
    if let Ok(mut gpu_volume) = gpu_volume_query.single_mut() {
        if reset {
            gpu_volume.rotation = Vec3::ZERO;
        } else {
            gpu_volume.rotation += rotation_delta;
        }
    }
}

/// System to regenerate textures when the volume rotation changes
fn update_procedural_volume(
    mut volume_query: Query<(&mut ProceduralVolume, &MeshMaterial2d<PositionMappedMaterial>)>,
    mut materials: ResMut<Assets<PositionMappedMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    for (mut proc_volume, material_handle) in volume_query.iter_mut() {
        if !proc_volume.needs_update {
            continue;
        }

        // Get the material
        let Some(material) = materials.get_mut(material_handle) else {
            continue;
        };

        // Regenerate the maps with the new rotation (256x256 for faster updates)
        let output_size = 256;
        
        let render_result = render_volume_to_maps(
            &proc_volume.volume,
            output_size,
            proc_volume.rotation,
        );

        // Create new images and replace the old ones
        let position_image = Image::new(
            bevy::render::render_resource::Extent3d {
                width: render_result.width,
                height: render_result.height,
                depth_or_array_layers: 1,
            },
            bevy::render::render_resource::TextureDimension::D2,
            render_result.position_map,
            bevy::render::render_resource::TextureFormat::Rgba8Unorm,
            bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD,
        );
        
        let normal_image = Image::new(
            bevy::render::render_resource::Extent3d {
                width: render_result.width,
                height: render_result.height,
                depth_or_array_layers: 1,
            },
            bevy::render::render_resource::TextureDimension::D2,
            render_result.normal_map,
            bevy::render::render_resource::TextureFormat::Rgba8Unorm,
            bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD,
        );
        
        let diffuse_image = Image::new(
            bevy::render::render_resource::Extent3d {
                width: render_result.width,
                height: render_result.height,
                depth_or_array_layers: 1,
            },
            bevy::render::render_resource::TextureDimension::D2,
            render_result.diffuse_map,
            bevy::render::render_resource::TextureFormat::Rgba8Unorm,
            bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD,
        );

        // Replace the images in the asset storage
        images.insert(&material.position_texture, position_image);
        images.insert(&material.normal_texture, normal_image);
        images.insert(&material.diffuse_texture, diffuse_image);

        proc_volume.needs_update = false;
    }
}

/// System to update the material's uniform data based on the light's transform and properties
fn update_material_light_info(
    light_query: Query<(&Transform, &MovableLightMarker)>,
    mut custom_materials: ResMut<Assets<PositionMappedMaterial>>,
    sprite_query: Query<(&MeshMaterial2d<PositionMappedMaterial>, &Transform), With<PositionMappedSprite>>,
) {
    if let Ok((light_transform, light_props)) = light_query.single()
        && let Ok((material_handle, sprite_transform)) = sprite_query.single()
        && let Some(material) = custom_materials.get_mut(material_handle)
    {
        // Light position: XY from transform (ground position), Z from virtual_height
        material.uniform_data.light_pos_world_3d = Vec3::new(
            light_transform.translation.x,
            light_transform.translation.y,
            light_props.virtual_height,
        );
        material.uniform_data.sprite_world_pos = sprite_transform.translation.truncate();
        material.uniform_data.light_color =
            LinearRgba::from(light_props.color) * light_props.intensity;
        material.uniform_data.ambient_light_color =
            LinearRgba::from(light_props.ambient_color) * light_props.ambient_intensity;
        material.uniform_data.light_radius = light_props.radius;
        material.uniform_data.light_falloff = light_props.falloff;
        material.uniform_data.position_scale = light_props.position_scale;
        material.uniform_data.debug_mode = light_props.debug_mode;
    }
}

/// System to handle GPU volume updates (runs every frame, no debouncing needed)
fn update_gpu_volume(
    mut gpu_renderer_query: Query<&mut GpuVolumeRenderer>,
) {
    // The GPU renderer will automatically render every frame
    // No explicit update needed - just ensure rotation is synchronized
    for _ in gpu_renderer_query.iter_mut() {
        // Could add performance metrics here if needed
    }
}

/// Toggle between CPU and GPU rendering modes
fn toggle_render_mode(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut render_mode: ResMut<VolumeRenderMode>,
    mut commands: Commands,
    procedural_scene_query: Query<Entity, With<ProceduralSceneEntity>>,
    ui_query: Query<(Entity, &SceneUi)>,
    asset_server: Res<AssetServer>,
    meshes: ResMut<Assets<Mesh>>,
    custom_materials: ResMut<Assets<PositionMappedMaterial>>,
    images: ResMut<Assets<Image>>,
    current_scene: Res<CurrentScene>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyM) && *current_scene == CurrentScene::Procedural {
        // Toggle mode
        *render_mode = match *render_mode {
            VolumeRenderMode::Cpu => VolumeRenderMode::Gpu,
            VolumeRenderMode::Gpu => VolumeRenderMode::Cpu,
        };

        // Despawn current procedural scene entities
        for entity in procedural_scene_query.iter() {
            commands.entity(entity).despawn();
        }
        
        // Despawn current UI
        despawn_scene_ui(commands.reborrow(), ui_query, SceneType::Procedural);

        // Recreate the procedural scene with new render mode (will spawn new UI)
        setup_procedural_scene(commands, asset_server, meshes, custom_materials, images, *render_mode);
    }
}

/// Update the debug mode display in the status panel
fn update_debug_mode_display(
    light_query: Query<&MovableLightMarker, Changed<MovableLightMarker>>,
    mut status_query: Query<&mut Text, With<StatusPanel>>,
) {
    // Only update if light properties changed
    if let Ok(light_props) = light_query.single() {
        if let Ok(mut text) = status_query.single_mut() {
            let mode_text = match light_props.debug_mode {
                0 => "Normal Lighting",
                1 => "Show Position Map (RGB = XYZ)",
                2 => "Show Normal Map",
                3 => "Show Distance to Light",
                4 => "Show Ground Level Only",
                5 => "Show 3D World Positions",
                _ => "Unknown",
            };
            **text = format!("Debug Mode: {}", mode_text);
        }
    }
}
