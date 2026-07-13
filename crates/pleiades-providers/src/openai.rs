use async_trait::async_trait;
use pleiades_core::conversation::{ContentBlock, Message, MessageRole};
use pleiades_core::error::Error;
use pleiades_core::model::{ModelCapabilities, ModelInfo};
use pleiades_core::provider::{
    ChatRequest, ChatResponse, EmbeddingResponse, Provider, ProviderCapabilities, StreamEvent,
    Usage,
};
use pleiades_core::tool::ToolDefinition;
use serde::{Deserialize, Serialize};

use crate::client;

const DEFAULT_API_URL: &str = "https://api.openai.com/v1";

/// OpenAI provider implementation.
pub struct OpenAIProvider {
    api_key: String,
    base_url: String,
    http_client: reqwest::Client,
}

impl OpenAIProvider {
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
            reqwest::header::AUTHORIZATION,
            reqwest::header::HeaderValue::from_str(&format!("Bearer {}", self.api_key)).unwrap(),
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
    ) -> Vec<OpenAIMessage> {
        let mut result = Vec::new();

        if let Some(system) = system_prompt {
            if !system.is_empty() {
                result.push(OpenAIMessage {
                    role: "system".to_string(),
                    content: Some(system.to_string()),
                    tool_calls: None,
                    tool_call_id: None,
                    name: None,
                });
            }
        }

        for msg in messages {
            let openai_msg = match msg.role {
                MessageRole::System => {
                    if system_prompt.is_some() {
                        continue;
                    }
                    OpenAIMessage {
                        role: "system".to_string(),
                        content: Some(msg.text_content()),
                        tool_calls: None,
                        tool_call_id: None,
                        name: None,
                    }
                }
                MessageRole::User => OpenAIMessage {
                    role: "user".to_string(),
                    content: Some(msg.text_content()),
                    tool_calls: None,
                    tool_call_id: None,
                    name: None,
                },
                MessageRole::Assistant => {
                    let mut tool_calls = Vec::new();
                    let mut text_content = String::new();

                    for block in &msg.content {
                        match block {
                            ContentBlock::Text(t) => text_content.push_str(t),
                            ContentBlock::ToolUse { id, name, input } => {
                                tool_calls.push(OpenAIToolCall {
                                    id: id.clone(),
                                    type_: "function".to_string(),
                                    function: OpenAIToolCallFunction {
                                        name: name.clone(),
                                        arguments: serde_json::to_string(input).unwrap_or_default(),
                                    },
                                });
                            }
                            _ => {}
                        }
                    }

                    OpenAIMessage {
                        role: "assistant".to_string(),
                        content: if text_content.is_empty() {
                            None
                        } else {
                            Some(text_content)
                        },
                        tool_calls: if tool_calls.is_empty() {
                            None
                        } else {
                            Some(tool_calls)
                        },
                        tool_call_id: None,
                        name: None,
                    }
                }
                MessageRole::Tool => {
                    let mut tool_call_id = String::new();
                    let mut content = String::new();

                    for block in &msg.content {
                        match block {
                            ContentBlock::ToolResult {
                                id,
                                content: c,
                                is_error: _,
                            } => {
                                tool_call_id = id.clone();
                                content = c.clone();
                            }
                            ContentBlock::Text(t) => content.push_str(t),
                            _ => {}
                        }
                    }

                    OpenAIMessage {
                        role: "tool".to_string(),
                        content: Some(content),
                        tool_calls: None,
                        tool_call_id: Some(tool_call_id),
                        name: None,
                    }
                }
            };
            result.push(openai_msg);
        }

        result
    }

    fn convert_tools(&self, tools: &[ToolDefinition]) -> Vec<OpenAITool> {
        tools
            .iter()
            .map(|t| OpenAITool {
                type_: "function".to_string(),
                function: OpenAIToolFunction {
                    name: t.name.clone(),
                    description: t.description.clone(),
                    parameters: t.input_schema.clone(),
                },
            })
            .collect()
    }

    fn convert_response(&self, response: &OpenAIChatResponse) -> Result<ChatResponse, Error> {
        let choice = response
            .choices
            .first()
            .ok_or_else(|| Error::provider("OpenAI returned no choices"))?;

        let mut content_blocks = Vec::new();

        if let Some(ref content) = choice.message.content {
            if !content.is_empty() {
                content_blocks.push(ContentBlock::Text(content.clone()));
            }
        }

        if let Some(ref tool_calls) = choice.message.tool_calls {
            for tc in tool_calls {
                let input: serde_json::Value = serde_json::from_str(&tc.function.arguments)
                    .unwrap_or_else(|_| serde_json::Value::String(tc.function.arguments.clone()));

                content_blocks.push(ContentBlock::ToolUse {
                    id: tc.id.clone(),
                    name: tc.function.name.clone(),
                    input,
                });
            }
        }

        let finish_reason = choice.finish_reason.clone();

        Ok(ChatResponse {
            message: Message {
                role: MessageRole::Assistant,
                content: content_blocks,
                reasoning: None,
                metadata: None,
            },
            usage: response.usage.as_ref().map(|u| Usage {
                input_tokens: u.prompt_tokens as u64,
                output_tokens: u.completion_tokens as u64,
                cache_read_tokens: u
                    .prompt_tokens_details
                    .as_ref()
                    .and_then(|d| d.cached_tokens),
                cache_write_tokens: None,
            }),
            finish_reason,
        })
    }
}

