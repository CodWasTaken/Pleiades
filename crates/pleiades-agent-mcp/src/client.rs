use crate::error::{McpError, Result};
use crate::protocol::{JsonRpcRequest, JsonRpcResponse, ToolsListResult};
use serde_json::Value;
use std::collections::HashMap;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};

/// Configuration for a stdio MCP client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct McpClientConfig {
    /// Stable server ID.
    pub id: String,
    /// Command to execute.
    pub command: String,
    /// Command arguments.
    pub args: Vec<String>,
    /// Child process environment.
    pub env: HashMap<String, String>,
    /// Request timeout.
    pub timeout: Duration,
}

impl McpClientConfig {
    /// Validate configuration before spawning a process.
    pub fn validate(&self) -> Result<()> {
        if self.id.trim().is_empty() {
            return Err(McpError::InvalidServer {
                server: self.id.clone(),
                message: "server ID cannot be empty".to_string(),
            });
        }
        if self.command.trim().is_empty() {
            return Err(McpError::InvalidServer {
                server: self.id.clone(),
                message: "stdio command cannot be empty".to_string(),
            });
        }
        if self.timeout.is_zero() {
            return Err(McpError::InvalidServer {
                server: self.id.clone(),
                message: "timeout must be greater than zero".to_string(),
            });
        }
        Ok(())
    }
}

/// Minimal stdio MCP client.
///
/// The client speaks line-delimited JSON-RPC over a child process' stdin/stdout.
/// It intentionally exposes protocol operations without making trust or
/// permission decisions.
pub struct StdioMcpClient {
    config: McpClientConfig,
    child: Child,
    stdin: ChildStdin,
    stdout: Lines<BufReader<ChildStdout>>,
    next_id: u64,
}

impl StdioMcpClient {
    /// Spawn the configured MCP server.
    pub async fn start(config: McpClientConfig) -> Result<Self> {
        config.validate()?;

        let mut command = Command::new(&config.command);
        command
            .args(&config.args)
            .envs(&config.env)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = command.spawn().map_err(McpError::Io)?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| McpError::Transport("failed to open MCP server stdin".to_string()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| McpError::Transport("failed to open MCP server stdout".to_string()))?;

        Ok(Self {
            config,
            child,
            stdin,
            stdout: BufReader::new(stdout).lines(),
            next_id: 1,
        })
    }

    /// Server ID.
    pub fn id(&self) -> &str {
        &self.config.id
    }

    /// Send initialize and return the raw result payload.
    pub async fn initialize(&mut self, client_name: &str, client_version: &str) -> Result<Value> {
        let id = self.next_request_id();
        let request = JsonRpcRequest::initialize(id, client_name, client_version);
        self.send_request(request).await
    }

    /// List available MCP tools.
    pub async fn list_tools(&mut self) -> Result<ToolsListResult> {
        let id = self.next_request_id();
        let result = self.send_request(JsonRpcRequest::tools_list(id)).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Call a named MCP tool.
    pub async fn call_tool(&mut self, name: &str, arguments: Value) -> Result<Value> {
        let id = self.next_request_id();
        self.send_request(JsonRpcRequest::tools_call(id, name, arguments))
            .await
    }

    /// Shut down the child process.
    pub async fn shutdown(&mut self) -> Result<()> {
        self.child.kill().await.map_err(McpError::Io)
    }

    async fn send_request(&mut self, request: JsonRpcRequest) -> Result<Value> {
        let expected_id = request.id.clone();
        let mut line = serde_json::to_string(&request)?;
        line.push('\n');
        self.stdin.write_all(line.as_bytes()).await?;
        self.stdin.flush().await?;

        let response = self.read_response().await?;
        if response.id.as_ref() != Some(&expected_id) {
            return Err(McpError::UnexpectedResponse(format!(
                "expected response id {}, got {:?}",
                expected_id, response.id
            )));
        }

        if let Some(error) = response.error {
            return Err(McpError::JsonRpc {
                code: error.code,
                message: error.message,
                data: error.data,
            });
        }

        response.result.ok_or_else(|| {
            McpError::UnexpectedResponse("response had no result or error".to_string())
        })
    }

    async fn read_response(&mut self) -> Result<JsonRpcResponse> {
        let timeout = self.config.timeout;
        let line = tokio::time::timeout(timeout, self.stdout.next_line())
            .await
            .map_err(|_| McpError::Timeout(timeout))?
            .map_err(McpError::Io)?
            .ok_or_else(|| McpError::Transport("MCP server closed stdout".to_string()))?;

        Ok(serde_json::from_str(&line)?)
    }

    fn next_request_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_config_rejects_empty_command() {
        let config = McpClientConfig {
            id: "server".to_string(),
            command: String::new(),
            args: Vec::new(),
            env: HashMap::new(),
            timeout: Duration::from_secs(30),
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn client_config_rejects_zero_timeout() {
        let config = McpClientConfig {
            id: "server".to_string(),
            command: "server".to_string(),
            args: Vec::new(),
            env: HashMap::new(),
            timeout: Duration::ZERO,
        };

        assert!(config.validate().is_err());
    }
}
