//! Model Context Protocol client foundation for Pleiades.
//!
//! This crate owns MCP protocol shapes and connection primitives. It does not
//! render UI, execute provider requests, or make policy decisions; higher-level
//! crates expose MCP servers as ordinary typed tools after configuration and
//! permission checks.

pub mod client;
pub mod error;
pub mod protocol;
pub mod registry;

pub use client::{McpClientConfig, StdioMcpClient};
pub use error::{McpError, Result};
pub use protocol::{
    ClientCapabilities, ImplementationInfo, InitializeParams, JsonRpcError, JsonRpcRequest,
    JsonRpcResponse, McpToolInfo, PROTOCOL_VERSION, ToolCallParams, ToolsListResult,
};
pub use registry::{
    McpAuthSource, McpServerDefinition, McpServerHealth, McpServerStatus, McpTransportDefinition,
    RemoteMcpEndpoint,
};
