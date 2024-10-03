//! This module implements the necessary types and function required to
//! integrate Vite into Rust backend projects.

mod error;
mod integrations;
mod manifest;
mod resource;
mod vite;

pub use vite::{Vite, ViteMode, ViteOptions, ViteReactRefresh};
