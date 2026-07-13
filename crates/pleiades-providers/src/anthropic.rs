use async_trait::async_trait;
use pleiades_core::conversation::{ContentBlock, Message, MessageRole};
use pleiades_core::error::Error;
use pleiades_core::model::{ModelCapabilities, ModelInfo, Pricing};
use pleiades_core::provider::{
    ChatRequest, ChatResponse, Provider, ProviderCapabilities, StreamEvent, Usage,
};
use pleiades_core::tool::ToolDefinition;
use serde::{Deserialize, Serialize};

use crate::client;

const DEFAULT_API_URL: &str = "https://api.anthropic.com/v1";
const ANTHROPIC_VERSION: &str = "2023-06-01";

/// Anthropic Claude provider implementation.
pub struct AnthropicProvider {
    api_key: String,
    base_url: String,
    http_client: reqwest::Client,
}

impl AnthropicProvider {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: DEFAULT_API_URL.to_string(),
            http_client: client::default_client(),
        }
    }

    pub fn with_base_url(api_key: impl Into<String>, base_url: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: base_url.into(),
            http_client: client::default_client(),
        }
    }

    fn build_headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "x-api-key",
            reqwest::header::HeaderValue::from_str(&self.api_key).unwrap(),
        );
        headers.insert(
            "anthropic-version",
            reqwest::header::HeaderValue::from_static(ANTHROPIC_VERSION),
        );
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            reqwest::header::HeaderValue::from_static("application/json"),
        );
        headers
    }

    fn convert_messages(
        &self,
        messages: &[Message],
        system_prompt: Option<&str>,
    ) -> AnthropicRequest {
        let mut system = system_prompt.map(|s| s.to_string());
        let mut anthropic_messages = Vec::new();

        for msg in messages {
            match msg.role {
                MessageRole::System => {
                    let text = msg.text_content();
                    if !text.is_empty() {
                        system = Some(system.map_or(text.clone(), |s| format!("{}\n{}", s, text)));
                    }
                }
                MessageRole::User | MessageRole::Tool => {
                    let content = self.convert_content_blocks(&msg.content, &msg.role);
                    anthropic_messages.push(AnthropicMessage {
                        role: "user".to_string(),
                        content,
                    });
                }
                MessageRole::Assistant => {
                    let content = self.convert_content_blocks(&msg.content, &msg.role);
                    anthropic_messages.push(AnthropicMessage {
                        role: "assistant".to_string(),
                        content,
                    });
                }
            }
        }

        AnthropicRequest {
            model: String::new(),
            messages: anthropic_messages,
            system,
            max_tokens: 4096,
            temperature: None,
            top_p: None,
            stop_sequences: None,
            tools: None,
            stream: false,
        }
    }

    fn convert_content_blocks(
        &self,
        blocks: &[ContentBlock],
        role: &MessageRole,
    ) -> Vec<AnthropicContent> {
        let mut result = Vec::new();

        for block in blocks {
            match block {
                ContentBlock::Text(text) => {
                    result.push(AnthropicContent {
                        content_type: "text".to_string(),
                        text: Some(text.clone()),
                        id: None,
                        name: None,
                        input: None,
                        tool_use_id: None,
                        is_error: None,
                        source: None,
                    });
                }
                ContentBlock::ToolUse { id, name, input } => {
                    if *role == MessageRole::Assistant {
                        result.push(AnthropicContent {
                            content_type: "tool_use".to_string(),
                            text: None,
                            id: Some(id.clone()),
                            name: Some(name.clone()),
                            input: Some(input.clone()),
                            tool_use_id: None,
                            is_error: None,
                            source: None,
                        });
                    }
                }
                ContentBlock::ToolResult {
                    id,
                    content,
                    is_error,
                } => {
                    result.push(AnthropicContent {
                        content_type: "tool_result".to_string(),
                        text: Some(content.clone()),
                        id: None,
                        name: None,
                        input: None,
                        tool_use_id: Some(id.clone()),
                        is_error: Some(*is_error),
                        source: None,
                    });
                }
                ContentBlock::ImageUrl { url, detail: _ } => {
                    result.push(AnthropicContent {
                        content_type: "image".to_string(),
                        text: None,
                        id: None,
                        name: None,
                        input: None,
                        tool_use_id: None,
                        is_error: None,
                        source: Some(AnthropicImageSource {
                            source_type: "url".to_string(),
                            url: Some(url.clone()),
                            media_type: None,
                            data: None,
                        }),
                    });
                }
                ContentBlock::ImageData { mime_type, data } => {
                    result.push(AnthropicContent {
                        content_type: "image".to_string(),
                        text: None,
                        id: None,
                        name: None,
                        input: None,
                        tool_use_id: None,
                        is_error: None,
                        source: Some(AnthropicImageSource {
                            source_type: "base64".to_string(),
                            url: None,
                            media_type: Some(mime_type.clone()),
                            data: Some(base64::Engine::encode(
                                &base64::engine::general_purpose::STANDARD,
                                data,
                            )),
                        }),
                    });
                }
            }
        }

        result
    }

    fn convert_tools(&self, tools: &[ToolDefinition]) -> Vec<AnthropicTool> {
        tools
            .iter()
            .map(|t| AnthropicTool {
                name: t.name.clone(),
                description: t.description.clone(),
                input_schema: t.input_schema.clone(),
            })
            .collect()
    }

    fn convert_response(&self, response: &AnthropicResponse) -> ChatResponse {
        let mut content_blocks = Vec::new();
        let finish_reason = response.stop_reason.clone();

        for block in &response.content {
            match block.content_type.as_str() {
                "text" => {
                    if let Some(text) = &block.text {
                        content_blocks.push(ContentBlock::Text(text.clone()));
                    }
                }
                "tool_use" => {
                    if let (Some(id), Some(name), Some(input)) =
                        (&block.id, &block.name, &block.input)
                    {
                        content_blocks.push(ContentBlock::ToolUse {
                            id: id.clone(),
                            name: name.clone(),
                            input: input.clone(),
                        });
                    }
                }
                _ => {}
            }
        }

        ChatResponse {
            message: Message {
                role: MessageRole::Assistant,
                content: content_blocks,
                reasoning: None,
                metadata: None,
            },
            usage: Some(Usage {
                input_tokens: response.usage.input_tokens as u64,
                output_tokens: response.usage.output_tokens as u64,
                cache_read_tokens: response.usage.cache_read_input_tokens,
                cache_write_tokens: response.usage.cache_creation_input_tokens,
            }),
            finish_reason,
        }
    }
}

