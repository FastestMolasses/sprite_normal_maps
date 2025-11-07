use bevy::prelude::*;

// Module declarations
mod world;
mod simulation;
mod rendering;

// Re-exports
use world::*;
use simulation::*;
use rendering::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "3D-Simulated Isometric World".to_string(),
                resolution: (1920.0, 1080.0).into(),
                ..default()
            }),
            ..default()
        }))
        // World management
        .init_resource::<ChunkManager>()
        .init_resource::<SpatialIndex>()
        .init_resource::<SimulationSettings>()
        // Rendering systems
        .add_plugins(IsometricVoxelRendererPlugin)
        .add_plugins(GpuRendererPlugin)
        .add_plugins(VoxelWorldMaterialPlugin)
        // Simulation systems
        .add_plugins(ComputeSimulationPlugin)
        .add_plugins(CpuSimulationPlugin) // CPU sim (GPU requires complex render world setup)
        // Setup and update systems
        .add_systems(Startup, (setup_test_world, setup_camera))
        .add_systems(Update, (
            manage_chunk_loading,
            update_chunk_textures,
            update_auto_spawners,
            spawn_test_elements,
            debug_info,
        ))
        .run();
}

/// Initial test world setup - creates a few chunks with simple geometry
fn setup_test_world(
    mut commands: Commands,
    mut chunk_manager: ResMut<ChunkManager>,
    mut images: ResMut<Assets<Image>>,
) {
    info!("Setting up test world...");

    // Initialize chunk manager with reasonable distances
    *chunk_manager = ChunkManager::new(
        4,  // Load chunks within 4 chunk radius
        2,  // Simulate chunks within 2 chunk radius
    );

    // Create a few test chunks around origin
    for x in -1..=1 {
        for y in -1..=1 {
            for z in 0..=0 {
                let chunk_pos = IVec3::new(x, y, z);
                spawn_test_chunk(&mut commands, &mut chunk_manager, &mut images, chunk_pos);
            }
        }
    }

    info!("Test world setup complete - {} chunks created", chunk_manager.chunks.len());
}

/// Spawn a single chunk with test geometry
fn spawn_test_chunk(
    commands: &mut Commands,
    chunk_manager: &mut ChunkManager,
    images: &mut Assets<Image>,
    chunk_pos: IVec3,
) {
    let mut chunk = WorldChunk::new(chunk_pos);
    
    // Fill bottom layer with rock
    if chunk_pos.z == 0 {
        chunk.fill_region(
            UVec3::new(0, 0, 0),
            UVec3::new(CHUNK_SIZE, CHUNK_SIZE, 4),
            VoxelData::rock(255),
        );
    }
    
    // Create the GPU texture for this chunk
    let texture_handle = create_chunk_texture(&chunk, images);
    chunk.gpu_texture = Some(texture_handle);
    chunk.dirty = false;

    // Spawn the chunk entity
    let entity = commands.spawn((
        chunk,
        Name::new(format!("Chunk_{}_{}_{}", chunk_pos.x, chunk_pos.y, chunk_pos.z)),
    )).id();

    // Register in chunk manager
    chunk_manager.register_chunk(chunk_pos, entity);
}

/// System to manage chunk loading/unloading based on player position
fn manage_chunk_loading(
    // This will be implemented when we add player movement
    // For now, we keep all chunks loaded
) {
    // TODO: Implement chunk loading/unloading based on player position
}

/// System to update chunk textures when they're marked dirty
fn update_chunk_textures(
    mut chunks: Query<&mut WorldChunk>,
    mut images: ResMut<Assets<Image>>,
) {
    for mut chunk in chunks.iter_mut() {
        if chunk.dirty {
            // Re-upload texture data to GPU
            if let Some(texture_handle) = &chunk.gpu_texture {
                if let Some(image) = images.get_mut(texture_handle) {
                    // Update the texture data
                    let voxel_data: Vec<u8> = chunk.voxels
                        .iter()
                        .flat_map(|v| v.as_u32().to_le_bytes())
                        .collect();
                    image.data = Some(voxel_data);
                }
            }
            chunk.dirty = false;
        }
    }
}

/// Debug information display
fn debug_info(
    chunks: Query<&WorldChunk>,
    _chunk_manager: Res<ChunkManager>,
    time: Res<Time>,
) {
    // Print debug info every 2 seconds
    if time.elapsed_secs() % 2.0 < time.delta_secs() {
        let total_chunks = chunks.iter().count();
        let dynamic_chunks = chunks.iter().filter(|c| c.has_dynamic_elements).count();
        
        // Count total active voxels
        let mut fire_count = 0;
        let mut smoke_count = 0;
        let mut water_count = 0;
        let mut debris_count = 0;
        
        for chunk in chunks.iter() {
            for voxel in &chunk.voxels {
                match voxel.material() {
                    MaterialType::Fire => fire_count += 1,
                    MaterialType::Smoke => smoke_count += 1,
                    MaterialType::Water => water_count += 1,
                    MaterialType::Debris => debris_count += 1,
                    _ => {}
                }
            }
        }
        
        info!(
            "Chunks: {} total, {} dynamic | Elements: Fire={}, Smoke={}, Water={}, Debris={} | FPS: {:.1}",
            total_chunks,
            dynamic_chunks,
            fire_count,
            smoke_count,
            water_count,
            debris_count,
            1.0 / time.delta_secs()
        );
    }
}

/// Setup the camera
fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

