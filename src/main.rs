mod app;

#[cfg(not(target_arch = "wasm32"))]
mod main_native;
#[cfg(not(target_arch = "wasm32"))]
use main_native::*;

#[cfg(target_arch = "wasm32")]
mod main_wasm;
#[cfg(target_arch = "wasm32")]
use main_wasm::*;
