// Compute shader for voxel element simulation (cellular automata)
// Implements rules for fire, smoke, water, debris, etc.

// Input: Current frame voxel data
@group(0) @binding(0) var voxel_input: texture_storage_3d<r32uint, read>;

// Output: Next frame voxel data (double-buffered)
@group(0) @binding(1) var voxel_output: texture_storage_3d<r32uint, write>;

struct SimulationParams {
    chunk_size: u32,        // Size of chunk (typically 64)
    delta_time: f32,        // Time since last simulation step
    time_elapsed: f32,      // Total time elapsed
    random_seed: u32,       // For randomness
}

@group(0) @binding(2) var<uniform> params: SimulationParams;

// Material type IDs (must match Rust enum)
const MATERIAL_AIR: u32 = 0u;
const MATERIAL_ROCK: u32 = 1u;
const MATERIAL_DIRT: u32 = 2u;
const MATERIAL_WOOD: u32 = 3u;
const MATERIAL_METAL: u32 = 4u;
const MATERIAL_FIRE: u32 = 5u;
const MATERIAL_SMOKE: u32 = 6u;
const MATERIAL_WATER: u32 = 7u;
const MATERIAL_DEBRIS: u32 = 8u;

// Voxel flags
const FLAG_COLLISION: u32 = 1u;
const FLAG_EMITS_LIGHT: u32 = 2u;
const FLAG_TEMPORARY: u32 = 4u;
const FLAG_STATIC: u32 = 8u;
const FLAG_TRANSPARENT: u32 = 16u;

// Unpack voxel data from u32
struct VoxelData {
    material: u32,
    density: u32,
    temperature: u32,
    flags: u32,
}

fn unpack_voxel(packed: u32) -> VoxelData {
    var voxel: VoxelData;
    voxel.material = packed & 0xFFu;
    voxel.density = (packed >> 8u) & 0xFFu;
    voxel.temperature = (packed >> 16u) & 0xFFu;
    voxel.flags = (packed >> 24u) & 0xFFu;
    return voxel;
}

// Pack voxel data to u32
fn pack_voxel(voxel: VoxelData) -> u32 {
    return voxel.material | 
           (voxel.density << 8u) | 
           (voxel.temperature << 16u) | 
           (voxel.flags << 24u);
}

// Read voxel with bounds checking
fn read_voxel(pos: vec3<i32>) -> VoxelData {
    let size = i32(params.chunk_size);
    if (pos.x < 0 || pos.x >= size ||
        pos.y < 0 || pos.y >= size ||
        pos.z < 0 || pos.z >= size) {
        var empty: VoxelData;
        empty.material = MATERIAL_AIR;
        empty.density = 0u;
        empty.temperature = 0u;
        empty.flags = 0u;
        return empty;
    }
    let packed = textureLoad(voxel_input, pos).r;
    return unpack_voxel(packed);
}

// Write voxel
fn write_voxel(pos: vec3<i32>, voxel: VoxelData) {
    textureStore(voxel_output, pos, vec4<u32>(pack_voxel(voxel), 0u, 0u, 0u));
}

// Simple pseudo-random number generator
fn random(seed: u32, pos: vec3<i32>) -> f32 {
    let n = seed + u32(pos.x) * 374761393u + u32(pos.y) * 668265263u + u32(pos.z) * 1274126177u;
    let m = (n ^ (n >> 13u)) * 1597334677u;
    let x = m ^ (m >> 16u);
    return f32(x) / 4294967296.0;
}

// Check if a material is flammable
fn is_flammable(material: u32) -> bool {
    return material == MATERIAL_WOOD || material == MATERIAL_DEBRIS;
}

// Check if voxel is empty (air)
fn is_empty(voxel: VoxelData) -> bool {
    return voxel.material == MATERIAL_AIR || voxel.density == 0u;
}

// Check if voxel is solid
fn is_solid(voxel: VoxelData) -> bool {
    return (voxel.flags & FLAG_COLLISION) != 0u || 
           voxel.material == MATERIAL_ROCK ||
           voxel.material == MATERIAL_DIRT ||
           voxel.material == MATERIAL_WOOD ||
           voxel.material == MATERIAL_METAL;
}

// ============================================================================
// SIMULATION RULES
// ============================================================================

