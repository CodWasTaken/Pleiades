//! Plugin system for Pleiades.
//!
//! Plugins extend Pleiades with new capabilities using a WASM-based
//! runtime for safe, isolated execution.

pub mod manifest;
pub mod hooks;
pub mod registry;

pub use manifest::PluginManifest;
pub use registry::PluginRegistry;
