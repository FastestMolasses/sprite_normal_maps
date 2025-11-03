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
    
    // Check bounds
    if (pos.x < 0.0 || pos.x >= vol_size ||
        pos.y < 0.0 || pos.y >= vol_size ||
        pos.z < 0.0 || pos.z >= vol_size) {
        return 0.0;
    }
    
    // Normalize to 0-1 range for texture sampling
    let uv = pos / vol_size;
    return textureSampleLevel(volume_texture, volume_sampler, uv, 0.0).r;
}

// Calculate gradient (normal) at a position using central differences
fn calculate_gradient(pos: vec3<f32>) -> vec3<f32> {
    let step = 1.0;
    
    let dx = sample_volume(pos + vec3<f32>(step, 0.0, 0.0)) - 
             sample_volume(pos - vec3<f32>(step, 0.0, 0.0));
    let dy = sample_volume(pos + vec3<f32>(0.0, step, 0.0)) - 
             sample_volume(pos - vec3<f32>(0.0, step, 0.0));
    let dz = sample_volume(pos + vec3<f32>(0.0, 0.0, step)) - 
             sample_volume(pos - vec3<f32>(0.0, 0.0, step));
    
    let normal = vec3<f32>(-dx, -dy, -dz);
    let len = length(normal);
    
    if (len > 0.0001) {
        return normal / len;
    } else {
        return vec3<f32>(0.0, 1.0, 0.0);
    }
}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let pixel_coords = global_id.xy;
    
    // Check bounds
    if (pixel_coords.x >= params.output_width || pixel_coords.y >= params.output_height) {
        return;
    }
    
    let vol_size = params.volume_size;
    let center = vol_size / 2.0;
    
    // Map pixel to screen space (centered)
    let screen_x = (f32(pixel_coords.x) / f32(params.output_width) - 0.5) * vol_size;
    let screen_y = (f32(pixel_coords.y) / f32(params.output_height) - 0.5) * vol_size;
    
    // Ray in screen space
    let ray_start = vec3<f32>(screen_x, screen_y, -vol_size);
    let ray_dir = vec3<f32>(0.0, 0.0, 1.0);
    
    // Create inverse rotation matrix (transpose for orthogonal matrices)
    let inv_rotation = transpose(params.rotation_matrix);
    
    // Raymarch through the volume
    let max_steps = u32(vol_size * 1.5);
    let step_size = 0.75;
    
    var hit = false;
    var hit_pos = vec3<f32>(0.0);
    
    for (var step = 0u; step < max_steps; step = step + 1u) {
        let t = f32(step) * step_size;
        let ray_pos = ray_start + ray_dir * t;
        
        // Rotate ray position to volume space
        let rotated_pos = rotate_point(ray_pos, inv_rotation) + vec3<f32>(center);
        
        // Sample the volume
        let density = sample_volume(rotated_pos);
        
        if (density > params.threshold) {
            hit = true;
            hit_pos = rotated_pos;
            break;
        }
    }
    
    if (hit) {
        // Position map: encode world position as RGB (normalized to 0-1)
        let pos_color = vec4<f32>(hit_pos / vol_size, 1.0);
        textureStore(position_output, pixel_coords, pos_color);
        
        // Normal map: calculate gradient in volume space, then rotate to world space
        let normal_volume = calculate_gradient(hit_pos);
        let normal_world = rotate_point(normal_volume, params.rotation_matrix);
        // Map from -1..1 to 0..1
        let normal_color = vec4<f32>(normal_world * 0.5 + 0.5, 1.0);
        textureStore(normal_output, pixel_coords, normal_color);
        
        // Diffuse map: height-based color variation
        let variation = (hit_pos.y / vol_size) * 0.2;
        let base_color = 0.5 + variation;
        let diffuse_color = vec4<f32>(
            base_color * 0.7,  // R
            base_color * 0.66, // G
            base_color * 0.62, // B
            1.0                // A
        );
        textureStore(diffuse_output, pixel_coords, diffuse_color);
    } else {
        // No hit: transparent
        textureStore(position_output, pixel_coords, vec4<f32>(0.0, 0.0, 0.0, 0.0));
        textureStore(normal_output, pixel_coords, vec4<f32>(0.0, 0.0, 0.0, 0.0));
        textureStore(diffuse_output, pixel_coords, vec4<f32>(0.0, 0.0, 0.0, 0.0));
    }
}
