use std::path::{Path, PathBuf};

use crate::loader::format_for_path;
use crate::types::Config;

/// Manages named configuration profiles.
///
/// Profiles allow users to define multiple named configurations
/// and switch between them (e.g., "work", "personal", "default").
pub struct ProfileManager {
    profiles_dir: PathBuf,
    active_profile: Option<String>,
}

impl ProfileManager {
    /// Create a new profile manager.
    pub fn new(config_dir: &Path) -> Self {
        Self {
            profiles_dir: config_dir.join("profiles"),
            active_profile: None,
        }
    }

    /// List all available profiles.
    pub fn list(&self) -> Result<Vec<String>, String> {
        if !self.profiles_dir.exists() {
            return Ok(Vec::new());
        }

        let mut profiles = Vec::new();
        let entries = std::fs::read_dir(&self.profiles_dir)
            .map_err(|e| format!("Failed to read profiles directory: {}", e))?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(stem) = path.file_stem() {
                    if let Some(name) = stem.to_str() {
                        if !name.starts_with('.') {
                            profiles.push(name.to_string());
                        }
                    }
                }
            }
        }

        profiles.sort();
        Ok(profiles)
    }

    /// Load a profile by name.
    pub fn load(&self, name: &str) -> Result<Config, String> {
        let path = self.profile_path(name);
        if !path.exists() {
            return Err(format!("Profile '{}' not found", name));
        }

        let format = format_for_path(&path)
            .ok_or_else(|| format!("Unsupported profile format for '{}'", path.display()))?;

        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read profile '{}': {}", name, e))?;

        match format {
            "toml" => {
                toml::from_str(&content).map_err(|e| format!("Failed to parse profile TOML: {}", e))
            }
            "json" => serde_json::from_str(&content)
                .map_err(|e| format!("Failed to parse profile JSON: {}", e)),
            "yaml" => serde_yaml::from_str(&content)
                .map_err(|e| format!("Failed to parse profile YAML: {}", e)),
            _ => Err(format!("Unsupported format: {}", format)),
        }
    }

    /// Save a configuration as a named profile.
    pub fn save(&self, name: &str, config: &Config) -> Result<(), String> {
        std::fs::create_dir_all(&self.profiles_dir)
            .map_err(|e| format!("Failed to create profiles directory: {}", e))?;

        let path = self.profile_path(name);
        let toml_string = toml::to_string_pretty(config)
            .map_err(|e| format!("Failed to serialize profile: {}", e))?;

        std::fs::write(&path, &toml_string)
            .map_err(|e| format!("Failed to write profile '{}': {}", name, e))?;

        Ok(())
    }

    /// Delete a profile by name.
    pub fn delete(&self, name: &str) -> Result<(), String> {
        let path = self.profile_path(name);
        if !path.exists() {
            return Err(format!("Profile '{}' not found", name));
        }
        std::fs::remove_file(&path)
            .map_err(|e| format!("Failed to delete profile '{}': {}", name, e))
    }

    /// Set the active profile.
    pub fn set_active(&mut self, name: Option<String>) {
        self.active_profile = name;
    }

    /// Get the active profile name.
    pub fn active(&self) -> Option<&str> {
        self.active_profile.as_deref()
    }

    /// Check if a profile exists.
    pub fn exists(&self, name: &str) -> bool {
        self.profile_path(name).exists()
    }

    /// Get the path for a profile file.
    fn profile_path(&self, name: &str) -> PathBuf {
        self.profiles_dir.join(format!("{}.toml", name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_list_empty() {
        let dir = tempfile::tempdir().unwrap();
        let manager = ProfileManager::new(dir.path());
        let profiles = manager.list().unwrap();
        assert!(profiles.is_empty());
    }

    #[test]
    fn test_profile_save_and_load() {
        let dir = tempfile::tempdir().unwrap();
        let manager = ProfileManager::new(dir.path());
        let config = Config::default();

        manager.save("test", &config).unwrap();
        let loaded = manager.load("test").unwrap();

        assert_eq!(loaded.core.default_provider, config.core.default_provider);
    }

    #[test]
    fn test_profile_delete() {
        let dir = tempfile::tempdir().unwrap();
        let manager = ProfileManager::new(dir.path());
        let config = Config::default();

        manager.save("test", &config).unwrap();
        assert!(manager.exists("test"));

        manager.delete("test").unwrap();
        assert!(!manager.exists("test"));
    }

    #[test]
    fn test_profile_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let manager = ProfileManager::new(dir.path());
        assert!(manager.load("nonexistent").is_err());
    }
}
