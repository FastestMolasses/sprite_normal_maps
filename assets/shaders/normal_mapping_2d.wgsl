#import bevy_sprite::{
    mesh2d_types::{Mesh2d},
    mesh2d_view_types,
    mesh2d_view_bindings::{view, globals},
    mesh2d_functions as mesh_functions,
}
#import bevy_sprite::mesh2d_bindings::{mesh}

@group(2) @binding(0) var diffuse_texture: texture_2d<f32>;
@group(2) @binding(1) var diffuse_sampler: sampler;
@group(2) @binding(2) var normal_texture: texture_2d<f32>;
@group(2) @binding(3) var normal_sampler: sampler;

@group(2) @binding(4) var<uniform> material_uniforms: MaterialUniforms;
struct MaterialUniforms {
    light_pos_world_2d: vec2<f32>,  // Light's xy position in world space
    light_color: vec4<f32>,         // Light's color (rgb) and intensity (a)
    ambient_light_color: vec4<f32>, // Ambient light (rgb) and intensity (a)
}

struct VertexInput {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) world_position: vec3<f32>,
    @location(2) world_normal: vec3<f32>,
}

@vertex
fn vertex(vertex_input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    var world_from_local = mesh_functions::get_world_from_local(vertex_input.instance_index);
    out.world_position = mesh_functions::mesh2d_position_local_to_world(
        world_from_local,
        vec4<f32>(vertex_input.position, 1.0)
    ).xyz;

    out.clip_position = mesh_functions::mesh2d_position_world_to_clip(
        vec4<f32>(out.world_position, 1.0)
    );

    // Transform normal
    out.world_normal = mesh_functions::mesh2d_normal_local_to_world(
        vertex_input.normal, 
        vertex_input.instance_index
    );

    out.uv = vertex_input.uv;

    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let diffuse_color_sample = textureSample(diffuse_texture, diffuse_sampler, in.uv);

    // Skip fully transparent pixels
    if (diffuse_color_sample.a < 0.01) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    // Calculate distance to light (in 2D space)
    let light_position_2d = material_uniforms.light_pos_world_2d;
    let surface_position_2d = in.world_position.xy;
    let distance_to_light = distance(surface_position_2d, light_position_2d);

    // Point light parameters
    let light_radius = 300.0;  // Radius of the light circle
    let light_falloff = 1.0;   // Controls how sharp the edge of the light is

    // Calculate pure distance-based attenuation
    let distance_ratio = min(distance_to_light / light_radius, 1.0);
    let light_attenuation = 1.0 - pow(distance_ratio, light_falloff);

    // Apply the light with the base diffuse color
    let light_rgb = material_uniforms.light_color.rgb * material_uniforms.light_color.a;
    var base_lit_color = diffuse_color_sample.rgb * light_rgb * light_attenuation;

    // Sample normal map
    var surface_normal_tangent = textureSample(normal_texture, normal_sampler, in.uv).rgb;
    surface_normal_tangent = normalize(surface_normal_tangent * 2.0 - 1.0);

    // Simple TBN matrix
    let T = vec3<f32>(1.0, 0.0, 0.0); 
    let B = vec3<f32>(0.0, 1.0, 0.0);
    let N = in.world_normal;
    let tbn = mat3x3<f32>(T, B, N);

    // World space normal
    let final_surface_normal = normalize(tbn * surface_normal_tangent);
    let light_direction = normalize(vec3<f32>(0.0, 0.0, 1.0));  // Light from directly above
    let normal_factor = 0.95 + 0.1 * max(dot(final_surface_normal, light_direction), 0.0);
    var lit_color = base_lit_color * normal_factor;

    let ambient_rgb = material_uniforms.ambient_light_color.rgb * material_uniforms.ambient_light_color.a;
    lit_color += diffuse_color_sample.rgb * ambient_rgb;

    return vec4<f32>(lit_color, diffuse_color_sample.a);
}
