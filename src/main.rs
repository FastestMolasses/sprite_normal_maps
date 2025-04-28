use bevy::{color::palettes::css::*, prelude::*};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, handle_input)
        .run();
}

#[derive(Component)]
struct LightMarker;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);
    commands.spawn((
        Sprite::from_image(asset_server.load("Tree Diffuse 1.png")),
        Transform::from_xyz(0., 0., 0.),
    ));
    commands.spawn((
        Sprite::from_image(asset_server.load("Tree Normal Map 1.png")),
        Transform::from_xyz(0., 0., 0.5),
    ));
    commands.spawn((
        SpotLight {
            intensity: 100_000.0,
            color: LIME.into(),
            shadows_enabled: true,
            inner_angle: 0.6,
            outer_angle: 0.8,
            ..default()
        },
        LightMarker,
        Mesh2d(meshes.add(Rectangle::default())),
        MeshMaterial2d(materials.add(Color::from(PURPLE))),
        Transform {
            translation: Vec3::new(0.0, 0.0, 1.0),
            rotation: Quat::from_rotation_z(0.0),
            scale: Vec3::splat(64.0),
        }
    ));
}

fn handle_input(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<LightMarker>>,
) {
    const SPEED: f32 = 300.0;
    for mut transform in query.iter_mut() {
        if keyboard_input.pressed(KeyCode::KeyW) {
            transform.translation.y += SPEED * time.delta_secs();
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            transform.translation.y -= SPEED * time.delta_secs();
        }
        if keyboard_input.pressed(KeyCode::KeyA) {
            transform.translation.x -= SPEED * time.delta_secs();
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            transform.translation.x += SPEED * time.delta_secs();
        }
    }
}
