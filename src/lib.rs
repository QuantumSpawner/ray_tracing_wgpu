pub mod ray_tracer;

pub use eframe::wgpu;

#[cfg(not(target_arch = "wasm32"))]
pub use std::time as time;
#[cfg(target_arch = "wasm32")]
pub use web_time as time;
