//! Session persistence and management.
//!
//! Saves/loads conversations as JSON files and provides a manager
//! for multi-session lifecycle (list, show metadata, delete, export).

use std::path::{Path, PathBuf};

use pleiades_agent_core::conversation::{Conversation, ConversationMetadata};
use pleiades_agent_core::error::Error;

/// Handles persistence of conversation sessions to disk.
pub struct SessionStore {
    sessions_dir: PathBuf,
    ephemeral: bool,
}

impl SessionStore {
    /// Create a new session store rooted at the given directory.
    pub fn new(sessions_dir: PathBuf) -> Self {
        Self {
            sessions_dir,
            ephemeral: false,
        }
    }

    /// Create a session store using the default platform session directory.
    pub fn default_dir() -> Self {
        let base = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("pleiades")
            .join("sessions");
        Self {
            sessions_dir: base,
            ephemeral: false,
        }
    }

    /// Create a session store with config-based directory.
    pub fn from_config(config: &pleiades_agent_config::Config) -> Self {
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
        Self {
            sessions_dir: dir,
            ephemeral: config.session.ephemeral,
        }
    }

    /// Save a conversation to disk.
    pub fn save(&self, conversation: &Conversation) -> Result<(), Error> {
        if self.ephemeral {
            return Ok(());
        }
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
        let resolved = self.resolve_id(id)?;
        let path = self.session_path(&resolved);
        if !path.exists() {
            return Err(Error::InvalidInput(format!("Session '{}' not found", id)));
        }

        let json = std::fs::read_to_string(&path)
            .map_err(|e| Error::Io(format!("Failed to read session: {}", e)))?;

        serde_json::from_str(&json).map_err(|e| Error::Serialization(e.to_string()))
    }