/// System to create a simple preview of the world
/// This is a temporary visualization until we implement full compute shader rendering
fn render_world_preview(
    mut commands: Commands,
    chunks: Query<&WorldChunk, Added<WorldChunk>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // For each newly added chunk, create a simple colored square to visualize it
    for chunk in chunks.iter() {
        let world_pos = chunk.chunk_position.as_vec3() * CHUNK_SIZE as f32;
        let center = world_pos + Vec3::splat(CHUNK_SIZE as f32 * 0.5);
        
        // Create a simple mesh to represent the chunk
        let size = CHUNK_SIZE as f32 * 0.9; // Slightly smaller to see gaps
        
        commands.spawn((
            Mesh2d(meshes.add(Rectangle::new(size, size))),
            MeshMaterial2d(materials.add(ColorMaterial {
                color: Color::srgba(0.3, 0.5, 0.8, 0.5),
                ..default()
            })),
            Transform::from_translation(Vec3::new(center.x, center.y, 0.0)),
            Name::new(format!("ChunkPreview_{}_{}_{}", 
                chunk.chunk_position.x, 
                chunk.chunk_position.y, 
                chunk.chunk_position.z
            )),
        ));
    }
}

/// Test system to spawn fire elements for demonstration
fn spawn_test_elements(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut chunks: Query<&mut WorldChunk>,
    chunk_manager: Res<ChunkManager>,
    time: Res<Time>,
) {
    // Spawn fire ball on spacebar press
    if keyboard.just_pressed(KeyCode::Space) {
        info!("Spawning fire sphere!");
        ElementSpawner::spawn_fire_sphere(
            Vec3::new(0.0, 0.0, 20.0),
            5.0,
            &mut chunks,
            &chunk_manager,
        );
    }
    
    // Spawn explosion on E key
    if keyboard.just_pressed(KeyCode::KeyE) {
        info!("Spawning explosion!");
        ElementSpawner::spawn_explosion(
            Vec3::new(32.0, 32.0, 10.0),
            8.0,
            &mut chunks,
            &chunk_manager,
        );
    }
    
    // Spawn water on W key
    if keyboard.just_pressed(KeyCode::KeyW) {
        info!("Spawning water!");
        ElementSpawner::spawn_water_sphere(
            Vec3::new(64.0, 0.0, 20.0),
            6.0,
            &mut chunks,
            &chunk_manager,
        );
    }
    
    // Spawn smoke on S key
    if keyboard.just_pressed(KeyCode::KeyS) {
        info!("Spawning smoke!");
        ElementSpawner::spawn_smoke_sphere(
            Vec3::new(-32.0, 32.0, 15.0),
            4.0,
            &mut chunks,
            &chunk_manager,
        );
    }
    
    // Auto-spawn a small fire every second for testing
    static mut LAST_SPAWN: f32 = 0.0;
    unsafe {
        if time.elapsed_secs() - LAST_SPAWN > 1.0 {
            LAST_SPAWN = time.elapsed_secs();
            
            // Spawn a small fire in the center chunk
            ElementSpawner::spawn_fire_sphere(
                Vec3::new(32.0, 32.0, 5.0),
                2.0,
                &mut chunks,
                &chunk_manager,
            );
        }
    }
}

/// Marker for dynamic voxel visualization sprites
#[derive(Component)]
struct DynamicVoxelMarker;

/// Visualize dynamic voxels (fire, smoke, water, debris) as colored pixels
fn visualize_dynamic_voxels(
    mut commands: Commands,
    all_chunks: Query<&WorldChunk>,
    changed_chunks: Query<&WorldChunk, Changed<WorldChunk>>,
    existing_markers: Query<Entity, With<DynamicVoxelMarker>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Only rebuild if something changed
    if changed_chunks.is_empty() {
        return;
    }
    
    // Clear old markers when chunks change
    for entity in existing_markers.iter() {
        commands.entity(entity).despawn();
    }
    
    // Visualize dynamic voxels in ALL chunks (not just changed ones)
    for chunk in all_chunks.iter() {
        let chunk_world_pos = chunk.chunk_position.as_vec3() * CHUNK_SIZE as f32;
        
        // Sample voxels (every 2nd voxel to reduce visual clutter)
        for z in (0..CHUNK_SIZE).step_by(2) {
            for y in (0..CHUNK_SIZE).step_by(2) {
                for x in (0..CHUNK_SIZE).step_by(2) {
                    if let Some(voxel) = chunk.get_voxel(x, y, z) {
                        let material = voxel.material();
                        
                        // Only visualize dynamic materials
                        let color = match material {
                            MaterialType::Fire => Some(Color::srgb(1.0, 0.5, 0.0)),
                            MaterialType::Smoke => Some(Color::srgba(0.3, 0.3, 0.3, 0.6)),
                            MaterialType::Water => Some(Color::srgb(0.2, 0.4, 0.8)),
                            MaterialType::Debris => Some(Color::srgb(0.6, 0.5, 0.4)),
                            _ => None,
                        };
                        
                        if let Some(color) = color {
                            // Calculate world position
                            let world_pos = chunk_world_pos + Vec3::new(
                                x as f32 + 0.5,
                                y as f32 + 0.5,
                                z as f32 * 0.1, // Flatten Z for 2D view
                            );
                            
                            // Spawn a small square to represent this voxel
                            commands.spawn((
                                Mesh2d(meshes.add(Rectangle::new(2.0, 2.0))),
                                MeshMaterial2d(materials.add(ColorMaterial {
                                    color,
                                    ..default()
                                })),
                                Transform::from_translation(world_pos),
                                DynamicVoxelMarker,
                            ));
                        }
                    }
                }
            }
        }
    }
}
