use std::path::{Path, PathBuf};

use pleiades_agent_core::Error;
use serde::{Deserialize, Serialize};

/// User-facing reusable skill report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillReport {
    pub name: String,
    pub description: String,
    pub instructions: String,
    pub enabled: bool,
    pub scope: String,
    pub path: PathBuf,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SkillFile {
    name: String,
    #[serde(default)]
    description: String,
    instructions: String,
    #[serde(default)]
    enabled: bool,
    #[serde(default)]
    permissions: Vec<String>,
}

/// Shared skill management service.
#[derive(Debug, Clone)]
pub struct SkillService {
    global_dir: PathBuf,
    project_dir: PathBuf,
}

impl SkillService {
    pub(crate) fn new(global_dir: PathBuf, project_dir: PathBuf) -> Self {
        Self {
            global_dir,
            project_dir,
        }
    }

    pub fn list(&self) -> Result<Vec<SkillReport>, Error> {
        let mut reports = Vec::new();
        reports.extend(self.read_dir(&self.global_dir, "global")?);
        reports.extend(self.read_dir(&self.project_dir, "project")?);
        reports.sort_by(|left, right| {
            left.name
                .cmp(&right.name)
                .then_with(|| left.scope.cmp(&right.scope))
        });
        Ok(reports)
    }

    pub fn show(&self, name: &str) -> Result<SkillReport, Error> {
        self.list()?
            .into_iter()
            .find(|skill| skill.name == name)
            .ok_or_else(|| Error::invalid_input(format!("skill `{name}` not found")))
    }

    pub fn create(&self, name: &str) -> Result<SkillReport, Error> {
        validate_name(name)?;
        std::fs::create_dir_all(&self.project_dir).map_err(Error::from)?;
        let path = self.project_dir.join(format!("{name}.toml"));
        if path.exists() {
            return Err(Error::invalid_input(format!(
                "skill `{name}` already exists at {}",
                path.display()
            )));
        }
        let skill = SkillFile {
            name: name.to_string(),
            description: format!("Reusable instructions for {name}"),
            instructions: format!(
                "When this skill is enabled, apply project-specific guidance for `{name}`."
            ),
            enabled: false,
            permissions: Vec::new(),
        };
        write_skill(&path, &skill)?;
        Ok(report_from_file(path, "project", skill))
    }

    pub fn enable(&self, name: &str) -> Result<(), Error> {
        self.set_enabled(name, true)
    }

    pub fn disable(&self, name: &str) -> Result<(), Error> {
        self.set_enabled(name, false)
    }

    pub fn enabled_instructions(&self) -> Result<Vec<SkillReport>, Error> {
        Ok(self
            .list()?
            .into_iter()
            .filter(|skill| skill.enabled)
            .collect())
    }

    fn set_enabled(&self, name: &str, enabled: bool) -> Result<(), Error> {
        let report = self.show(name)?;
        let mut skill = read_skill(&report.path)?;
        skill.enabled = enabled;
        write_skill(&report.path, &skill)
    }

    fn read_dir(&self, dir: &Path, scope: &str) -> Result<Vec<SkillReport>, Error> {
        let mut reports = Vec::new();
        let entries = match std::fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(reports),
            Err(error) => return Err(Error::from(error)),
        };
        for entry in entries {
            let entry = entry.map_err(Error::from)?;
            let path = entry.path();
            if path
                .extension()
                .is_some_and(|extension| extension == "toml")
            {
                let skill = read_skill(&path)?;
                reports.push(report_from_file(path, scope, skill));
            }
        }
        Ok(reports)
    }
}

fn validate_name(name: &str) -> Result<(), Error> {
    if name.trim().is_empty()
        || !name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
    {
        return Err(Error::invalid_input(
            "skill name must contain only letters, numbers, '-' or '_'",
        ));
    }
    Ok(())
}

fn read_skill(path: &Path) -> Result<SkillFile, Error> {
    let text = std::fs::read_to_string(path).map_err(Error::from)?;
    toml::from_str(&text).map_err(|error| Error::config(error.to_string()))
}

fn write_skill(path: &Path, skill: &SkillFile) -> Result<(), Error> {
    let text = toml::to_string_pretty(skill).map_err(|error| Error::config(error.to_string()))?;
    std::fs::write(path, text).map_err(Error::from)
}

fn report_from_file(path: PathBuf, scope: &str, skill: SkillFile) -> SkillReport {
    SkillReport {
        name: skill.name,
        description: skill.description,
        instructions: skill.instructions,
        enabled: skill.enabled,
        scope: scope.to_string(),
        path,
        permissions: skill.permissions,
    }
}

#[cfg(test)]
mod tests {
    use super::SkillService;

    #[test]
    fn create_enable_disable_and_list_skill() {
        let temp = tempfile::tempdir().unwrap();
        let service = SkillService::new(temp.path().join("global"), temp.path().join("project"));

        let created = service.create("review").unwrap();
        assert_eq!(created.name, "review");
        assert!(!created.enabled);

        service.enable("review").unwrap();
        let skill = service.show("review").unwrap();
        assert!(skill.enabled);
        assert_eq!(service.enabled_instructions().unwrap().len(), 1);

        service.disable("review").unwrap();
        assert!(!service.show("review").unwrap().enabled);
    }
}
