use std::path::PathBuf;

use pleiades_agent_core::error::Error;
use pleiades_agent_memory::{MemoryEntry, ProjectMemory, SessionMemory, UserMemory};

#[derive(Debug, Clone)]
pub struct MemoryRecord {
    pub tier: String,
    pub entry: MemoryEntry,
}

#[derive(Debug, Clone)]
pub struct MemorySourceReport {
    pub tier: String,
    pub count: usize,
    pub sources: Vec<String>,
}

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

    pub fn recent(&self, limit: usize) -> Result<Vec<MemoryRecord>, Error> {
        let mut records = Vec::new();
        records.extend(
            self.session
                .recent(limit)?
                .into_iter()
                .map(|entry| MemoryRecord {
                    tier: "session".to_string(),
                    entry,
                }),
        );
        records.extend(
            self.project
                .recent(limit)?
                .into_iter()
                .map(|entry| MemoryRecord {
                    tier: "project".to_string(),
                    entry,
                }),
        );
        records.extend(
            self.user
                .recent(limit)?
                .into_iter()
                .map(|entry| MemoryRecord {
                    tier: "user".to_string(),
                    entry,
                }),
        );
        records.sort_by_key(|record| std::cmp::Reverse(record.entry.timestamp));
        records.truncate(limit);
        Ok(records)
    }

    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<MemoryRecord>, Error> {
        let mut records = Vec::new();
        records.extend(
            self.session
                .search(query, limit)?
                .into_iter()
                .map(|entry| MemoryRecord {
                    tier: "session".to_string(),
                    entry,
                }),
        );
        records.extend(
            self.project
                .search(query, limit)?
                .into_iter()
                .map(|entry| MemoryRecord {
                    tier: "project".to_string(),
                    entry,
                }),
        );
        records.extend(
            self.user
                .search(query, limit)?
                .into_iter()
                .map(|entry| MemoryRecord {
                    tier: "user".to_string(),
                    entry,
                }),
        );
        records.sort_by_key(|record| std::cmp::Reverse(record.entry.timestamp));
        records.truncate(limit);
        Ok(records)
    }

    pub fn add_user(&self, content: &str) -> Result<(), Error> {
        self.user.add(content, "user")
    }

    pub fn forget(&self, id: &str) -> Result<bool, Error> {
        let resolved = self.resolve_id(id)?;
        Ok(self.session.delete(&resolved)?
            || self.project.delete(&resolved)?
            || self.user.delete(&resolved)?)
    }

    pub fn clear_all(&self) -> Result<(), Error> {
        self.session.clear()?;
        self.project.clear()?;
        self.user.clear()
    }

    pub fn sources(&self) -> Result<Vec<MemorySourceReport>, Error> {
        Ok([
            ("session", self.session.recent(usize::MAX)?),
            ("project", self.project.recent(usize::MAX)?),
            ("user", self.user.recent(usize::MAX)?),
        ]
        .into_iter()
        .map(|(tier, entries)| {
            let mut sources = entries
                .iter()
                .map(|entry| entry.source.clone())
                .collect::<Vec<_>>();
            sources.sort();
            sources.dedup();
            MemorySourceReport {
                tier: tier.to_string(),
                count: entries.len(),
                sources,
            }
        })
        .collect())
    }

    fn resolve_id(&self, id: &str) -> Result<String, Error> {
        let matches = self
            .recent(usize::MAX)?
            .into_iter()
            .filter(|record| record.entry.id == id || record.entry.id.starts_with(id))
            .map(|record| record.entry.id)
            .collect::<Vec<_>>();
        match matches.as_slice() {
            [one] => Ok(one.clone()),
            [] => Ok(id.to_string()),
            _ => Err(Error::invalid_input(format!(
                "Memory id prefix `{id}` is ambiguous"
            ))),
        }
    }
}

impl Default for MemoryManager {
    fn default() -> Self {
        Self::new()
    }
}
