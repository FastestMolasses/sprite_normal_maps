use bevy::prelude::*;
use noise::{Fbm, NoiseFn, Perlin};

/// Represents a 3D voxel volume with density values
#[derive(Clone)]
pub struct Volume {
    /// Width, Height, Depth dimensions
    pub dimensions: UVec3,
    /// Density values at each voxel (0.0 = empty, 1.0 = solid)
    pub data: Vec<f32>,
}

impl Volume {
    /// Create a new empty volume with given dimensions
    pub fn new(width: u32, height: u32, depth: u32) -> Self {
        let size = (width * height * depth) as usize;
        Self {
            dimensions: UVec3::new(width, height, depth),
            data: vec![0.0; size],
        }
    }

    /// Get the index for a 3D coordinate
    #[inline]
    fn index(&self, x: u32, y: u32, z: u32) -> usize {
        (z * self.dimensions.x * self.dimensions.y + y * self.dimensions.x + x) as usize
    }

    /// Get density at a specific voxel position
    pub fn get(&self, x: u32, y: u32, z: u32) -> f32 {
        if x >= self.dimensions.x || y >= self.dimensions.y || z >= self.dimensions.z {
            return 0.0;
        }
        self.data[self.index(x, y, z)]
    }

    /// Set density at a specific voxel position
    pub fn set(&mut self, x: u32, y: u32, z: u32, value: f32) {
        if x >= self.dimensions.x || y >= self.dimensions.y || z >= self.dimensions.z {
            return;
        }
        let idx = self.index(x, y, z);
        self.data[idx] = value;
    }

    /// Sample the volume with trilinear interpolation
    pub fn sample(&self, pos: Vec3) -> f32 {
        // Clamp to volume bounds
        let x = pos.x.clamp(0.0, (self.dimensions.x - 1) as f32);
        let y = pos.y.clamp(0.0, (self.dimensions.y - 1) as f32);
        let z = pos.z.clamp(0.0, (self.dimensions.z - 1) as f32);

        // Get integer and fractional parts
        let x0 = x.floor() as u32;
        let y0 = y.floor() as u32;
        let z0 = z.floor() as u32;
        let x1 = (x0 + 1).min(self.dimensions.x - 1);
        let y1 = (y0 + 1).min(self.dimensions.y - 1);
        let z1 = (z0 + 1).min(self.dimensions.z - 1);

        let fx = x.fract();
        let fy = y.fract();
        let fz = z.fract();

        // Trilinear interpolation
        let c000 = self.get(x0, y0, z0);
        let c100 = self.get(x1, y0, z0);
        let c010 = self.get(x0, y1, z0);
        let c110 = self.get(x1, y1, z0);
        let c001 = self.get(x0, y0, z1);
        let c101 = self.get(x1, y0, z1);
        let c011 = self.get(x0, y1, z1);
        let c111 = self.get(x1, y1, z1);

        let c00 = c000 * (1.0 - fx) + c100 * fx;
        let c01 = c001 * (1.0 - fx) + c101 * fx;
        let c10 = c010 * (1.0 - fx) + c110 * fx;
        let c11 = c011 * (1.0 - fx) + c111 * fx;

        let c0 = c00 * (1.0 - fy) + c10 * fy;
        let c1 = c01 * (1.0 - fy) + c11 * fy;

        c0 * (1.0 - fz) + c1 * fz
    }

    /// Calculate the gradient (normal) at a position using central differences
    pub fn gradient(&self, x: u32, y: u32, z: u32) -> Vec3 {
        let step = 1.0;
        
        let dx = if x > 0 && x < self.dimensions.x - 1 {
            (self.get(x + 1, y, z) - self.get(x - 1, y, z)) / (2.0 * step)
        } else {
            0.0
        };

        let dy = if y > 0 && y < self.dimensions.y - 1 {
            (self.get(x, y + 1, z) - self.get(x, y - 1, z)) / (2.0 * step)
        } else {
            0.0
        };

        let dz = if z > 0 && z < self.dimensions.z - 1 {
            (self.get(x, y, z + 1) - self.get(x, y, z - 1)) / (2.0 * step)
        } else {
            0.0
        };

        // Negate for outward-facing normals
        let normal = Vec3::new(-dx, -dy, -dz);
        
        if normal.length_squared() > 0.0001 {
            normal.normalize()
        } else {
            Vec3::Y // Default up vector if no gradient
        }
    }
}

