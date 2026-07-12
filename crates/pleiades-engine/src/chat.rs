use pleiades_core::conversation::{Conversation, Message};
use pleiades_core::error::Error;

use crate::engine::Engine;

/// A managed chat session with streaming support.
pub struct ChatSession<'a> {
    engine: &'a Engine,
    conversation: Conversation,
    provider: String,
}

impl<'a> ChatSession<'a> {
    /// Create a new chat session.
    pub fn new(engine: &'a Engine, provider: impl Into<String>) -> Self {
        let id = uuid::Uuid::new_v4().to_string();
        Self {
            engine,
            conversation: Conversation::new(id),
            provider: provider.into(),
        }
    }

    /// Create a chat session with an existing conversation (for resume).
    pub fn from_conversation(engine: &'a Engine, conversation: Conversation, provider: impl Into<String>) -> Self {
        Self {
            engine,
            conversation,
            provider: provider.into(),
        }
    }

    /// Send a message and get a response (blocking, non-streaming).
    pub async fn send(&mut self, content: impl Into<String>) -> Result<Message, Error> {
        let message = Message::user(content);
        self.conversation.add_message(message);
        self.engine.chat(&mut self.conversation, &self.provider).await
    }

    /// Stream a chat response, processing tokens as they arrive.
    pub async fn send_stream(
        &mut self,
        content: impl Into<String>,
    ) -> Result<tokio::sync::mpsc::Receiver<pleiades_core::provider::StreamEvent>, Error> {
        let message = Message::user(content);
        self.conversation.add_message(message);
        self.engine
            .chat_stream(&mut self.conversation, &self.provider)
            .await
    }

    /// Get a reference to the conversation.
    pub fn conversation(&self) -> &Conversation {
        &self.conversation
    }

    /// Get a mutable reference to the conversation.
    pub fn conversation_mut(&mut self) -> &mut Conversation {
        &mut self.conversation
    }

    /// Get the conversation ID.
    pub fn id(&self) -> &str {
        &self.conversation.id
    }

    /// Get the provider name.
    pub fn provider(&self) -> &str {
        &self.provider
    }
}