    /// Delete a session file by ID.
    pub fn delete(&self, id: &str) -> Result<(), Error> {
        let resolved = self.resolve_id(id)?;
        let path = self.session_path(&resolved);
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

    /// Search saved sessions by id, title, provider, model, tags, and message text.
    pub fn search(&self, query: &str) -> Result<Vec<SessionInfo>, Error> {
        let query = query.to_lowercase();
        let mut matches = Vec::new();
        for session in self.list()? {
            let conversation = self.load(&session.id)?;
            let haystack = session_search_text(&conversation);
            if haystack.to_lowercase().contains(&query) {
                matches.push(session);
            }
        }
        Ok(matches)
    }

    /// Rename a saved session title.
    pub fn rename(&self, id: &str, title: impl Into<String>) -> Result<Conversation, Error> {
        let mut conversation = self.load(id)?;
        conversation.metadata.title = Some(title.into());
        conversation.metadata.updated_at = chrono::Utc::now();
        self.save(&conversation)?;
        Ok(conversation)
    }

    /// Fork a saved session to a new id without modifying the parent.
    pub fn fork(&self, id: &str) -> Result<Conversation, Error> {
        let conversation = self.load(id)?;
        self.fork_conversation(&conversation)
    }

    /// Fork an in-memory conversation to a new id.
    pub fn fork_conversation(&self, conversation: &Conversation) -> Result<Conversation, Error> {
        let mut conversation = conversation.clone();
        let parent_id = conversation.id.clone();
        conversation.id = self.next_fork_id(&parent_id);
        conversation.metadata.created_at = chrono::Utc::now();
        conversation.metadata.updated_at = conversation.metadata.created_at;
        let title = conversation.metadata.title.as_deref().unwrap_or("Untitled");
        conversation.metadata.title = Some(format!("Fork of {title}"));
        if !conversation.metadata.tags.iter().any(|tag| tag == "fork") {
            conversation.metadata.tags.push("fork".to_string());
        }
        self.save(&conversation)?;
        Ok(conversation)
    }

    /// Export a conversation to markdown format.
    pub fn export_markdown(&self, id: &str) -> Result<String, Error> {
        let conversation = self.load(id)?;
        let title = conversation.metadata.title.as_deref().unwrap_or("Untitled");
        let mut output = format!("# {}\n\n", title);
        output.push_str(&format!(
            "- **Created**: {}\n",
            conversation
                .metadata
                .created_at
                .format("%Y-%m-%d %H:%M UTC")
        ));
        output.push_str(&format!(
            "- **Updated**: {}\n",
            conversation
                .metadata
                .updated_at
                .format("%Y-%m-%d %H:%M UTC")
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
            output.push_str(&format!(
                "- **Tags**: {}\n",
                conversation.metadata.tags.join(", ")
            ));
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
        serde_json::to_string_pretty(&conversation).map_err(|e| Error::Serialization(e.to_string()))
    }

    /// Load only the metadata from a session (without full conversation).
    fn load_metadata(&self, id: &str) -> Result<ConversationMetadata, Error> {
        let path = self.session_path(id);
        let json = std::fs::read_to_string(&path)
            .map_err(|e| Error::Io(format!("Failed to read session: {}", e)))?;

        let conv: Conversation =
            serde_json::from_str(&json).map_err(|e| Error::Serialization(e.to_string()))?;

        Ok(conv.metadata)
    }

    /// Get the file path for a session ID.
    fn session_path(&self, id: &str) -> PathBuf {
        self.sessions_dir.join(format!("{}.json", id))
    }

    /// Resolve a full id or unique prefix to a full session id.
    pub fn resolve_id(&self, id: &str) -> Result<String, Error> {
        let exact = self.session_path(id);
        if exact.exists() {
            return Ok(id.to_string());
        }
        let matches = self
            .list()?
            .into_iter()
            .filter(|session| session.id.starts_with(id))
            .map(|session| session.id)
            .collect::<Vec<_>>();
        match matches.as_slice() {
            [one] => Ok(one.clone()),
            [] => Err(Error::InvalidInput(format!("Session '{}' not found", id))),
            _ => Err(Error::InvalidInput(format!(
                "Session prefix '{}' is ambiguous",
                id
            ))),
        }
    }

    /// Get the sessions directory path.
    pub fn dir(&self) -> &Path {
        &self.sessions_dir
    }

    /// Count saved sessions.
    pub fn count(&self) -> Result<usize, Error> {
        Ok(self.list()?.len())
    }

    /// Whether this store skips persistence for the active process.
    pub fn is_ephemeral(&self) -> bool {
        self.ephemeral
    }

    fn next_fork_id(&self, parent_id: &str) -> String {
        let safe_parent = parent_id
            .chars()
            .filter(|ch| ch.is_ascii_alphanumeric() || *ch == '-' || *ch == '_')
            .take(24)
            .collect::<String>();
        for offset in 0..1000 {
            let id = format!(
                "{}-fork-{}-{}",
                safe_parent,
                chrono::Utc::now().timestamp_millis(),
                offset
            );
            if !self.session_path(&id).exists() {
                return id;
            }
        }
        format!("{}-fork-{}", safe_parent, uuid_like_timestamp())
    }
}

fn uuid_like_timestamp() -> String {
    format!(
        "{}{}",
        chrono::Utc::now().timestamp_micros(),
        std::process::id()
    )
}

fn session_search_text(conversation: &Conversation) -> String {
    let mut text = String::new();
    text.push_str(&conversation.id);
    text.push('\n');
    if let Some(title) = &conversation.metadata.title {
        text.push_str(title);
        text.push('\n');
    }
    if let Some(provider) = &conversation.metadata.provider {
        text.push_str(provider);
        text.push('\n');
    }
    if let Some(model) = &conversation.metadata.model {
        text.push_str(model);
        text.push('\n');
    }
    for tag in &conversation.metadata.tags {
        text.push_str(tag);
        text.push('\n');
    }
    for message in &conversation.messages {
        text.push_str(&message.text_content());
        text.push('\n');
    }
    text
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
        let tokens = self
            .metadata
            .total_tokens
            .map(|t| t.to_string())
            .unwrap_or_else(|| "?".to_string());
        format!(
            "{} | {} | model: {} | tokens: {}",
            &self.id[..8],
            created,
            model,
            tokens
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pleiades_agent_core::conversation::Message;

    fn conversation(id: &str, title: &str, body: &str) -> Conversation {
        let mut conversation = Conversation::new(id);
        conversation.metadata.title = Some(title.to_string());
        conversation.metadata.provider = Some("openai".to_string());
        conversation.metadata.model = Some("gpt-test".to_string());
        conversation.metadata.tags.push("regression".to_string());
        conversation.add_message(Message::user(body));
        conversation
    }

    #[test]
    fn search_matches_metadata_and_message_text() {
        let temp = tempfile::tempdir().unwrap();
        let store = SessionStore::new(temp.path().to_path_buf());
        store
            .save(&conversation(
                "session-alpha",
                "Auth refresh",
                "token expired",
            ))
            .unwrap();

        assert_eq!(store.search("refresh").unwrap().len(), 1);
        assert_eq!(store.search("token expired").unwrap().len(), 1);
        assert_eq!(store.search("missing").unwrap().len(), 0);
    }

    #[test]
    fn rename_and_fork_do_not_modify_parent() {
        let temp = tempfile::tempdir().unwrap();
        let store = SessionStore::new(temp.path().to_path_buf());
        store
            .save(&conversation("session-alpha", "Original", "hello"))
            .unwrap();

        store.rename("session-alpha", "Renamed").unwrap();
        let fork = store.fork("session-alpha").unwrap();

        let parent = store.load("session-alpha").unwrap();
        assert_eq!(parent.metadata.title.as_deref(), Some("Renamed"));
        assert_ne!(fork.id, parent.id);
        assert_eq!(fork.metadata.title.as_deref(), Some("Fork of Renamed"));
        assert_eq!(fork.messages.len(), parent.messages.len());
    }

    #[test]
    fn resolves_unique_prefixes() {
        let temp = tempfile::tempdir().unwrap();
        let store = SessionStore::new(temp.path().to_path_buf());
        store
            .save(&conversation("abcdef-session", "A", "one"))
            .unwrap();

        assert_eq!(store.resolve_id("abcdef").unwrap(), "abcdef-session");
        assert!(store.resolve_id("missing").is_err());
    }

    #[test]
    fn ephemeral_store_does_not_persist_saves() {
        let temp = tempfile::tempdir().unwrap();
        let mut config = pleiades_agent_config::Config::default();
        config.session.history_dir = Some(temp.path().display().to_string());
        config.session.ephemeral = true;
        let store = SessionStore::from_config(&config);

        store
            .save(&conversation("session-alpha", "Ephemeral", "discard"))
            .unwrap();

        assert!(store.is_ephemeral());
        assert!(store.list().unwrap().is_empty());
    }
}
