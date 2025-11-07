use bevy::prelude::*;
use crate::world::voxel::VoxelData;

/// Size of a chunk in voxels (each dimension)
pub const CHUNK_SIZE: u32 = 64;

/// Calculate the number of voxels in a chunk
pub const VOXELS_PER_CHUNK: usize = (CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) as usize;

/// A 3D chunk of voxel data
/// Represents a 64x64x64 section of the world
#[derive(Component, Clone)]
pub struct WorldChunk {
    /// Position of this chunk in chunk coordinates (not voxel coordinates)
    pub chunk_position: IVec3,
    
    /// Voxel data stored as packed u32 values
    /// Indexed as: z * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE + x
    pub voxels: Vec<VoxelData>,
    
    /// GPU texture handle for this chunk (3D texture)
    pub gpu_texture: Option<Handle<Image>>,
    
    /// Whether this chunk has been modified and needs re-upload to GPU
    pub dirty: bool,
    
    /// Whether this chunk contains any dynamic elements that need simulation
    pub has_dynamic_elements: bool,
    
    /// Bounding box in world space (for culling)
    pub world_bounds: BoundingBox,
}

/// Bounding box for spatial queries
#[derive(Debug, Clone, Copy)]
pub struct BoundingBox {
    pub min: Vec3,
    pub max: Vec3,
}

impl BoundingBox {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    pub fn contains_point(&self, point: Vec3) -> bool {
        point.x >= self.min.x && point.x <= self.max.x &&
        point.y >= self.min.y && point.y <= self.max.y &&
        point.z >= self.min.z && point.z <= self.max.z
    }

    pub fn intersects(&self, other: &BoundingBox) -> bool {
        self.min.x <= other.max.x && self.max.x >= other.min.x &&
        self.min.y <= other.max.y && self.max.y >= other.min.y &&
        self.min.z <= other.max.z && self.max.z >= other.min.z
    }
}

impl WorldChunk {
    /// Create a new empty chunk at the given chunk position
    pub fn new(chunk_position: IVec3) -> Self {
        let voxels = vec![VoxelData::air(); VOXELS_PER_CHUNK];
        
        // Calculate world-space bounds
        let world_min = chunk_position.as_vec3() * CHUNK_SIZE as f32;
        let world_max = world_min + Vec3::splat(CHUNK_SIZE as f32);
        
        Self {
            chunk_position,
            voxels,
            gpu_texture: None,
            dirty: true,
            has_dynamic_elements: false,
            world_bounds: BoundingBox::new(world_min, world_max),
        }
    }

    /// Get the flat index for a voxel position within this chunk
    #[inline]
    fn voxel_index(&self, x: u32, y: u32, z: u32) -> Option<usize> {
        if x >= CHUNK_SIZE || y >= CHUNK_SIZE || z >= CHUNK_SIZE {
            return None;
        }
        Some((z * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE + x) as usize)
    }

    /// Get voxel at local chunk coordinates (0-63)
    pub fn get_voxel(&self, x: u32, y: u32, z: u32) -> Option<VoxelData> {
        self.voxel_index(x, y, z).map(|idx| self.voxels[idx])
    }

    /// Set voxel at local chunk coordinates
    pub fn set_voxel(&mut self, x: u32, y: u32, z: u32, voxel: VoxelData) {
        if let Some(idx) = self.voxel_index(x, y, z) {
            self.voxels[idx] = voxel;
            self.dirty = true;
            
            // Check if this adds a dynamic element
            if voxel.material().is_dynamic() {
                self.has_dynamic_elements = true;
            }
        }
    }

    /// Get voxel at world position (converts to local coordinates)
    pub fn get_voxel_world(&self, world_pos: Vec3) -> Option<VoxelData> {
        let local_pos = self.world_to_local(world_pos)?;
        self.get_voxel(local_pos.x, local_pos.y, local_pos.z)
    }

    /// Set voxel at world position
    pub fn set_voxel_world(&mut self, world_pos: Vec3, voxel: VoxelData) {
        if let Some(local_pos) = self.world_to_local(world_pos) {
            self.set_voxel(local_pos.x, local_pos.y, local_pos.z, voxel);
        }
    }

    /// Convert world position to local chunk coordinates
    fn world_to_local(&self, world_pos: Vec3) -> Option<UVec3> {
        if !self.world_bounds.contains_point(world_pos) {
            return None;
        }

        let local = world_pos - self.world_bounds.min;
        let x = local.x.floor() as u32;
        let y = local.y.floor() as u32;
        let z = local.z.floor() as u32;

        if x < CHUNK_SIZE && y < CHUNK_SIZE && z < CHUNK_SIZE {
            Some(UVec3::new(x, y, z))
        } else {
            None
        }
    }

    /// Convert local chunk coordinates to world position (center of voxel)
    pub fn local_to_world(&self, x: u32, y: u32, z: u32) -> Vec3 {
        self.world_bounds.min + Vec3::new(x as f32 + 0.5, y as f32 + 0.5, z as f32 + 0.5)
    }

    /// Fill a region with a specific voxel type
    pub fn fill_region(
        &mut self,
        min: UVec3,
        max: UVec3,
        voxel: VoxelData,
    ) {
        let min_x = min.x.min(CHUNK_SIZE - 1);
        let min_y = min.y.min(CHUNK_SIZE - 1);
        let min_z = min.z.min(CHUNK_SIZE - 1);
        let max_x = max.x.min(CHUNK_SIZE);
        let max_y = max.y.min(CHUNK_SIZE);
        let max_z = max.z.min(CHUNK_SIZE);

        for z in min_z..max_z {
            for y in min_y..max_y {
                for x in min_x..max_x {
                    self.set_voxel(x, y, z, voxel);
                }
            }
        }
    }

