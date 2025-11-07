use bevy::prelude::*;
use bevy::render::renderer::{RenderDevice, RenderQueue};
use bevy::render::render_resource::*;
use crate::world::{WorldChunk, ChunkManager, CHUNK_SIZE};

/// Plugin for GPU-accelerated voxel simulation
/// Uses compute shaders to simulate voxels on the GPU
pub struct GpuSimulationPlugin;

impl Plugin for GpuSimulationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GpuSimulationState>()
           .add_systems(Update, run_gpu_simulation);
    }
}

/// Resource to track GPU simulation state
#[derive(Resource, Default)]
struct GpuSimulationState {
    pipeline: Option<ComputePipeline>,
    bind_group_layout: Option<BindGroupLayout>,
    last_sim_time: f32,
}

/// Run GPU simulation on chunks
fn run_gpu_simulation(
    time: Res<Time>,
    mut state: ResMut<GpuSimulationState>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    manager: Res<ChunkManager>,
    mut chunks: Query<&mut WorldChunk>,
) {
    // Run at 15Hz for now (every ~0.066 seconds)
    const SIM_RATE: f32 = 0.066;
    
    if time.elapsed_secs() - state.last_sim_time < SIM_RATE {
        return;
    }
    state.last_sim_time = time.elapsed_secs();
    
    // Initialize pipeline if needed
    if state.pipeline.is_none() {
        initialize_pipeline(&mut state, &render_device);
    }
    
    // For now, just log that we would run GPU sim
    // Full implementation requires complex texture management
    let active_chunks = chunks.iter().filter(|c| c.has_dynamic_elements).count();
    
    if active_chunks > 0 {
        debug!("Would run GPU simulation on {} chunks", active_chunks);
    }
}

/// Initialize the compute pipeline
fn initialize_pipeline(state: &mut GpuSimulationState, device: &RenderDevice) {
    // Create bind group layout
    let bind_group_layout = device.create_bind_group_layout(
        "gpu_sim_bind_group_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::COMPUTE,
            (
                // Input texture
                texture_storage_3d(TextureFormat::R32Uint, StorageTextureAccess::ReadOnly),
                // Output texture  
                texture_storage_3d(TextureFormat::R32Uint, StorageTextureAccess::WriteOnly),
                // Params uniform
                uniform_buffer::<SimParams>(false),
            ),
        ),
    );
    
    state.bind_group_layout = Some(bind_group_layout);
    
    info!("GPU simulation pipeline initialized");
}

#[derive(ShaderType)]
struct SimParams {
    chunk_size: u32,
    delta_time: f32,
    time_elapsed: f32,
    random_seed: u32,
}

fn texture_storage_3d(format: TextureFormat, access: StorageTextureAccess) -> BindGroupLayoutEntry {
    BindGroupLayoutEntry {
        binding: u32::MAX,
        visibility: ShaderStages::COMPUTE,
        ty: BindingType::StorageTexture {
            access,
            format,
            view_dimension: TextureViewDimension::D3,
        },
        count: None,
    }
}

fn uniform_buffer<T: ShaderType>(has_dynamic_offset: bool) -> BindGroupLayoutEntry {
    BindGroupLayoutEntry {
        binding: u32::MAX,
        visibility: ShaderStages::COMPUTE,
        ty: BindingType::Buffer {
            ty: BufferBindingType::Uniform,
            has_dynamic_offset,
            min_binding_size: Some(T::min_size()),
        },
        count: None,
    }
}
