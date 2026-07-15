use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::manifest::PluginError;
use crate::plugin::{Plugin, PluginDefinition, PluginKind};
use crate::registry::{PluginEntry, PluginRegistry};

const REGISTRY_FILE: &str = "installed.json";
const SETTINGS_FILE: &str = "settings.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstalledPluginRecord {
    pub kind: PluginKind,
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub install_path: PathBuf,
    /// Original local source used for future updates. Older registries omit it.
    #[serde(default)]
    pub source_path: Option<PathBuf>,
    pub installed_at_unix_ms: u128,
    pub updated_at_unix_ms: u128,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstalledPluginRegistry {
    #[serde(default)]
    pub plugins: BTreeMap<String, InstalledPluginRecord>,
}

#[derive(Debug, Clone)]
pub struct InstallOutcome {
    pub plugin_id: String,
    pub version: String,
    pub install_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct UpdateOutcome {
    pub plugin_id: String,
    pub old_version: String,
    pub new_version: String,
    pub install_path: PathBuf,
}

pub struct PluginManager {
    config_home: PathBuf,
    enabled_plugins: BTreeMap<String, bool>,
    trusted_plugins: BTreeMap<String, bool>,
}

impl PluginManager {
    pub fn new(config_home: impl Into<PathBuf>) -> Self {
        let config_home = config_home.into();
        let enabled_plugins = Self::load_enabled_state(&config_home);
        let trusted_plugins = Self::load_trusted_state(&config_home);
        Self {
            config_home,
            enabled_plugins,
            trusted_plugins,
        }
    }

    fn install_root(&self) -> PathBuf {
        self.config_home.join("plugins").join("installed")
    }

    fn registry_path(&self) -> PathBuf {
        self.config_home.join("plugins").join(REGISTRY_FILE)
    }

    fn settings_path(&self) -> PathBuf {
        self.config_home.join(SETTINGS_FILE)
    }

    pub fn plugin_registry(&self) -> Result<PluginRegistry, PluginError> {
        self.discover_plugins()
    }

    pub fn list_plugins(&self) -> Result<Vec<crate::registry::PluginSummary>, PluginError> {
        Ok(self.plugin_registry()?.summaries())
    }

    /// Install a plugin from a local directory path.
    pub fn install(&mut self, source: &str) -> Result<InstallOutcome, PluginError> {
        let source_path = PathBuf::from(source);
        if !source_path.is_dir() {
            return Err(PluginError::NotFound(format!(
                "plugin source `{source}` is not a valid directory"
            )));
        }

        let source_path = source_path.canonicalize()?;
        let manifest = crate::manifest::PluginManifest::load_from_directory(&source_path)?;
        let plugin_id = format!("{}-external", manifest.name);
        let install_path = self.install_root().join(&plugin_id);

        if install_path.exists() {
            std::fs::remove_dir_all(&install_path)?;
        }
        std::fs::create_dir_all(&install_path)?;
        copy_dir_recursive(&source_path, &install_path)?;

        let now = unix_ms();
        let record = InstalledPluginRecord {
            kind: PluginKind::External,
            id: plugin_id.clone(),
            name: manifest.name,
            version: manifest.version.clone(),
            description: manifest.description,
            install_path: install_path.clone(),
            source_path: Some(source_path),
            installed_at_unix_ms: now,
            updated_at_unix_ms: now,
        };

        let mut registry = self.load_registry()?;
        registry.plugins.insert(plugin_id.clone(), record);
        self.store_registry(&registry)?;
        self.set_enabled(&plugin_id, Some(false))?;
        self.enabled_plugins.insert(plugin_id.clone(), false);

        Ok(InstallOutcome {
            plugin_id,
            version: manifest.version,
            install_path,
        })
    }

