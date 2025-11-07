use bevy::prelude::*;
use bevy::render::extract_component::{ExtractComponent, ExtractComponentPlugin};
use bevy::render::render_asset::{RenderAssets, RenderAssetUsages};
use bevy::render::render_graph::{self, RenderGraph, RenderLabel};
use bevy::render::render_resource::*;
use bevy::render::renderer::{RenderContext, RenderDevice};
use bevy::render::{Render, RenderApp, RenderSet};
use bevy::render::texture::GpuImage;

use crate::volume::Volume;

/// Component for entities that use GPU volume rendering
#[derive(Component, Clone)]
pub struct GpuVolumeRenderer {
    pub volume_texture: Handle<Image>,
    pub position_output: Handle<Image>,
    pub normal_output: Handle<Image>,
    pub diffuse_output: Handle<Image>,
    pub rotation: Vec3,
    pub volume_size: f32,
    pub output_size: u32,
}

impl ExtractComponent for GpuVolumeRenderer {
    type QueryData = &'static Self;
    type QueryFilter = ();
    type Out = Self;

    fn extract_component(item: bevy::ecs::query::QueryItem<Self::QueryData>) -> Option<Self::Out> {
        Some(item.clone())
    }
}

/// Shader uniform for volume rendering parameters
#[derive(ShaderType, Clone, Copy)]
struct VolumeParamsUniform {
    rotation_matrix: Mat3,
    volume_size: f32,
    threshold: f32,
    output_width: u32,
    output_height: u32,
}

/// Resource containing the compute pipeline
#[derive(Resource)]
struct VolumeComputePipeline {
    bind_group_layout: BindGroupLayout,
    pipeline: CachedComputePipelineId,
}

/// Label for the volume rendering compute node
#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct VolumeRenderLabel;

/// Upload volume data to GPU as a 3D texture
pub fn create_volume_texture(
    volume: &Volume,
    images: &mut ResMut<Assets<Image>>,
) -> Handle<Image> {
    let size = volume.dimensions.x;
    
    // Convert f32 density data to u8 grayscale
    let mut texture_data = Vec::with_capacity((size * size * size) as usize);
    for density in &volume.data {
        texture_data.push((*density * 255.0) as u8);
    }
    
    // Create 3D texture
    let mut image = Image::new(
        Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: size,
        },
        TextureDimension::D3,
        texture_data,
        TextureFormat::R8Unorm,
        RenderAssetUsages::RENDER_WORLD,
    );
    
    // Set texture settings for 3D sampling
    image.sampler = bevy::image::ImageSampler::Descriptor(bevy::image::ImageSamplerDescriptor {
        address_mode_u: bevy::image::ImageAddressMode::ClampToEdge,
        address_mode_v: bevy::image::ImageAddressMode::ClampToEdge,
        address_mode_w: bevy::image::ImageAddressMode::ClampToEdge,
        mag_filter: bevy::image::ImageFilterMode::Linear,
        min_filter: bevy::image::ImageFilterMode::Linear,
        mipmap_filter: bevy::image::ImageFilterMode::Linear,
        ..default()
    });
    
    images.add(image)
}

