// Library target for native testing of pure-Rust modules.
// WASM-dependent modules are only available when compiling to wasm32.

pub mod markdown;
pub mod models;

#[cfg(target_arch = "wasm32")]
pub mod api;
#[cfg(target_arch = "wasm32")]
pub mod app;
#[cfg(target_arch = "wasm32")]
pub mod components;
#[cfg(target_arch = "wasm32")]
pub mod pages;
#[cfg(target_arch = "wasm32")]
pub mod theme;
