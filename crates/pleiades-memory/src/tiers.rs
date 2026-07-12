use std::sync::Mutex;

use pleiades_core::error::Error;
use pleiades_core::conversation::Message;

use crate::store::{InMemoryStore, MemoryEntry, MemoryStore};

/// Working memory for the current conversation context.
pub struct WorkingMemory {
    messages: Vec<Message>,
    max_tokens: usize,
}

impl WorkingMemory {
    pub fn new(max_tokens: usize) -> Self {
        Self {
            messages: Vec::new(),
            max_tokens,
        }
    }

    pub fn add(&mut self, message: Message) {
        self.messages.push(message);
    }

    pub fn estimated_tokens(&self) -> usize {
        self.messages.iter().map(|m| m.text_content().len() / 4).sum()
    }

    pub fn needs_compression(&self) -> bool {
        self.estimated_tokens() > self.max_tokens
    }

    pub fn messages(&self) -> &[Message] {
        &self.messages
    }
}

/// Session memory for the current session (across conversations).
pub struct SessionMemory {
    store: Mutex<Box<dyn MemoryStore>>,
}

impl SessionMemory {
    pub fn new() -> Self {
        Self {
            store: Mutex::new(Box::new(InMemoryStore::new())),
        }
    }

    pub fn add(&self, content: impl Into<String>, source: impl Into<String>) -> Result<(), Error> {
        let entry = MemoryEntry {
            id: uuid::Uuid::new_v4().to_string(),
            content: content.into(),
            source: source.into(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            metadata: None,
        };
        self.store.lock().unwrap().insert(entry)
    }

    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<MemoryEntry>, Error> {
        self.store.lock().unwrap().search(query, limit)
    }
}

impl Default for SessionMemory {
    fn default() -> Self {
        Self::new()
    }
}

/// Project memory for project-specific knowledge.
pub struct ProjectMemory {
    store: Mutex<Box<dyn MemoryStore>>,
}

impl ProjectMemory {
    pub fn new() -> Self {
        Self {
            store: Mutex::new(Box::new(InMemoryStore::new())),
        }
    }

    pub fn add(&self, content: impl Into<String>, source: impl Into<String>) -> Result<(), Error> {
        let entry = MemoryEntry {
            id: uuid::Uuid::new_v4().to_string(),
            content: content.into(),
            source: source.into(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            metadata: None,
        };
        self.store.lock().unwrap().insert(entry)
    }

    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<MemoryEntry>, Error> {
        self.store.lock().unwrap().search(query, limit)
    }
}

impl Default for ProjectMemory {
    fn default() -> Self {
        Self::new()
    }
}

/// User memory for long-term user preferences and patterns.
pub struct UserMemory {
    store: Mutex<Box<dyn MemoryStore>>,
}

impl UserMemory {
    pub fn new() -> Self {
        Self {
            store: Mutex::new(Box::new(InMemoryStore::new())),
        }
    }

    pub fn add(&self, content: impl Into<String>, source: impl Into<String>) -> Result<(), Error> {
        let entry = MemoryEntry {
            id: uuid::Uuid::new_v4().to_string(),
            content: content.into(),
            source: source.into(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            metadata: None,
        };
        self.store.lock().unwrap().insert(entry)
    }

    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<MemoryEntry>, Error> {
        self.store.lock().unwrap().search(query, limit)
    }
}

impl Default for UserMemory {
    fn default() -> Self {
        Self::new()
    }
}
