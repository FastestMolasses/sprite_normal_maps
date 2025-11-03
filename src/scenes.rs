use bevy::prelude::*;

/// Resource to track the current scene
#[derive(Resource, Default, Clone, Copy, PartialEq, Eq)]
pub enum CurrentScene {
    #[default]
    TextureMapped,
    Procedural,
}

/// Resource to select CPU or GPU rendering for procedural volumes
#[derive(Resource, Default, Clone, Copy, PartialEq, Eq)]
pub enum VolumeRenderMode {
    #[default]
    Cpu,
    Gpu,
}

impl VolumeRenderMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            VolumeRenderMode::Cpu => "CPU (Software Raymarching)",
            VolumeRenderMode::Gpu => "GPU (Compute Shader)",
        }
    }
}

// Marker components for scene entities
#[derive(Component)]
pub struct TextureMappedSceneEntity;

#[derive(Component)]
pub struct ProceduralSceneEntity;

#[derive(Component)]
pub struct PositionMappedSprite;
