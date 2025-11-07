use bevy::prelude::*;
use crate::world::{WorldChunk, MaterialType, CHUNK_SIZE};

/// Resource to cache the isometric cube mesh
#[derive(Resource)]
struct IsometricMeshCache {
    cube_mesh: Handle<Mesh>,
}

/// Plugin for rendering voxels in isometric projection
pub struct IsometricVoxelRendererPlugin;

impl Plugin for IsometricVoxelRendererPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_mesh_cache)
           .add_systems(Update, render_voxels_isometric);
    }
}

/// Setup mesh cache on startup
fn setup_mesh_cache(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let cube_mesh = meshes.add(create_isometric_cube_mesh());
    commands.insert_resource(IsometricMeshCache { cube_mesh });
}

/// Marker component for isometric voxel sprites
#[derive(Component)]
pub struct IsometricVoxelSprite {
    pub chunk_entity: Entity,
    pub voxel_pos: UVec3,
}

/// Render voxels in isometric projection
/// Uses diamond/cube sprites with depth sorting
fn render_voxels_isometric(
    mut commands: Commands,
    chunks: Query<(Entity, &WorldChunk), Changed<WorldChunk>>,
    existing_sprites: Query<Entity, With<IsometricVoxelSprite>>,
    mesh_cache: Res<IsometricMeshCache>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Only rebuild when chunks change
    if chunks.is_empty() {
        return;
    }
    
    // Clear old sprites
    for entity in existing_sprites.iter() {
        commands.entity(entity).despawn();
    }
    
    // Render all chunks
    for (chunk_entity, chunk) in chunks.iter() {
        render_chunk_isometric(
            &mut commands,
            chunk_entity,
            chunk,
            &mesh_cache.cube_mesh,
            &mut materials,
        );
    }
}

/// Render a single chunk in isometric view
fn render_chunk_isometric(
    commands: &mut Commands,
    chunk_entity: Entity,
    chunk: &WorldChunk,
    cube_mesh: &Handle<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) {
    let chunk_world_pos = chunk.chunk_position.as_vec3() * CHUNK_SIZE as f32;
    
    // Dynamic sample rate: render more detail for chunks with dynamic elements
    let sample_rate = if chunk.has_dynamic_elements {
        1  // Render every voxel for active chunks
    } else {
        4  // Skip most voxels for static chunks
    };
    
    for z in (0..CHUNK_SIZE).step_by(sample_rate) {
        for y in (0..CHUNK_SIZE).step_by(sample_rate) {
            for x in (0..CHUNK_SIZE).step_by(sample_rate) {
                if let Some(voxel) = chunk.get_voxel(x, y, z) {
                    let material = voxel.material();
                    
                    // Only render visible materials
                    if material == MaterialType::Air {
                        continue;
                    }
                    
                    let world_pos = chunk_world_pos + Vec3::new(x as f32, y as f32, z as f32);
                    
                    // Get base color with height-based shading
                    let color = get_material_color_with_shading(material, world_pos.y);
                    
                    // Convert 3D position to isometric 2D coordinates
                    let iso_pos = world_to_isometric(world_pos);
                    
                    // Spawn isometric sprite
                    commands.spawn((
                        Mesh2d(cube_mesh.clone()),
                        MeshMaterial2d(materials.add(ColorMaterial { color, ..default() })),
                        Transform::from_translation(Vec3::new(iso_pos.x, iso_pos.y, iso_pos.z)),
                        IsometricVoxelSprite {
                            chunk_entity,
                            voxel_pos: UVec3::new(x, y, z),
                        },
                    ));
                }
            }
        }
    }
}

/// Convert 3D world position to 2D isometric screen position
/// Uses classic isometric projection (Diablo/SimCity style)
fn world_to_isometric(world_pos: Vec3) -> Vec3 {
    // Isometric projection: 
    // Looking from above-right, so positive X goes right, positive Z goes up-left
    // This matches a 2:1 pixel ratio isometric view
    
    let iso_x = world_pos.x - world_pos.z;
    let iso_y = (world_pos.x + world_pos.z) * 0.5 - world_pos.y;
    
    // Z coordinate for depth sorting (further back = lower z)
    let depth = world_pos.y - world_pos.x * 0.01 - world_pos.z * 0.01;
    
    Vec3::new(iso_x, iso_y, depth)
}

/// Create a small diamond/cube shape for isometric voxels
fn create_isometric_cube_mesh() -> Mesh {
    // Create an isometric diamond/rhombus shape
    // This represents a cube viewed from 45° angle
    
    // Diamond points (rhombus for isometric view)
    let size = 4.0;
    let half = size / 2.0;
    
    // Isometric diamond vertices
    let vertices = vec![
        [0.0, half, 0.0],      // Top
        [half, 0.0, 0.0],      // Right
        [0.0, -half, 0.0],     // Bottom
        [-half, 0.0, 0.0],     // Left
    ];
    
    // Triangle indices for the diamond
    let indices = vec![
        0, 1, 2,  // Top-right-bottom
        0, 2, 3,  // Top-bottom-left
    ];
    
    // UVs for texturing (if needed later)
    let uvs = vec![
        [0.5, 1.0],
        [1.0, 0.5],
        [0.5, 0.0],
        [0.0, 0.5],
    ];
    
    // Normals (all facing camera)
    let normals = vec![
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
    ];
    
    Mesh::new(
        bevy::render::render_resource::PrimitiveTopology::TriangleList,
        bevy::render::render_asset::RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(bevy::render::mesh::Indices::U32(indices))
}

/// Get color for material type
fn get_material_color(material: MaterialType) -> Color {
    match material {
        MaterialType::Air => Color::NONE,
        MaterialType::Rock => Color::srgb(0.5, 0.5, 0.5),
        MaterialType::Dirt => Color::srgb(0.6, 0.4, 0.2),
        MaterialType::Wood => Color::srgb(0.6, 0.4, 0.1),
        MaterialType::Metal => Color::srgb(0.7, 0.7, 0.8),
        MaterialType::Fire => Color::srgb(1.0, 0.5, 0.0),
        MaterialType::Smoke => Color::srgba(0.3, 0.3, 0.3, 0.6),
        MaterialType::Water => Color::srgb(0.2, 0.4, 0.8),
        MaterialType::Debris => Color::srgb(0.6, 0.5, 0.4),
    }
}

/// Get color with height-based shading for depth perception
fn get_material_color_with_shading(material: MaterialType, height: f32) -> Color {
    let mut base_color = get_material_color(material);
    
    // Skip shading for emissive/transparent materials
    match material {
        MaterialType::Fire | MaterialType::Smoke | MaterialType::Water => return base_color,
        _ => {}
    }
    
    // Add subtle height-based shading (higher = slightly brighter)
    let shade_factor = 0.8 + (height / 64.0) * 0.4; // 0.8 to 1.2 range
    let shade_factor = shade_factor.clamp(0.7, 1.3);
    
    // Apply shading to RGB channels
    if let Color::Srgba(srgba) = &mut base_color {
        srgba.red *= shade_factor;
        srgba.green *= shade_factor;
        srgba.blue *= shade_factor;
    }
    
    base_color
}
