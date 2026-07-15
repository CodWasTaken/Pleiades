use std::time::Duration;

use pleiades_agent_config::{ConfigLoader, McpAuthConfig, McpServerConfig, McpTransportConfig};
use pleiades_agent_core::Error;
use pleiades_agent_mcp::{
    McpAuthSource, McpServerDefinition, McpServerStatus, McpTransportDefinition, RemoteMcpEndpoint,
};

/// Secret-safe MCP server information for CLI and live workspace renderers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct McpServerReport {
    pub id: String,
    pub enabled: bool,
    pub transport: String,
    pub health: String,
    pub timeout_secs: u64,
    pub tool_count: Option<usize>,
    pub allowlist: Vec<String>,
    pub denylist: Vec<String>,
    pub last_error: Option<String>,
}

/// Tool exposure report for one MCP server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct McpToolReport {
    pub server: String,
    pub tool: String,
    pub exposed: bool,
    pub schema_available: bool,
    pub notes: String,
}

/// Shared MCP configuration and status operations.
pub struct McpService {
    loader: ConfigLoader,
}

impl McpService {
    pub(crate) fn new(loader: ConfigLoader) -> Self {
        Self { loader }
    }

    /// List configured MCP servers without resolving or exposing secrets.
    pub fn list(&self) -> Result<Vec<McpServerReport>, Error> {
        let config = self.loader.load().map_err(Error::config)?;
        let mut reports = config
            .mcp
            .servers
            .iter()
            .map(|(id, server)| {
                let definition = definition_from_config(id, server);
                let status = McpServerStatus::from_definition(&definition);
                report_from_status(server, status)
            })
            .collect::<Vec<_>>();
        reports.sort_by(|left, right| left.id.cmp(&right.id));
        Ok(reports)
    }

    /// Inspect a configured MCP server.
    pub fn info(&self, id: &str) -> Result<McpServerReport, Error> {
        self.list()?
            .into_iter()
            .find(|server| server.id == id)
            .ok_or_else(|| Error::invalid_input(format!("MCP server `{id}` is not configured")))
    }

    /// Enable a configured MCP server.
    pub fn enable(&self, id: &str) -> Result<(), Error> {
        self.set_enabled(id, true)
    }

    /// Disable a configured MCP server.
    pub fn disable(&self, id: &str) -> Result<(), Error> {
        self.set_enabled(id, false)
    }

    /// Remove a configured MCP server.
    pub fn remove(&self, id: &str) -> Result<(), Error> {
        let mut config = self.loader.load().map_err(Error::config)?;
        if config.mcp.servers.remove(id).is_none() {
            return Err(Error::invalid_input(format!(
                "MCP server `{id}` is not configured"
            )));
        }
        self.loader.save_project(&config).map_err(Error::config)
    }

    /// Return the configured exposure filters for a server. Runtime discovery
    /// will replace these reports with live schemas when connections are wired.
    pub fn tools(&self, id: &str) -> Result<Vec<McpToolReport>, Error> {
        let config = self.loader.load().map_err(Error::config)?;
        let server =
            config.mcp.servers.get(id).ok_or_else(|| {
                Error::invalid_input(format!("MCP server `{id}` is not configured"))
            })?;
        let definition = definition_from_config(id, server);
        let mut names = server
            .tool_allowlist
            .iter()
            .chain(server.tool_denylist.iter())
            .cloned()
            .collect::<Vec<_>>();
        names.sort();
        names.dedup();

        Ok(names
            .into_iter()
            .map(|tool| McpToolReport {
                server: id.to_string(),
                exposed: definition.allows_tool(&tool),
                tool,
                schema_available: false,
                notes: "Configured filter only; live schema discovery is not connected yet."
                    .to_string(),
            })
            .collect())
    }

    /// Inspect one configured MCP tool exposure rule.
    pub fn tool_info(&self, id: &str, tool: &str) -> Result<McpToolReport, Error> {
        let config = self.loader.load().map_err(Error::config)?;
        let server =
            config.mcp.servers.get(id).ok_or_else(|| {
                Error::invalid_input(format!("MCP server `{id}` is not configured"))
            })?;
        let definition = definition_from_config(id, server);
        Ok(McpToolReport {
            server: id.to_string(),
            tool: tool.to_string(),
            exposed: definition.allows_tool(tool),
            schema_available: false,
            notes:
                "Live schema discovery will be available after MCP runtime connections are wired."
                    .to_string(),
        })
    }

    fn set_enabled(&self, id: &str, enabled: bool) -> Result<(), Error> {
        let mut config = self.loader.load().map_err(Error::config)?;
        let server =
            config.mcp.servers.get_mut(id).ok_or_else(|| {
                Error::invalid_input(format!("MCP server `{id}` is not configured"))
            })?;
        server.enabled = enabled;
        self.loader.save_project(&config).map_err(Error::config)
    }
}

