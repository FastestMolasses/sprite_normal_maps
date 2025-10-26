#import bevy_sprite::{
    mesh2d_types::{Mesh2d},
    mesh2d_view_types,
    mesh2d_view_bindings::{view, globals},
    mesh2d_functions as mesh_functions,
}
#import bevy_sprite::mesh2d_bindings::{mesh}

@group(2) @binding(0) var diffuse_texture: texture_2d<f32>;
@group(2) @binding(1) var diffuse_sampler: sampler;
@group(2) @binding(2) var position_texture: texture_2d<f32>;
@group(2) @binding(3) var position_sampler: sampler;
@group(2) @binding(4) var normal_texture: texture_2d<f32>;
@group(2) @binding(5) var normal_sampler: sampler;

@group(2) @binding(6) var<uniform> material_uniforms: MaterialUniforms;
struct MaterialUniforms {
    light_pos_world_3d: vec3<f32>,  // XY = ground position, Z = virtual height in game world
    sprite_world_pos: vec2<f32>,    // Sprite's position on the ground (XY plane)
    light_color: vec4<f32>,         // Light's color and intensity
    ambient_light_color: vec4<f32>, // Ambient light color and intensity
    light_radius: f32,              // Light radius (in 3D space)
    light_falloff: f32,             // Light falloff exponent
    position_scale: f32,            // Scale factor to convert position map units to world units
    debug_mode: u32,                // 0=normal, 1=positions, 2=normals, 3=distance, 4=ground level, 5=3D coords
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
    // Sample diffuse color
    let diffuse_color = textureSample(diffuse_texture, diffuse_sampler, in.uv);

    // Skip fully transparent pixels
    if (diffuse_color.a < 0.01) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    // Sample the position map - this contains local 3D positions relative to the sprite origin
    let position_sample = textureSample(position_texture, position_sampler, in.uv);

    // Ignore black pixels (no geometry) FIRST - these represent empty space
    // Black pixels should always be transparent regardless of debug mode
    if (length(position_sample.rgb) < 0.01) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);  // Fully transparent
    }

    // Debug mode 1: Show raw position map
    if (material_uniforms.debug_mode == 1u) {
        return vec4<f32>(position_sample.rgb, diffuse_color.a);
    }

    // Debug mode 4: Show ground level only
    // Highlights pixels where blue channel is close to 0 (ground level)
    if (material_uniforms.debug_mode == 4u) {
        // Threshold for what we consider "ground level"
        // Adjust this value to be more/less strict
        let ground_threshold = 0.05;  // 5% of height range

        if (position_sample.b < ground_threshold) {
            // Ground level pixels show in bright green
            return vec4<f32>(0.0, 1.0, 0.0, diffuse_color.a);
        } else {
            // Other pixels show darkened diffuse
            return vec4<f32>(diffuse_color.rgb * 0.3, diffuse_color.a);
        }
    }

    // For normal rendering, apply only ambient lighting to non-geometry pixels
    // For normal rendering, apply only ambient lighting to non-geometry pixels
    if (length(position_sample.rgb) < 0.01) {
        let ambient_contribution = material_uniforms.ambient_light_color.rgb * 
                                  material_uniforms.ambient_light_color.a;
        let final_color = diffuse_color.rgb * ambient_contribution;
        return vec4<f32>(final_color, diffuse_color.a);
    }

    // Position map interpretation for isometric game:
    // - The sprite is placed at sprite_world_pos (XY on the ground plane)
    // - Position map R = X offset from sprite center (0.5 = center)
    // - Position map G = Y offset from sprite center (0.5 = center)
    // - Position map B = virtual Z height above ground (0 = ground, 1 = max height)
    //
    // Coordinate system:
    // - XY plane = ground (where player walks)
    // - Virtual Z = height in the game world (not Bevy's rendering Z)

    let normalized_pos = position_sample.rgb;

    // Convert position map to world 3D coordinates:
    // X and Y: offset from sprite's ground position
    // Z: absolute virtual height (0 = ground level)
    let pixel_world_pos_3d = vec3<f32>(
        material_uniforms.sprite_world_pos.x + (normalized_pos.r - 0.5) * material_uniforms.position_scale,
        material_uniforms.sprite_world_pos.y + (normalized_pos.g - 0.5) * material_uniforms.position_scale,
        normalized_pos.b * material_uniforms.position_scale  // Z is absolute height from ground
    );

    // Debug mode 5: Show computed 3D world positions
    if (material_uniforms.debug_mode == 5u) {
        // Normalize the world position for visualization
        // This helps see if coordinates are correct
        let vis_x = fract(pixel_world_pos_3d.x / 100.0);  // Cycle every 100 units
        let vis_y = fract(pixel_world_pos_3d.y / 100.0);
        let vis_z = clamp(pixel_world_pos_3d.z / 500.0, 0.0, 1.0);  // 0-500 range
        return vec4<f32>(vis_x, vis_y, vis_z, diffuse_color.a);
    }

    // Calculate the vector from the pixel to the light in full 3D space
    let light_vector = material_uniforms.light_pos_world_3d - pixel_world_pos_3d;
    let distance_to_light_3d = length(light_vector);

    // Debug mode 3: Show distance as grayscale
    if (material_uniforms.debug_mode == 3u) {
        let normalized_distance = clamp(distance_to_light_3d / material_uniforms.light_radius, 0.0, 1.0);
        // Show in color: red=too far, yellow=medium, green=close
        let close_factor = 1.0 - normalized_distance;
        return vec4<f32>(normalized_distance, close_factor, 0.0, diffuse_color.a);
    }

    // Calculate light attenuation based on 3D distance
    let distance_ratio = clamp(distance_to_light_3d / material_uniforms.light_radius, 0.0, 1.0);
    let light_attenuation = max(0.0, 1.0 - pow(distance_ratio, material_uniforms.light_falloff));

    // Hard cutoff: if outside radius, no light at all
    if (distance_to_light_3d > material_uniforms.light_radius) {
        // Only ambient lighting
        let ambient_contribution = material_uniforms.ambient_light_color.rgb * 
                                  material_uniforms.ambient_light_color.a;
        let final_color = diffuse_color.rgb * ambient_contribution;
        return vec4<f32>(final_color, diffuse_color.a);
    }

    // Sample and process normal map for surface-angle-based lighting
    var normal_sample = textureSample(normal_texture, normal_sampler, in.uv).rgb;

    // Debug mode 2: Show normal map
    if (material_uniforms.debug_mode == 2u) {
        return vec4<f32>(normal_sample, diffuse_color.a);
    }

    // Convert from [0,1] range to [-1,1] range
    normal_sample = (normal_sample * 2.0 - 1.0);

    // The normal map from Blender should already be in tangent space
    // For 2D sprites, we can use it directly or transform it to world space
    // Since we're working with pre-rendered sprites, the normals are baked in the correct space
    let surface_normal = normalize(normal_sample);

    // Calculate the light direction (normalized)
    let light_direction = normalize(light_vector);

    // Calculate diffuse lighting using Lambertian reflectance
    // This gives us how much the surface is facing the light
    let n_dot_l = max(dot(surface_normal, light_direction), 0.0);

    // Combine distance attenuation with surface angle
    let final_light_intensity = light_attenuation * n_dot_l;

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
