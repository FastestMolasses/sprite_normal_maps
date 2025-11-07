/// GPU-accelerated simulation systems
/// 
/// This module handles compute shader-based cellular automata
/// for simulating fire, smoke, liquids, and other dynamic elements.

pub mod compute_pipeline;
pub mod cpu_simulation;
pub mod element_spawner;

pub use compute_pipeline::*;
pub use cpu_simulation::*;
pub use element_spawner::*;
