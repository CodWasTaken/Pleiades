use std::collections::HashMap;
use std::sync::Arc;

use pleiades_core::conversation::{Conversation, Message, MessageRole};
use pleiades_core::error::Error;
use pleiades_core::event::Event;
use pleiades_core::provider::{ChatRequest, Provider, StreamEvent};
use pleiades_core::tool::{Tool, ToolContext};
use pleiades_core::model::ModelRegistry;

use pleiades_config::types::Config;

/// The main engine that orchestrates AI interactions.
///
/// The engine ties together providers, tools, conversations,
/// and the model registry into a cohesive interaction loop.
pub struct Engine {
    providers: HashMap<String, Box<dyn Provider>>,
    tools: Vec<Box<dyn Tool>>,
    model_registry: ModelRegistry,
    config: Arc<Config>,
    event_sender: Option<tokio::sync::mpsc::Sender<Event>>,
}

impl Engine {
    /// Create a new engine with the given configuration.
    pub fn new(config: Config) -> Self {
        Self {
            providers: HashMap::new(),
            tools: Vec::new(),
            model_registry: ModelRegistry::new(),
            config: Arc::new(config),
            event_sender: None,
        }
    }

    /// Register a provider with the engine.
    pub fn register_provider(&mut self, provider: Box<dyn Provider>) {
        self.providers.insert(provider.name().to_string(), provider);
    }

    /// Register a tool with the engine.
    pub fn register_tool(&mut self, tool: Box<dyn Tool>) {
        self.tools.push(tool);
    }

    /// Set the event sender for emitting events.
    pub fn set_event_sender(&mut self, sender: tokio::sync::mpsc::Sender<Event>) {
        self.event_sender = Some(sender);
    }

    /// Get a registered provider by name.
    pub fn get_provider(&self, name: &str) -> Result<&dyn Provider, Error> {
        self.providers
            .get(name)
            .map(|p| p.as_ref())
            .ok_or_else(|| Error::ProviderNotFound { provider: name.to_string() })
    }

    /// Get the model registry.
    pub fn model_registry(&self) -> &ModelRegistry {
        &self.model_registry
    }

    /// Get a mutable reference to the model registry.
    pub fn model_registry_mut(&mut self) -> &mut ModelRegistry {
        &mut self.model_registry
    }

    /// Get available tools as definitions for provider requests.
    pub fn tool_definitions(&self) -> Vec<pleiades_core::tool::ToolDefinition> {
        self.tools.iter().map(|t| t.definition()).collect()
    }

    /// Process a chat message through the engine.
    pub async fn chat(&self, conversation: &mut Conversation, provider_name: &str) -> Result<Message, Error> {
        self.prepare_conversation(conversation);
        let provider = self.get_provider(provider_name)?;
        let model = self.config.core.default_model.clone()
            .unwrap_or_else(|| provider.default_model().to_string());

        let request = ChatRequest {
            model,
            messages: conversation.messages.clone(),
            system_prompt: conversation.config.system_prompt.clone(),
            temperature: conversation.config.temperature,
            top_p: conversation.config.top_p,
            max_tokens: conversation.config.max_tokens,
            stop: None,
            tools: Some(self.tool_definitions()),
        };

        let response = provider.chat(request).await?;
        conversation.add_message(response.message.clone());

        if let Some(usage) = response.usage {
            conversation.metadata.total_tokens = Some(
                conversation.metadata.total_tokens.unwrap_or(0) + usage.input_tokens + usage.output_tokens
            );
        }

        self.emit(Event::MessageAdded {
            conversation_id: conversation.id.clone(),
        });

        Ok(response.message)
    }

    /// Stream a chat response, processing tokens as they arrive.
    pub async fn chat_stream(
        &self,
        conversation: &mut Conversation,
        provider_name: &str,
    ) -> Result<tokio::sync::mpsc::Receiver<StreamEvent>, Error> {
        self.prepare_conversation(conversation);
        let provider = self.get_provider(provider_name)?;
        let model = self.config.core.default_model.clone()
            .unwrap_or_else(|| provider.default_model().to_string());

        let request = ChatRequest {
            model,
            messages: conversation.messages.clone(),
            system_prompt: conversation.config.system_prompt.clone(),
            temperature: conversation.config.temperature,
            top_p: conversation.config.top_p,
            max_tokens: conversation.config.max_tokens,
            stop: None,
            tools: Some(self.tool_definitions()),
        };

        let receiver = provider.chat_stream(request).await?;

        Ok(receiver)
    }

