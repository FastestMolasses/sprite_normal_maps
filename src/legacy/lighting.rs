use bevy::prelude::*;

#[derive(Component)]
pub struct MovableLightMarker {
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

impl Default for MovableLightMarker {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            intensity: 1.0,
            ambient_color: Color::srgb(0.1, 0.1, 0.15),
            ambient_intensity: 0.2,
            radius: 300.0,
            falloff: 1.5,
            position_scale: 1.0,
            debug_mode: 0,
            virtual_height: 0.0,
        }
    }
}
