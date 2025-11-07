use bevy::prelude::*;

/// Simple spatial index using a grid-based approach
/// This will be used for fast spatial queries (raycasting, collision detection)
#[derive(Resource, Default)]
pub struct SpatialIndex {
    // In the future, this could be a proper octree or BVH
    // For now, we'll use the chunk system itself as the spatial index
}

impl SpatialIndex {
    pub fn new() -> Self {
        Self::default()
    }

    /// Perform a raycast through the world
    /// Returns the hit position and normal if a solid voxel is hit
    pub fn raycast(
        &self,
        _origin: Vec3,
        _direction: Vec3,
        _max_distance: f32,
    ) -> Option<RaycastHit> {
        // TODO: Implement DDA (Digital Differential Analyzer) raycast
        // This will be implemented in a future phase
        None
    }
}

/// Result of a raycast query
#[derive(Debug, Clone)]
pub struct RaycastHit {
    pub position: Vec3,
    pub normal: Vec3,
    pub distance: f32,
}
