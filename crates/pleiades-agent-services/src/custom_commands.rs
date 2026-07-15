use std::path::{Path, PathBuf};

use pleiades_agent_core::Error;
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomArgumentReport {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub default: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomCommandDefinition {
    pub name: String,
    pub path: Vec<String>,
    pub aliases: Vec<String>,
    pub description: String,
    pub prompt: String,
    pub arguments: Vec<CustomArgumentReport>,
    pub permission: String,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub skills: Vec<String>,
    pub workflow: Option<String>,
    pub background: bool,
    pub scope: String,
    pub source: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomCommandReport {
    pub name: String,
    pub path: Vec<String>,
    pub description: String,
    pub scope: String,
    pub source: PathBuf,
    pub valid: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct CustomCommandFile {
    name: Option<String>,
    path: Option<Vec<String>>,
    #[serde(default)]
    aliases: Vec<String>,
    #[serde(default)]
    description: String,
    prompt: Option<String>,
    #[serde(default)]
    arguments: Vec<CustomArgumentFile>,
    #[serde(default = "default_permission")]
    permission: String,
    provider: Option<String>,
    model: Option<String>,
    #[serde(default)]
    skills: Vec<String>,
    workflow: Option<String>,
    #[serde(default)]
    background: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct CustomArgumentFile {
    name: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    required: bool,
    default: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CustomCommandService {
    global_dir: PathBuf,
    project_dir: PathBuf,
}

impl CustomCommandService {
    pub(crate) fn new(global_dir: PathBuf, project_dir: PathBuf) -> Self {
        Self {
            global_dir,
            project_dir,
        }
    }

    pub fn list(&self) -> Result<Vec<CustomCommandReport>, Error> {
        let mut reports = Vec::new();
        reports.extend(self.read_reports(&self.global_dir, "global")?);
        reports.extend(self.read_reports(&self.project_dir, "project")?);
        reports.sort_by(|left, right| {
            left.path
                .cmp(&right.path)
                .then_with(|| left.scope.cmp(&right.scope))
        });
        Ok(reports)
    }

    pub fn definitions(&self) -> Result<Vec<CustomCommandDefinition>, Error> {
        let mut definitions = Vec::new();
        definitions.extend(self.read_definitions(&self.global_dir, "global")?);
        definitions.extend(self.read_definitions(&self.project_dir, "project")?);
        definitions.sort_by(|left, right| {
            left.path
                .cmp(&right.path)
                .then_with(|| left.scope.cmp(&right.scope))
        });
        Ok(definitions)
    }

    pub fn show(&self, name: &str) -> Result<CustomCommandDefinition, Error> {
        self.definitions()?
            .into_iter()
            .find(|definition| definition.name == name || definition.path.join(" ") == name)
            .ok_or_else(|| Error::invalid_input(format!("custom command `{name}` not found")))
    }

    fn read_reports(&self, dir: &Path, scope: &str) -> Result<Vec<CustomCommandReport>, Error> {
        let mut reports = Vec::new();
        for path in command_files(dir)? {
            match read_definition(&path, scope) {
                Ok(definition) => reports.push(CustomCommandReport {
                    name: definition.name,
                    path: definition.path,
                    description: definition.description,
                    scope: scope.to_string(),
                    source: path,
                    valid: true,
                    error: None,
                }),
                Err(error) => reports.push(CustomCommandReport {
                    name: fallback_name(&path),
                    path: vec![fallback_name(&path)],
                    description: String::new(),
                    scope: scope.to_string(),
                    source: path,
                    valid: false,
                    error: Some(error.to_string()),
                }),
            }
        }
        Ok(reports)
    }

    fn read_definitions(
        &self,
        dir: &Path,
        scope: &str,
    ) -> Result<Vec<CustomCommandDefinition>, Error> {
        let mut definitions = Vec::new();
        for path in command_files(dir)? {
            if let Ok(definition) = read_definition(&path, scope) {
                definitions.push(definition);
            }
        }
        Ok(definitions)
    }
}

fn default_permission() -> String {
    "read".to_string()
}

fn command_files(dir: &Path) -> Result<Vec<PathBuf>, Error> {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(Error::from(error)),
    };
    let mut paths = Vec::new();
    for entry in entries {
        let path = entry.map_err(Error::from)?.path();
        if path
            .extension()
            .is_some_and(|extension| extension == "toml")
        {
            paths.push(path);
        }
    }
    paths.sort();
    Ok(paths)
}

fn read_definition(path: &Path, scope: &str) -> Result<CustomCommandDefinition, Error> {
    let text = std::fs::read_to_string(path).map_err(Error::from)?;
    let file: CustomCommandFile =
        toml::from_str(&text).map_err(|error| Error::config(error.to_string()))?;
    let name = file.name.unwrap_or_else(|| fallback_name(path));
    validate_segment(&name)?;
    let command_path = file.path.unwrap_or_else(|| vec![name.clone()]);
    if command_path.is_empty() {
        return Err(Error::invalid_input("custom command path cannot be empty"));
    }
    for segment in &command_path {
        validate_segment(segment)?;
    }
    for alias in &file.aliases {
        validate_alias(alias)?;
    }
    let prompt = file
        .prompt
        .ok_or_else(|| Error::invalid_input("custom command requires a prompt"))?;
    if prompt.trim().is_empty() {
        return Err(Error::invalid_input(
            "custom command prompt cannot be empty",
        ));
    }
    let permission = file.permission.to_ascii_lowercase();
    if !matches!(permission.as_str(), "none" | "read" | "write" | "dangerous") {
        return Err(Error::invalid_input(
            "custom command permission must be none, read, write, or dangerous",
        ));
    }
    let arguments = file
        .arguments
        .into_iter()
        .map(|argument| {
            validate_segment(&argument.name)?;
            Ok(CustomArgumentReport {
                name: argument.name,
                description: argument.description,
                required: argument.required,
                default: argument.default,
            })
        })
        .collect::<Result<Vec<_>, Error>>()?;

    Ok(CustomCommandDefinition {
        name,
        path: command_path,
        aliases: file.aliases,
        description: file.description,
        prompt,
        arguments,
        permission,
        provider: file.provider,
        model: file.model,
        skills: file.skills,
        workflow: file.workflow,
        background: file.background,
        scope: scope.to_string(),
        source: path.to_path_buf(),
    })
}

fn fallback_name(path: &Path) -> String {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("command")
        .to_string()
}

fn validate_segment(value: &str) -> Result<(), Error> {
    if value.trim().is_empty()
        || !value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
    {
        return Err(Error::invalid_input(format!(
            "invalid custom command segment `{value}`"
        )));
    }
    Ok(())
}

fn validate_alias(alias: &str) -> Result<(), Error> {
    if alias.trim().is_empty() {
        return Err(Error::invalid_input("custom command alias cannot be empty"));
    }
    for segment in alias.split_whitespace() {
        validate_segment(segment)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::CustomCommandService;

    #[test]
    fn lists_valid_and_invalid_command_files_without_crashing() {
        let temp = tempfile::tempdir().unwrap();
        let project = temp.path().join("project");
        std::fs::create_dir_all(&project).unwrap();
        std::fs::write(
            project.join("release.toml"),
            r#"
description = "Prepare a release"
prompt = "Prepare release {{version}}"

[[arguments]]
name = "version"
required = true
"#,
        )
        .unwrap();
        std::fs::write(project.join("broken.toml"), "description = 1").unwrap();

        let service = CustomCommandService::new(temp.path().join("global"), project);
        let reports = service.list().unwrap();
        assert_eq!(reports.len(), 2);
        assert!(reports.iter().any(|report| report.valid));
        assert!(reports.iter().any(|report| !report.valid));

        let definitions = service.definitions().unwrap();
        assert_eq!(definitions.len(), 1);
        assert_eq!(definitions[0].path, vec!["release"]);
    }
}
