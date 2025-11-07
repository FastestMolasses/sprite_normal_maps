// Compute shader for chunk-based world rendering
// Performs raymarching through voxel chunks and outputs position/normal/diffuse maps
// This supports the isometric camera view for Diablo-style gameplay

@group(0) @binding(0) var chunk_texture: texture_3d<u32>;
@group(0) @binding(1) var chunk_sampler: sampler;

@group(0) @binding(2) var position_output: texture_storage_2d<rgba16float, write>;
@group(0) @binding(3) var normal_output: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(4) var diffuse_output: texture_storage_2d<rgba8unorm, write>;

struct ChunkRenderParams {
    camera_position: vec3<f32>,     // Camera position in world space
    camera_forward: vec3<f32>,      // Camera forward direction
    camera_right: vec3<f32>,        // Camera right direction
    camera_up: vec3<f32>,           // Camera up direction
    chunk_world_offset: vec3<f32>,  // World position of this chunk
    fov: f32,                       // Field of view
    aspect_ratio: f32,              // Aspect ratio (width/height)
    chunk_size: f32,                // Size of the chunk (typically 64)
    output_width: u32,              // Output texture width
    output_height: u32,             // Output texture height
}

@group(0) @binding(5) var<uniform> params: ChunkRenderParams;

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

// Sample voxel from chunk texture with bounds checking
fn sample_voxel(pos: vec3<f32>) -> VoxelData {
    let chunk_size = params.chunk_size;
    
    // Check if position is within chunk bounds
    if (pos.x < 0.0 || pos.x >= chunk_size ||
        pos.y < 0.0 || pos.y >= chunk_size ||
        pos.z < 0.0 || pos.z >= chunk_size) {
        var empty: VoxelData;
        empty.material = MATERIAL_AIR;
        empty.density = 0u;
        empty.temperature = 0u;
        empty.flags = 0u;
        return empty;
    }
    
    // Convert to texture coordinates (0-1 range)
    let uv = pos / chunk_size;
    let coords = vec3<i32>(i32(pos.x), i32(pos.y), i32(pos.z));
    
    // Sample the texture
    let packed = textureLoad(chunk_texture, coords, 0).r;
    return unpack_voxel(packed);
}

// Calculate normal using central differences
fn calculate_normal(pos: vec3<f32>) -> vec3<f32> {
    let step = 1.0;
    
    let voxel_x_pos = sample_voxel(pos + vec3<f32>(step, 0.0, 0.0));
    let voxel_x_neg = sample_voxel(pos - vec3<f32>(step, 0.0, 0.0));
    let voxel_y_pos = sample_voxel(pos + vec3<f32>(0.0, step, 0.0));
    let voxel_y_neg = sample_voxel(pos - vec3<f32>(0.0, step, 0.0));
    let voxel_z_pos = sample_voxel(pos + vec3<f32>(0.0, 0.0, step));
    let voxel_z_neg = sample_voxel(pos - vec3<f32>(0.0, 0.0, step));
    
    // Use density to calculate gradient
    let dx = f32(voxel_x_pos.density) - f32(voxel_x_neg.density);
    let dy = f32(voxel_y_pos.density) - f32(voxel_y_neg.density);
    let dz = f32(voxel_z_pos.density) - f32(voxel_z_neg.density);
    
    let normal = vec3<f32>(-dx, -dy, -dz);
    let len = length(normal);
    
    if (len > 0.01) {
        return normalize(normal);
    } else {
        return vec3<f32>(0.0, 1.0, 0.0); // Default up
    }
}

