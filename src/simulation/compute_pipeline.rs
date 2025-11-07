use bevy::prelude::*;
use bevy::render::extract_resource::{ExtractResource, ExtractResourcePlugin};
use bevy::render::render_resource::*;
use bevy::render::renderer::{RenderDevice, RenderQueue};
use bevy::render::{Render, RenderApp, RenderSet};
use bevy::render::render_asset::RenderAssets;
use bevy::render::texture::GpuImage;
use crate::world::chunk::CHUNK_SIZE;
use std::collections::HashMap;

/// Uniform data for simulation compute shader
#[derive(ShaderType, Clone, Copy)]
struct SimulationParams {
    chunk_size: u32,
    delta_time: f32,
    time_elapsed: f32,
    random_seed: u32,
}

/// Resource to control simulation timing
#[derive(Resource, Clone, ExtractResource)]
pub struct SimulationSettings {
    pub enabled: bool,
    pub fixed_timestep: f32, // Simulate at fixed rate (e.g., 1/60)
    pub time_accumulator: f32,
    pub time_elapsed: f32,
}

impl Default for SimulationSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            fixed_timestep: 1.0 / 60.0, // 60Hz simulation
            time_accumulator: 0.0,
            time_elapsed: 0.0,
        }
    }
}

/// Plugin for GPU compute simulation
pub struct ComputeSimulationPlugin;

impl Plugin for ComputeSimulationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SimulationSettings>()
           .add_plugins(ExtractResourcePlugin::<SimulationSettings>::default())
           .add_systems(Update, update_simulation_time);

        // Add render world systems (will initialize pipeline when render world is ready)
        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .init_resource::<SimulationPipeline>()
                .add_systems(Render, prepare_simulation_pipeline.in_set(RenderSet::Prepare));
        }

        info!("Compute simulation plugin initialized");
    }
}

/// Update simulation timing in main world
fn update_simulation_time(
    time: Res<Time>,
    mut settings: ResMut<SimulationSettings>,
) {
    if settings.enabled {
        settings.time_accumulator += time.delta_secs();
        settings.time_elapsed += time.delta_secs();
    }
}

/// Resource containing the compute pipeline
#[derive(Resource, Default)]
struct SimulationPipeline {
    bind_group_layout: Option<BindGroupLayout>,
    pipeline: Option<CachedComputePipelineId>,
    initialized: bool,
}

impl SimulationPipeline {
    fn ensure_initialized(&mut self, world: &mut World) {
        if self.initialized {
            return;
        }
        
        let render_device = world.resource::<RenderDevice>();
        let pipeline_cache = world.resource::<PipelineCache>();

        // Create bind group layout
        let bind_group_layout = render_device.create_bind_group_layout(
            "simulation_bind_group_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    // Input voxel texture (3D, read)
                    texture_storage_3d(TextureFormat::R32Uint, StorageTextureAccess::ReadOnly),
                    // Output voxel texture (3D, write)
                    texture_storage_3d(TextureFormat::R32Uint, StorageTextureAccess::WriteOnly),
                    // Simulation parameters uniform
                    BindGroupLayoutEntry {
                        binding: u32::MAX,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: Some(SimulationParams::min_size()),
                        },
                        count: None,
                    },
                ),
            ),
        );

        // Create compute pipeline
        let shader = world.resource::<AssetServer>()
            .load("shaders/element_simulation.wgsl");

        let pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("element_simulation_pipeline".into()),
            layout: vec![bind_group_layout.clone()],
            push_constant_ranges: vec![],
            shader,
            shader_defs: vec![],
            entry_point: "main".into(),
            zero_initialize_workgroup_memory: true,
        });

        self.bind_group_layout = Some(bind_group_layout);
        self.pipeline = Some(pipeline);
        self.initialized = true;
        
        info!("Simulation pipeline initialized");
    }
}

/// Prepare simulation pipeline (placeholder for actual dispatch)
fn prepare_simulation_pipeline(
    world: &mut World,
) {
    // Lazy initialize pipeline
    world.resource_scope(|world, mut pipeline: Mut<SimulationPipeline>| {
        pipeline.ensure_initialized(world);
    });
    
    let mut settings = world.resource_mut::<SimulationSettings>();
    
    if !settings.enabled {
        return;
    }
    
    // Check if we should run a simulation step (fixed timestep)
    if settings.time_accumulator >= settings.fixed_timestep {
        settings.time_accumulator -= settings.fixed_timestep;
        
        // TODO: Dispatch compute shader for active chunks
        // For now, we'll implement this once we have proper texture double-buffering
        // The infrastructure is ready, we just need to:
        // 1. Extract chunk GPU textures to render world
        // 2. Create read/write texture pairs  
        // 3. Create bind groups with input/output textures
        // 4. Dispatch workgroups (CHUNK_SIZE/8 per dimension)
        // 5. Copy results back to chunks
        
        // This would look like:
        // for each chunk with has_dynamic_elements:
        //   - create bind group with (read_texture, write_texture, params_uniform)
        //   - encoder.dispatch_workgroups(8, 8, 8) // 64/8 = 8 workgroups per dimension
        //   - swap read/write textures
    }
}

// Helper function for texture storage binding
fn texture_storage_3d(
    format: TextureFormat,
    access: StorageTextureAccess,
) -> BindGroupLayoutEntry {
    BindGroupLayoutEntry {
        binding: u32::MAX, // Sequential layout will assign this
        visibility: ShaderStages::COMPUTE,
        ty: BindingType::StorageTexture {
            access,
            format,
            view_dimension: TextureViewDimension::D3,
        },
        count: None,
    }
}
