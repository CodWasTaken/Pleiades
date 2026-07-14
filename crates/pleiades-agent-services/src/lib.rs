//! Shared application services used by every Pleiades frontend.
//!
//! Services return typed reports and domain errors. They do not print, render
//! Ratatui widgets, or parse Clap arguments. The CLI and live workspace can
//! therefore present the same operation without duplicating business logic.

mod models;
mod plugins;
mod providers;

use std::path::PathBuf;

pub use models::{ModelDiscoveryReport, ModelPreferences, ModelProviderResult, ModelService};
pub use plugins::{PluginInstallReport, PluginReport, PluginService};
pub use providers::{ProviderFactory, ProviderReport, ProviderService, ProviderTestReport};

/// Root service container shared by headless and interactive frontends.
#[derive(Debug, Clone)]
pub struct ApplicationServices {
    global_config_dir: PathBuf,
    project_config_dir: PathBuf,
}

impl ApplicationServices {
    /// Use platform-default global configuration and `./.pleiades` project
    /// configuration.
    pub fn new() -> Self {
        let loader = pleiades_agent_config::ConfigLoader::new();
        Self::with_config_dirs(
            loader.global_dir().to_path_buf(),
            loader.project_dir().to_path_buf(),
        )
    }

    /// Construct services for explicit configuration roots. This is used by
    /// tests and embedders that must not touch a user's real configuration.
    pub fn with_config_dirs(global: PathBuf, project: PathBuf) -> Self {
        Self {
            global_config_dir: global,
            project_config_dir: project,
        }
    }

    pub fn provider(&self) -> ProviderService {
        ProviderService::new(self.loader())
    }

    pub fn plugin(&self) -> PluginService {
        PluginService::new(self.global_config_dir.clone())
    }

    pub fn model(&self) -> ModelService {
        ModelService::new(self.loader())
    }

    pub fn loader(&self) -> pleiades_agent_config::ConfigLoader {
        pleiades_agent_config::ConfigLoader::with_dirs(
            self.global_config_dir.clone(),
            self.project_config_dir.clone(),
        )
    }
}

impl Default for ApplicationServices {
    fn default() -> Self {
        Self::new()
    }
}
