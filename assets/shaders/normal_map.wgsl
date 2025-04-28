#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(1) @binding(0)
var<uniform> light_position: vec3<f32>;
@group(1) @binding(1)
var<uniform> light_color: vec4<f32>;
@group(1) @binding(2)
var<uniform> ambient_color: vec4<f32>;
@group(1) @binding(3)
var diffuse_texture: texture_2d<f32>;
@group(1) @binding(4)
var diffuse_sampler: sampler;
@group(1) @binding(5)
var normal_texture: texture_2d<f32>;
@group(1) @binding(6)
var normal_sampler: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Sample the diffuse and normal textures
    let diffuse_color = textureSample(diffuse_texture, diffuse_sampler, in.uv);
    var normal = textureSample(normal_texture, normal_sampler, in.uv).rgb;
    
    // Convert normal from [0,1] to [-1,1] range
    normal = normal * 2.0 - 1.0;
    
    // Calculate the position of the fragment in world space
    let frag_position = vec3<f32>(in.world_position.xy, 0.0);
    
    // Calculate the light direction (from fragment to light)
    let light_dir = normalize(light_position - frag_position);
    
    // Calculate the light distance and attenuation
    let distance = length(light_position - frag_position);
    let attenuation = 1.0 / (1.0 + 0.1 * distance + 0.01 * distance * distance);
    
    // Calculate the diffuse lighting intensity using the normal map
    let diffuse_intensity = max(dot(normal, light_dir), 0.0);
    
    // Calculate the final lighting color
    let lighting = ambient_color.rgb + light_color.rgb * diffuse_intensity * attenuation;
    
    // Apply lighting to the diffuse color
    let final_color = diffuse_color.rgb * lighting;
    
    return vec4<f32>(final_color, diffuse_color.a);
}
