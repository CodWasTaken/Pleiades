//! Multi-level configuration system for Pleiades.
//!
//! Supports five levels of configuration with merging:
//! 1. Defaults
//! 2. Global config (`~/.config/pleiades/`)
//! 3. Project config (`./.pleiades/`)
//! 4. Environment variables (`PLEIADES_*`)
//! 5. CLI flags

pub mod loader;
pub mod types;
pub mod validate;

pub use loader::ConfigLoader;
pub use types::{Config, Profile, ProviderConfig};
pub use validate::ValidationError;
