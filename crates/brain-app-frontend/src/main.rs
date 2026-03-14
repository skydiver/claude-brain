#[cfg(target_arch = "wasm32")]
mod api;
#[cfg(target_arch = "wasm32")]
mod app;
#[cfg(target_arch = "wasm32")]
mod components;
mod markdown;
mod models;
#[cfg(target_arch = "wasm32")]
mod pages;
#[cfg(target_arch = "wasm32")]
mod theme;

#[cfg(target_arch = "wasm32")]
use leptos::prelude::*;

#[cfg(target_arch = "wasm32")]
fn main() {
    mount_to_body(app::App);
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    // This crate is intended to be compiled to WASM.
    // This main exists only so `cargo test` works on native targets.
    eprintln!("brain-app-frontend is a WASM application. Build with `trunk build` instead.");
}
