//! Session persistence and management.
//!
//! Saves/loads conversations as JSON files and provides a manager
//! for multi-session lifecycle (list, show metadata, delete, export).

use std::path::{Path, PathBuf};

use pleiades_core::conversation::{Conversation, ConversationMetadata};
use pleiades_core::error::Error;

/// Handles persistence of conversation sessions to disk.
pub struct SessionStore {
    sessions_dir: PathBuf,
}

impl SessionStore {
    /// Create a new session store rooted at the given directory.
    pub fn new(sessions_dir: PathBuf) -> Self {
        Self { sessions_dir }
    }

    /// Create a session store using the default platform session directory.
    pub fn default_dir() -> Self {
        let base = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("pleiades")
            .join("sessions");
        Self { sessions_dir: base }
    }

    /// Create a session store with config-based directory.
    pub fn from_config(config: &pleiades_config::Config) -> Self {
        let dir = config
            .session
            .history_dir
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                dirs::data_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join("pleiades")
                    .join("sessions")
            });
        Self { sessions_dir: dir }
    }

    /// Save a conversation to disk.
    pub fn save(&self, conversation: &Conversation) -> Result<(), Error> {
        std::fs::create_dir_all(&self.sessions_dir)
            .map_err(|e| Error::Io(format!("Failed to create sessions dir: {}", e)))?;

        let path = self.session_path(&conversation.id);
        let json = serde_json::to_string_pretty(conversation)
            .map_err(|e| Error::Serialization(e.to_string()))?;

        std::fs::write(&path, &json)
            .map_err(|e| Error::Io(format!("Failed to save session: {}", e)))
    }

    /// Load a conversation from disk by ID.
    pub fn load(&self, id: &str) -> Result<Conversation, Error> {
        let path = self.session_path(id);
        if !path.exists() {
            return Err(Error::InvalidInput(format!("Session '{}' not found", id)));
        }

        let json = std::fs::read_to_string(&path)
            .map_err(|e| Error::Io(format!("Failed to read session: {}", e)))?;

        serde_json::from_str(&json)
            .map_err(|e| Error::Serialization(e.to_string()))
    }

    /// Delete a session file by ID.
    pub fn delete(&self, id: &str) -> Result<(), Error> {
        let path = self.session_path(id);
        if !path.exists() {
            return Err(Error::InvalidInput(format!("Session '{}' not found", id)));
        }
        std::fs::remove_file(&path)
            .map_err(|e| Error::Io(format!("Failed to delete session: {}", e)))
    }

    /// List all saved sessions with their metadata.
    pub fn list(&self) -> Result<Vec<SessionInfo>, Error> {
        if !self.sessions_dir.exists() {
            return Ok(Vec::new());
        }

        let mut sessions = Vec::new();
        let entries = std::fs::read_dir(&self.sessions_dir)
            .map_err(|e| Error::Io(format!("Failed to read sessions dir: {}", e)))?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "json") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    match self.load_metadata(stem) {
                        Ok(meta) => sessions.push(SessionInfo {
                            id: stem.to_string(),
                            metadata: meta,
                        }),
                        Err(_) => {
                            // If we can't read metadata, still include basic info
                            sessions.push(SessionInfo {
                                id: stem.to_string(),
                                metadata: ConversationMetadata::default(),
                            });
                        }
                    }
                }
            }
        }

        sessions.sort_by_key(|b| std::cmp::Reverse(b.metadata.updated_at));
        Ok(sessions)
    }

    /// Export a conversation to markdown format.
    pub fn export_markdown(&self, id: &str) -> Result<String, Error> {
        let conversation = self.load(id)?;
        let title = conversation.metadata.title.as_deref().unwrap_or("Untitled");
        let mut output = format!("# {}\n\n", title);
        output.push_str(&format!(
            "- **Created**: {}\n",
            conversation.metadata.created_at.format("%Y-%m-%d %H:%M UTC")
        ));
        output.push_str(&format!(
            "- **Updated**: {}\n",
            conversation.metadata.updated_at.format("%Y-%m-%d %H:%M UTC")
        ));
        if let Some(ref model) = conversation.metadata.model {
            output.push_str(&format!("- **Model**: {}\n", model));
        }
        if let Some(ref provider) = conversation.metadata.provider {
            output.push_str(&format!("- **Provider**: {}\n", provider));
        }
        if let Some(tokens) = conversation.metadata.total_tokens {
            output.push_str(&format!("- **Total Tokens**: {}\n", tokens));
        }
        if !conversation.metadata.tags.is_empty() {
            output.push_str(&format!("- **Tags**: {}\n", conversation.metadata.tags.join(", ")));
        }
        output.push_str("\n---\n\n");

        for msg in &conversation.messages {
            let role = format!("{:?}", msg.role).to_lowercase();
            let content = msg.text_content();
            output.push_str(&format!("### {}\n\n{}\n\n", role, content));
        }

        Ok(output)
    }

    /// Export a conversation to JSON format.
    pub fn export_json(&self, id: &str) -> Result<String, Error> {
        let conversation = self.load(id)?;
        serde_json::to_string_pretty(&conversation)
            .map_err(|e| Error::Serialization(e.to_string()))
    }

    /// Load only the metadata from a session (without full conversation).
    fn load_metadata(&self, id: &str) -> Result<ConversationMetadata, Error> {
        let path = self.session_path(id);
        let json = std::fs::read_to_string(&path)
            .map_err(|e| Error::Io(format!("Failed to read session: {}", e)))?;

        let conv: Conversation = serde_json::from_str(&json)
            .map_err(|e| Error::Serialization(e.to_string()))?;

        Ok(conv.metadata)
    }

    /// Get the file path for a session ID.
    fn session_path(&self, id: &str) -> PathBuf {
        self.sessions_dir.join(format!("{}.json", id))
    }

    /// Get the sessions directory path.
    pub fn dir(&self) -> &Path {
        &self.sessions_dir
    }

    /// Count saved sessions.
    pub fn count(&self) -> Result<usize, Error> {
        Ok(self.list()?.len())
    }
}

/// Summary info for a saved session.
#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub id: String,
    pub metadata: ConversationMetadata,
}

impl SessionInfo {
    /// Get a human-readable summary line.
    pub fn summary(&self) -> String {
        let created = self.metadata.created_at.format("%Y-%m-%d %H:%M");
        let model = self.metadata.model.as_deref().unwrap_or("?");
        let tokens = self.metadata.total_tokens.map(|t| t.to_string()).unwrap_or_else(|| "?".to_string());
        format!("{} | {} | model: {} | tokens: {}", &self.id[..8], created, model, tokens)
    }
}