    /// Refresh an external plugin from the source used during installation.
    /// New content is validated in a staging directory before replacing the
    /// installed copy, so a malformed update leaves the active plugin intact.
    pub fn update(&mut self, plugin_id: &str) -> Result<UpdateOutcome, PluginError> {
        let mut registry = self.load_registry()?;
        let record = registry.plugins.get(plugin_id).cloned().ok_or_else(|| {
            PluginError::NotFound(format!("plugin `{plugin_id}` is not installed"))
        })?;
        if record.kind != PluginKind::External {
            return Err(PluginError::CommandFailed(format!(
                "plugin `{plugin_id}` is not external and cannot be updated"
            )));
        }
        let source = record.source_path.as_ref().ok_or_else(|| {
            PluginError::CommandFailed(format!(
                "plugin `{plugin_id}` predates source tracking; reinstall it before updating"
            ))
        })?;
        if !source.is_dir() {
            return Err(PluginError::NotFound(format!(
                "plugin source `{}` is not a valid directory",
                source.display()
            )));
        }

        let manifest = crate::manifest::PluginManifest::load_from_directory(source)?;
        let expected_id = format!("{}-external", manifest.name);
        if expected_id != plugin_id {
            return Err(PluginError::CommandFailed(format!(
                "updated manifest resolves to `{expected_id}`, expected `{plugin_id}`"
            )));
        }

        let staging = record.install_path.with_extension("update-staging");
        let backup = record.install_path.with_extension("update-backup");
        remove_dir_if_exists(&staging)?;
        remove_dir_if_exists(&backup)?;
        std::fs::create_dir_all(&staging)?;
        if let Err(error) = copy_dir_recursive(source, &staging).and_then(|_| {
            crate::manifest::PluginManifest::load_from_directory(&staging)
                .map(|_| ())
                .map_err(plugin_error_to_io)
        }) {
            let _ = std::fs::remove_dir_all(&staging);
            return Err(PluginError::Io(error));
        }

        std::fs::rename(&record.install_path, &backup)?;
        if let Err(error) = std::fs::rename(&staging, &record.install_path) {
            let _ = std::fs::rename(&backup, &record.install_path);
            return Err(PluginError::Io(error));
        }

        let mut updated = record.clone();
        updated.name = manifest.name;
        updated.version = manifest.version.clone();
        updated.description = manifest.description;
        updated.updated_at_unix_ms = unix_ms();
        registry.plugins.insert(plugin_id.to_string(), updated);
        if let Err(error) = self.store_registry(&registry) {
            let _ = std::fs::remove_dir_all(&record.install_path);
            let _ = std::fs::rename(&backup, &record.install_path);
            return Err(error);
        }
        remove_dir_if_exists(&backup)?;

        Ok(UpdateOutcome {
            plugin_id: plugin_id.to_string(),
            old_version: record.version,
            new_version: manifest.version,
            install_path: record.install_path,
        })
    }

    /// Uninstall a plugin.
    pub fn uninstall(&mut self, plugin_id: &str) -> Result<(), PluginError> {
        let mut registry = self.load_registry()?;
        let record = registry.plugins.remove(plugin_id).ok_or_else(|| {
            PluginError::NotFound(format!("plugin `{plugin_id}` is not installed"))
        })?;

        if record.kind == PluginKind::Bundled {
            registry.plugins.insert(plugin_id.to_string(), record);
            return Err(PluginError::CommandFailed(format!(
                "plugin `{plugin_id}` is bundled; disable it instead"
            )));
        }

        if record.install_path.exists() {
            std::fs::remove_dir_all(&record.install_path)?;
        }
        self.store_registry(&registry)?;
        self.set_enabled(plugin_id, None)?;
        self.set_trusted(plugin_id, None)?;
        self.enabled_plugins.remove(plugin_id);
        self.trusted_plugins.remove(plugin_id);
        Ok(())
    }

    /// Enable a plugin.
    pub fn enable(&mut self, plugin_id: &str) -> Result<(), PluginError> {
        let plugin = self.ensure_known(plugin_id)?;
        if plugin.metadata().kind == PluginKind::External && !self.is_trusted(plugin_id) {
            return Err(PluginError::CommandFailed(format!(
                "plugin `{plugin_id}` must be trusted before it can be enabled"
            )));
        }
        self.set_enabled(plugin_id, Some(true))?;
        self.enabled_plugins.insert(plugin_id.to_string(), true);
        Ok(())
    }

    /// Mark a plugin as explicitly trusted.
    pub fn trust(&mut self, plugin_id: &str) -> Result<(), PluginError> {
        self.ensure_known(plugin_id)?;
        self.set_trusted(plugin_id, Some(true))?;
        self.trusted_plugins.insert(plugin_id.to_string(), true);
        Ok(())
    }

    /// Revoke a plugin trust decision and disable it.
    pub fn untrust(&mut self, plugin_id: &str) -> Result<(), PluginError> {
        self.ensure_known(plugin_id)?;
        self.set_enabled(plugin_id, Some(false))?;
        self.enabled_plugins.insert(plugin_id.to_string(), false);
        self.set_trusted(plugin_id, Some(false))?;
        self.trusted_plugins.insert(plugin_id.to_string(), false);
        Ok(())
    }

