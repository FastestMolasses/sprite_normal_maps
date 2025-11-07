use bevy::prelude::*;
use crate::world::{WorldChunk, ChunkManager, VoxelData, MaterialType};

// Simple random number generator for simulation
fn simple_random() -> f32 {
    static mut SEED: u32 = 12345;
    unsafe {
        SEED = (SEED * 1664525 + 1013904223) & 0xFFFFFFFF;
        (SEED as f32) / (u32::MAX as f32)
    }
}

/// Plugin for simple CPU-based voxel simulation (temporary, will move to GPU)
pub struct CpuSimulationPlugin;

impl Plugin for CpuSimulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, simulate_fire_cpu);
    }
}

/// Simple CPU simulation: make fire spread, rise, and turn to smoke
fn simulate_fire_cpu(
    time: Res<Time>,
    manager: Res<ChunkManager>,
    mut chunks: Query<&mut WorldChunk>,
) {
    // Run simulation at ~15Hz (every 0.066 seconds) for smoother animation
    static mut ACCUMULATOR: f32 = 0.0;
    const SIM_RATE: f32 = 0.066;
    
    unsafe {
        ACCUMULATOR += time.delta_secs();
        if ACCUMULATOR < SIM_RATE {
            return;
        }
        ACCUMULATOR -= SIM_RATE;
    }
    
    // Simulate each chunk with dynamic elements
    for (_chunk_pos, &entity) in manager.chunks.iter() {
        if let Ok(mut chunk) = chunks.get_mut(entity) {
            if !chunk.has_dynamic_elements {
                continue;
            }
            
            simulate_chunk(&mut chunk);
        }
    }
}

/// Simulate a single chunk
fn simulate_chunk(chunk: &mut WorldChunk) {
    let chunk_size = 64u32;
    
    // Build a list of changes to apply (can't modify while iterating)
    let mut changes: Vec<(u32, u32, u32, VoxelData)> = Vec::new();
    
    // Iterate through all voxels
    for z in 0..chunk_size {
        for y in 0..chunk_size {
            for x in 0..chunk_size {
                if let Some(voxel) = chunk.get_voxel(x, y, z) {
                    match voxel.material() {
                        MaterialType::Fire => {
                            simulate_fire_voxel(chunk, x, y, z, voxel, &mut changes);
                        }
                        MaterialType::Smoke => {
                            simulate_smoke_voxel(chunk, x, y, z, voxel, &mut changes);
                        }
                        MaterialType::Water => {
                            simulate_water_voxel(chunk, x, y, z, voxel, &mut changes);
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    
    // Apply all changes
    for (x, y, z, new_voxel) in changes {
        chunk.set_voxel(x, y, z, new_voxel);
    }
}

/// Simulate fire: spread to neighbors, rise, convert to smoke
fn simulate_fire_voxel(
    chunk: &WorldChunk,
    x: u32,
    y: u32,
    z: u32,
    voxel: VoxelData,
    changes: &mut Vec<(u32, u32, u32, VoxelData)>,
) {
    // Fire has a chance to turn into smoke
    if simple_random() < 0.05 { // 5% chance per tick
        let smoke = VoxelData::new(MaterialType::Smoke, 200, 150, 0);
        changes.push((x, y, z, smoke));
        return;
    }
    
    // Try to rise (fire is buoyant)
    if y < 63 {
        if let Some(above) = chunk.get_voxel(x, y + 1, z) {
            if above.material() == MaterialType::Air {
                // Move fire up
                changes.push((x, y, z, VoxelData::air()));
                changes.push((x, y + 1, z, voxel));
                return;
            }
        }
    }
    
    // Try to spread horizontally (25% chance)
    if simple_random() < 0.25 {
        let dirs = [(1, 0), (-1, 0), (0, 1), (0, -1)];
        let (dx, dz) = dirs[(simple_random() * 4.0) as usize];
        
        let nx = (x as i32 + dx) as u32;
        let nz = (z as i32 + dz) as u32;
        
        if nx < 64 && nz < 64 {
            if let Some(neighbor) = chunk.get_voxel(nx, y, nz) {
                // Spread to flammable materials
                match neighbor.material() {
                    MaterialType::Air => {
                        // Spread fire to air
                        let new_fire = VoxelData::new(MaterialType::Fire, 255, 200, 0);
                        changes.push((nx, y, nz, new_fire));
                    }
                    MaterialType::Wood => {
                        // Ignite wood
                        let new_fire = VoxelData::new(MaterialType::Fire, 255, 250, 0);
                        changes.push((nx, y, nz, new_fire));
                    }
                    _ => {}
                }
            }
        }
    }
}

/// Simulate smoke: rise slowly
fn simulate_smoke_voxel(
    chunk: &WorldChunk,
    x: u32,
    y: u32,
    z: u32,
    voxel: VoxelData,
    changes: &mut Vec<(u32, u32, u32, VoxelData)>,
) {
    // Smoke dissipates over time
    if simple_random() < 0.02 { // 2% chance to disappear
        changes.push((x, y, z, VoxelData::air()));
        return;
    }
    
    // Try to rise (smoke is buoyant but slower than fire)
    if y < 63 && simple_random() < 0.3 { // 30% chance to rise
        if let Some(above) = chunk.get_voxel(x, y + 1, z) {
            if above.material() == MaterialType::Air {
                // Move smoke up
                changes.push((x, y, z, VoxelData::air()));
                changes.push((x, y + 1, z, voxel));
            }
        }
    }
}

/// Simulate water: fall down
fn simulate_water_voxel(
    chunk: &WorldChunk,
    x: u32,
    y: u32,
    z: u32,
    voxel: VoxelData,
    changes: &mut Vec<(u32, u32, u32, VoxelData)>,
) {
    // Try to fall down
    if y > 0 {
        if let Some(below) = chunk.get_voxel(x, y - 1, z) {
            match below.material() {
                MaterialType::Air => {
                    // Fall down
                    changes.push((x, y, z, VoxelData::air()));
                    changes.push((x, y - 1, z, voxel));
                    return;
                }
                MaterialType::Fire => {
                    // Extinguish fire
                    changes.push((x, y, z, VoxelData::air()));
                    changes.push((x, y - 1, z, VoxelData::new(MaterialType::Smoke, 150, 50, 0)));
                    return;
                }
                _ => {}
            }
        }
    }
    
    // Try to spread horizontally if can't fall
    if simple_random() < 0.5 {
        let dirs = [(1, 0), (-1, 0), (0, 1), (0, -1)];
        let (dx, dz) = dirs[(simple_random() * 4.0) as usize];
        
        let nx = (x as i32 + dx) as u32;
        let nz = (z as i32 + dz) as u32;
        
        if nx < 64 && nz < 64 {
            if let Some(neighbor) = chunk.get_voxel(nx, y, nz) {
                if neighbor.material() == MaterialType::Air {
                    // Spread water horizontally
                    changes.push((nx, y, nz, voxel));
                }
            }
        }
    }
}