#[async_trait]
impl Provider for OpenAIProvider {
    fn name(&self) -> &str {
        "openai"
    }

    fn display_name(&self) -> &str {
        "OpenAI"
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            streaming: true,
            tools: true,
            vision: true,
            embeddings: true,
            thinking: false,
            json_mode: true,
            function_calling: true,
        }
    }

    fn default_model(&self) -> &str {
        "gpt-4o"
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>, Error> {
        let url = format!("{}/models", self.base_url);
        let req = self.http_client.get(&url).headers(self.build_headers());
        let response = client::send_request(&self.http_client, req, "openai").await?;
        let status = response.status();
        let body = response.text().await.unwrap_or_default();

        if !status.is_success() {
            return Err(client::map_api_error(status.as_u16(), &body, "openai"));
        }

        let models: OpenAIListModelsResponse = serde_json::from_str(&body)?;

        let infos: Vec<ModelInfo> = models
            .data
            .into_iter()
            .filter(|m| m.id.starts_with("gpt-") || m.id.starts_with("o"))
            .map(|m| ModelInfo {
                id: m.id,
                provider: "openai".to_string(),
                display_name: None,
                description: None,
                capabilities: ModelCapabilities {
                    max_context_length: 128000,
                    max_output_tokens: 4096,
                    supports_tools: true,
                    supports_vision: true,
                    supports_streaming: true,
                    supports_thinking: false,
                    supports_json_mode: true,
                },
                pricing: None,
            })
            .collect();

        Ok(infos)
    }

    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, Error> {
        let messages = self.convert_messages(&request.messages, request.system_prompt.as_deref());

        let mut api_request = OpenAIChatRequest {
            model: request.model,
            messages,
            temperature: request.temperature,
            top_p: request.top_p,
            max_tokens: request.max_tokens,
            stop: request.stop,
            tools: None,
            tool_choice: None,
            stream: false,
        };

        if let Some(ref tools) = request.tools {
            if !tools.is_empty() {
                api_request.tools = Some(self.convert_tools(tools));
                api_request.tool_choice = Some(serde_json::Value::String("auto".to_string()));
            }
        }

        let body = serde_json::to_value(&api_request)?;

        let url = format!("{}/chat/completions", self.base_url);
        let req = self
            .http_client
            .post(&url)
            .headers(self.build_headers())
            .json(&body);

        let response = client::send_request(&self.http_client, req, "openai").await?;
        let status = response.status();
        let response_body = response.text().await.unwrap_or_default();

        if !status.is_success() {
            return Err(client::map_api_error(
                status.as_u16(),
                &response_body,
                "openai",
            ));
        }

        let openai_response: OpenAIChatResponse = serde_json::from_str(&response_body)?;
        self.convert_response(&openai_response)
    }

    async fn chat_stream(
        &self,
        request: ChatRequest,
    ) -> Result<tokio::sync::mpsc::Receiver<StreamEvent>, Error> {
        let messages = self.convert_messages(&request.messages, request.system_prompt.as_deref());

        let mut api_request = OpenAIChatRequest {
            model: request.model,
            messages,
            temperature: request.temperature,
            top_p: request.top_p,
            max_tokens: request.max_tokens,
            stop: request.stop,
            tools: None,
            tool_choice: None,
            stream: true,
        };

        if let Some(ref tools) = request.tools {
            if !tools.is_empty() {
                api_request.tools = Some(self.convert_tools(tools));
                api_request.tool_choice = Some(serde_json::Value::String("auto".to_string()));
            }
        }

        let body = serde_json::to_value(&api_request)?;

        let url = format!("{}/chat/completions", self.base_url);
        let req = self
            .http_client
            .post(&url)
            .headers(self.build_headers())
            .json(&body);

        let response = client::send_request(&self.http_client, req, "openai").await?;
        let status = response.status();

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(client::map_api_error(status.as_u16(), &body, "openai"));
        }

        let mut sse_rx = client::parse_sse_stream(response);
        let (tx, rx) = tokio::sync::mpsc::channel(256);

        tokio::spawn(async move {
            let mut input_tokens: u64 = 0;
            let mut output_tokens: u64 = 0;
            let mut finish_reason: Option<String> = None;
            let mut current_tool_calls: std::collections::HashMap<String, OpenAIToolCallPartial> =
                std::collections::HashMap::new();

            while let Some(event_result) = sse_rx.recv().await {
                match event_result {
                    Ok(event) => {
                        if event.data == "[DONE]" {
                            let _ = tx
                                .send(StreamEvent::Done {
                                    finish_reason: finish_reason
                                        .clone()
                                        .unwrap_or_else(|| "stop".to_string()),
                                    usage: Some(Usage {
                                        input_tokens,
                                        output_tokens,
                                        cache_read_tokens: None,
                                        cache_write_tokens: None,
                                    }),
                                })
                                .await;
                            continue;
                        }

                        if let Ok(chunk) = event.parse_json::<OpenAIStreamChunk>() {
                            if let Some(ref usage) = chunk.usage {
                                input_tokens = usage.prompt_tokens as u64;
                                output_tokens = usage.completion_tokens as u64;
                            }

                            for choice in chunk.choices {
                                finish_reason = choice.finish_reason.clone();

                                if let Some(ref delta) = choice.delta {
                                    if let Some(ref content) = delta.content {
                                        if !content.is_empty() {
                                            let _ =
                                                tx.send(StreamEvent::Token(content.clone())).await;
                                        }
                                    }

                                    if let Some(ref tool_calls) = delta.tool_calls {
                                        for tc in tool_calls {
                                            let index = tc.index;
                                            let entry = current_tool_calls
                                                .entry(index.to_string())
                                                .or_insert_with(|| OpenAIToolCallPartial {
                                                    id: String::new(),
                                                    name: String::new(),
                                                    arguments: String::new(),
                                                });

                                            if let Some(ref id) = tc.id {
                                                entry.id.push_str(id);
                                            }
                                            if let Some(ref func) = tc.function {
                                                if let Some(ref name) = func.name {
                                                    entry.name.push_str(name);
                                                }
                                                if let Some(ref args) = func.arguments {
                                                    entry.arguments.push_str(args);
                                                }
                                            }
                                        }
                                    }
                                }

                                if choice.finish_reason.is_some() {
                                    for (_idx, tc) in current_tool_calls.drain() {
                                        let input: serde_json::Value =
                                            serde_json::from_str(&tc.arguments).unwrap_or_else(
                                                |_| serde_json::Value::String(tc.arguments.clone()),
                                            );

                                        let _ = tx
                                            .send(StreamEvent::ToolCall {
                                                id: tc.id,
                                                name: tc.name,
                                                input,
                                            })
                                            .await;
                                    }
                                }
                            }
                        }
                    }
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

    async fn embed(&self, input: Vec<String>, model: &str) -> Result<EmbeddingResponse, Error> {
        let request = OpenAIEmbeddingRequest {
            model: model.to_string(),
            input,
        };

        let body = serde_json::to_value(&request)?;
        let url = format!("{}/embeddings", self.base_url);
        let req = self
            .http_client
            .post(&url)
            .headers(self.build_headers())
            .json(&body);

        let response = client::send_request(&self.http_client, req, "openai").await?;
        let status = response.status();
        let response_body = response.text().await.unwrap_or_default();

        if !status.is_success() {
            return Err(client::map_api_error(
                status.as_u16(),
                &response_body,
                "openai",
            ));
        }

        let embedding_response: OpenAIEmbeddingResponse = serde_json::from_str(&response_body)?;

        Ok(EmbeddingResponse {
            embeddings: embedding_response
                .data
                .into_iter()
                .map(|d| d.embedding)
                .collect(),
            model: embedding_response.model,
            usage: embedding_response.usage.map(|u| Usage {
                input_tokens: u.prompt_tokens as u64,
                output_tokens: 0,
                cache_read_tokens: None,
                cache_write_tokens: None,
            }),
        })
    }
}

// OpenAI API types

#[derive(Debug, Serialize)]
struct OpenAIChatRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<OpenAITool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<serde_json::Value>,
    stream: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<OpenAIToolCall>>,
    #[serde(rename = "tool_call_id", skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIToolCall {
    id: String,
    #[serde(rename = "type")]
    type_: String,
    function: OpenAIToolCallFunction,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIToolCallFunction {
    name: String,
    arguments: String,
}

#[derive(Debug, Serialize)]
struct OpenAITool {
    #[serde(rename = "type")]
    type_: String,
    function: OpenAIToolFunction,
}

#[derive(Debug, Serialize)]
struct OpenAIToolFunction {
    name: String,
    description: String,
    parameters: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct OpenAIChatResponse {
    choices: Vec<OpenAIChoice>,
    usage: Option<OpenAIUsage>,
    #[allow(dead_code)]
    model: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
    finish_reason: Option<String>,
    #[allow(dead_code)]
    index: u32,
}

#[derive(Debug, Deserialize)]
struct OpenAIUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    #[allow(dead_code)]
    total_tokens: u32,
    prompt_tokens_details: Option<OpenAIPromptTokensDetails>,
}

#[derive(Debug, Deserialize)]
struct OpenAIPromptTokensDetails {
    cached_tokens: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct OpenAIListModelsResponse {
    data: Vec<OpenAIModelEntry>,
    #[allow(dead_code)]
    object: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIModelEntry {
    id: String,
    #[allow(dead_code)]
    object: String,
    #[allow(dead_code)]
    created: u64,
    #[allow(dead_code)]
    owned_by: String,
}

// Streaming types

#[derive(Debug, Deserialize)]
struct OpenAIStreamChunk {
    choices: Vec<OpenAIStreamChoice>,
    usage: Option<OpenAIUsage>,
    #[allow(dead_code)]
    model: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAIStreamChoice {
    delta: Option<OpenAIStreamDelta>,
    finish_reason: Option<String>,
    #[allow(dead_code)]
    index: u32,
}

#[derive(Debug, Deserialize)]
struct OpenAIStreamDelta {
    content: Option<String>,
    tool_calls: Option<Vec<OpenAIStreamToolCall>>,
    #[allow(dead_code)]
    role: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAIStreamToolCall {
    index: u32,
    id: Option<String>,
    #[allow(dead_code)]
    #[serde(rename = "type")]
    type_: Option<String>,
    function: Option<OpenAIStreamToolCallFunction>,
}

#[derive(Debug, Deserialize)]
struct OpenAIStreamToolCallFunction {
    name: Option<String>,
    arguments: Option<String>,
}

#[derive(Debug)]
struct OpenAIToolCallPartial {
    id: String,
    name: String,
    arguments: String,
}

// Embedding types

#[derive(Debug, Serialize)]
struct OpenAIEmbeddingRequest {
    model: String,
    input: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAIEmbeddingResponse {
    data: Vec<OpenAIEmbeddingData>,
    model: String,
    usage: Option<OpenAIUsage>,
}

#[derive(Debug, Deserialize)]
struct OpenAIEmbeddingData {
    embedding: Vec<f32>,
    #[allow(dead_code)]
    index: u32,
}
