// Simple shader to display the voxel world render targets
// Takes position, normal, and diffuse textures and composites them

#import bevy_sprite::{
    mesh2d_view_bindings::view,
    mesh2d_bindings::mesh,
    mesh2d_functions,
}

@group(2) @binding(0) var position_texture: texture_2d<f32>;
@group(2) @binding(1) var position_sampler: sampler;
@group(2) @binding(2) var normal_texture: texture_2d<f32>;
@group(2) @binding(3) var normal_sampler: sampler;
@group(2) @binding(4) var diffuse_texture: texture_2d<f32>;
@group(2) @binding(5) var diffuse_sampler: sampler;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    
    let world_from_local = mesh2d_functions::get_world_from_local(vertex.instance_index);
    let world_position = mesh2d_functions::mesh2d_position_local_to_world(
        world_from_local,
        vec4<f32>(vertex.position, 1.0)
    );
    
    out.clip_position = mesh2d_functions::mesh2d_position_world_to_clip(world_position);
    out.uv = vertex.uv;
    
    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Sample all textures
    let position = textureSample(position_texture, position_sampler, in.uv);
    let normal = textureSample(normal_texture, normal_sampler, in.uv);
    let diffuse = textureSample(diffuse_texture, diffuse_sampler, in.uv);
    
    // If no geometry (alpha = 0), return transparent
    if (diffuse.a < 0.01) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
    
    // Simple lighting calculation
    // Decode normal from 0-1 range to -1..1
    let world_normal = normalize(normal.xyz * 2.0 - 1.0);
    
    // Simple directional light from above
    let light_dir = normalize(vec3<f32>(0.3, 0.7, 0.5));
    let light_intensity = max(dot(world_normal, light_dir), 0.0);
    
    // Ambient + diffuse lighting
    let ambient = 0.3;
    let lighting = ambient + (1.0 - ambient) * light_intensity;
    
    // Apply lighting to diffuse color
    let lit_color = diffuse.rgb * lighting;
    
    return vec4<f32>(lit_color, diffuse.a);
}
