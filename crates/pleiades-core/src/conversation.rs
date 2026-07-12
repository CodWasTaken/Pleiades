use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Content block types for multi-modal messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentBlock {
    Text(String),
    ImageUrl {
        url: String,
        detail: Option<String>,
    },
    ImageData {
        mime_type: String,
        data: Vec<u8>,
    },
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    ToolResult {
        id: String,
        content: String,
        is_error: bool,
    },
}

/// A single message in a conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: MessageRole,
    pub content: Vec<ContentBlock>,
    pub reasoning: Option<String>,
    pub metadata: Option<HashMap<String, String>>,
}

/// Message role types.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

impl Message {
    /// Create a new text message.
    pub fn text(role: MessageRole, content: impl Into<String>) -> Self {
        Self {
            role,
            content: vec![ContentBlock::Text(content.into())],
            reasoning: None,
            metadata: None,
        }
    }

    /// Create a system message.
    pub fn system(content: impl Into<String>) -> Self {
        Self::text(MessageRole::System, content)
    }

    /// Create a user message.
    pub fn user(content: impl Into<String>) -> Self {
        Self::text(MessageRole::User, content)
    }

    /// Create an assistant message.
    pub fn assistant(content: impl Into<String>) -> Self {
        Self::text(MessageRole::Assistant, content)
    }

    /// Get the text content of this message.
    pub fn text_content(&self) -> String {
        self.content
            .iter()
            .filter_map(|block| match block {
                ContentBlock::Text(t) => Some(t.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// Metadata about a conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMetadata {
    pub title: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub model: Option<String>,
    pub provider: Option<String>,
    pub total_tokens: Option<u64>,
    pub tags: Vec<String>,
}

impl Default for ConversationMetadata {
    fn default() -> Self {
        Self {
            title: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            model: None,
            provider: None,
            total_tokens: None,
            tags: Vec::new(),
        }
    }
}

/// Configuration for a conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationConfig {
    pub max_tokens: Option<u64>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub system_prompt: Option<String>,
    pub auto_compress: bool,
    pub compression_threshold: f32,
}

impl Default for ConversationConfig {
    fn default() -> Self {
        Self {
            max_tokens: None,
            temperature: None,
            top_p: None,
            system_prompt: None,
            auto_compress: true,
            compression_threshold: 0.8,
        }
    }
}

/// A complete conversation with message history.
#[derive(Debug, Clone)]
pub struct Conversation {
    pub id: String,
    pub messages: Vec<Message>,
    pub metadata: ConversationMetadata,
    pub config: ConversationConfig,
}

impl Conversation {
    /// Create a new conversation.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            messages: Vec::new(),
            metadata: ConversationMetadata::default(),
            config: ConversationConfig::default(),
        }
    }

    /// Add a message to the conversation.
    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
        self.metadata.updated_at = chrono::Utc::now();
    }

    /// Get the current token estimate for the conversation.
    pub fn estimated_tokens(&self) -> usize {
        self.messages
            .iter()
            .map(|m| m.text_content().len() / 4)
            .sum()
    }

    /// Check if the conversation exceeds the context window threshold.
    pub fn exceeds_threshold(&self, max_context: usize) -> bool {
        self.estimated_tokens() > (max_context as f32 * self.config.compression_threshold) as usize
    }

    /// Clear all messages (with confirmation).
    pub fn clear(&mut self) {
        self.messages.clear();
        self.metadata.updated_at = chrono::Utc::now();
    }

    /// Get the number of messages.
    pub fn len(&self) -> usize {
        self.messages.len()
    }

    /// Check if the conversation is empty.
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }
}
