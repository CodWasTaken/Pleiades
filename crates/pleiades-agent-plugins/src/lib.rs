pub mod hooks;
pub mod manager;
pub mod manifest;
pub mod plugin;
pub mod registry;

pub use hooks::HookRunner;
pub use manager::PluginManager;
pub use manifest::PluginManifest;
pub use plugin::{Plugin, PluginDefinition, PluginKind, PluginMetadata, PluginTool};
pub use registry::{PluginEntry, PluginRegistry};
