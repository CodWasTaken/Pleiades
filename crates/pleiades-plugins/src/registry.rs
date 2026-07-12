use std::collections::HashMap;

use pleiades_core::error::Error;

use crate::manifest::PluginManifest;

/// Plugin state.
#[derive(Debug, Clone, PartialEq)]
pub enum PluginState {
    Installed,
    Loaded,
    Enabled,
    Disabled,
    Error(String),
}

/// A registered plugin.
#[derive(Debug, Clone)]
pub struct PluginEntry {
    pub manifest: PluginManifest,
    pub state: PluginState,
    pub path: std::path::PathBuf,
}

/// Plugin registry managing all installed plugins.
pub struct PluginRegistry {
    plugins: HashMap<String, PluginEntry>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    /// Install a plugin from a path.
    pub fn install(&mut self, path: &std::path::Path) -> Result<&PluginEntry, Error> {
        let manifest_path = path.join("pleiades.toml");
        let manifest_content = std::fs::read_to_string(&manifest_path)
            .map_err(|e| Error::plugin(format!("Failed to read manifest: {}", e)))?;

        let manifest: PluginManifest = toml::from_str(&manifest_content)
            .map_err(|e| Error::plugin(format!("Failed to parse manifest: {}", e)))?;

        let name = manifest.plugin.name.clone();

        if self.plugins.contains_key(&name) {
            return Err(Error::plugin(format!("Plugin '{}' is already installed", name)));
        }

        let entry = PluginEntry {
            manifest,
            state: PluginState::Installed,
            path: path.to_path_buf(),
        };

        self.plugins.insert(name.clone(), entry);
        Ok(self.plugins.get(&name).unwrap())
    }

    /// Remove a plugin.
    pub fn remove(&mut self, name: &str) -> Result<(), Error> {
        self.plugins.remove(name)
            .ok_or_else(|| Error::plugin(format!("Plugin '{}' not found", name)))?;
        Ok(())
    }

    /// Enable a plugin.
    pub fn enable(&mut self, name: &str) -> Result<(), Error> {
        let entry = self.plugins.get_mut(name)
            .ok_or_else(|| Error::plugin(format!("Plugin '{}' not found", name)))?;
        entry.state = PluginState::Enabled;
        Ok(())
    }

    /// Disable a plugin.
    pub fn disable(&mut self, name: &str) -> Result<(), Error> {
        let entry = self.plugins.get_mut(name)
            .ok_or_else(|| Error::plugin(format!("Plugin '{}' not found", name)))?;
        entry.state = PluginState::Disabled;
        Ok(())
    }

    /// List all plugins.
    pub fn list(&self) -> Vec<&PluginEntry> {
        self.plugins.values().collect()
    }

    /// Get a specific plugin.
    pub fn get(&self, name: &str) -> Option<&PluginEntry> {
        self.plugins.get(name)
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}