// Update fire element
fn update_fire(pos: vec3<i32>, current: VoxelData) -> VoxelData {
    var result = current;
    
    // Fire consumes itself over time (lifetime)
    if (result.density > 5u) {
        result.density -= 2u; // Burn out gradually
    } else {
        // Fire dies out, becomes air
        result.material = MATERIAL_AIR;
        result.density = 0u;
        result.temperature = 0u;
        result.flags = 0u;
        return result;
    }
    
    // Cool down temperature
    if (result.temperature > 100u) {
        result.temperature -= 1u;
    }
    
    // Fire rises (swap with air above)
    let above = read_voxel(pos + vec3<i32>(0, 0, 1));
    if (is_empty(above) && random(params.random_seed, pos) > 0.3) {
        // Move up (will be handled by swapping logic in main)
        result.density -= 10u; // Lose some density when rising
    }
    
    // Fire spreads to neighbors
    let rand = random(params.random_seed + 1u, pos);
    
    // Check 6 neighbors for spreading
    let offsets = array<vec3<i32>, 6>(
        vec3<i32>(1, 0, 0),
        vec3<i32>(-1, 0, 0),
        vec3<i32>(0, 1, 0),
        vec3<i32>(0, -1, 0),
        vec3<i32>(0, 0, 1),
        vec3<i32>(0, 0, -1)
    );
    
    // Random neighbor selection
    let neighbor_idx = u32(rand * 6.0);
    let neighbor_pos = pos + offsets[neighbor_idx];
    let neighbor = read_voxel(neighbor_pos);
    
    // Spread to flammable materials
    if (is_flammable(neighbor.material) && rand > 0.7) {
        // Neighbor will catch fire (handled in main loop)
        // For now, heat it up
        result.temperature = min(result.temperature + 10u, 255u);
    }
    
    // Spread to air (fire expands)
    if (is_empty(neighbor) && rand > 0.85) {
        // Fire can spread to empty space
        result.temperature = min(result.temperature + 5u, 255u);
    }
    
    return result;
}

// Update smoke element
fn update_smoke(pos: vec3<i32>, current: VoxelData) -> VoxelData {
    var result = current;
    
    // Smoke dissipates over time
    if (result.density > 3u) {
        result.density -= 1u;
    } else {
        // Smoke disappears
        result.material = MATERIAL_AIR;
        result.density = 0u;
        result.temperature = 0u;
        result.flags = 0u;
        return result;
    }
    
    // Smoke rises (like fire but more aggressive)
    let above = read_voxel(pos + vec3<i32>(0, 0, 1));
    if (is_empty(above)) {
        // Smoke rises quickly
        result.density = max(result.density - 5u, 1u);
    }
    
    // Smoke spreads horizontally
    let rand = random(params.random_seed + 2u, pos);
    if (rand > 0.6) {
        // Slight horizontal drift
        result.density = max(result.density - 1u, 1u);
    }
    
    return result;
}

// Update water element
fn update_water(pos: vec3<i32>, current: VoxelData) -> VoxelData {
    var result = current;
    
    // Water flows downward
    let below = read_voxel(pos + vec3<i32>(0, 0, -1));
    if (is_empty(below)) {
        // Move down (swap will be handled in main)
        return result;
    }
    
    // If can't move down, spread horizontally
    let rand = random(params.random_seed + 3u, pos);
    
    // Try to spread to horizontal neighbors
    let horizontal_offsets = array<vec3<i32>, 4>(
        vec3<i32>(1, 0, 0),
        vec3<i32>(-1, 0, 0),
        vec3<i32>(0, 1, 0),
        vec3<i32>(0, -1, 0)
    );
    
    let h_idx = u32(rand * 4.0);
    let h_neighbor_pos = pos + horizontal_offsets[h_idx];
    let h_neighbor = read_voxel(h_neighbor_pos);
    
    if (is_empty(h_neighbor) && rand > 0.5) {
        // Can spread horizontally
        result.density = max(result.density - 20u, 50u);
    }
    
    return result;
}

// Update debris element (falling particles)
fn update_debris(pos: vec3<i32>, current: VoxelData) -> VoxelData {
    var result = current;
    
    // Debris falls down
    let below = read_voxel(pos + vec3<i32>(0, 0, -1));
    if (is_empty(below)) {
        // Move down (will be swapped)
        return result;
    }
    
    // If can't fall, settle and become static
    if (is_solid(below)) {
        result.flags = result.flags | FLAG_STATIC;
        result.material = MATERIAL_DIRT; // Settled debris becomes dirt
    }
    
    return result;
}

