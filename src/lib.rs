pub mod ray_tracer;

pub use eframe::wgpu;

#[cfg(not(target_arch = "wasm32"))]
pub use std::time as timer;
#[cfg(target_arch = "wasm32")]
pub use wasm_timer as timer;