    /// Fill a sphere with voxels (useful for spawning elements like fire)
    pub fn fill_sphere(
        &mut self,
        center_world: Vec3,
        radius: f32,
        voxel: VoxelData,
    ) {
        let radius_sq = radius * radius;
        
        // Calculate bounding box of sphere in local coordinates
        let local_min = self.world_to_local(center_world - Vec3::splat(radius));
        let local_max = self.world_to_local(center_world + Vec3::splat(radius));
        
        if local_min.is_none() && local_max.is_none() {
            return; // Sphere doesn't intersect this chunk
        }

        let min = local_min.unwrap_or(UVec3::ZERO);
        let max = local_max.unwrap_or(UVec3::splat(CHUNK_SIZE - 1));

        for z in min.z..=max.z.min(CHUNK_SIZE - 1) {
            for y in min.y..=max.y.min(CHUNK_SIZE - 1) {
                for x in min.x..=max.x.min(CHUNK_SIZE - 1) {
                    let voxel_world = self.local_to_world(x, y, z);
                    let dist_sq = center_world.distance_squared(voxel_world);
                    
                    if dist_sq <= radius_sq {
                        self.set_voxel(x, y, z, voxel);
                    }
                }
            }
        }
    }

    /// Get raw voxel data as u32 slice (for GPU upload)
    pub fn as_u32_slice(&self) -> Vec<u32> {
        self.voxels.iter().map(|v| v.as_u32()).collect()
    }

    /// Check if this chunk needs dynamic simulation
    pub fn needs_simulation(&self) -> bool {
        self.has_dynamic_elements
    }

    /// Recalculate whether this chunk has dynamic elements
    pub fn recalculate_dynamic_status(&mut self) {
        self.has_dynamic_elements = self.voxels.iter()
            .any(|v| v.material().is_dynamic());
    }
}

/// Resource managing all active chunks in the world
#[derive(Resource, Default)]
pub struct ChunkManager {
    /// Map of chunk position to entity ID
    pub chunks: std::collections::HashMap<IVec3, Entity>,
    
    /// Distance from player to load/unload chunks
    pub load_distance: i32,
    
    /// Distance from player to simulate chunks
    pub simulation_distance: i32,
}

impl ChunkManager {
    pub fn new(load_distance: i32, simulation_distance: i32) -> Self {
        Self {
            chunks: std::collections::HashMap::new(),
            load_distance,
            simulation_distance,
        }
    }

    /// Get chunk position from world position
    pub fn world_to_chunk_pos(world_pos: Vec3) -> IVec3 {
        IVec3::new(
            (world_pos.x / CHUNK_SIZE as f32).floor() as i32,
            (world_pos.y / CHUNK_SIZE as f32).floor() as i32,
            (world_pos.z / CHUNK_SIZE as f32).floor() as i32,
        )
    }

    /// Check if a chunk position should be loaded based on player position
    pub fn should_load_chunk(&self, chunk_pos: IVec3, player_chunk_pos: IVec3) -> bool {
        let distance = (chunk_pos - player_chunk_pos).abs().max_element();
        distance <= self.load_distance
    }

    /// Check if a chunk should be actively simulated
    pub fn should_simulate_chunk(&self, chunk_pos: IVec3, player_chunk_pos: IVec3) -> bool {
        let distance = (chunk_pos - player_chunk_pos).abs().max_element();
        distance <= self.simulation_distance
    }

    /// Get entity for a chunk at given position
    pub fn get_chunk_entity(&self, chunk_pos: IVec3) -> Option<Entity> {
        self.chunks.get(&chunk_pos).copied()
    }

    /// Register a new chunk
    pub fn register_chunk(&mut self, chunk_pos: IVec3, entity: Entity) {
        self.chunks.insert(chunk_pos, entity);
    }

    /// Unregister a chunk
    pub fn unregister_chunk(&mut self, chunk_pos: IVec3) -> Option<Entity> {
        self.chunks.remove(&chunk_pos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::voxel::MaterialType;

    #[test]
    fn test_chunk_creation() {
        let chunk = WorldChunk::new(IVec3::new(0, 0, 0));
        assert_eq!(chunk.voxels.len(), VOXELS_PER_CHUNK);
        assert!(chunk.dirty);
    }

    #[test]
    fn test_voxel_indexing() {
        let mut chunk = WorldChunk::new(IVec3::ZERO);
        let rock = VoxelData::rock(255);
        
        chunk.set_voxel(10, 20, 30, rock);
        let retrieved = chunk.get_voxel(10, 20, 30).unwrap();
        
        assert_eq!(retrieved.material(), MaterialType::Rock);
        assert_eq!(retrieved.density(), 255);
    }

    #[test]
    fn test_world_to_chunk_pos() {
        assert_eq!(
            ChunkManager::world_to_chunk_pos(Vec3::new(0.0, 0.0, 0.0)),
            IVec3::new(0, 0, 0)
        );
        assert_eq!(
            ChunkManager::world_to_chunk_pos(Vec3::new(64.0, 64.0, 64.0)),
            IVec3::new(1, 1, 1)
        );
        assert_eq!(
            ChunkManager::world_to_chunk_pos(Vec3::new(-1.0, -1.0, -1.0)),
            IVec3::new(-1, -1, -1)
        );
    }
}
