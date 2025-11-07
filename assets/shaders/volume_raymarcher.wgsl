// Compute shader for real-time volume rendering
// Performs raymarching through a 3D volume texture and outputs position/normal/diffuse maps

@group(0) @binding(0) var volume_texture: texture_3d<f32>;
@group(0) @binding(1) var volume_sampler: sampler;

@group(0) @binding(2) var position_output: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(3) var normal_output: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(4) var diffuse_output: texture_storage_2d<rgba8unorm, write>;

struct VolumeParams {
    rotation_matrix: mat3x3<f32>, // Rotation transformation
    volume_size: f32,              // Size of the volume
    threshold: f32,                // Density threshold for hit detection
    output_width: u32,             // Output texture width
    output_height: u32,            // Output texture height
}

@group(0) @binding(5) var<uniform> params: VolumeParams;

// Rotate a point using the rotation matrix
fn rotate_point(point: vec3<f32>, matrix: mat3x3<f32>) -> vec3<f32> {
    return matrix * point;
}

// Sample the volume with bounds checking
fn sample_volume(pos: vec3<f32>) -> f32 {
    let vol_size = params.volume_size;
    
    // Branchless bounds check using step functions
    let in_bounds = step(0.0, pos.x) * step(pos.x, vol_size - 0.001) *
                    step(0.0, pos.y) * step(pos.y, vol_size - 0.001) *
                    step(0.0, pos.z) * step(pos.z, vol_size - 0.001);
    
    // Normalize to 0-1 range for texture sampling
    let uv = pos / vol_size;
    return textureSampleLevel(volume_texture, volume_sampler, uv, 0.0).r * in_bounds;
}

// Calculate gradient (normal) at a position using central differences
fn calculate_gradient(pos: vec3<f32>) -> vec3<f32> {
    let step = 1.0;
    
    // Use vec3 offsets for cleaner code
    let offset_x = vec3<f32>(step, 0.0, 0.0);
    let offset_y = vec3<f32>(0.0, step, 0.0);
    let offset_z = vec3<f32>(0.0, 0.0, step);
    
    let dx = sample_volume(pos + offset_x) - sample_volume(pos - offset_x);
    let dy = sample_volume(pos + offset_y) - sample_volume(pos - offset_y);
    let dz = sample_volume(pos + offset_z) - sample_volume(pos - offset_z);
    
    let normal = vec3<f32>(-dx, -dy, -dz);
    let len = length(normal);
    
    // Branchless normalization using select
    let normalized = normal / max(len, 0.0001);
    return select(vec3<f32>(0.0, 1.0, 0.0), normalized, len > 0.0001);
}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let pixel_coords = global_id.xy;
    
    // Check bounds
    if (pixel_coords.x >= params.output_width || pixel_coords.y >= params.output_height) {
        return;
    }
    
    let vol_size = params.volume_size;
    let center = vol_size * 0.5;
    let inv_vol_size = 1.0 / vol_size;
    
    // Precompute reciprocals for division optimization
    let inv_output_width = 1.0 / f32(params.output_width);
    let inv_output_height = 1.0 / f32(params.output_height);
    
    // Map pixel to screen space (centered)
    let screen_x = (f32(pixel_coords.x) * inv_output_width - 0.5) * vol_size;
    let screen_y = (f32(pixel_coords.y) * inv_output_height - 0.5) * vol_size;
    
    // Ray in screen space
    let ray_start = vec3<f32>(screen_x, screen_y, -vol_size);
    let ray_dir = vec3<f32>(0.0, 0.0, 1.0);
    
    // Create inverse rotation matrix (transpose for orthogonal matrices)
    let inv_rotation = transpose(params.rotation_matrix);
    
    // Precompute center offset
    let center_offset = vec3<f32>(center);
    
    // Raymarch through the volume
    let max_steps = u32(vol_size * 1.5);
    let step_size = 0.75;
    
    var hit = false;
    var hit_pos = vec3<f32>(0.0);
    
    for (var step = 0u; step < max_steps; step = step + 1u) {
        let t = f32(step) * step_size;
        let ray_pos = ray_start + ray_dir * t;
        
        // Rotate ray position to volume space
        let rotated_pos = rotate_point(ray_pos, inv_rotation) + center_offset;
        
        // Sample the volume
        let density = sample_volume(rotated_pos);
        
        if (density > params.threshold) {
            hit = true;
            hit_pos = rotated_pos;
            break;
        }
    }
    
    // Multiply by hit flag (0.0 or 1.0)
    let hit_f = f32(hit);
    
    // Position map: encode world position as RGB (normalized to 0-1)
    let pos_color = vec4<f32>(hit_pos * inv_vol_size, 1.0) * hit_f;
    textureStore(position_output, pixel_coords, pos_color);
    
    // Normal map: calculate gradient in volume space, then rotate to world space
    // Only calculate if we hit (avoid expensive gradient calculation when hit = false)
    let normal_volume = select(vec3<f32>(0.0), calculate_gradient(hit_pos), hit);
    let normal_world = rotate_point(normal_volume, params.rotation_matrix);
    // Map from -1..1 to 0..1 using fma
    let normal_color = vec4<f32>(fma(normal_world, vec3<f32>(0.5), vec3<f32>(0.5)), 1.0) * hit_f;
    textureStore(normal_output, pixel_coords, normal_color);
    
    // Diffuse map: height-based color variation
    let variation = hit_pos.y * inv_vol_size * 0.2;
    let base_color = 0.5 + variation;
    let diffuse_color = vec4<f32>(
        base_color * 0.7,  // R
        base_color * 0.66, // G
        base_color * 0.62, // B
        1.0                // A
    ) * hit_f;
    textureStore(diffuse_output, pixel_coords, diffuse_color);
}
