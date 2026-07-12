pub mod hooks;
pub mod manifest;
pub mod manager;
pub mod plugin;
pub mod registry;

pub use hooks::HookRunner;
pub use manifest::PluginManifest;
pub use manager::PluginManager;
pub use plugin::{Plugin, PluginDefinition, PluginKind, PluginMetadata, PluginTool};
pub use registry::{PluginEntry, PluginRegistry};
