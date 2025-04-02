#[cfg(not(target_arch = "wasm32"))]
pub mod client;

#[cfg(target_arch = "wasm32")]
pub mod client_wasm;
