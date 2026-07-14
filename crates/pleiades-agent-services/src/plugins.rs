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
}

pub struct PluginService {
    config_home: PathBuf,
}

impl PluginService {
    pub(crate) fn new(config_home: PathBuf) -> Self {
        Self { config_home }
    }

    pub fn list(&self) -> Result<Vec<PluginReport>, Error> {
        let mut reports = PluginManager::new(&self.config_home)
            .list_plugins()
            .map_err(|error| Error::plugin(error.to_string()))?
            .into_iter()
            .map(|plugin| PluginReport {
                id: plugin.id,
                name: plugin.name,
                version: plugin.version,
                description: plugin.description,
                kind: plugin.kind,
                enabled: plugin.enabled,
                tool_count: plugin.tool_count,
                has_hooks: plugin.has_hooks,
            })
            .collect::<Vec<_>>();
        reports.sort_by(|left, right| left.id.cmp(&right.id));
        Ok(reports)
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
}