    /// Disable a plugin.
    pub fn disable(&mut self, plugin_id: &str) -> Result<(), PluginError> {
        self.ensure_known(plugin_id)?;
        self.set_enabled(plugin_id, Some(false))?;
        self.enabled_plugins.insert(plugin_id.to_string(), false);
        Ok(())
    }

    pub fn is_trusted(&self, plugin_id: &str) -> bool {
        self.trusted_plugins
            .get(plugin_id)
            .copied()
            .unwrap_or(false)
    }

    fn ensure_known(&self, plugin_id: &str) -> Result<PluginEntry, PluginError> {
        self.plugin_registry()?
            .get(plugin_id)
            .cloned()
            .ok_or_else(|| PluginError::NotFound(format!("plugin `{plugin_id}` is not installed")))
    }

    fn discover_plugins(&self) -> Result<PluginRegistry, PluginError> {
        let registry = self.load_registry()?;
        let mut entries = Vec::new();

        // Discover builtin plugins
        entries.push(self.builtin_entry());

        // Discover installed plugins
        for record in registry.plugins.values() {
            if !record.install_path.exists() {
                continue;
            }
            match PluginDefinition::load_from_directory(
                &record.install_path,
                record.kind,
                record.install_path.display().to_string(),
                record.kind.marketplace(),
            ) {
                Ok(def) => {
                    let enabled = self.is_enabled(def.metadata());
                    entries.push(PluginEntry {
                        definition: def,
                        enabled,
                    });
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to load plugin `{}`: {e}",
                        record.install_path.display()
                    );
                }
            }
        }

        Ok(PluginRegistry::new(entries))
    }

    fn builtin_entry(&self) -> PluginEntry {
        let metadata = crate::plugin::PluginMetadata {
            id: "pleiades-agent-core-builtin".to_string(),
            name: "pleiades-agent-core".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "Core Pleiades built-in capabilities".to_string(),
            kind: PluginKind::Builtin,
            source: "builtin".to_string(),
            default_enabled: true,
            root: None,
            requested_paths: Vec::new(),
            env_vars: Vec::new(),
            network: None,
            checksum: None,
            signature: None,
            commands: Vec::new(),
        };
        let enabled = self
            .enabled_plugins
            .get(&metadata.id)
            .copied()
            .unwrap_or(true);
        PluginEntry {
            definition: PluginDefinition::Builtin(crate::plugin::BuiltinPlugin {
                metadata,
                hooks: crate::manifest::PluginHooks::default(),
                lifecycle: crate::manifest::PluginLifecycle::default(),
                tools: Vec::new(),
            }),
            enabled,
        }
    }

    fn is_enabled(&self, metadata: &crate::plugin::PluginMetadata) -> bool {
        self.enabled_plugins
            .get(&metadata.id)
            .copied()
            .unwrap_or(match metadata.kind {
                PluginKind::External => false,
                PluginKind::Builtin | PluginKind::Bundled => metadata.default_enabled,
            })
    }

    fn load_registry(&self) -> Result<InstalledPluginRegistry, PluginError> {
        let path = self.registry_path();
        match std::fs::read_to_string(&path) {
            Ok(content) if content.trim().is_empty() => Ok(InstalledPluginRegistry::default()),
            Ok(content) => Ok(serde_json::from_str(&content)?),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                Ok(InstalledPluginRegistry::default())
            }
            Err(e) => Err(PluginError::Io(e)),
        }
    }

    fn store_registry(&self, registry: &InstalledPluginRegistry) -> Result<(), PluginError> {
        let path = self.registry_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, serde_json::to_string_pretty(registry)?)?;
        Ok(())
    }

    fn set_enabled(&self, plugin_id: &str, enabled: Option<bool>) -> Result<(), PluginError> {
        self.set_bool_setting("enabledPlugins", plugin_id, enabled)
    }

    fn set_trusted(&self, plugin_id: &str, trusted: Option<bool>) -> Result<(), PluginError> {
        self.set_bool_setting("trustedPlugins", plugin_id, trusted)
    }

    fn set_bool_setting(
        &self,
        key: &str,
        plugin_id: &str,
        value: Option<bool>,
    ) -> Result<(), PluginError> {
        let path = self.settings_path();
        let mut settings: serde_json::Value = std::fs::read_to_string(&path)
            .ok()
            .and_then(|c| serde_json::from_str(&c).ok())
            .unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new()));

        let entries = settings
            .as_object_mut()
            .unwrap()
            .entry(key)
            .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()))
            .as_object_mut()
            .unwrap();

        match value {
            Some(val) => {
                entries.insert(plugin_id.to_string(), serde_json::json!(val));
            }
            None => {
                entries.remove(plugin_id);
            }
        }

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, serde_json::to_string_pretty(&settings)?)?;
        Ok(())
    }

    fn load_enabled_state(config_home: &Path) -> BTreeMap<String, bool> {
        let path = config_home.join(SETTINGS_FILE);
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => return BTreeMap::new(),
        };
        let settings: serde_json::Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(_) => return BTreeMap::new(),
        };
        let mut map = BTreeMap::new();
        if let Some(enabled) = settings.get("enabledPlugins").and_then(|v| v.as_object()) {
            for (id, val) in enabled {
                if let Some(state) = val.as_bool() {
                    map.insert(id.clone(), state);
                }
            }
        }
        map
    }

    fn load_trusted_state(config_home: &Path) -> BTreeMap<String, bool> {
        Self::load_bool_state(config_home, "trustedPlugins")
    }

    fn load_bool_state(config_home: &Path, key: &str) -> BTreeMap<String, bool> {
        let path = config_home.join(SETTINGS_FILE);
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => return BTreeMap::new(),
        };
        let settings: serde_json::Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(_) => return BTreeMap::new(),
        };
        let mut map = BTreeMap::new();
        if let Some(entries) = settings.get(key).and_then(|v| v.as_object()) {
            for (id, val) in entries {
                if let Some(state) = val.as_bool() {
                    map.insert(id.clone(), state);
                }
            }
        }
        map
    }
}

