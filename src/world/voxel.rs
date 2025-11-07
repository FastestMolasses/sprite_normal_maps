use bevy::prelude::*;

/// Material types for voxels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum MaterialType {
    Air = 0,
    Rock = 1,
    Dirt = 2,
    Wood = 3,
    Metal = 4,
    Fire = 5,
    Smoke = 6,
    Water = 7,
    Debris = 8,
    // Add more as needed
}

impl MaterialType {
    /// Convert from u8 back to MaterialType
    pub fn from_u8(value: u8) -> Self {
        match value {
            1 => MaterialType::Rock,
            2 => MaterialType::Dirt,
            3 => MaterialType::Wood,
            4 => MaterialType::Metal,
            5 => MaterialType::Fire,
            6 => MaterialType::Smoke,
            7 => MaterialType::Water,
            8 => MaterialType::Debris,
            _ => MaterialType::Air,
        }
    }

    /// Check if this material is solid (for collision)
    pub fn is_solid(&self) -> bool {
        matches!(self, 
            MaterialType::Rock | 
            MaterialType::Dirt | 
            MaterialType::Wood | 
            MaterialType::Metal
        )
    }

    /// Check if this material is dynamic (needs simulation)
    pub fn is_dynamic(&self) -> bool {
        matches!(self,
            MaterialType::Fire |
            MaterialType::Smoke |
            MaterialType::Water |
            MaterialType::Debris
        )
    }

    /// Get the default color for this material (for diffuse rendering)
    pub fn default_color(&self) -> Color {
        match self {
            MaterialType::Air => Color::NONE,
            MaterialType::Rock => Color::srgb(0.5, 0.5, 0.5),
            MaterialType::Dirt => Color::srgb(0.4, 0.3, 0.2),
            MaterialType::Wood => Color::srgb(0.6, 0.4, 0.2),
            MaterialType::Metal => Color::srgb(0.7, 0.7, 0.8),
            MaterialType::Fire => Color::srgb(1.0, 0.5, 0.1),
            MaterialType::Smoke => Color::srgba(0.2, 0.2, 0.2, 0.5),
            MaterialType::Water => Color::srgba(0.2, 0.4, 0.8, 0.6),
            MaterialType::Debris => Color::srgb(0.6, 0.5, 0.4),
        }
    }
}

/// Voxel data packed into 32 bits (4 bytes)
/// Layout: [material_id: 8 bits][density: 8 bits][temperature: 8 bits][flags: 8 bits]
#[derive(Debug, Clone, Copy, Default)]
pub struct VoxelData {
    data: u32,
}

/// Flags for voxel properties
pub mod voxel_flags {
    pub const NONE: u8 = 0;
    pub const COLLISION: u8 = 1 << 0;  // Blocks movement
    pub const EMITS_LIGHT: u8 = 1 << 1; // Emits light
    pub const TEMPORARY: u8 = 1 << 2;   // Will be removed after lifetime
    pub const STATIC: u8 = 1 << 3;      // Part of static geometry (no simulation)
    pub const TRANSPARENT: u8 = 1 << 4; // Allows light to pass through
}

impl VoxelData {
    /// Create a new voxel with given properties
    pub fn new(material: MaterialType, density: u8, temperature: u8, flags: u8) -> Self {
        let data = (material as u32) 
            | ((density as u32) << 8) 
            | ((temperature as u32) << 16) 
            | ((flags as u32) << 24);
        Self { data }
    }

    /// Create an empty (air) voxel
    pub fn air() -> Self {
        Self::new(MaterialType::Air, 0, 0, voxel_flags::NONE)
    }

    /// Create a solid rock voxel
    pub fn rock(density: u8) -> Self {
        Self::new(
            MaterialType::Rock, 
            density, 
            0, 
            voxel_flags::COLLISION | voxel_flags::STATIC
        )
    }

    /// Get the raw packed u32 value (for GPU upload)
    #[inline]
    pub fn as_u32(&self) -> u32 {
        self.data
    }

    /// Create from raw u32 value (for GPU readback)
    #[inline]
    pub fn from_u32(data: u32) -> Self {
        Self { data }
    }

    /// Get material type
    #[inline]
    pub fn material(&self) -> MaterialType {
        MaterialType::from_u8((self.data & 0xFF) as u8)
    }

    /// Get density (0-255)
    #[inline]
    pub fn density(&self) -> u8 {
        ((self.data >> 8) & 0xFF) as u8
    }

    /// Get temperature (0-255)
    #[inline]
    pub fn temperature(&self) -> u8 {
        ((self.data >> 16) & 0xFF) as u8
    }

    /// Get flags
    #[inline]
    pub fn flags(&self) -> u8 {
        ((self.data >> 24) & 0xFF) as u8
    }

    /// Set material type
    pub fn set_material(&mut self, material: MaterialType) {
        self.data = (self.data & 0xFFFFFF00) | (material as u32);
    }

    /// Set density
    pub fn set_density(&mut self, density: u8) {
        self.data = (self.data & 0xFFFF00FF) | ((density as u32) << 8);
    }

    /// Set temperature
    pub fn set_temperature(&mut self, temperature: u8) {
        self.data = (self.data & 0xFF00FFFF) | ((temperature as u32) << 16);
    }

    /// Set flags
    pub fn set_flags(&mut self, flags: u8) {
        self.data = (self.data & 0x00FFFFFF) | ((flags as u32) << 24);
    }

    /// Check if voxel has a specific flag
    #[inline]
    pub fn has_flag(&self, flag: u8) -> bool {
        (self.flags() & flag) != 0
    }

    /// Add a flag
    pub fn add_flag(&mut self, flag: u8) {
        let new_flags = self.flags() | flag;
        self.set_flags(new_flags);
    }

    /// Remove a flag
    pub fn remove_flag(&mut self, flag: u8) {
        let new_flags = self.flags() & !flag;
        self.set_flags(new_flags);
    }

    /// Check if this is an empty (air) voxel
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.material() == MaterialType::Air
    }

    /// Check if this voxel is solid (blocks collision)
    #[inline]
    pub fn is_solid(&self) -> bool {
        self.has_flag(voxel_flags::COLLISION) || self.material().is_solid()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_voxel_packing() {
        let voxel = VoxelData::new(
            MaterialType::Fire,
            200,
            255,
            voxel_flags::EMITS_LIGHT | voxel_flags::TEMPORARY
        );

        assert_eq!(voxel.material(), MaterialType::Fire);
        assert_eq!(voxel.density(), 200);
        assert_eq!(voxel.temperature(), 255);
        assert!(voxel.has_flag(voxel_flags::EMITS_LIGHT));
        assert!(voxel.has_flag(voxel_flags::TEMPORARY));
    }

    #[test]
    fn test_voxel_round_trip() {
        let original = VoxelData::new(MaterialType::Rock, 128, 50, voxel_flags::COLLISION);
        let packed = original.as_u32();
        let unpacked = VoxelData::from_u32(packed);

        assert_eq!(unpacked.material(), MaterialType::Rock);
        assert_eq!(unpacked.density(), 128);
        assert_eq!(unpacked.temperature(), 50);
        assert_eq!(unpacked.flags(), voxel_flags::COLLISION);
    }
}
