use std::collections::BTreeMap;
use std::path::PathBuf;

use pleiades_agent_core::Error;
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectCommandReport {
    pub name: String,
    pub command: String,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectDetectionReport {
    pub markers: Vec<String>,
    pub suggested: Vec<ProjectCommandReport>,
}

#[derive(Debug, Clone)]
pub struct ProjectService {
    workspace: PathBuf,
    project_config_dir: PathBuf,
}

impl ProjectService {
    pub fn new(workspace: PathBuf, project_config_dir: PathBuf) -> Self {
        Self {
            workspace,
            project_config_dir,
        }
    }

    pub fn detect(&self) -> ProjectDetectionReport {
        let mut markers = Vec::new();
        let mut suggested = Vec::new();
        if self.workspace.join("Cargo.toml").exists() {
            markers.push("Cargo.toml".to_string());
            suggested.extend([
                ProjectCommandReport {
                    name: "test".to_string(),
                    command: "cargo test --workspace".to_string(),
                    source: "detected rust".to_string(),
                },
                ProjectCommandReport {
                    name: "lint".to_string(),
                    command: "cargo clippy --workspace --all-targets --all-features -- -D warnings".to_string(),
                    source: "detected rust".to_string(),
                },
                ProjectCommandReport {
                    name: "format".to_string(),
                    command: "cargo fmt --all -- --check".to_string(),
                    source: "detected rust".to_string(),
                },
                ProjectCommandReport {
                    name: "verify".to_string(),
                    command: "cargo fmt --all -- --check && cargo clippy --workspace --all-targets --all-features -- -D warnings && cargo test --workspace".to_string(),
                    source: "detected rust".to_string(),
                },
            ]);
        }
        if self.workspace.join("package.json").exists() {
            markers.push("package.json".to_string());
            suggested.extend([
                ProjectCommandReport {
                    name: "dev".to_string(),
                    command: "npm run dev".to_string(),
                    source: "detected npm".to_string(),
                },
                ProjectCommandReport {
                    name: "test".to_string(),
                    command: "npm test".to_string(),
                    source: "detected npm".to_string(),
                },
            ]);
        }
        ProjectDetectionReport { markers, suggested }
    }

    pub fn commands(&self) -> Result<Vec<ProjectCommandReport>, Error> {
        let mut commands = self.detect().suggested;
        let configured = self.read_configured_commands()?;
        commands.retain(|command| !configured.contains_key(&command.name));
        commands.extend(
            configured
                .into_iter()
                .map(|(name, command)| ProjectCommandReport {
                    name,
                    command,
                    source: self.project_file().display().to_string(),
                }),
        );
        commands.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(commands)
    }

    pub fn command(&self, name: &str) -> Result<ProjectCommandReport, Error> {
        self.commands()?
            .into_iter()
            .find(|command| command.name == name)
            .ok_or_else(|| Error::invalid_input(format!("project command `{name}` not found")))
    }

    pub fn verify_command(&self) -> Result<ProjectCommandReport, Error> {
        self.command("verify").or_else(|_| {
            let commands = self.commands()?;
            let chain = ["format", "lint", "test"]
                .into_iter()
                .filter_map(|name| {
                    commands
                        .iter()
                        .find(|command| command.name == name)
                        .map(|command| command.command.clone())
                })
                .collect::<Vec<_>>();
            if chain.is_empty() {
                Err(Error::invalid_input("no project verify recipe detected"))
            } else {
                Ok(ProjectCommandReport {
                    name: "verify".to_string(),
                    command: chain.join(" && "),
                    source: "composed detected recipes".to_string(),
                })
            }
        })
    }

    fn read_configured_commands(&self) -> Result<BTreeMap<String, String>, Error> {
        let path = self.project_file();
        if !path.exists() {
            return Ok(BTreeMap::new());
        }
        let content = std::fs::read_to_string(&path).map_err(Error::from)?;
        let config: ProjectFile = toml::from_str(&content)
            .map_err(|error| Error::config(format!("invalid {}: {error}", path.display())))?;
        Ok(config.project.commands)
    }

    fn project_file(&self) -> PathBuf {
        self.project_config_dir.join("project.toml")
    }
}

#[derive(Debug, Deserialize)]
struct ProjectFile {
    #[serde(default)]
    project: ProjectSection,
}

#[derive(Debug, Default, Deserialize)]
struct ProjectSection {
    #[serde(default)]
    commands: BTreeMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_rust_recipes() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::write(temp.path().join("Cargo.toml"), "[workspace]\n").unwrap();
        let service = ProjectService::new(temp.path().to_path_buf(), temp.path().join(".pleiades"));
        let report = service.detect();
        assert!(report.markers.contains(&"Cargo.toml".to_string()));
        assert!(
            report
                .suggested
                .iter()
                .any(|recipe| recipe.name == "verify")
        );
    }

    #[test]
    fn configured_commands_override_detected() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::write(temp.path().join("Cargo.toml"), "[workspace]\n").unwrap();
        let project = temp.path().join(".pleiades");
        std::fs::create_dir_all(&project).unwrap();
        std::fs::write(
            project.join("project.toml"),
            "[project.commands]\ntest = \"cargo nextest run\"\n",
        )
        .unwrap();
        let service = ProjectService::new(temp.path().to_path_buf(), project);
        let command = service.command("test").unwrap();
        assert_eq!(command.command, "cargo nextest run");
    }
}
