use std::path::PathBuf;

use pleiades_core::error::Error;
use pleiades_memory::{ProjectMemory, SessionMemory, UserMemory};

/// Manages the multi-tier memory system for the engine.
///
/// Initializes persistent stores from config and provides
/// methods to store/retrieve conversation summaries and context.
pub struct MemoryManager {
    pub session: SessionMemory,
    pub project: ProjectMemory,
    pub user: UserMemory,
}

impl MemoryManager {
    /// Create a new memory manager with in-memory stores.
    pub fn new() -> Self {
        Self {
            session: SessionMemory::new(),
            project: ProjectMemory::new(),
            user: UserMemory::new(),
        }
    }

    /// Create a memory manager with persistent file-based stores.
    pub fn persisted(base_dir: PathBuf) -> Self {
        Self {
            session: SessionMemory::persisted(base_dir.clone()),
            project: ProjectMemory::persisted(base_dir.clone()),
            user: UserMemory::persisted(base_dir),
        }
    }

    /// Store a conversation summary in session memory.
    pub fn store_summary(&self, summary: &str) -> Result<(), Error> {
        self.session.add(summary, "conversation_summary")
    }

    /// Retrieve recent conversation summaries.
    pub fn recent_summaries(&self, limit: usize) -> Result<Vec<String>, Error> {
        let entries = self.session.recent(limit)?;
        Ok(entries.into_iter().map(|e| e.content).collect())
    }
}

impl Default for MemoryManager {
    fn default() -> Self {
        Self::new()
    }
}