/// Create output textures for position, normal, and diffuse maps
pub fn create_output_textures(
    size: u32,
    images: &mut ResMut<Assets<Image>>,
) -> (Handle<Image>, Handle<Image>, Handle<Image>) {
    let create_texture = |format: TextureFormat| {
        let mut img = Image::new(
            Extent3d {
                width: size,
                height: size,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            vec![0u8; (size * size * 4) as usize],
            format,
            RenderAssetUsages::RENDER_WORLD,
        );
        // Mark as storage texture for GPU compute shader writes
        img.texture_descriptor.usage = TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST;
        img
    };
    
    let position = images.add(create_texture(TextureFormat::Rgba8Unorm));
    let normal = images.add(create_texture(TextureFormat::Rgba8Unorm));
    let diffuse = images.add(create_texture(TextureFormat::Rgba8Unorm));
    
    (position, normal, diffuse)
}

/// Setup the compute pipeline
fn prepare_pipeline(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    pipeline_cache: ResMut<PipelineCache>,
    existing_pipeline: Option<Res<VolumeComputePipeline>>,
) {
    // Only prepare once
    if existing_pipeline.is_some() {
        return;
    }
    
    // Create bind group layout
    let bind_group_layout = render_device.create_bind_group_layout(
        "volume_compute_bind_group_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::COMPUTE,
            (
                // Volume texture (3D) - binding 0
                BindGroupLayoutEntry {
                    binding: u32::MAX, // Sequential ignores this
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D3,
                        multisampled: false,
                    },
                    count: None,
                },
                // Volume sampler - binding 1
                BindGroupLayoutEntry {
                    binding: u32::MAX,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                // Position output (storage texture) - binding 2
                BindGroupLayoutEntry {
                    binding: u32::MAX,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::WriteOnly,
                        format: TextureFormat::Rgba8Unorm,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
                // Normal output (storage texture) - binding 3
                BindGroupLayoutEntry {
                    binding: u32::MAX,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::WriteOnly,
                        format: TextureFormat::Rgba8Unorm,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
                // Diffuse output (storage texture) - binding 4
                BindGroupLayoutEntry {
                    binding: u32::MAX,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::WriteOnly,
                        format: TextureFormat::Rgba8Unorm,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
                // Params uniform - binding 5
                BindGroupLayoutEntry {
                    binding: u32::MAX,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(VolumeParamsUniform::min_size()),
                    },
                    count: None,
                },
            ),
        ),
    );
    
    // Create compute pipeline
    let pipeline_id = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        label: Some("volume_render_pipeline".into()),
        layout: vec![bind_group_layout.clone()],
        push_constant_ranges: vec![],
        shader: VOLUME_SHADER_HANDLE,
        shader_defs: vec![],
        entry_point: "main".into(),
        zero_initialize_workgroup_memory: false,
    });
    
    commands.insert_resource(VolumeComputePipeline {
        bind_group_layout,
        pipeline: pipeline_id,
    });
}

/// Compute node that dispatches the volume rendering shader
struct VolumeRenderNode;

impl render_graph::Node for VolumeRenderNode {
    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        // Get resources
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<VolumeComputePipeline>();
        let gpu_images = world.resource::<RenderAssets<GpuImage>>();
        let render_device = world.resource::<RenderDevice>();
        let render_queue = world.resource::<bevy::render::renderer::RenderQueue>();
        
        // Get the prepared pipeline
        let Some(compute_pipeline) = pipeline_cache.get_compute_pipeline(pipeline.pipeline) else {
            return Ok(());
        };
        
        // Query for volume renderers
        let entity_renderer_pairs: Vec<(Entity, &GpuVolumeRenderer)> = world
            .iter_entities()
            .filter_map(|entity_ref| {
                entity_ref.get::<GpuVolumeRenderer>()
                    .map(|renderer| (entity_ref.id(), renderer))
            })
            .collect();
        
        if entity_renderer_pairs.is_empty() {
            return Ok(());
        }
        
