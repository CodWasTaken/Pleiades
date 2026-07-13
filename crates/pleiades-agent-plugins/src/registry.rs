use std::collections::BTreeMap;

use crate::hooks::HookRunner;
use crate::manifest::{PluginError, PluginHooks};
use crate::plugin::{Plugin, PluginDefinition, PluginKind, PluginMetadata, PluginTool};

#[derive(Debug, Clone)]
pub struct PluginEntry {
    pub definition: PluginDefinition,
    pub enabled: bool,
}

impl PluginEntry {
    pub fn metadata(&self) -> &PluginMetadata {
        self.definition.metadata()
    }

    pub fn hooks(&self) -> &PluginHooks {
        self.definition.hooks()
    }

    pub fn tools(&self) -> &[PluginTool] {
        self.definition.tools()
    }
}

#[derive(Debug, Clone)]
pub struct PluginRegistry {
    plugins: Vec<PluginEntry>,
}

impl PluginRegistry {
    pub fn new(plugins: Vec<PluginEntry>) -> Self {
        Self { plugins }
    }

    pub fn plugins(&self) -> &[PluginEntry] {
        &self.plugins
    }

    pub fn get(&self, plugin_id: &str) -> Option<&PluginEntry> {
        self.plugins.iter().find(|p| p.metadata().id == plugin_id)
    }

    pub fn contains(&self, plugin_id: &str) -> bool {
        self.get(plugin_id).is_some()
    }

    pub fn enabled_plugins(&self) -> impl Iterator<Item = &PluginEntry> {
        self.plugins.iter().filter(|p| p.enabled)
    }

    pub fn aggregated_hooks(&self) -> PluginHooks {
        self.enabled_plugins()
            .map(|p| p.hooks())
            .fold(PluginHooks::default(), |acc, hooks| acc.merged_with(hooks))
    }

    pub fn hook_runner(&self) -> HookRunner {
        HookRunner::new(self.aggregated_hooks())
    }

    pub fn aggregated_tools(&self) -> Result<Vec<PluginTool>, PluginError> {
        let mut tools = Vec::new();
        let mut seen_names = BTreeMap::new();
        for entry in self.enabled_plugins() {
            for tool in entry.tools() {
                if let Some(existing) = seen_names.insert(tool.name.clone(), tool.plugin_id.clone())
                {
                    return Err(PluginError::CommandFailed(format!(
                        "tool `{}` is defined by both `{}` and `{}`",
                        tool.name, existing, tool.plugin_id
                    )));
                }
                tools.push(tool.clone());
            }
        }
        Ok(tools)
    }

    pub fn summaries(&self) -> Vec<PluginSummary> {
        self.plugins
            .iter()
            .map(|p| PluginSummary {
                id: p.metadata().id.clone(),
                name: p.metadata().name.clone(),
                version: p.metadata().version.clone(),
                description: p.metadata().description.clone(),
                kind: p.metadata().kind,
                enabled: p.enabled,
                tool_count: p.tools().len(),
                has_hooks: !p.hooks().is_empty(),
            })
            .collect()
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

#[derive(Debug, Clone)]
pub struct PluginSummary {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub kind: PluginKind,
    pub enabled: bool,
    pub tool_count: usize,
    pub has_hooks: bool,
}