fn plugin_error_to_io(error: PluginError) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, error.to_string())
}

fn remove_dir_if_exists(path: &Path) -> Result<(), std::io::Error> {
    match std::fs::remove_dir_all(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), std::io::Error> {
    for entry in walkdir::WalkDir::new(src).min_depth(1) {
        let entry = entry?;
        let relative = entry.path().strip_prefix(src).unwrap();
        let target = dst.join(relative);
        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&target)?;
        } else {
            std::fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

fn unix_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write_test_plugin(dir: &Path, name: &str, enabled: Option<bool>) {
        let plugin_dir = dir.join(name);
        std::fs::create_dir_all(plugin_dir.join(".pleiades-plugin")).unwrap();
        std::fs::create_dir_all(plugin_dir.join("hooks")).unwrap();
        std::fs::write(
            plugin_dir.join("hooks").join("pre.sh"),
            "#!/bin/sh\nprintf 'ok'\n",
        )
        .unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(
                plugin_dir.join("hooks").join("pre.sh"),
                std::fs::Permissions::from_mode(0o755),
            )
            .ok();
        }
        std::fs::write(
            plugin_dir.join(".pleiades-plugin").join("plugin.json"),
            serde_json::to_string_pretty(&serde_json::json!({
                "name": name,
                "version": "1.0.0",
                "description": format!("test plugin {}", name),
                "defaultEnabled": enabled.unwrap_or(false),
                "hooks": {
                    "PreToolUse": ["./hooks/pre.sh"]
                }
            }))
            .unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn install_and_list_plugin() {
        let tmp = TempDir::new().unwrap();
        let config_home = tmp.path().join("config");
        write_test_plugin(tmp.path(), "my-plugin", Some(true));

        let mut manager = PluginManager::new(&config_home);
        let outcome = manager
            .install(tmp.path().join("my-plugin").to_str().unwrap())
            .expect("install should succeed");

        assert_eq!(outcome.plugin_id, "my-plugin-external");

        let plugins = manager.list_plugins().expect("list should succeed");
        let plugin = plugins.iter().find(|p| p.id == outcome.plugin_id);
        assert!(plugin.is_some());
        let p = plugin.unwrap();
        assert_eq!(p.name, "my-plugin");
        assert!(!p.enabled);
    }

    #[test]
    fn update_replaces_valid_content_and_preserves_enabled_state() {
        let tmp = TempDir::new().unwrap();
        let config_home = tmp.path().join("config");
        write_test_plugin(tmp.path(), "updated-plugin", Some(true));
        let source = tmp.path().join("updated-plugin");
        let mut manager = PluginManager::new(&config_home);
        let installed = manager.install(source.to_str().unwrap()).unwrap();
        manager.disable(&installed.plugin_id).unwrap();

        let manifest_path = source.join(".pleiades-plugin/plugin.json");
        let mut manifest: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&manifest_path).unwrap()).unwrap();
        manifest["version"] = serde_json::json!("2.0.0");
        std::fs::write(
            &manifest_path,
            serde_json::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();

        let outcome = manager.update(&installed.plugin_id).unwrap();
        assert_eq!(outcome.old_version, "1.0.0");
        assert_eq!(outcome.new_version, "2.0.0");
        let plugin = manager
            .plugin_registry()
            .unwrap()
            .plugins()
            .iter()
            .find(|plugin| plugin.metadata().id == installed.plugin_id)
            .unwrap()
            .clone();
        assert_eq!(plugin.metadata().version, "2.0.0");
        assert!(!plugin.enabled);
    }

    #[test]
    fn invalid_update_keeps_installed_content() {
        let tmp = TempDir::new().unwrap();
        let config_home = tmp.path().join("config");
        write_test_plugin(tmp.path(), "safe-plugin", Some(true));
        let source = tmp.path().join("safe-plugin");
        let mut manager = PluginManager::new(&config_home);
        let installed = manager.install(source.to_str().unwrap()).unwrap();

        std::fs::write(source.join(".pleiades-plugin/plugin.json"), "not json").unwrap();
        assert!(manager.update(&installed.plugin_id).is_err());
        let plugin = manager
            .plugin_registry()
            .unwrap()
            .plugins()
            .iter()
            .find(|plugin| plugin.metadata().id == installed.plugin_id)
            .unwrap()
            .clone();
        assert_eq!(plugin.metadata().version, "1.0.0");
    }

    #[test]
    fn enable_disable_plugin() {
        let tmp = TempDir::new().unwrap();
        let config_home = tmp.path().join("config");
        write_test_plugin(tmp.path(), "test-plugin", Some(false));

        let mut manager = PluginManager::new(&config_home);
        manager
            .install(tmp.path().join("test-plugin").to_str().unwrap())
            .expect("install should succeed");

        let plugin_id = "test-plugin-external";

        manager.disable(plugin_id).expect("disable should succeed");
        let plugins = manager.list_plugins().expect("list should succeed");
        let p = plugins.iter().find(|p| p.id == plugin_id).unwrap();
        assert!(!p.enabled);

        assert!(manager.enable(plugin_id).is_err());
        manager.trust(plugin_id).expect("trust should succeed");
        manager.enable(plugin_id).expect("enable should succeed");
        let plugins = manager.list_plugins().expect("list should succeed");
        let p = plugins.iter().find(|p| p.id == plugin_id).unwrap();
        assert!(p.enabled);
    }

    #[test]
    fn builtin_plugin_honors_enabled_state() {
        let tmp = TempDir::new().unwrap();
        let mut manager = PluginManager::new(tmp.path());
        let id = "pleiades-agent-core-builtin";
        manager.disable(id).unwrap();
        assert!(!manager.plugin_registry().unwrap().get(id).unwrap().enabled);
        manager.enable(id).unwrap();
        assert!(manager.plugin_registry().unwrap().get(id).unwrap().enabled);
    }

    #[test]
    fn uninstall_removes_plugin() {
        let tmp = TempDir::new().unwrap();
        let config_home = tmp.path().join("config");
        write_test_plugin(tmp.path(), "remove-me", None);

        let mut manager = PluginManager::new(&config_home);
        manager
            .install(tmp.path().join("remove-me").to_str().unwrap())
            .expect("install should succeed");

        let plugin_id = "remove-me-external";
        assert!(manager.plugin_registry().unwrap().contains(plugin_id));

        manager
            .uninstall(plugin_id)
            .expect("uninstall should succeed");
        assert!(!manager.plugin_registry().unwrap().contains(plugin_id));
    }

    #[test]
    fn plugin_registry_aggregates_hooks() {
        let tmp = TempDir::new().unwrap();
        let config_home = tmp.path().join("config");
        write_test_plugin(tmp.path(), "alpha", Some(true));
        write_test_plugin(tmp.path(), "beta", Some(true));

        let mut manager = PluginManager::new(&config_home);
        manager
            .install(tmp.path().join("alpha").to_str().unwrap())
            .expect("install alpha");
        manager.trust("alpha-external").expect("trust alpha");
        manager.enable("alpha-external").expect("enable alpha");
        manager
            .install(tmp.path().join("beta").to_str().unwrap())
            .expect("install beta");
        manager.trust("beta-external").expect("trust beta");
        manager.enable("beta-external").expect("enable beta");

        let registry = manager.plugin_registry().expect("registry");
        let hooks = registry.aggregated_hooks();
        assert_eq!(hooks.pre_tool_use.len(), 2);
    }
}
