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
    light_color: vec4<f32>,         // Light's color and intensity
    ambient_light_color: vec4<f32>, // Ambient light color and intensity
    light_radius: f32,              // Light radius
    light_falloff: f32,             // Light falloff exponent
    light_height: f32,              // Light Z position
    normal_strength: f32,           // Normal map strength multiplier
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

    // Get the world_from_local transform using Bevy's helper function
    var world_from_local = mesh_functions::get_world_from_local(vertex_input.instance_index);

    // Transform position using Bevy's helper functions
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
    let diffuse_color = textureSample(diffuse_texture, diffuse_sampler, in.uv);

    // Skip fully transparent pixels
    if (diffuse_color.a < 0.01) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    // Calculate circular light attenuation (creates perfect circle boundary)
    let light_pos_2d = material_uniforms.light_pos_world_2d;
    let surface_pos_2d = in.world_position.xy;
    let distance_to_light = distance(surface_pos_2d, light_pos_2d);
    let distance_ratio = clamp(distance_to_light / material_uniforms.light_radius, 0.0, 1.0);
    let circle_attenuation = max(0.0, 1.0 - pow(distance_ratio, material_uniforms.light_falloff));

    // Calculate surface lighting with normal map
    var surface_light_factor = 1.0; // Default to full lighting within circle
    
    if (material_uniforms.normal_strength > 0.0) {
        // Sample and process normal map
        var normal_sample = textureSample(normal_texture, normal_sampler, in.uv).rgb;
        normal_sample = (normal_sample * 2.0 - 1.0); // Convert to [-1,1] range
        
        // Apply normal strength
        normal_sample.x *= material_uniforms.normal_strength;
        normal_sample.y *= material_uniforms.normal_strength;
        
        // Build TBN matrix for 2D sprite
        let tangent = vec3<f32>(1.0, 0.0, 0.0);
        let bitangent = vec3<f32>(0.0, 1.0, 0.0);
        let normal_base = vec3<f32>(0.0, 0.0, 1.0);
        
        // Transform normal to world space
        let world_normal = normalize(
            tangent * normal_sample.x + 
            bitangent * normal_sample.y + 
            normal_base * normal_sample.z
        );
        
        // Calculate light direction from surface to light
        let light_direction = normalize(vec3<f32>(
            light_pos_2d.x - surface_pos_2d.x,
            light_pos_2d.y - surface_pos_2d.y,
            material_uniforms.light_height
        ));
        
        // Calculate how much this surface faces the light
        let normal_dot_light = dot(world_normal, light_direction);
        
        // Dynamic lighting range based on normal strength
        // Higher normal strength = more dramatic lighting contrast
        let max_contrast = material_uniforms.normal_strength;
        let min_light = max(0.1, 1.0 - max_contrast);  // Gets darker as strength increases
        let max_light = 1.0;  // Always full brightness for surfaces facing light
        
        // Map normal dot product from [-1,1] to [min_light, max_light]
        let normal_factor = (normal_dot_light + 1.0) * 0.5; // Convert to [0,1]
        surface_light_factor = mix(min_light, max_light, normal_factor);
    }
    
    // Combine circle attenuation with surface lighting
    let final_light_intensity = circle_attenuation * surface_light_factor;
    
    // Apply light color and intensity
    let light_contribution = material_uniforms.light_color.rgb * 
                            material_uniforms.light_color.a * final_light_intensity;
    
    // Add ambient lighting
    let ambient_contribution = material_uniforms.ambient_light_color.rgb * 
                              material_uniforms.ambient_light_color.a;
    
    // Final color
    let final_color = diffuse_color.rgb * (light_contribution + ambient_contribution);
    return vec4<f32>(final_color, diffuse_color.a);
}
