use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::manifest::{
    PluginError, PluginHooks, PluginLifecycle, PluginManifest, PluginToolPermission,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginKind {
    Builtin,
    Bundled,
    External,
}

impl PluginKind {
    pub fn marketplace(self) -> &'static str {
        match self {
            Self::Builtin => "builtin",
            Self::Bundled => "bundled",
            Self::External => "external",
        }
    }
}

impl std::fmt::Display for PluginKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Builtin => write!(f, "builtin"),
            Self::Bundled => write!(f, "bundled"),
            Self::External => write!(f, "external"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PluginMetadata {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub kind: PluginKind,
    pub source: String,
    pub default_enabled: bool,
    pub root: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct PluginTool {
    pub plugin_id: String,
    pub plugin_name: String,
    pub name: String,
    pub description: String,
    pub input_schema: Value,
    pub command: String,
    pub args: Vec<String>,
    pub required_permission: PluginToolPermission,
    pub root: Option<PathBuf>,
}

impl PluginTool {
    pub fn execute(&self, input: &Value) -> Result<String, PluginError> {
        let input_json = input.to_string();
        let mut cmd = Command::new(&self.command);
        cmd.args(&self.args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .env("PLEIADES_PLUGIN_ID", &self.plugin_id)
            .env("PLEIADES_PLUGIN_NAME", &self.plugin_name)
            .env("PLEIADES_TOOL_NAME", &self.name)
            .env("PLEIADES_TOOL_INPUT", &input_json);

        if let Some(root) = &self.root {
            cmd.current_dir(root)
                .env("PLEIADES_PLUGIN_ROOT", root.display().to_string());
        }

        let mut child = cmd.spawn()?;
        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            let _ = stdin.write_all(input_json.as_bytes());
        }

        let output = child.wait_with_output()?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            Err(PluginError::CommandFailed(format!(
                "plugin tool `{}` from `{}` failed: {}",
                self.name,
                self.plugin_id,
                if stderr.is_empty() {
                    format!("exit status {}", output.status)
                } else {
                    stderr
                },
            )))
        }
    }
}

pub trait Plugin: std::fmt::Debug {
    fn metadata(&self) -> &PluginMetadata;
    fn hooks(&self) -> &PluginHooks;
    fn lifecycle(&self) -> &PluginLifecycle;
    fn tools(&self) -> &[PluginTool];
}

#[derive(Debug, Clone)]
pub struct BuiltinPlugin {
    pub metadata: PluginMetadata,
    pub hooks: PluginHooks,
    pub lifecycle: PluginLifecycle,
    pub tools: Vec<PluginTool>,
}

impl Plugin for BuiltinPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
    fn hooks(&self) -> &PluginHooks {
        &self.hooks
    }
    fn lifecycle(&self) -> &PluginLifecycle {
        &self.lifecycle
    }
    fn tools(&self) -> &[PluginTool] {
        &self.tools
    }
}

#[derive(Debug, Clone)]
pub struct BundledPlugin {
    pub metadata: PluginMetadata,
    pub hooks: PluginHooks,
    pub lifecycle: PluginLifecycle,
    pub tools: Vec<PluginTool>,
}

impl Plugin for BundledPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
    fn hooks(&self) -> &PluginHooks {
        &self.hooks
    }
    fn lifecycle(&self) -> &PluginLifecycle {
        &self.lifecycle
    }
    fn tools(&self) -> &[PluginTool] {
        &self.tools
    }
}

#[derive(Debug, Clone)]
pub struct ExternalPlugin {
    pub metadata: PluginMetadata,
    pub hooks: PluginHooks,
    pub lifecycle: PluginLifecycle,
    pub tools: Vec<PluginTool>,
}

impl Plugin for ExternalPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
    fn hooks(&self) -> &PluginHooks {
        &self.hooks
    }
    fn lifecycle(&self) -> &PluginLifecycle {
        &self.lifecycle
    }
    fn tools(&self) -> &[PluginTool] {
        &self.tools
    }
}

#[derive(Debug, Clone)]
pub enum PluginDefinition {
    Builtin(BuiltinPlugin),
    Bundled(BundledPlugin),
    External(ExternalPlugin),
}

impl Plugin for PluginDefinition {
    fn metadata(&self) -> &PluginMetadata {
        match self {
            Self::Builtin(p) => p.metadata(),
            Self::Bundled(p) => p.metadata(),
            Self::External(p) => p.metadata(),
        }
    }

    fn hooks(&self) -> &PluginHooks {
        match self {
            Self::Builtin(p) => p.hooks(),
            Self::Bundled(p) => p.hooks(),
            Self::External(p) => p.hooks(),
        }
    }

    fn lifecycle(&self) -> &PluginLifecycle {
        match self {
            Self::Builtin(p) => p.lifecycle(),
            Self::Bundled(p) => p.lifecycle(),
            Self::External(p) => p.lifecycle(),
        }
    }

    fn tools(&self) -> &[PluginTool] {
        match self {
            Self::Builtin(p) => p.tools(),
            Self::Bundled(p) => p.tools(),
            Self::External(p) => p.tools(),
        }
    }
}

impl PluginDefinition {
    pub fn load_from_directory(
        root: &Path,
        kind: PluginKind,
        source: String,
        marketplace: &str,
    ) -> Result<Self, PluginError> {
        let manifest = PluginManifest::load_from_directory(root)?;
        let plugin_id = format!("{}-{}", manifest.name, marketplace);
        let metadata = PluginMetadata {
            id: plugin_id,
            name: manifest.name,
            version: manifest.version,
            description: manifest.description,
            kind,
            source,
            default_enabled: manifest.default_enabled,
            root: Some(root.to_path_buf()),
        };
        let hooks = manifest.hooks;
        let lifecycle = manifest.lifecycle;
        let tools = manifest
            .tools
            .into_iter()
            .map(|t| PluginTool {
                plugin_id: metadata.id.clone(),
                plugin_name: metadata.name.clone(),
                name: t.name,
                description: t.description,
                input_schema: t.input_schema,
                command: resolve_path(root, &t.command),
                args: t.args,
                required_permission: t.required_permission,
                root: Some(root.to_path_buf()),
            })
            .collect();

        Ok(match kind {
            PluginKind::Builtin => Self::Builtin(BuiltinPlugin {
                metadata,
                hooks,
                lifecycle,
                tools,
            }),
            PluginKind::Bundled => Self::Bundled(BundledPlugin {
                metadata,
                hooks,
                lifecycle,
                tools,
            }),
            PluginKind::External => Self::External(ExternalPlugin {
                metadata,
                hooks,
                lifecycle,
                tools,
            }),
        })
    }
}

fn resolve_path(root: &Path, path: &str) -> String {
    if Path::new(path).is_absolute() {
        path.to_string()
    } else {
        root.join(path).to_string_lossy().to_string()
    }
}