#[async_trait]
impl Provider for AnthropicProvider {
    fn name(&self) -> &str {
        "anthropic"
    }

    fn display_name(&self) -> &str {
        "Anthropic Claude"
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            streaming: true,
            tools: true,
            vision: true,
            embeddings: false,
            thinking: true,
            json_mode: false,
            function_calling: true,
        }
    }

    fn default_model(&self) -> &str {
        "claude-sonnet-4-20250514"
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>, Error> {
        Ok(vec![
            ModelInfo {
                id: "claude-sonnet-4-20250514".to_string(),
                provider: "anthropic".to_string(),
                display_name: Some("Claude Sonnet 4".to_string()),
                description: Some("Best balance of speed and capability".to_string()),
                capabilities: ModelCapabilities {
                    max_context_length: 200000,
                    max_output_tokens: 8192,
                    supports_tools: true,
                    supports_vision: true,
                    supports_streaming: true,
                    supports_thinking: true,
                    supports_json_mode: false,
                },
                pricing: Some(Pricing {
                    input_per_million: 3.0,
                    output_per_million: 15.0,
                    cache_read_per_million: Some(0.30),
                    cache_write_per_million: Some(3.75),
                }),
            },
            ModelInfo {
                id: "claude-opus-4-20250514".to_string(),
                provider: "anthropic".to_string(),
                display_name: Some("Claude Opus 4".to_string()),
                description: Some("Most capable Claude model".to_string()),
                capabilities: ModelCapabilities {
                    max_context_length: 200000,
                    max_output_tokens: 8192,
                    supports_tools: true,
                    supports_vision: true,
                    supports_streaming: true,
                    supports_thinking: true,
                    supports_json_mode: false,
                },
                pricing: Some(Pricing {
                    input_per_million: 15.0,
                    output_per_million: 75.0,
                    cache_read_per_million: Some(1.50),
                    cache_write_per_million: Some(18.75),
                }),
            },
            ModelInfo {
                id: "claude-haiku-3-5-20241022".to_string(),
                provider: "anthropic".to_string(),
                display_name: Some("Claude Haiku 3.5".to_string()),
                description: Some("Fastest Claude model for simple tasks".to_string()),
                capabilities: ModelCapabilities {
                    max_context_length: 200000,
                    max_output_tokens: 8192,
                    supports_tools: true,
                    supports_vision: true,
                    supports_streaming: true,
                    supports_thinking: true,
                    supports_json_mode: false,
                },
                pricing: Some(Pricing {
                    input_per_million: 0.80,
                    output_per_million: 4.0,
                    cache_read_per_million: Some(0.08),
                    cache_write_per_million: Some(1.0),
                }),
            },
        ])
    }

    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, Error> {
        let mut api_request =
            self.convert_messages(&request.messages, request.system_prompt.as_deref());
        api_request.model = request.model;
        api_request.max_tokens = request.max_tokens.unwrap_or(4096) as u32;
        api_request.temperature = request.temperature;
        api_request.top_p = request.top_p;
        api_request.stop_sequences = request.stop;

        if let Some(ref tools) = request.tools {
            if !tools.is_empty() {
                api_request.tools = Some(self.convert_tools(tools));
            }
        }

        let body = serde_json::to_value(&api_request)?;

        let url = format!("{}/messages", self.base_url);
        let req = self
            .http_client
            .post(&url)
            .headers(self.build_headers())
            .json(&body);

        let response = client::send_request(&self.http_client, req, "anthropic").await?;
        let status = response.status();
        let response_body = response.text().await.unwrap_or_default();

        if !status.is_success() {
            return Err(client::map_api_error(
                status.as_u16(),
                &response_body,
                "anthropic",
            ));
        }

        let anthropic_response: AnthropicResponse = serde_json::from_str(&response_body)?;
        Ok(self.convert_response(&anthropic_response))
    }

    async fn chat_stream(
        &self,
        request: ChatRequest,
    ) -> Result<tokio::sync::mpsc::Receiver<StreamEvent>, Error> {
        let mut api_request =
            self.convert_messages(&request.messages, request.system_prompt.as_deref());
        api_request.model = request.model;
        api_request.max_tokens = request.max_tokens.unwrap_or(4096) as u32;
        api_request.temperature = request.temperature;
        api_request.top_p = request.top_p;
        api_request.stop_sequences = request.stop;
        api_request.stream = true;

        if let Some(ref tools) = request.tools {
            if !tools.is_empty() {
                api_request.tools = Some(self.convert_tools(tools));
            }
        }

        let body = serde_json::to_value(&api_request)?;

        let url = format!("{}/messages", self.base_url);
        let req = self
            .http_client
            .post(&url)
            .headers(self.build_headers())
            .json(&body);

        let response = client::send_request(&self.http_client, req, "anthropic").await?;
        let status = response.status();

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(client::map_api_error(status.as_u16(), &body, "anthropic"));
        }

        let mut sse_rx = client::parse_sse_stream(response);
        let (tx, rx) = tokio::sync::mpsc::channel(256);

        tokio::spawn(async move {
            let mut content_buffer = String::new();
            let mut input_tokens: u64 = 0;
            let mut output_tokens: u64 = 0;
            let mut cache_read: Option<u64> = None;
            let mut cache_write: Option<u64> = None;
            let mut finish_reason: Option<String> = None;
            let mut pending_tool_id: Option<String> = None;
            let mut pending_tool_name: Option<String> = None;
            let mut pending_tool_input: String = String::new();

            while let Some(event_result) = sse_rx.recv().await {
                match event_result {
                    Ok(event) => match event.event.as_str() {
                        "message_start" => {
                            if let Ok(start) = event.parse_json::<AnthropicStreamMessageStart>() {
                                input_tokens = start.message.usage.input_tokens as u64;
                                cache_read = start.message.usage.cache_read_input_tokens;
                                cache_write = start.message.usage.cache_creation_input_tokens;
                            }
                        }
                        "content_block_start" => {
                            if let Ok(block) =
                                event.parse_json::<AnthropicStreamContentBlockStart>()
                            {
                                if block.content_block.type_ == "tool_use" {
                                    pending_tool_id = block.content_block.id;
                                    pending_tool_name = block.content_block.name;
                                    pending_tool_input.clear();
                                }
                            }
                        }
                        "content_block_delta" => {
                            if let Ok(delta) =
                                event.parse_json::<AnthropicStreamContentBlockDelta>()
                            {
                                match delta.delta.type_.as_str() {
                                    "text_delta" => {
                                        if let Some(text) = &delta.delta.text {
                                            content_buffer.push_str(text);
                                            let _ = tx.send(StreamEvent::Token(text.clone())).await;
                                        }
                                    }
                                    "input_json_delta" => {
                                        if let Some(partial) = &delta.delta.partial_json {
                                            pending_tool_input.push_str(partial);
                                        }
                                    }
                                    "thinking_delta" => {
                                        if let Some(thinking) = &delta.delta.thinking {
                                            let _ = tx
                                                .send(StreamEvent::ReasoningToken(thinking.clone()))
                                                .await;
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        "content_block_stop" => {
                            if let (Some(id), Some(name)) =
                                (pending_tool_id.take(), pending_tool_name.take())
                            {
                                let input: serde_json::Value =
                                    serde_json::from_str(&pending_tool_input).unwrap_or_else(
                                        |_| serde_json::Value::String(pending_tool_input.clone()),
                                    );
                                pending_tool_input.clear();
                                let _ = tx.send(StreamEvent::ToolCall { id, name, input }).await;
                            }
                        }
                        "message_delta" => {
                            if let Ok(delta) = event.parse_json::<AnthropicStreamMessageDelta>() {
                                output_tokens = delta.usage.output_tokens as u64;
                                finish_reason = delta.delta.stop_reason;
                            }
                        }
                        "message_stop" => {
                            let _ = tx
                                .send(StreamEvent::Done {
                                    finish_reason: finish_reason
                                        .clone()
                                        .unwrap_or_else(|| "end_turn".to_string()),
                                    usage: Some(Usage {
                                        input_tokens,
                                        output_tokens,
                                        cache_read_tokens: cache_read,
                                        cache_write_tokens: cache_write,
                                    }),
                                })
                                .await;
                        }
                        "error" => {
                            let err_msg =
                                if let Ok(err) = event.parse_json::<AnthropicStreamError>() {
                                    err.error.message
                                } else {
                                    event.data.clone()
                                };
                            let _ = tx
                                .send(StreamEvent::Error {
                                    message: err_msg,
                                    code: Some("stream_error".to_string()),
                                })
                                .await;
                        }
                        _ => {}
                    },
                    Err(e) => {
                        let _ = tx
                            .send(StreamEvent::Error {
                                message: e.to_string(),
                                code: None,
                            })
                            .await;
                    }
                }
            }
        });

        Ok(rx)
    }
}

// Anthropic API types

#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_sequences: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<AnthropicTool>>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    stream: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnthropicMessage {
    role: String,
    content: Vec<AnthropicContent>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnthropicContent {
    #[serde(rename = "type")]
    content_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    input: Option<serde_json::Value>,
    #[serde(rename = "tool_use_id", skip_serializing_if = "Option::is_none")]
    tool_use_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_error: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source: Option<AnthropicImageSource>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnthropicImageSource {
    #[serde(rename = "type")]
    source_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,
    #[serde(rename = "media_type", skip_serializing_if = "Option::is_none")]
    media_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnthropicTool {
    name: String,
    description: String,
    #[serde(rename = "input_schema")]
    input_schema: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    #[allow(dead_code)]
    id: String,
    #[allow(dead_code)]
    model: String,
    content: Vec<AnthropicContent>,
    #[serde(rename = "stop_reason")]
    stop_reason: Option<String>,
    usage: AnthropicUsage,
}

#[derive(Debug, Deserialize)]
struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
    cache_creation_input_tokens: Option<u64>,
    cache_read_input_tokens: Option<u64>,
}

// Stream event types

#[derive(Debug, Deserialize)]
struct AnthropicStreamMessageStart {
    #[allow(dead_code)]
    message: AnthropicStreamMessage,
}

#[derive(Debug, Deserialize)]
struct AnthropicStreamMessage {
    #[allow(dead_code)]
    id: String,
    usage: AnthropicUsage,
}

#[derive(Debug, Deserialize)]
struct AnthropicStreamContentBlockStart {
    #[serde(rename = "content_block")]
    content_block: AnthropicStreamContentBlock,
}

#[derive(Debug, Deserialize)]
struct AnthropicStreamContentBlock {
    #[serde(rename = "type")]
    type_: String,
    id: Option<String>,
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AnthropicStreamContentBlockDelta {
    #[allow(dead_code)]
    index: u32,
    delta: AnthropicStreamDelta,
}

#[derive(Debug, Deserialize)]
struct AnthropicStreamDelta {
    #[serde(rename = "type")]
    type_: String,
    text: Option<String>,
    #[serde(rename = "partial_json")]
    partial_json: Option<String>,
    thinking: Option<String>,
    #[serde(rename = "stop_reason")]
    stop_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AnthropicStreamMessageDelta {
    delta: AnthropicStreamDelta,
    usage: AnthropicStreamUsage,
}

#[derive(Debug, Deserialize)]
struct AnthropicStreamUsage {
    output_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct AnthropicStreamError {
    error: AnthropicStreamErrorDetail,
    #[allow(dead_code)]
    #[serde(rename = "type")]
    type_: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AnthropicStreamErrorDetail {
    #[allow(dead_code)]
    #[serde(rename = "type")]
    type_: Option<String>,
    message: String,
}
