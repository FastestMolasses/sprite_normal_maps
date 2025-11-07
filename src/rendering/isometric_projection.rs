use bevy::prelude::*;

/// Camera setup for isometric view
#[derive(Component)]
pub struct IsometricCamera {
    /// Angle of the camera on the X axis (typically 45 degrees for Diablo-style)
    pub pitch: f32,
    
    /// Height of the camera above the ground
    pub height: f32,
    
    /// Distance from the focal point
    pub distance: f32,
}

impl Default for IsometricCamera {
    fn default() -> Self {
        Self {
            pitch: 45.0_f32.to_radians(),
            height: 100.0,
            distance: 200.0,
        }
    }
}

/// Plugin for isometric camera and projection setup
pub struct IsometricProjectionPlugin;

impl Plugin for IsometricProjectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_isometric_camera)
           .add_systems(Update, update_camera_transform);
    }
}

fn setup_isometric_camera(mut commands: Commands) {
    // Spawn the main camera with isometric settings
    commands.spawn((
        Camera2d,
        IsometricCamera::default(),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    info!("Isometric camera spawned");
}

fn update_camera_transform(
    mut query: Query<(&IsometricCamera, &mut Transform)>,
) {
    for (iso_cam, mut transform) in query.iter_mut() {
        // Calculate isometric view position
        // This is a simplified version - we'll enhance it later
        let x = 0.0;
        let y = iso_cam.height;
        let z = iso_cam.distance;
        
        transform.translation = Vec3::new(x, y, z);
    }
}
