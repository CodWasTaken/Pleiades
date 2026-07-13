use std::collections::HashMap;
use std::sync::Arc;

use pleiades_agent_core::conversation::{Conversation, Message, MessageRole};
use pleiades_agent_core::error::Error;
use pleiades_agent_core::event::Event;
use pleiades_agent_core::model::ModelRegistry;
use pleiades_agent_core::provider::{ChatRequest, Provider, StreamEvent};
use pleiades_agent_core::tool::{PermissionLevel, Tool, ToolContext};

use pleiades_agent_config::types::Config;

use crate::memory::MemoryManager;

/// The main engine that orchestrates AI interactions.
///
/// The engine ties together providers, tools, conversations,
/// and the model registry into a cohesive interaction loop.
pub struct Engine {
    providers: HashMap<String, Box<dyn Provider>>,
    tools: Vec<Box<dyn Tool>>,
    model_registry: ModelRegistry,
    config: Arc<Config>,
    sandbox_mode: String,
    event_sender: Option<tokio::sync::mpsc::Sender<Event>>,
    memory: MemoryManager,
}

impl Engine {
    /// Create a new engine with the given configuration.
    pub fn new(config: Config) -> Self {
        let mem_dir = dirs::data_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("pleiades")
            .join("memory");
        Self {
            providers: HashMap::new(),
            tools: Vec::new(),
            model_registry: ModelRegistry::new(),
            config: Arc::new(config),
            sandbox_mode: "workspace-write".to_string(),
            event_sender: None,
            memory: MemoryManager::persisted(mem_dir),
        }
    }

    /// Build a fully configured engine from application configuration.
    ///
    /// Provider and tool construction belongs to the engine layer so terminal
    /// frontends only exchange commands and events with the runtime.
    pub fn configured(config: Config, sandbox_mode: &str) -> Self {
        let mut engine = Self::new(config.clone());
        engine.sandbox_mode = sandbox_mode.to_string();
        let providers = pleiades_agent_providers::ProviderRegistry::from_config(&config);
        for provider in providers.into_providers() {
            engine.register_provider(provider);
        }
        if config.providers.contains_key("openai-subscription") {
            engine.register_provider(Box::new(
                pleiades_agent_providers::codex::CodexCliProvider::new()
                    .with_sandbox_mode(sandbox_mode),
            ));
        }

        let mut tools = pleiades_agent_tools::ToolRegistry::new();
        tools.register_defaults();
        for tool in tools.into_tools() {
            engine.register_tool(tool);
        }
        engine
    }

    /// Create a new engine with a custom memory manager.
    pub fn with_memory(config: Config, memory: MemoryManager) -> Self {
        Self {
            providers: HashMap::new(),
            tools: Vec::new(),
            model_registry: ModelRegistry::new(),
            config: Arc::new(config),
            sandbox_mode: "workspace-write".to_string(),
            event_sender: None,
            memory,
        }
    }

