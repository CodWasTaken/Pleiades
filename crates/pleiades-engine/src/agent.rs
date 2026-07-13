use pleiades_core::conversation::{Conversation, Message};
use pleiades_core::error::Error;

use crate::Engine;

/// Agent execution for multi-step, tool-using AI interactions.
///
/// A configured entry point for executing a single task through an [`Engine`].
pub struct Agent {
    engine: Option<Engine>,
    provider_name: String,
}

impl Default for Agent {
    fn default() -> Self {
        Self::new()
    }
}

impl Agent {
    pub fn new() -> Self {
        Self {
            engine: None,
            provider_name: "openai".to_string(),
        }
    }

    /// Create an agent backed by an engine and registered provider.
    pub fn with_engine(engine: Engine, provider_name: impl Into<String>) -> Self {
        Self {
            engine: Some(engine),
            provider_name: provider_name.into(),
        }
    }

    /// Execute a task and return the assistant's text response.
    pub async fn execute(&self, task: &str) -> Result<String, Error> {
        if task.trim().is_empty() {
            return Err(Error::invalid_input("agent task cannot be empty"));
        }
        let engine = self.engine.as_ref().ok_or_else(|| {
            Error::config("agent is not configured; construct it with Agent::with_engine")
        })?;
        let mut conversation = Conversation::new(format!(
            "agent_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
        ));
        conversation.add_message(Message::user(task));
        let response = engine.chat(&mut conversation, &self.provider_name).await?;
        Ok(response.text_content())
    }
}