    /// Execute a tool by name.
    pub async fn execute_tool(
        &self,
        name: &str,
        input: serde_json::Value,
    ) -> Result<pleiades_core::tool::ToolResult, Error> {
        let tool = self.tools.iter()
            .find(|t| t.name() == name)
            .ok_or_else(|| Error::ToolNotFound { name: name.to_string() })?;

        let ctx = ToolContext {
            cwd: std::env::current_dir().map_err(|e| Error::Io(e.to_string()))?,
            working_directory: std::env::current_dir().map_err(|e| Error::Io(e.to_string()))?,
            permission_mode: if self.config.permissions.ask_always {
                pleiades_core::tool::PermissionMode::Ask
            } else {
                pleiades_core::tool::PermissionMode::Allow
            },
            config: Arc::new(serde_json::to_value(&*self.config).unwrap_or_default()),
        };

        self.emit(Event::ToolCalled {
            tool: name.to_string(),
            input: input.clone(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        });

        let start = std::time::Instant::now();
        let result = tool.execute(input, &ctx).await;
        let duration = start.elapsed().as_millis() as u64;

        self.emit(Event::ToolCompleted {
            tool: name.to_string(),
            success: result.is_ok(),
            duration_ms: duration,
        });

        result
    }

    /// Estimate token count for a conversation using a simple heuristic.
    ///
    /// Roughly 4 characters per token for English text, plus overhead per message.
    pub fn estimate_tokens(conversation: &Conversation) -> usize {
        conversation.messages.iter().map(|m| {
            let text_len = m.text_content().len();
            let role_overhead = 4; // role tag
            text_len / 4 + role_overhead
        }).sum()
    }

    /// Check if a conversation exceeds the configured context window.
    pub fn exceeds_context_limit(&self, conversation: &Conversation) -> bool {
        let max_context = self.config.session.context_size * 4; // rough: 4 chars per token
        conversation.estimated_tokens() > max_context
    }

    /// Truncate a conversation to fit within the context window.
    ///
    /// Strategy: keeps system message, most recent messages, removes oldest non-system messages.
    /// Returns the number of messages removed.
    pub fn truncate_conversation(&self, conversation: &mut Conversation) -> usize {
        let max_messages = self.config.session.context_size;
        if conversation.messages.len() <= max_messages {
            return 0;
        }

        let mut system_msgs: Vec<Message> = Vec::new();
        let mut non_system: Vec<Message> = Vec::new();

        for msg in conversation.messages.drain(..) {
            if msg.role == MessageRole::System {
                system_msgs.push(msg);
            } else {
                non_system.push(msg);
            }
        }

        let remove_count = non_system.len().saturating_sub(max_messages.saturating_sub(system_msgs.len()));
        let truncated: Vec<Message> = non_system.into_iter().skip(remove_count).collect();

        let mut result = system_msgs;
        result.extend(truncated);
        conversation.messages = result;

        remove_count
    }

    /// Apply automatic compression when the conversation exceeds the threshold.
    ///
    /// Returns a summary message if compression was applied.
    pub fn compress_conversation(&self, conversation: &mut Conversation) -> Option<String> {
        if !conversation.config.auto_compress {
            return None;
        }
        if !self.exceeds_context_limit(conversation) {
            return None;
        }

        let target = (conversation.estimated_tokens() as f32 * 0.5) as usize;
        let mut cumulative = 0;
        let mut compress_up_to = 0;

        for (i, msg) in conversation.messages.iter().enumerate() {
            if msg.role == MessageRole::System {
                continue;
            }
            cumulative += msg.text_content().len() / 4;
            if cumulative > target {
                compress_up_to = i;
                break;
            }
        }

        if compress_up_to < 2 {
            return None;
        }

        let removed_count = compress_up_to;
        let kept: Vec<Message> = conversation.messages.drain(compress_up_to..).collect();
        let removed_summary: Vec<String> = conversation.messages.iter().map(|m| {
            let text = m.text_content();
            let words: Vec<&str> = text.split_whitespace().collect();
            if words.len() > 10 {
                format!("{}...", words[..10.min(words.len())].join(" "))
            } else {
                text
            }
        }).collect::<Vec<_>>();

        conversation.messages = kept;

        let summary = format!(
            "[{} earlier messages compressed. Topics discussed: {}]",
            removed_count,
            removed_summary.join(" | "),
        );

        Some(summary)
    }

    /// Ensure the conversation fits within the context window before a request.
    pub fn prepare_conversation(&self, conversation: &mut Conversation) {
        let removed = self.truncate_conversation(conversation);
        if removed > 0 {
            tracing::info!("Truncated {} messages from conversation", removed);
        }

        let compression = self.compress_conversation(conversation);
        if let Some(summary) = compression {
            tracing::info!("Compressed conversation: {}", summary);
        }
    }

    /// Emit an event if a sender is configured.
    fn emit(&self, event: Event) {
        if let Some(sender) = &self.event_sender {
            let _ = sender.try_send(event);
        }
    }
}