/// Parameters for procedural rock generation
#[derive(Clone)]
pub struct RockGenerationParams {
    pub size: u32,
    pub scale: f32,
    pub octaves: usize,
    pub lacunarity: f32,
    pub persistence: f32,
    pub threshold: f32,
    pub seed: u32,
}

impl Default for RockGenerationParams {
    fn default() -> Self {
        Self {
            size: 64,
            scale: 4.0,
            octaves: 4,
            lacunarity: 2.0,
            persistence: 0.5,
            threshold: 0.0,
            seed: 42,
        }
    }
}

/// Generate a procedural rock volume using noise
pub fn generate_rock_volume(params: &RockGenerationParams) -> Volume {
    let mut volume = Volume::new(params.size, params.size, params.size);
    
    // Create Fractal Brownian Motion noise
    let fbm = Fbm::<Perlin>::new(params.seed);
    
    let center = params.size as f32 / 2.0;
    let radius = center * 0.8; // Make it slightly smaller than the volume
    
    for z in 0..params.size {
        for y in 0..params.size {
            for x in 0..params.size {
                // Position relative to center
                let px = x as f32 - center;
                let py = y as f32 - center;
                let pz = z as f32 - center;
                
                // Distance from center (sphere)
                let dist = (px * px + py * py + pz * pz).sqrt();
                let dist_normalized = dist / radius;
                
                // Base sphere shape
                let sphere_value = 1.0 - dist_normalized.clamp(0.0, 1.0);
                
                // Add noise for rock-like surface
                let noise_pos = [
                    (px / params.size as f32 * params.scale) as f64,
                    (py / params.size as f32 * params.scale) as f64,
                    (pz / params.size as f32 * params.scale) as f64,
                ];
                
                let noise_value = fbm.get(noise_pos) as f32;
                
                // Combine sphere with noise
                let density = sphere_value + noise_value * 0.3;
                
                // Apply threshold
                let final_density = if density > params.threshold {
                    density
                } else {
                    0.0
                };
                
                volume.set(x, y, z, final_density.clamp(0.0, 1.0));
            }
        }
    }
    
    volume
}

