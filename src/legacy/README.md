# Legacy Code Reference

This directory contains the original implementation of the sprite normal mapping system.

## Files

- **main_old.rs** - Original main file with texture-mapped scene and procedural volume demo
- **lighting.rs** - Light marker component for 2D lighting
- **scenes.rs** - Scene management and switching system
- **ui.rs** - UI for debugging and controls
- **volume.rs** - CPU-based voxel volume with noise generation
- **gpu_volume.rs** - GPU compute shader for volume rendering

## Features Demonstrated

1. **Position-mapped lighting** - Using position maps to simulate 3D lighting in 2D
2. **Normal mapping** - Standard normal mapping for surface details
3. **Procedural rock generation** - Noise-based 3D rock with rotation
4. **GPU raymarching** - Compute shader for rendering 3D volumes to 2D sprites

## Usage

This code is preserved for reference. The new system builds upon these concepts but with a completely restructured architecture focused on:
- Chunk-based world management
- Noita-like element simulation
- Efficient GPU-first processing
- Scalable 3D-simulated isometric world

To reference this code, see the individual files. Many concepts (like the shader implementations) will be adapted for the new system.
