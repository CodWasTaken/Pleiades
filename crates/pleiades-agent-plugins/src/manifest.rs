use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub const MANIFEST_FILE_NAME: &str = "plugin.json";
pub const MANIFEST_RELATIVE_PATH: &str = ".pleiades-plugin/plugin.json";

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct PluginHooks {
    #[serde(rename = "PreToolUse", default)]
    pub pre_tool_use: Vec<String>,
    #[serde(rename = "PostToolUse", default)]
    pub post_tool_use: Vec<String>,
    #[serde(rename = "PostToolUseFailure", default)]
    pub post_tool_use_failure: Vec<String>,
}

impl PluginHooks {
    pub fn is_empty(&self) -> bool {
        self.pre_tool_use.is_empty()
            && self.post_tool_use.is_empty()
            && self.post_tool_use_failure.is_empty()
    }

    pub fn merged_with(&self, other: &Self) -> Self {
        let mut merged = self.clone();
        merged
            .pre_tool_use
            .extend(other.pre_tool_use.iter().cloned());
        merged
            .post_tool_use
            .extend(other.post_tool_use.iter().cloned());
        merged
            .post_tool_use_failure
            .extend(other.post_tool_use_failure.iter().cloned());
        merged
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginLifecycle {
    #[serde(rename = "Init", default)]
    pub init: Vec<String>,
    #[serde(rename = "Shutdown", default)]
    pub shutdown: Vec<String>,
}

impl PluginLifecycle {
    pub fn is_empty(&self) -> bool {
        self.init.is_empty() && self.shutdown.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PluginPermission {
    Read,
    Write,
    Execute,
}

impl PluginPermission {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Write => "write",
            Self::Execute => "execute",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PluginToolManifest {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: serde_json::Value,
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(rename = "requiredPermission", default = "default_tool_permission")]
    pub required_permission: PluginToolPermission,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PluginToolPermission {
    ReadOnly,
    WorkspaceWrite,
    DangerFullAccess,
}

impl PluginToolPermission {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ReadOnly => "read-only",
            Self::WorkspaceWrite => "workspace-write",
            Self::DangerFullAccess => "danger-full-access",
        }
    }
}

fn default_tool_permission() -> PluginToolPermission {
    PluginToolPermission::DangerFullAccess
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginCommandManifest {
    pub name: String,
    pub description: String,
    pub command: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    #[serde(default)]
    pub permissions: Vec<String>,
    #[serde(rename = "defaultEnabled", default)]
    pub default_enabled: bool,
    #[serde(default)]
    pub hooks: PluginHooks,
    #[serde(default)]
    pub lifecycle: PluginLifecycle,
    #[serde(default)]
    pub tools: Vec<PluginToolManifest>,
    #[serde(default)]
    pub commands: Vec<PluginCommandManifest>,
}

impl PluginManifest {
    pub fn validate(&self, root: &Path) -> Result<(), PluginError> {
        let mut errors = Vec::new();

        if self.name.trim().is_empty() {
            errors.push(PluginErrorKind::EmptyField { field: "name" });
        }
        if self.version.trim().is_empty() {
            errors.push(PluginErrorKind::EmptyField { field: "version" });
        }
        if self.description.trim().is_empty() {
            errors.push(PluginErrorKind::EmptyField {
                field: "description",
            });
        }

        let mut seen_perms = BTreeSet::new();
        for perm in &self.permissions {
            if !seen_perms.insert(perm.clone()) {
                errors.push(PluginErrorKind::DuplicatePermission {
                    permission: perm.clone(),
                });
            }
            match perm.as_str() {
                "read" | "write" | "execute" => {}
                other => errors.push(PluginErrorKind::InvalidPermission {
                    permission: other.to_string(),
                }),
            }
        }

        Self::validate_paths(root, &self.hooks.pre_tool_use, "hook", &mut errors);
        Self::validate_paths(root, &self.hooks.post_tool_use, "hook", &mut errors);
        Self::validate_paths(root, &self.hooks.post_tool_use_failure, "hook", &mut errors);
        Self::validate_paths(root, &self.lifecycle.init, "lifecycle", &mut errors);
        Self::validate_paths(root, &self.lifecycle.shutdown, "lifecycle", &mut errors);

        if !errors.is_empty() {
            return Err(PluginError::Validation(errors));
        }
        Ok(())
    }

    fn validate_paths(
        root: &Path,
        paths: &[String],
        kind: &str,
        errors: &mut Vec<PluginErrorKind>,
    ) {
        for path in paths {
            let full_path = if Path::new(path).is_absolute() {
                PathBuf::from(path)
            } else {
                root.join(path)
            };
            if !full_path.exists() {
                errors.push(PluginErrorKind::MissingPath {
                    kind: kind.to_string(),
                    path: full_path,
                });
            }
        }
    }

    pub fn load_from_directory(root: &Path) -> Result<Self, PluginError> {
        let manifest_path = find_manifest_path(root)?;
        let content = std::fs::read_to_string(&manifest_path).map_err(|e| {
            PluginError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!(
                    "plugin manifest not found at {}: {e}",
                    manifest_path.display()
                ),
            ))
        })?;

        let manifest: Self = serde_json::from_str(&content)?;
        manifest.validate(root)?;
        Ok(manifest)
    }
}

fn find_manifest_path(root: &Path) -> Result<PathBuf, PluginError> {
    let direct = root.join(MANIFEST_FILE_NAME);
    if direct.exists() {
        return Ok(direct);
    }
    let packaged = root.join(MANIFEST_RELATIVE_PATH);
    if packaged.exists() {
        return Ok(packaged);
    }
    Err(PluginError::Io(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        format!(
            "plugin manifest not found at {} or {}",
            direct.display(),
            packaged.display()
        ),
    )))
}

#[derive(Debug)]
pub enum PluginError {
    Io(std::io::Error),
    Json(serde_json::Error),
    Validation(Vec<PluginErrorKind>),
    NotFound(String),
    CommandFailed(String),
}

impl std::fmt::Display for PluginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "{e}"),
            Self::Json(e) => write!(f, "{e}"),
            Self::Validation(errors) => {
                for (i, e) in errors.iter().enumerate() {
                    if i > 0 {
                        write!(f, "; ")?;
                    }
                    write!(f, "{e}")?;
                }
                Ok(())
            }
            Self::NotFound(msg) | Self::CommandFailed(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for PluginError {}

impl From<std::io::Error> for PluginError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<serde_json::Error> for PluginError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

#[derive(Debug, Clone)]
pub enum PluginErrorKind {
    EmptyField { field: &'static str },
    InvalidPermission { permission: String },
    DuplicatePermission { permission: String },
    MissingPath { kind: String, path: PathBuf },
}

impl std::fmt::Display for PluginErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyField { field } => write!(f, "plugin manifest {field} cannot be empty"),
            Self::InvalidPermission { permission } => {
                write!(
                    f,
                    "invalid permission `{permission}`, must be read/write/execute"
                )
            }
            Self::DuplicatePermission { permission } => {
                write!(f, "duplicate permission `{permission}`")
            }
            Self::MissingPath { kind, path } => {
                write!(f, "{kind} path `{}` does not exist", path.display())
            }
        }
    }
}
