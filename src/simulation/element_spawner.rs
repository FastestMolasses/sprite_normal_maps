use bevy::prelude::*;
use crate::world::{WorldChunk, VoxelData, MaterialType, voxel_flags, ChunkManager, CHUNK_SIZE};

/// High-level API for spawning dynamic elements in the world
pub struct ElementSpawner;

impl ElementSpawner {
    /// Spawn a sphere of fire at a world position
    pub fn spawn_fire_sphere(
        world_pos: Vec3,
        radius: f32,
        chunks: &mut Query<&mut WorldChunk>,
        chunk_manager: &ChunkManager,
    ) {
        Self::spawn_element_sphere(
            world_pos,
            radius,
            VoxelData::new(
                MaterialType::Fire,
                255,
                255,
                voxel_flags::EMITS_LIGHT | voxel_flags::TEMPORARY,
            ),
            chunks,
            chunk_manager,
        );
    }

    /// Spawn a sphere of smoke at a world position
    pub fn spawn_smoke_sphere(
        world_pos: Vec3,
        radius: f32,
        chunks: &mut Query<&mut WorldChunk>,
        chunk_manager: &ChunkManager,
    ) {
        Self::spawn_element_sphere(
            world_pos,
            radius,
            VoxelData::new(
                MaterialType::Smoke,
                200,
                50,
                voxel_flags::TEMPORARY | voxel_flags::TRANSPARENT,
            ),
            chunks,
            chunk_manager,
        );
    }

    /// Spawn a sphere of water at a world position
    pub fn spawn_water_sphere(
        world_pos: Vec3,
        radius: f32,
        chunks: &mut Query<&mut WorldChunk>,
        chunk_manager: &ChunkManager,
    ) {
        Self::spawn_element_sphere(
            world_pos,
            radius,
            VoxelData::new(
                MaterialType::Water,
                255,
                20,
                voxel_flags::TRANSPARENT,
            ),
            chunks,
            chunk_manager,
        );
    }

    /// Spawn debris from an explosion (scattered in a sphere)
    pub fn spawn_explosion_debris(
        world_pos: Vec3,
        radius: f32,
        chunks: &mut Query<&mut WorldChunk>,
        chunk_manager: &ChunkManager,
    ) {
        // Create debris with some variation
        Self::spawn_element_sphere(
            world_pos,
            radius * 0.7, // Debris is more concentrated
            VoxelData::new(
                MaterialType::Debris,
                180,
                100,
                voxel_flags::TEMPORARY,
            ),
            chunks,
            chunk_manager,
        );
    }

    /// Spawn a complete explosion effect (fire + smoke + debris)
    pub fn spawn_explosion(
        world_pos: Vec3,
        radius: f32,
        chunks: &mut Query<&mut WorldChunk>,
        chunk_manager: &ChunkManager,
    ) {
        // Inner core of fire
        Self::spawn_fire_sphere(world_pos, radius * 0.5, chunks, chunk_manager);
        
        // Outer smoke ring
        Self::spawn_smoke_sphere(world_pos, radius, chunks, chunk_manager);
        
        // Debris scattered around
        Self::spawn_explosion_debris(world_pos, radius * 1.2, chunks, chunk_manager);
    }

    /// Generic sphere spawner
    fn spawn_element_sphere(
        world_pos: Vec3,
        radius: f32,
        voxel: VoxelData,
        chunks: &mut Query<&mut WorldChunk>,
        chunk_manager: &ChunkManager,
    ) {
        let radius_sq = radius * radius;
        
        // Calculate affected chunk range
        let min_chunk = ChunkManager::world_to_chunk_pos(world_pos - Vec3::splat(radius));
        let max_chunk = ChunkManager::world_to_chunk_pos(world_pos + Vec3::splat(radius));
        
        // Iterate through all potentially affected chunks
        for cx in min_chunk.x..=max_chunk.x {
            for cy in min_chunk.y..=max_chunk.y {
                for cz in min_chunk.z..=max_chunk.z {
                    let chunk_pos = IVec3::new(cx, cy, cz);
                    
                    // Get the chunk entity
                    if let Some(entity) = chunk_manager.get_chunk_entity(chunk_pos) {
                        if let Ok(mut chunk) = chunks.get_mut(entity) {
                            // Fill sphere within this chunk
                            chunk.fill_sphere(world_pos, radius, voxel);
                            
                            // Mark as having dynamic elements
                            if voxel.material().is_dynamic() {
                                chunk.has_dynamic_elements = true;
                            }
                        }
                    }
                }
            }
        }
    }

    /// Spawn a line of elements (useful for testing)
    pub fn spawn_fire_line(
        start: Vec3,
        end: Vec3,
        thickness: f32,
        chunks: &mut Query<&mut WorldChunk>,
        chunk_manager: &ChunkManager,
    ) {
        let direction = (end - start).normalize();
        let length = start.distance(end);
        let steps = (length / thickness).ceil() as i32;
        
        for i in 0..steps {
            let t = i as f32 / steps as f32;
            let pos = start + direction * length * t;
            Self::spawn_fire_sphere(pos, thickness, chunks, chunk_manager);
        }
    }
}

/// Component to mark an entity as an element spawner with automatic spawning
#[derive(Component)]
pub struct AutoElementSpawner {
    pub element_type: ElementType,
    pub spawn_interval: f32,
    pub spawn_radius: f32,
    pub timer: f32,
}

/// Type of element to spawn
#[derive(Clone, Copy)]
pub enum ElementType {
    Fire,
    Smoke,
    Water,
    Explosion,
}

/// System to handle automatic element spawning
pub fn update_auto_spawners(
    time: Res<Time>,
    mut spawners: Query<(&mut AutoElementSpawner, &Transform)>,
    mut chunks: Query<&mut WorldChunk>,
    chunk_manager: Res<ChunkManager>,
) {
    for (mut spawner, transform) in spawners.iter_mut() {
        spawner.timer += time.delta_secs();
        
        if spawner.timer >= spawner.spawn_interval {
            spawner.timer = 0.0;
            
            let pos = transform.translation;
            
            match spawner.element_type {
                ElementType::Fire => {
                    ElementSpawner::spawn_fire_sphere(
                        pos,
                        spawner.spawn_radius,
                        &mut chunks,
                        &chunk_manager,
                    );
                }
                ElementType::Smoke => {
                    ElementSpawner::spawn_smoke_sphere(
                        pos,
                        spawner.spawn_radius,
                        &mut chunks,
                        &chunk_manager,
                    );
                }
                ElementType::Water => {
                    ElementSpawner::spawn_water_sphere(
                        pos,
                        spawner.spawn_radius,
                        &mut chunks,
                        &chunk_manager,
                    );
                }
                ElementType::Explosion => {
                    ElementSpawner::spawn_explosion(
                        pos,
                        spawner.spawn_radius,
                        &mut chunks,
                        &chunk_manager,
                    );
                }
            }
        }
    }
}