    /// Get a reference to the memory manager.
    pub fn memory(&self) -> &MemoryManager {
        &self.memory
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
            .ok_or_else(|| Error::ProviderNotFound {
                provider: name.to_string(),
            })
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
    pub fn tool_definitions(&self) -> Vec<pleiades_agent_core::tool::ToolDefinition> {
        self.tools.iter().map(|t| t.definition()).collect()
    }

    /// Return the permission level declared by a registered tool.
    pub fn tool_permission_level(&self, name: &str) -> Result<PermissionLevel, Error> {
        self.tools
            .iter()
            .find(|tool| tool.name() == name)
            .map(|tool| tool.permission_level())
            .ok_or_else(|| Error::ToolNotFound {
                name: name.to_string(),
            })
    }

    /// Return the human-readable description declared by a registered tool.
    pub fn tool_description(&self, name: &str) -> Result<&str, Error> {
        self.tools
            .iter()
            .find(|tool| tool.name() == name)
            .map(|tool| tool.description())
            .ok_or_else(|| Error::ToolNotFound {
                name: name.to_string(),
            })
    }

    /// Process a chat message through the engine.
    pub async fn chat(
        &self,
        conversation: &mut Conversation,
        provider_name: &str,
    ) -> Result<Message, Error> {
        self.inject_memory_context(conversation);
        self.prepare_conversation(conversation).await;
        let provider = self.get_provider(provider_name)?;
        let model = self
            .config
            .core
            .default_model
            .clone()
            .unwrap_or_else(|| provider.default_model().to_string());

        let request = ChatRequest {
            model,
            messages: conversation.messages.clone(),
            system_prompt: self.resolve_system_prompt(conversation),
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
                conversation.metadata.total_tokens.unwrap_or(0)
                    + usage.input_tokens
                    + usage.output_tokens,
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
        self.inject_memory_context(conversation);
        self.prepare_conversation(conversation).await;
        let provider = self.get_provider(provider_name)?;
        let model = self
            .config
            .core
            .default_model
            .clone()
            .unwrap_or_else(|| provider.default_model().to_string());

        let request = ChatRequest {
            model,
            messages: conversation.messages.clone(),
            system_prompt: self.resolve_system_prompt(conversation),
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
    ) -> Result<pleiades_agent_core::tool::ToolResult, Error> {
        let tool = self
            .tools
            .iter()
            .find(|t| t.name() == name)
            .ok_or_else(|| Error::ToolNotFound {
                name: name.to_string(),
            })?;

        let ctx = ToolContext {
            cwd: std::env::current_dir().map_err(|e| Error::Io(e.to_string()))?,
            working_directory: std::env::current_dir().map_err(|e| Error::Io(e.to_string()))?,
            permission_mode: if self.config.permissions.ask_always {
                pleiades_agent_core::tool::PermissionMode::Ask
            } else {
                pleiades_agent_core::tool::PermissionMode::Allow
            },
            sandbox_mode: self.sandbox_mode.clone(),
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

        let timeout_dur =
            std::time::Duration::from_secs(self.config.session.context_size.max(120) as u64);

        let result = tokio::time::timeout(timeout_dur, tool.execute(input.clone(), &ctx)).await;
        let duration = start.elapsed().as_millis() as u64;

        let result = match result {
            Ok(Ok(r)) => r,
            Ok(Err(e)) => return Err(e),
            Err(_) => {
                return Err(Error::tool(format!(
                    "Tool '{}' timed out after {}s",
                    name,
                    timeout_dur.as_secs(),
                )));
            }
        };

        self.emit(Event::ToolCompleted {
            tool: name.to_string(),
            success: result.success,
            duration_ms: duration,
        });

        Ok(result)
    }

    /// Estimate token count for a conversation using a simple heuristic.
    ///
    /// Roughly 4 characters per token for English text, plus overhead per message.
    pub fn estimate_tokens(conversation: &Conversation) -> usize {
        conversation
            .messages
            .iter()
            .map(|m| {
                let text_len = m.text_content().len();
                let role_overhead = 4; // role tag
                text_len / 4 + role_overhead
            })
            .sum()
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

        let remove_count = non_system
            .len()
            .saturating_sub(max_messages.saturating_sub(system_msgs.len()));
        let truncated: Vec<Message> = non_system.into_iter().skip(remove_count).collect();

        let mut result = system_msgs;
        result.extend(truncated);
        conversation.messages = result;

        remove_count
    }

    /// Summarize a batch of messages using the provider.
    ///
    /// Uses a simple heuristic: runs the provider with a summarization prompt.
    /// Falls back to a text-based summary if the provider call fails.
    pub async fn summarize_messages(&self, messages: &[Message]) -> String {
        if messages.is_empty() {
            return String::new();
        }

        let provider_name = self
            .config
            .core
            .default_provider
            .clone()
            .unwrap_or_else(|| "openai".to_string());
        let model = self
            .config
            .core
            .default_model
            .clone()
            .unwrap_or_else(|| "gpt-4o".to_string());

        let conversation_text: String = messages
            .iter()
            .map(|m| {
                let role = format!("{:?}", m.role).to_lowercase();
                format!("<{}>\n{}\n</{}>", role, m.text_content(), role)
            })
            .collect::<Vec<_>>()
            .join("\n\n");

        let prompt = format!(
            "Summarize the following conversation concisely in 2-3 sentences. \
             Focus on key decisions, code changes, and important context.\n\n{}",
            conversation_text,
        );

        let request = ChatRequest {
            model,
            messages: vec![Message::user(prompt)],
            system_prompt: Some(
                "You are a precise summarizer. Output ONLY the summary, no preamble.".to_string(),
            ),
            temperature: Some(0.3),
            top_p: None,
            max_tokens: Some(256),
            stop: None,
            tools: None,
        };

        if let Ok(provider) = self.get_provider(&provider_name) {
            if let Ok(response) = provider.chat(request).await {
                let summary = response.message.text_content();
                if !summary.is_empty() {
                    return summary;
                }
            }
        }

        // Fallback: simple text compression
        let words: Vec<&str> = conversation_text.split_whitespace().collect();
        if words.len() > 50 {
            format!(
                "Summary of {} messages: {}...",
                messages.len(),
                words[..50].join(" ")
            )
        } else {
            conversation_text
        }
    }

    /// Apply automatic compression when the conversation exceeds the threshold.
    ///
    /// Uses LLM-based summarization and stores summaries in persistent memory.
    /// Returns a summary message if compression was applied.
    pub async fn compress_conversation(&self, conversation: &mut Conversation) -> Option<String> {
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

        let removed: Vec<Message> = conversation.messages.drain(..compress_up_to).collect();

        let summary_text = self.summarize_messages(&removed).await;

        self.memory.store_summary(&summary_text).ok();

        let summary_msg =
            Message::system(format!("[Conversation History Summary]\n{}", summary_text,));

        conversation.messages.insert(0, summary_msg);

        tracing::info!(
            "Compressed {} messages. Summary: {}",
            removed.len(),
            summary_text
        );
        Some(summary_text)
    }

    /// Inject relevant memory context into the conversation as system messages.
    pub fn inject_memory_context(&self, conversation: &mut Conversation) {
        if let Ok(summaries) = self.memory.recent_summaries(3) {
            if !summaries.is_empty() {
                let context = summaries.join("\n---\n");
                let memory_msg =
                    Message::system(format!("[Previous Session Context]\n{}", context,));
                conversation.messages.insert(0, memory_msg);
            }
        }
    }

    /// Ensure the conversation fits within the context window before a request.
    pub async fn prepare_conversation(&self, conversation: &mut Conversation) {
        let removed = self.truncate_conversation(conversation);
        if removed > 0 {
            tracing::info!("Truncated {} messages from conversation", removed);
        }

        let compression = self.compress_conversation(conversation).await;
        if compression.is_some() {
            tracing::info!("Conversation compressed with LLM summary");
        }
    }

    /// Resolve the system prompt for a request.
    ///
    /// Uses the conversation-configured prompt when present, otherwise falls
    /// back to the built-in default assistant prompt from the prompt library.
    fn resolve_system_prompt(&self, conversation: &Conversation) -> Option<String> {
        if let Some(existing) = &conversation.config.system_prompt {
            if !existing.trim().is_empty() {
                return Some(existing.clone());
            }
        }
        let lib = pleiades_agent_prompts::PromptLibrary::with_builtins();
        let mut vars = std::collections::HashMap::new();
        vars.insert("os".to_string(), std::env::consts::OS.to_string());
        vars.insert(
            "cwd".to_string(),
            std::env::current_dir()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| ".".to_string()),
        );
        lib.render("default-assistant", &vars).ok()
    }

    /// Emit an event if a sender is configured.
    fn emit(&self, event: Event) {
        if let Some(sender) = &self.event_sender {
            let _ = sender.try_send(event);
        }
    }
}
