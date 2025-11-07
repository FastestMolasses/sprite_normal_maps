use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::*;
use crate::world::chunk::{WorldChunk, CHUNK_SIZE};

/// Upload chunk voxel data to GPU as a 3D texture
pub fn create_chunk_texture(
    chunk: &WorldChunk,
    images: &mut Assets<Image>,
) -> Handle<Image> {
    // Convert voxel data to bytes for GPU upload
    let voxel_data: Vec<u8> = chunk.voxels
        .iter()
        .flat_map(|v| v.as_u32().to_le_bytes())
        .collect();
    
    // Create 3D texture
    let mut image = Image::new(
        Extent3d {
            width: CHUNK_SIZE,
            height: CHUNK_SIZE,
            depth_or_array_layers: CHUNK_SIZE,
        },
        TextureDimension::D3,
        voxel_data,
        TextureFormat::R32Uint, // Store packed u32 voxel data
        RenderAssetUsages::RENDER_WORLD,
    );
    
    // Set texture sampling parameters
    image.sampler = bevy::image::ImageSampler::Descriptor(bevy::image::ImageSamplerDescriptor {
        address_mode_u: bevy::image::ImageAddressMode::ClampToEdge,
        address_mode_v: bevy::image::ImageAddressMode::ClampToEdge,
        address_mode_w: bevy::image::ImageAddressMode::ClampToEdge,
        mag_filter: bevy::image::ImageFilterMode::Nearest, // Use nearest for voxel data
        min_filter: bevy::image::ImageFilterMode::Nearest,
        mipmap_filter: bevy::image::ImageFilterMode::Nearest,
        ..default()
    });
    
    images.add(image)
}

/// Create output render targets (position, normal, diffuse)
pub fn create_render_targets(
    width: u32,
    height: u32,
    images: &mut Assets<Image>,
) -> RenderTargets {
    let create_texture = |format: TextureFormat| {
        let mut img = Image::new(
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            vec![0u8; (width * height * 4) as usize],
            format,
            RenderAssetUsages::RENDER_WORLD,
        );
        img.texture_descriptor.usage = 
            TextureUsages::STORAGE_BINDING | 
            TextureUsages::TEXTURE_BINDING | 
            TextureUsages::COPY_SRC |
            TextureUsages::COPY_DST;
        img
    };
    
    RenderTargets {
        position: images.add(create_texture(TextureFormat::Rgba16Float)),
        normal: images.add(create_texture(TextureFormat::Rgba8Unorm)),
        diffuse: images.add(create_texture(TextureFormat::Rgba8Unorm)),
    }
}

/// Handles to the render target textures
#[derive(Resource, Clone)]
pub struct RenderTargets {
    pub position: Handle<Image>,
    pub normal: Handle<Image>,
    pub diffuse: Handle<Image>,
}

/// Plugin for GPU rendering systems
pub struct GpuRendererPlugin;

impl Plugin for GpuRendererPlugin {
    fn build(&self, _app: &mut App) {
        info!("GPU renderer plugin initialized");
        // TODO: Add render world systems for compute-based rendering
    }
}
