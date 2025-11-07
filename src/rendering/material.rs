use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use bevy::sprite::{Material2d, Material2dPlugin};

/// Material for displaying rendered voxel world
#[derive(AsBindGroup, Debug, Clone, Asset, TypePath)]
pub struct VoxelWorldMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub position_texture: Handle<Image>,

    #[texture(2)]
    #[sampler(3)]
    pub normal_texture: Handle<Image>,

    #[texture(4)]
    #[sampler(5)]
    pub diffuse_texture: Handle<Image>,
}

impl Material2d for VoxelWorldMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/voxel_world_display.wgsl".into()
    }
}

/// Plugin for voxel world material
pub struct VoxelWorldMaterialPlugin;

impl Plugin for VoxelWorldMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<VoxelWorldMaterial>::default());
    }
}
