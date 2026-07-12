use serde::{Deserialize, Serialize};

/// Plugin manifest metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub plugin: PluginInfo,
    pub permissions: PluginPermissions,
    pub hooks: PluginHooks,
    pub tools: Option<Vec<ToolEntry>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub license: Option<String>,
    pub min_pleiades_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginPermissions {
    pub required: Vec<String>,
    pub optional: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginHooks {
    pub on_tool_call: Option<String>,
    pub on_message: Option<String>,
    pub on_startup: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolEntry {
    pub name: String,
    pub description: String,
}