// Get color for a material type
fn get_material_color(material: u32) -> vec4<f32> {
    switch (material) {
        case MATERIAL_ROCK: {
            return vec4<f32>(0.5, 0.5, 0.5, 1.0);
        }
        case MATERIAL_DIRT: {
            return vec4<f32>(0.4, 0.3, 0.2, 1.0);
        }
        case MATERIAL_WOOD: {
            return vec4<f32>(0.6, 0.4, 0.2, 1.0);
        }
        case MATERIAL_METAL: {
            return vec4<f32>(0.7, 0.7, 0.8, 1.0);
        }
        case MATERIAL_FIRE: {
            return vec4<f32>(1.0, 0.5, 0.1, 1.0);
        }
        case MATERIAL_SMOKE: {
            return vec4<f32>(0.2, 0.2, 0.2, 0.5);
        }
        case MATERIAL_WATER: {
            return vec4<f32>(0.2, 0.4, 0.8, 0.6);
        }
        case MATERIAL_DEBRIS: {
            return vec4<f32>(0.6, 0.5, 0.4, 1.0);
        }
        default: {
            return vec4<f32>(0.0, 0.0, 0.0, 0.0); // Air is transparent
        }
    }
}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let pixel_coords = global_id.xy;
    
    // Check bounds
    if (pixel_coords.x >= params.output_width || pixel_coords.y >= params.output_height) {
        return;
    }
    
    // Calculate normalized device coordinates (-1 to 1)
    let ndc_x = (f32(pixel_coords.x) / f32(params.output_width)) * 2.0 - 1.0;
    let ndc_y = (f32(pixel_coords.y) / f32(params.output_height)) * 2.0 - 1.0;
    
    // Apply aspect ratio and FOV
    let ray_x = ndc_x * params.aspect_ratio * tan(params.fov * 0.5);
    let ray_y = -ndc_y * tan(params.fov * 0.5); // Flip Y for screen space
    
    // Construct ray direction in world space
    let ray_dir = normalize(
        params.camera_forward + 
        ray_x * params.camera_right + 
        ray_y * params.camera_up
    );
    
    // Ray origin
    let ray_origin = params.camera_position;
    
    // Raymarch through the chunk
    let max_distance = params.chunk_size * 2.0;
    let step_size = 0.5; // Half-voxel steps for better quality
    var t = 0.0;
    
    var hit = false;
    var hit_pos_world = vec3<f32>(0.0);
    var hit_pos_local = vec3<f32>(0.0);
    var hit_voxel: VoxelData;
    
    // March until we hit something or exceed max distance
    while (t < max_distance && !hit) {
        let pos_world = ray_origin + ray_dir * t;
        let pos_local = pos_world - params.chunk_world_offset;
        
        // Check if we're inside the chunk bounds
        if (pos_local.x >= 0.0 && pos_local.x < params.chunk_size &&
            pos_local.y >= 0.0 && pos_local.y < params.chunk_size &&
            pos_local.z >= 0.0 && pos_local.z < params.chunk_size) {
            
            let voxel = sample_voxel(pos_local);
            
            // Check if this voxel is solid (not air)
            if (voxel.material != MATERIAL_AIR && voxel.density > 0u) {
                hit = true;
                hit_pos_world = pos_world;
                hit_pos_local = pos_local;
                hit_voxel = voxel;
                break;
            }
        }
        
        t += step_size;
    }
    
    // Write outputs
    if (hit) {
        // Position map (world space position encoded as color)
        let pos_normalized = hit_pos_world / 1000.0; // Scale to 0-1 range
        textureStore(position_output, pixel_coords, vec4<f32>(pos_normalized, 1.0));
        
        // Normal map
        let normal = calculate_normal(hit_pos_local);
        let normal_encoded = (normal + 1.0) * 0.5; // Encode -1..1 to 0..1
        textureStore(normal_output, pixel_coords, vec4<f32>(normal_encoded, 1.0));
        
        // Diffuse map (material color)
        let color = get_material_color(hit_voxel.material);
        textureStore(diffuse_output, pixel_coords, color);
    } else {
        // No hit - write transparent/empty
        textureStore(position_output, pixel_coords, vec4<f32>(0.0, 0.0, 0.0, 0.0));
        textureStore(normal_output, pixel_coords, vec4<f32>(0.5, 0.5, 1.0, 0.0));
        textureStore(diffuse_output, pixel_coords, vec4<f32>(0.0, 0.0, 0.0, 0.0));
    }
}