        for (_, renderer) in entity_renderer_pairs {
            // Get GPU textures
            let Some(volume_texture) = gpu_images.get(&renderer.volume_texture) else {
                continue;
            };
            let Some(position_output) = gpu_images.get(&renderer.position_output) else {
                continue;
            };
            let Some(normal_output) = gpu_images.get(&renderer.normal_output) else {
                continue;
            };
            let Some(diffuse_output) = gpu_images.get(&renderer.diffuse_output) else {
                continue;
            };
            
            // Create rotation matrix
            let (sx, cx) = renderer.rotation.x.sin_cos();
            let (sy, cy) = renderer.rotation.y.sin_cos();
            let (sz, cz) = renderer.rotation.z.sin_cos();
            
            let rx = Mat3::from_cols(
                Vec3::new(1.0, 0.0, 0.0),
                Vec3::new(0.0, cx, sx),
                Vec3::new(0.0, -sx, cx),
            );
            
            let ry = Mat3::from_cols(
                Vec3::new(cy, 0.0, -sy),
                Vec3::new(0.0, 1.0, 0.0),
                Vec3::new(sy, 0.0, cy),
            );
            
            let rz = Mat3::from_cols(
                Vec3::new(cz, sz, 0.0),
                Vec3::new(-sz, cz, 0.0),
                Vec3::new(0.0, 0.0, 1.0),
            );
            
            let rotation_matrix = rz * ry * rx;
            
            // Create uniform data
            let params = VolumeParamsUniform {
                rotation_matrix,
                volume_size: renderer.volume_size,
                threshold: 0.3,
                output_width: renderer.output_size,
                output_height: renderer.output_size,
            };
            
            // Create uniform buffer
            let mut uniform_buffer = UniformBuffer::from(params);
            uniform_buffer.write_buffer(render_device, render_queue);
            
            let Some(uniform_binding) = uniform_buffer.binding() else {
                continue;
            };
            
            // Create bind group
            let bind_group = render_device.create_bind_group(
                "volume_compute_bind_group",
                &pipeline.bind_group_layout,
                &BindGroupEntries::sequential((
                    &volume_texture.texture_view,
                    &volume_texture.sampler,
                    &position_output.texture_view,
                    &normal_output.texture_view,
                    &diffuse_output.texture_view,
                    uniform_binding.clone(),
                )),
            );
            
            // Dispatch compute shader
            let mut pass = render_context
                .command_encoder()
                .begin_compute_pass(&ComputePassDescriptor {
                    label: Some("volume_render_pass"),
                    timestamp_writes: None,
                });
            
            pass.set_pipeline(compute_pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            
            // Dispatch with 8x8 workgroups
            let workgroup_count_x = renderer.output_size.div_ceil(8);
            let workgroup_count_y = renderer.output_size.div_ceil(8);
            pass.dispatch_workgroups(workgroup_count_x, workgroup_count_y, 1);
        }
        
        Ok(())
    }
}

/// Handle for the compute shader
use bevy::asset::weak_handle;
const VOLUME_SHADER_HANDLE: Handle<Shader> = weak_handle!("12345678-90AB-CDEF-1234-567890ABCDEF");

/// Plugin to add GPU volume rendering support
pub struct GpuVolumeRenderPlugin;

impl Plugin for GpuVolumeRenderPlugin {
    fn build(&self, app: &mut App) {
        // Load the compute shader
        let mut shaders = app.world_mut().resource_mut::<Assets<Shader>>();
        shaders.insert(
            &VOLUME_SHADER_HANDLE,
            Shader::from_wgsl(
                include_str!("../assets/shaders/volume_raymarcher.wgsl"),
                "volume_raymarcher.wgsl",
            ),
        );
        
        // Add extraction plugin
        app.add_plugins(ExtractComponentPlugin::<GpuVolumeRenderer>::default());
        
        // Setup render app
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        
        render_app
            .add_systems(Render, prepare_pipeline.in_set(RenderSet::Prepare))
            .add_systems(Render, queue_bind_groups.in_set(RenderSet::Queue));
        
        // Add compute node to render graph - should run before camera driver
        let mut render_graph = render_app.world_mut().resource_mut::<RenderGraph>();
        render_graph.add_node(VolumeRenderLabel, VolumeRenderNode);
        // Run before camera driver so textures are ready for rendering
        render_graph.add_node_edge(bevy::render::graph::CameraDriverLabel, VolumeRenderLabel);
    }
}

fn queue_bind_groups() {
    // Placeholder - bind groups are created in the node
}

