use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// Protocol version currently used by Pleiades MCP clients.
pub const PROTOCOL_VERSION: &str = "2024-11-05";

/// JSON-RPC request sent to an MCP server.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JsonRpcRequest {
    /// JSON-RPC protocol marker.
    pub jsonrpc: &'static str,
    /// Request identifier.
    pub id: Value,
    /// Method name.
    pub method: String,
    /// Optional request parameters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

impl JsonRpcRequest {
    /// Create a new JSON-RPC 2.0 request.
    pub fn new(id: impl Into<Value>, method: impl Into<String>, params: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0",
            id: id.into(),
            method: method.into(),
            params,
        }
    }

    /// Create an MCP initialize request.
    pub fn initialize(id: impl Into<Value>, client_name: &str, client_version: &str) -> Self {
        Self::new(
            id,
            "initialize",
            Some(json!(InitializeParams {
                protocol_version: PROTOCOL_VERSION.to_string(),
                capabilities: ClientCapabilities::default(),
                client_info: ImplementationInfo {
                    name: client_name.to_string(),
                    version: client_version.to_string(),
                },
            })),
        )
    }

    /// Create an MCP tools/list request.
    pub fn tools_list(id: impl Into<Value>) -> Self {
        Self::new(id, "tools/list", None)
    }

    /// Create an MCP tools/call request.
    pub fn tools_call(id: impl Into<Value>, name: &str, arguments: Value) -> Self {
        Self::new(
            id,
            "tools/call",
            Some(json!(ToolCallParams {
                name: name.to_string(),
                arguments,
            })),
        )
    }
}

/// JSON-RPC response returned by an MCP server.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JsonRpcResponse {
    /// JSON-RPC protocol marker.
    pub jsonrpc: String,
    /// Response identifier, absent on some invalid responses.
    pub id: Option<Value>,
    /// Successful result payload.
    pub result: Option<Value>,
    /// Error payload.
    pub error: Option<JsonRpcError>,
}

/// JSON-RPC error object.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JsonRpcError {
    /// Error code.
    pub code: i64,
    /// Human-readable error message.
    pub message: String,
    /// Optional structured error details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// MCP initialize request params.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct InitializeParams {
    /// MCP protocol version.
    pub protocol_version: String,
    /// Client capabilities.
    pub capabilities: ClientCapabilities,
    /// Client implementation information.
    pub client_info: ImplementationInfo,
}

/// Client capability declaration.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClientCapabilities {}

/// Implementation name and version.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ImplementationInfo {
    /// Implementation name.
    pub name: String,
    /// Implementation version.
    pub version: String,
}

/// MCP tool descriptor.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct McpToolInfo {
    /// Tool name.
    pub name: String,
    /// Tool description if provided.
    #[serde(default)]
    pub description: Option<String>,
    /// JSON Schema for tool input.
    #[serde(default)]
    pub input_schema: Option<Value>,
}

/// MCP tools/list result.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ToolsListResult {
    /// Exposed tools.
    #[serde(default)]
    pub tools: Vec<McpToolInfo>,
}

/// MCP tools/call params.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolCallParams {
    /// Tool to call.
    pub name: String,
    /// Tool arguments.
    pub arguments: Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initialize_request_uses_mcp_shape() {
        let request = JsonRpcRequest::initialize(1, "pleiades", "2.0.0");
        let value = serde_json::to_value(request).unwrap();

        assert_eq!(value["jsonrpc"], "2.0");
        assert_eq!(value["method"], "initialize");
        assert_eq!(value["params"]["protocolVersion"], PROTOCOL_VERSION);
        assert_eq!(value["params"]["clientInfo"]["name"], "pleiades");
    }

    #[test]
    fn tools_call_serializes_arguments() {
        let request = JsonRpcRequest::tools_call(7, "read_file", json!({ "path": "README.md" }));
        let value = serde_json::to_value(request).unwrap();

        assert_eq!(value["method"], "tools/call");
        assert_eq!(value["params"]["name"], "read_file");
        assert_eq!(value["params"]["arguments"]["path"], "README.md");
    }
}