// Main simulation dispatch
@compute @workgroup_size(4, 4, 4)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let pos = vec3<i32>(global_id);
    let size = i32(params.chunk_size);
    
    // Bounds check
    if (pos.x >= size || pos.y >= size || pos.z >= size) {
        return;
    }
    
    // Read current voxel
    let current = read_voxel(pos);
    
    // Skip static voxels (don't simulate)
    if ((current.flags & FLAG_STATIC) != 0u) {
        write_voxel(pos, current);
        return;
    }
    
    // Skip air
    if (is_empty(current)) {
        write_voxel(pos, current);
        return;
    }
    
    // Apply simulation rules based on material type
    var updated: VoxelData;
    
    switch (current.material) {
        case MATERIAL_FIRE: {
            updated = update_fire(pos, current);
        }
        case MATERIAL_SMOKE: {
            updated = update_smoke(pos, current);
        }
        case MATERIAL_WATER: {
            updated = update_water(pos, current);
        }
        case MATERIAL_DEBRIS: {
            updated = update_debris(pos, current);
        }
        default: {
            // No simulation for other materials
            updated = current;
        }
    }
    
    // Special rule: Fire creates smoke
    if (current.material == MATERIAL_FIRE && current.density < 50u) {
        // Dying fire produces smoke above
        let above_pos = pos + vec3<i32>(0, 0, 1);
        let above = read_voxel(above_pos);
        if (is_empty(above)) {
            var smoke: VoxelData;
            smoke.material = MATERIAL_SMOKE;
            smoke.density = 150u;
            smoke.temperature = 100u;
            smoke.flags = FLAG_TEMPORARY;
            write_voxel(above_pos, smoke);
        }
    }
    
    // Rising behavior (fire and smoke swap with air above)
    if ((current.material == MATERIAL_FIRE || current.material == MATERIAL_SMOKE)) {
        let above_pos = pos + vec3<i32>(0, 0, 1);
        let above = read_voxel(above_pos);
        
        if (is_empty(above) && random(params.random_seed + 4u, pos) > 0.5) {
            // Swap: current voxel moves up
            write_voxel(above_pos, updated);
            
            // Leave air in current position
            var air: VoxelData;
            air.material = MATERIAL_AIR;
            air.density = 0u;
            air.temperature = 0u;
            air.flags = 0u;
            write_voxel(pos, air);
            return;
        }
    }
    
    // Falling behavior (water and debris swap with air below)
    if (current.material == MATERIAL_WATER || current.material == MATERIAL_DEBRIS) {
        let below_pos = pos + vec3<i32>(0, 0, -1);
        let below = read_voxel(below_pos);
        
        if (is_empty(below)) {
            // Swap: current voxel moves down
            write_voxel(below_pos, updated);
            
            // Leave air in current position
            var air: VoxelData;
            air.material = MATERIAL_AIR;
            air.density = 0u;
            air.temperature = 0u;
            air.flags = 0u;
            write_voxel(pos, air);
            return;
        }
    }
    
    // Fire spreading (ignite flammable neighbors)
    if (current.material == MATERIAL_FIRE && current.temperature > 150u) {
        let offsets = array<vec3<i32>, 6>(
            vec3<i32>(1, 0, 0),
            vec3<i32>(-1, 0, 0),
            vec3<i32>(0, 1, 0),
            vec3<i32>(0, -1, 0),
            vec3<i32>(0, 0, 1),
            vec3<i32>(0, 0, -1)
        );
        
        let rand = random(params.random_seed + 5u, pos);
        let idx = u32(rand * 6.0);
        let neighbor_pos = pos + offsets[idx];
        let neighbor = read_voxel(neighbor_pos);
        
        if (is_flammable(neighbor.material) && rand > 0.8) {
            // Ignite the neighbor
            var fire: VoxelData;
            fire.material = MATERIAL_FIRE;
            fire.density = 255u;
            fire.temperature = 255u;
            fire.flags = FLAG_EMITS_LIGHT | FLAG_TEMPORARY;
            write_voxel(neighbor_pos, fire);
        }
    }
    
    // Write the updated voxel
    write_voxel(pos, updated);
}
