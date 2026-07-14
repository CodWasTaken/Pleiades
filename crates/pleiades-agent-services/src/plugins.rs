use std::path::PathBuf;

use pleiades_agent_core::Error;
use pleiades_agent_plugins::{PluginKind, PluginManager};

/// Plugin metadata shared by CLI and TUI renderers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginReport {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub kind: PluginKind,
    pub enabled: bool,
    pub tool_count: usize,
    pub has_hooks: bool,
    pub source: String,
    pub permissions: Vec<String>,
}

/// Outcome of installing a local plugin.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginInstallReport {
    pub id: String,
    pub version: String,
    pub install_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginUpdateReport {
    pub id: String,
    pub old_version: String,
    pub new_version: String,
    pub install_path: PathBuf,
}

pub struct PluginService {
    config_home: PathBuf,
}

impl PluginService {
    pub(crate) fn new(config_home: PathBuf) -> Self {
        Self { config_home }
    }

    pub fn list(&self) -> Result<Vec<PluginReport>, Error> {
        let registry = PluginManager::new(&self.config_home)
            .plugin_registry()
            .map_err(|error| Error::plugin(error.to_string()))?;
        let mut reports = registry
            .plugins()
            .iter()
            .map(|plugin| {
                let mut permissions = plugin
                    .tools()
                    .iter()
                    .map(|tool| format!("tool:{}:{}", tool.name, tool.required_permission.as_str()))
                    .collect::<Vec<_>>();
                if !plugin.hooks().is_empty() {
                    permissions.push("executable-hooks".to_string());
                }
                PluginReport {
                    id: plugin.metadata().id.clone(),
                    name: plugin.metadata().name.clone(),
                    version: plugin.metadata().version.clone(),
                    description: plugin.metadata().description.clone(),
                    kind: plugin.metadata().kind,
                    enabled: plugin.enabled,
                    tool_count: plugin.tools().len(),
                    has_hooks: !plugin.hooks().is_empty(),
                    source: plugin.metadata().source.clone(),
                    permissions,
                }
            })
            .collect::<Vec<_>>();
        reports.sort_by(|left, right| left.id.cmp(&right.id));
        Ok(reports)
    }

    pub fn info(&self, id: &str) -> Result<PluginReport, Error> {
        self.list()?
            .into_iter()
            .find(|plugin| plugin.id == id)
            .ok_or_else(|| Error::plugin(format!("plugin `{id}` is not installed")))
    }

    pub fn install(&self, source: &str) -> Result<PluginInstallReport, Error> {
        let outcome = PluginManager::new(&self.config_home)
            .install(source)
            .map_err(|error| Error::plugin(error.to_string()))?;
        Ok(PluginInstallReport {
            id: outcome.plugin_id,
            version: outcome.version,
            install_path: outcome.install_path,
        })
    }

    pub fn uninstall(&self, id: &str) -> Result<(), Error> {
        PluginManager::new(&self.config_home)
            .uninstall(id)
            .map_err(|error| Error::plugin(error.to_string()))
    }

    pub fn enable(&self, id: &str) -> Result<(), Error> {
        PluginManager::new(&self.config_home)
            .enable(id)
            .map_err(|error| Error::plugin(error.to_string()))
    }

    pub fn disable(&self, id: &str) -> Result<(), Error> {
        PluginManager::new(&self.config_home)
            .disable(id)
            .map_err(|error| Error::plugin(error.to_string()))
    }

    pub fn update(&self, id: &str) -> Result<PluginUpdateReport, Error> {
        let outcome = PluginManager::new(&self.config_home)
            .update(id)
            .map_err(|error| Error::plugin(error.to_string()))?;
        Ok(PluginUpdateReport {
            id: outcome.plugin_id,
            old_version: outcome.old_version,
            new_version: outcome.new_version,
            install_path: outcome.install_path,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::PluginService;

    #[test]
    fn builtin_plugin_is_available_through_the_service() {
        let temp = tempfile::tempdir().unwrap();
        let reports = PluginService::new(temp.path().to_path_buf())
            .list()
            .unwrap();
        assert!(
            reports
                .iter()
                .any(|plugin| { plugin.id == "pleiades-agent-core-builtin" && plugin.enabled })
        );
    }

    #[test]
    fn enable_and_disable_are_reflected_in_reports() {
        let temp = tempfile::tempdir().unwrap();
        let service = PluginService::new(temp.path().to_path_buf());
        service.disable("pleiades-agent-core-builtin").unwrap();
        assert!(!service.info("pleiades-agent-core-builtin").unwrap().enabled);
        service.enable("pleiades-agent-core-builtin").unwrap();
        assert!(service.info("pleiades-agent-core-builtin").unwrap().enabled);
    }
}