/// Result of rendering a volume to 2D textures
pub struct VolumeRenderResult {
    pub position_map: Vec<u8>,
    pub normal_map: Vec<u8>,
    pub diffuse_map: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

/// Render a volume to 2D position, normal, and diffuse maps using orthographic projection
pub fn render_volume_to_maps(volume: &Volume, output_size: u32, rotation: Vec3) -> VolumeRenderResult {
    let width = output_size;
    let height = output_size;
    let pixel_count = (width * height) as usize;
    
    let mut position_map = vec![0u8; pixel_count * 4]; // RGBA
    let mut normal_map = vec![0u8; pixel_count * 4];   // RGBA
    let mut diffuse_map = vec![0u8; pixel_count * 4];  // RGBA
    
    let vol_size = volume.dimensions.x as f32;
    let threshold = 0.3; // Density threshold for "solid"
    let center = vol_size / 2.0;
    
    // Create rotation matrix from Euler angles (in radians)
    let rotation_matrix = create_rotation_matrix(rotation);
    let inverse_rotation = create_rotation_matrix(-rotation);
    
    // Orthographic projection: shoot rays from front (Z+) toward back (Z-)
    for py in 0..height {
        for px in 0..width {
            let pixel_idx = (py * width + px) as usize * 4;
            
            // Map pixel to volume coordinates (XY plane, centered)
            let screen_x = (px as f32 / width as f32 - 0.5) * vol_size;
            let screen_y = (py as f32 / height as f32 - 0.5) * vol_size;
            
            // Raycast from front to back along Z axis
            let mut hit = false;
            let mut hit_pos = Vec3::ZERO;
            let mut hit_voxel = UVec3::ZERO;
            
            // Ray in screen space (before rotation)
            let ray_start = Vec3::new(screen_x, screen_y, -vol_size);
            let ray_dir = Vec3::new(0.0, 0.0, 1.0);
            
            // March along the ray with adaptive step size
            let max_steps = (vol_size * 1.5) as usize; // Reduced from 2.0
            let step_size = 0.75; // Increased from 0.5 for faster marching
            
            for step in 0..max_steps {
                let t = step as f32 * step_size;
                let ray_pos = ray_start + ray_dir * t;
                
                // Rotate ray position to volume space
                let rotated_pos = rotate_point(ray_pos, inverse_rotation) + Vec3::splat(center);
                
                // Check if we're inside the volume
                if rotated_pos.x < 0.0 || rotated_pos.x >= vol_size ||
                   rotated_pos.y < 0.0 || rotated_pos.y >= vol_size ||
                   rotated_pos.z < 0.0 || rotated_pos.z >= vol_size {
                    continue;
                }
                
                let vx = rotated_pos.x as u32;
                let vy = rotated_pos.y as u32;
                let vz = rotated_pos.z as u32;
                
                let density = volume.get(vx, vy, vz);
                
                if density > threshold {
                    // Hit! Record the position
                    hit = true;
                    hit_pos = rotated_pos;
                    hit_voxel = UVec3::new(vx, vy, vz);
                    break;
                }
            }
            
            if hit {
                // Position map: encode world position as RGB
                // Normalize to 0-255 range based on volume size
                position_map[pixel_idx] = ((hit_pos.x / vol_size) * 255.0) as u8;
                position_map[pixel_idx + 1] = ((hit_pos.y / vol_size) * 255.0) as u8;
                position_map[pixel_idx + 2] = ((hit_pos.z / vol_size) * 255.0) as u8;
                position_map[pixel_idx + 3] = 255; // Alpha
                
                // Normal map: calculate gradient in volume space, then rotate to world space
                let normal_volume = volume.gradient(hit_voxel.x, hit_voxel.y, hit_voxel.z);
                let normal_world = rotate_point(normal_volume, rotation_matrix);
                
                // Map from -1..1 to 0..255
                normal_map[pixel_idx] = ((normal_world.x * 0.5 + 0.5) * 255.0) as u8;
                normal_map[pixel_idx + 1] = ((normal_world.y * 0.5 + 0.5) * 255.0) as u8;
                normal_map[pixel_idx + 2] = ((normal_world.z * 0.5 + 0.5) * 255.0) as u8;
                normal_map[pixel_idx + 3] = 255; // Alpha
                
                // Diffuse map: simple gray rock color with slight variation based on position
                let variation = (hit_pos.y / vol_size) * 0.2; // Height-based variation
                let base_color = 0.5 + variation;
                let r = (base_color * 180.0) as u8;
                let g = (base_color * 170.0) as u8;
                let b = (base_color * 160.0) as u8;
                
                diffuse_map[pixel_idx] = r;
                diffuse_map[pixel_idx + 1] = g;
                diffuse_map[pixel_idx + 2] = b;
                diffuse_map[pixel_idx + 3] = 255; // Alpha
            } else {
                // No hit: transparent
                position_map[pixel_idx + 3] = 0;
                normal_map[pixel_idx + 3] = 0;
                diffuse_map[pixel_idx + 3] = 0;
            }
        }
    }
    
    VolumeRenderResult {
        position_map,
        normal_map,
        diffuse_map,
        width,
        height,
    }
}

/// Create a 3D rotation matrix from Euler angles (XYZ order)
fn create_rotation_matrix(rotation: Vec3) -> Mat3 {
    let (sx, cx) = rotation.x.sin_cos();
    let (sy, cy) = rotation.y.sin_cos();
    let (sz, cz) = rotation.z.sin_cos();
    
    // Rotation around X axis
    let rx = Mat3::from_cols(
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(0.0, cx, sx),
        Vec3::new(0.0, -sx, cx),
    );
    
    // Rotation around Y axis
    let ry = Mat3::from_cols(
        Vec3::new(cy, 0.0, -sy),
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::new(sy, 0.0, cy),
    );
    
    // Rotation around Z axis
    let rz = Mat3::from_cols(
        Vec3::new(cz, sz, 0.0),
        Vec3::new(-sz, cz, 0.0),
        Vec3::new(0.0, 0.0, 1.0),
    );
    
    // Combine rotations: Z * Y * X
    rz * ry * rx
}

/// Rotate a point using a rotation matrix
fn rotate_point(point: Vec3, rotation_matrix: Mat3) -> Vec3 {
    rotation_matrix * point
}
