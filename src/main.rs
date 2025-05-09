use bevy::{
    color::palettes::css::*,
    math::primitives::Rectangle,
    prelude::*,
    reflect::TypePath,
    render::render_resource::{AsBindGroup, ShaderRef, ShaderType},
    sprite::{AlphaMode2d, Material2d, Material2dPlugin},
};

#[derive(ShaderType, Debug, Clone, Default)]
pub struct MyMaterialUniformData {
    light_pos_world_2d: Vec2,
    light_color: LinearRgba,
    ambient_light_color: LinearRgba,
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
    pub uniform_data: MyMaterialUniformData,
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
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut custom_materials: ResMut<Assets<NormalMappedMaterial>>,
) {
    commands.spawn(Camera2d);

    let diffuse_handle: Handle<Image> = asset_server.load("Tree Diffuse 1.png");
    let normal_handle: Handle<Image> = asset_server.load("Tree Normal Map 1.png");

    // Define initial light properties
    let initial_light_props = MovableLightMarker {
        color: WHITE.into(),
        intensity: 75_000.0,
        ambient_color: DARK_SLATE_GRAY.into(),
        ambient_intensity: 0.2,
    };
    let initial_light_pos_xy = Vec2::new(100.0, 50.0);

    // Create the material instance
    let tree_material = custom_materials.add(NormalMappedMaterial {
        diffuse_texture: diffuse_handle,
        normal_texture: normal_handle,
        uniform_data: MyMaterialUniformData {
            light_pos_world_2d: initial_light_pos_xy,
            light_color: LinearRgba::from(initial_light_props.color)
                * initial_light_props.intensity
                / 75_000.0,
            ambient_light_color: LinearRgba::from(initial_light_props.ambient_color)
                * initial_light_props.ambient_intensity,
        },
    });

    let sprite_width = 1024.0;
    let sprite_height = 1024.0;

    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(sprite_width, sprite_height))),
        MeshMaterial2d(tree_material),
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
        // Control light intensity
        if keyboard_input.pressed(KeyCode::KeyI) {
            light_props.intensity += 0.2 * time.delta_secs();
        }
        if keyboard_input.pressed(KeyCode::KeyK) {
            light_props.intensity = (light_props.intensity - 0.2 * time.delta_secs()).max(0.0);
        }

        // Control ambient intensity
        if keyboard_input.pressed(KeyCode::KeyO) {
            light_props.ambient_intensity += 0.05 * time.delta_secs();
        }
        if keyboard_input.pressed(KeyCode::KeyL) {
            light_props.ambient_intensity =
                (light_props.ambient_intensity - 0.05 * time.delta_secs()).max(0.0);
        }

        // Control spotlight radius
        // if keyboard_input.pressed(KeyCode::BracketRight) {
        //     light_props.radius += 50.0 * time.delta_secs();
        // }
        // if keyboard_input.pressed(KeyCode::BracketLeft) {
        //     light_props.radius = (light_props.radius - 50.0 * time.delta_secs()).max(10.0);
        // }
    }
}

fn handle_input(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<MovableLightMarker>>,
) {
    const SPEED: f32 = 200.0; // Adjusted speed
    if let Ok(mut light_transform) = query.single_mut() {
        if keyboard_input.pressed(KeyCode::KeyW) {
            light_transform.translation.y += SPEED * time.delta_secs();
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            light_transform.translation.y -= SPEED * time.delta_secs();
        }
        if keyboard_input.pressed(KeyCode::KeyA) {
            light_transform.translation.x -= SPEED * time.delta_secs();
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            light_transform.translation.x += SPEED * time.delta_secs();
        }
    }
}

/// System to update the material's uniform data based on the light's transform and properties
fn update_material_light_info(
    light_query: Query<(&Transform, &MovableLightMarker)>,
    mut custom_materials: ResMut<Assets<NormalMappedMaterial>>,
    sprite_query: Query<&MeshMaterial2d<NormalMappedMaterial>>,
) {
    if let Ok((light_transform, light_props)) = light_query.single() {
        if let Ok(material_handle) = sprite_query.single() {
            if let Some(material) = custom_materials.get_mut(material_handle) {
                material.uniform_data.light_pos_world_2d = light_transform.translation.truncate();
                // Normalize intensity somewhat for shader, or adjust shader to handle larger values
                material.uniform_data.light_color =
                    LinearRgba::from(light_props.color) * light_props.intensity / 75_000.0;
                material.uniform_data.ambient_light_color =
                    LinearRgba::from(light_props.ambient_color) * light_props.ambient_intensity;
            }
        }
    }
}
