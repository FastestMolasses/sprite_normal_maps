use bevy::{
    color::palettes::css::*,
    math::primitives::Rectangle,
    prelude::*,
    reflect::TypePath,
    render::render_resource::{AsBindGroup, ShaderRef, ShaderType},
    sprite::{AlphaMode2d, Material2d, Material2dPlugin},
};

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
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                handle_input,
                update_material_light_info,
                control_light_properties,
            ),
        )
        .run();
}

#[derive(Component)]
struct PositionMappedSprite;

#[derive(Component)]
struct MovableLightMarker {
    pub color: Color,
    pub intensity: f32,
    pub ambient_color: Color,
    pub ambient_intensity: f32,
    pub radius: f32,
    pub falloff: f32,
    pub position_scale: f32,
    pub debug_mode: u32,
    pub virtual_height: f32, // The virtual Z height in game world
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut custom_materials: ResMut<Assets<PositionMappedMaterial>>,
) {
    commands.spawn(Camera2d);

    let diffuse_handle: Handle<Image> = asset_server.load("tree_diffuse_color.png");
    let position_handle: Handle<Image> = asset_server.load("tree_position.png");
    let normal_handle: Handle<Image> = asset_server.load("tree_normal.png");

    // Define initial light properties
    let initial_light_props = MovableLightMarker {
        color: WHITE.into(),
        intensity: 1.0,
        ambient_color: DARK_SLATE_GRAY.into(),
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
    ));

    // Spawn a visible marker for the light source
    commands.spawn((
        initial_light_props,
        Sprite {
            color: LIME.into(),
            custom_size: Some(Vec2::splat(16.0)),
            ..default()
        },
        Transform::from_xyz(initial_light_pos_xy.x, initial_light_pos_xy.y, 10.0),
    ));
}

fn control_light_properties(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut light_query: Query<(&mut MovableLightMarker, &Transform)>,
) {
    if let Ok((mut light_props, light_transform)) = light_query.single_mut() {
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
            println!("Debug Mode: {}", match light_props.debug_mode {
                0 => "Normal Lighting",
                1 => "Show Position Map (RGB = XYZ)",
                2 => "Show Normal Map",
                3 => "Show Distance to Light",
                4 => "Show Ground Level Only (Blue channel near 0)",
                5 => "Show 3D World Positions (for debugging coordinates)",
                _ => "Unknown",
            });
        }

        // Display current values
        if keyboard_input.just_pressed(KeyCode::Space) {
            println!("Light Properties:");
            println!("  Ground Position (XY): ({:.1}, {:.1})", 
                light_transform.translation.x,
                light_transform.translation.y);
            println!("  Virtual Height (game Z): {:.1}", light_props.virtual_height);
            println!("  Intensity: {:.2}", light_props.intensity);
            println!("  Ambient: {:.2}", light_props.ambient_intensity);
            println!("  Radius: {:.1}", light_props.radius);
            println!("  Falloff: {:.2}", light_props.falloff);
            println!("  Position Scale: {:.2}", light_props.position_scale);
            println!("  Debug Mode: {}", light_props.debug_mode);
            println!("Controls: WASD=move on ground (XY), U/J=virtual height, I/K=intensity, O/L=ambient, [/]=radius, +/-=falloff, P/;=scale, V=debug");
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
