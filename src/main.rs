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
    light_pos_world_2d: Vec2,
    light_color: LinearRgba,
    ambient_light_color: LinearRgba,
    light_radius: f32,
    light_falloff: f32,
    light_height: f32,
    normal_strength: f32,
}

#[derive(AsBindGroup, Debug, Clone, Asset, TypePath)]
pub struct NormalMappedMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub diffuse_texture: Handle<Image>,

    #[texture(2)]
    #[sampler(3)]
    pub normal_texture: Handle<Image>,

    #[uniform(4)]
    pub uniform_data: LightUniformData,
}

impl Material2d for NormalMappedMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/normal_mapping_2d.wgsl".into()
    }

    fn vertex_shader() -> ShaderRef {
        "shaders/normal_mapping_2d.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(Material2dPlugin::<NormalMappedMaterial>::default())
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
struct NormalMappedSprite;

#[derive(Component)]
struct MovableLightMarker {
    pub color: Color,
    pub intensity: f32,
    pub ambient_color: Color,
    pub ambient_intensity: f32,
    pub radius: f32,
    pub falloff: f32,
    pub height: f32,
    pub normal_strength: f32,
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut custom_materials: ResMut<Assets<NormalMappedMaterial>>,
) {
    commands.spawn(Camera2d);

    let diffuse_handle: Handle<Image> = asset_server.load("tree_diffuse.png");
    let normal_handle: Handle<Image> = asset_server.load("tree_normal.png");

    // Define initial light properties with more control
    let initial_light_props = MovableLightMarker {
        color: WHITE.into(),
        intensity: 1.0,
        ambient_color: DARK_SLATE_GRAY.into(),
        ambient_intensity: 0.2,
        radius: 300.0,
        falloff: 1.5,
        height: 50.0,
        normal_strength: 0.5,
    };
    let initial_light_pos_xy = Vec2::new(100.0, 50.0);

    // Create the material instance
    let tree_material = custom_materials.add(NormalMappedMaterial {
        diffuse_texture: diffuse_handle,
        normal_texture: normal_handle,
        uniform_data: LightUniformData {
            light_pos_world_2d: initial_light_pos_xy,
            light_color: LinearRgba::from(initial_light_props.color)
                * initial_light_props.intensity,
            ambient_light_color: LinearRgba::from(initial_light_props.ambient_color)
                * initial_light_props.ambient_intensity,
            light_radius: initial_light_props.radius,
            light_falloff: initial_light_props.falloff,
            light_height: initial_light_props.height,
            normal_strength: initial_light_props.normal_strength,
        },
    });

    let sprite_width = 2048.0;
    let sprite_height = 2048.0;

    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(sprite_width, sprite_height))),
        MeshMaterial2d(tree_material),
        Transform::from_xyz(0.0, 100.0, 0.0),
        NormalMappedSprite,
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
    mut light_query: Query<&mut MovableLightMarker>,
) {
    if let Ok(mut light_props) = light_query.single_mut() {
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

        // Control light height
        if keyboard_input.pressed(KeyCode::KeyU) {
            light_props.height += 50.0 * dt;
        }
        if keyboard_input.pressed(KeyCode::KeyJ) {
            light_props.height = (light_props.height - 50.0 * dt).max(1.0);
        }

        // Control normal map strength
        if keyboard_input.pressed(KeyCode::KeyP) {
            light_props.normal_strength += 1.0 * dt; // Faster increase
        }
        if keyboard_input.pressed(KeyCode::Semicolon) {
            light_props.normal_strength = (light_props.normal_strength - 1.0 * dt).max(0.0);
        }

        // Display current values (you might want to use a UI system for this)
        if keyboard_input.just_pressed(KeyCode::Space) {
            println!("Light Properties:");
            println!("  Intensity: {:.2}", light_props.intensity);
            println!("  Ambient: {:.2}", light_props.ambient_intensity);
            println!("  Radius: {:.1}", light_props.radius);
            println!("  Falloff: {:.2}", light_props.falloff);
            println!("  Height: {:.1}", light_props.height);
            println!("  Normal Strength: {:.2}", light_props.normal_strength);
            println!(
                "Controls: I/K=intensity, O/L=ambient, [/]=radius, +/-=falloff, U/J=height, P/;=normal"
            );
            println!("Normal Strength Guide: 0=no effect, 0.5=subtle, 1.0=strong, 2.0=very strong");
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
    mut custom_materials: ResMut<Assets<NormalMappedMaterial>>,
    sprite_query: Query<&MeshMaterial2d<NormalMappedMaterial>>,
) {
    if let Ok((light_transform, light_props)) = light_query.single()
        && let Ok(material_handle) = sprite_query.single()
        && let Some(material) = custom_materials.get_mut(material_handle)
    {
        material.uniform_data.light_pos_world_2d = light_transform.translation.truncate();
        material.uniform_data.light_color =
            LinearRgba::from(light_props.color) * light_props.intensity;
        material.uniform_data.ambient_light_color =
            LinearRgba::from(light_props.ambient_color) * light_props.ambient_intensity;
        material.uniform_data.light_radius = light_props.radius;
        material.uniform_data.light_falloff = light_props.falloff;
        material.uniform_data.light_height = light_props.height;
        material.uniform_data.normal_strength = light_props.normal_strength;
    }
}
