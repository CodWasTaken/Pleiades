use std::path::PathBuf;

use pleiades_agent_config::ConfigLoader;
use pleiades_agent_core::Error;
use pleiades_agent_permissions::{
    Decision, PermissionAction, PermissionEngine, PermissionRule, ToolInvocation,
};

/// Persisted permission rules and legacy allow/deny lists.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PermissionReport {
    pub rules: Vec<PermissionRuleReport>,
    pub always_allow: Vec<String>,
    pub always_deny: Vec<String>,
}

/// User-facing permission rule with its stable one-based display index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PermissionRuleReport {
    pub index: usize,
    pub rule: PermissionRule,
}

/// Result of testing a command against the configured permission engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PermissionTestReport {
    pub decision: Decision,
    pub command: String,
    pub cwd: PathBuf,
    pub workspace_root: PathBuf,
}

/// Shared permission-rule management operations.
pub struct PermissionService {
    loader: ConfigLoader,
}

impl PermissionService {
    pub(crate) fn new(loader: ConfigLoader) -> Self {
        Self { loader }
    }

    pub fn show(&self) -> Result<PermissionReport, Error> {
        let config = self.loader.load().map_err(Error::config)?;
        Ok(PermissionReport {
            rules: config
                .permissions
                .rules
                .into_iter()
                .enumerate()
                .map(|(index, rule)| PermissionRuleReport {
                    index: index + 1,
                    rule,
                })
                .collect(),
            always_allow: config.permissions.always_allow,
            always_deny: config.permissions.always_deny,
        })
    }

    pub fn add_bash_rule(&self, action: PermissionAction, pattern: &str) -> Result<(), Error> {
        self.add_rule(PermissionRule {
            tool: "bash".to_string(),
            pattern: validate_pattern(pattern)?,
            action,
            cwd: None,
            network: None,
            mcp_server: None,
            mcp_tool: None,
        })
    }

    pub fn add_rule(&self, rule: PermissionRule) -> Result<(), Error> {
        let mut config = self.loader.load().map_err(Error::config)?;
        let mut rules = config.permissions.rules.clone();
        rules.push(rule);
        PermissionEngine::new(rules.clone())
            .map_err(|error| Error::invalid_input(error.to_string()))?;
        config.permissions.rules = rules;
        self.loader.save_project(&config).map_err(Error::config)
    }

    pub fn reset(&self) -> Result<(), Error> {
        let mut config = self.loader.load().map_err(Error::config)?;
        config.permissions.rules.clear();
        config.permissions.always_allow.clear();
        config.permissions.always_deny.clear();
        self.loader.save_project(&config).map_err(Error::config)
    }

    pub fn test_bash_command(&self, command: &str) -> Result<PermissionTestReport, Error> {
        let config = self.loader.load().map_err(Error::config)?;
        let engine = PermissionEngine::new(config.permissions.rules)
            .map_err(|error| Error::config(error.to_string()))?;
        let cwd = std::env::current_dir().map_err(Error::from)?;
        let workspace_root = cwd.clone();
        let invocation = ToolInvocation {
            tool: "bash".to_string(),
            command: Some(command.to_string()),
            cwd: cwd.clone(),
            workspace_root: workspace_root.clone(),
            ..ToolInvocation::default()
        };
        Ok(PermissionTestReport {
            decision: engine.evaluate(&invocation),
            command: command.to_string(),
            cwd,
            workspace_root,
        })
    }
}

fn validate_pattern(pattern: &str) -> Result<String, Error> {
    let trimmed = pattern.trim();
    if trimmed.is_empty() {
        return Err(Error::invalid_input("permission pattern cannot be empty"));
    }
    Ok(trimmed.to_string())
}

#[cfg(test)]
mod tests {
    use pleiades_agent_permissions::{DecisionKind, PermissionAction};

    use super::PermissionService;

    #[test]
    fn add_show_and_reset_rules() {
        let temp = tempfile::tempdir().unwrap();
        let loader = pleiades_agent_config::ConfigLoader::with_dirs(
            temp.path().join("global"),
            temp.path().join("project"),
        );
        let service = PermissionService::new(loader);
        service
            .add_bash_rule(PermissionAction::Allow, "cargo test *")
            .unwrap();

        let report = service.show().unwrap();
        assert_eq!(report.rules.len(), 1);
        assert_eq!(report.rules[0].index, 1);
        assert_eq!(report.rules[0].rule.pattern, "cargo test *");

        service.reset().unwrap();
        assert!(service.show().unwrap().rules.is_empty());
    }

    #[test]
    fn test_bash_command_uses_configured_rules() {
        let temp = tempfile::tempdir().unwrap();
        let loader = pleiades_agent_config::ConfigLoader::with_dirs(
            temp.path().join("global"),
            temp.path().join("project"),
        );
        let service = PermissionService::new(loader);
        service
            .add_bash_rule(PermissionAction::Deny, "git push *")
            .unwrap();

        let report = service.test_bash_command("git push origin main").unwrap();
        assert_eq!(report.decision.kind, DecisionKind::Deny);
    }
}
