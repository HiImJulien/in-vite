//! This module implements the necessary types and function required to
//! integrate Vite into Rust backend projects.

mod integrations;
mod manifest;
mod resource;
mod vite;
mod error;

pub use vite::{ViteMode, ViteOptions, Vite};