fn report_from_status(server: &McpServerConfig, status: McpServerStatus) -> McpServerReport {
    McpServerReport {
        id: status.id,
        enabled: status.enabled,
        transport: status.transport,
        health: format!("{:?}", status.health).to_lowercase(),
        timeout_secs: server.timeout_secs,
        tool_count: status.tool_count,
        allowlist: server.tool_allowlist.clone(),
        denylist: server.tool_denylist.clone(),
        last_error: status.last_error,
    }
}

fn definition_from_config(id: &str, server: &McpServerConfig) -> McpServerDefinition {
    McpServerDefinition {
        id: id.to_string(),
        enabled: server.enabled,
        transport: transport_from_config(&server.transport),
        timeout: Duration::from_secs(server.timeout_secs),
        tool_allowlist: server.tool_allowlist.clone(),
        tool_denylist: server.tool_denylist.clone(),
    }
}

fn transport_from_config(transport: &McpTransportConfig) -> McpTransportDefinition {
    match transport {
        McpTransportConfig::Stdio { command, args, env } => McpTransportDefinition::Stdio {
            command: command.clone(),
            args: args.clone(),
            env: env.clone(),
        },
        McpTransportConfig::Http { url, auth } => McpTransportDefinition::Http(RemoteMcpEndpoint {
            url: url.clone(),
            auth: auth.as_ref().map(auth_from_config),
        }),
        McpTransportConfig::StreamableHttp { url, auth } => {
            McpTransportDefinition::StreamableHttp(RemoteMcpEndpoint {
                url: url.clone(),
                auth: auth.as_ref().map(auth_from_config),
            })
        }
    }
}

fn auth_from_config(auth: &McpAuthConfig) -> McpAuthSource {
    match auth {
        McpAuthConfig::Bearer { token_env } => McpAuthSource::Bearer {
            token_env: token_env.clone(),
        },
        McpAuthConfig::OAuth {
            client_id_env,
            token_env,
        } => McpAuthSource::OAuth {
            client_id_env: client_id_env.clone(),
            token_env: token_env.clone(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::McpService;
    use pleiades_agent_config::{Config, McpServerConfig, McpTransportConfig};
    use std::collections::HashMap;

    fn service(temp: &tempfile::TempDir, config: &Config) -> McpService {
        let loader = pleiades_agent_config::ConfigLoader::with_dirs(
            temp.path().join("global"),
            temp.path().join("project"),
        );
        loader.save_project(config).unwrap();
        McpService::new(loader)
    }

    #[test]
    fn list_returns_redacted_reports() {
        let temp = tempfile::tempdir().unwrap();
        let mut config = Config::default();
        config.mcp.servers.insert(
            "docs".to_string(),
            McpServerConfig {
                transport: McpTransportConfig::Http {
                    url: "https://example.test/mcp?token=secret".to_string(),
                    auth: None,
                },
                tool_allowlist: vec!["search".to_string()],
                ..McpServerConfig::default()
            },
        );

        let reports = service(&temp, &config).list().unwrap();
        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].id, "docs");
        assert_eq!(
            reports[0].transport,
            "http:https://example.test/mcp?token=REDACTED"
        );
        assert_eq!(reports[0].allowlist, vec!["search"]);
    }

    #[test]
    fn enable_disable_and_remove_persist_to_project_config() {
        let temp = tempfile::tempdir().unwrap();
        let mut config = Config::default();
        config.mcp.servers.insert(
            "local".to_string(),
            McpServerConfig {
                transport: McpTransportConfig::Stdio {
                    command: "server".to_string(),
                    args: Vec::new(),
                    env: HashMap::new(),
                },
                ..McpServerConfig::default()
            },
        );
        let service = service(&temp, &config);

        service.disable("local").unwrap();
        assert!(!service.info("local").unwrap().enabled);
        service.enable("local").unwrap();
        assert!(service.info("local").unwrap().enabled);
        service.remove("local").unwrap();
        assert!(service.info("local").is_err());
    }

    #[test]
    fn tool_reports_reflect_filters_without_claiming_schema_discovery() {
        let temp = tempfile::tempdir().unwrap();
        let mut config = Config::default();
        config.mcp.servers.insert(
            "local".to_string(),
            McpServerConfig {
                transport: McpTransportConfig::Stdio {
                    command: "server".to_string(),
                    args: Vec::new(),
                    env: HashMap::new(),
                },
                tool_allowlist: vec!["read".to_string()],
                tool_denylist: vec!["write".to_string()],
                ..McpServerConfig::default()
            },
        );
        let service = service(&temp, &config);

        let read = service.tool_info("local", "read").unwrap();
        assert!(read.exposed);
        assert!(!read.schema_available);
        let write = service.tool_info("local", "write").unwrap();
        assert!(!write.exposed);
    }
}
