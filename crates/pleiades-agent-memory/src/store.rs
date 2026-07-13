use std::path::PathBuf;

use pleiades_agent_core::error::Error;
use serde::{Deserialize, Serialize};

/// A memory entry with text and metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: String,
    pub content: String,
    pub source: String,
    pub timestamp: u64,
    pub metadata: Option<serde_json::Value>,
}

/// Abstract memory storage interface.
pub trait MemoryStore: Send + Sync {
    fn insert(&mut self, entry: MemoryEntry) -> Result<(), Error>;
    fn search(&self, query: &str, limit: usize) -> Result<Vec<MemoryEntry>, Error>;
    fn recent(&self, limit: usize) -> Result<Vec<MemoryEntry>, Error>;
    fn clear(&mut self) -> Result<(), Error>;
}

/// Simple in-memory vector store for development.
pub struct InMemoryStore {
    entries: Vec<MemoryEntry>,
}

impl InMemoryStore {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}

impl MemoryStore for InMemoryStore {
    fn insert(&mut self, entry: MemoryEntry) -> Result<(), Error> {
        self.entries.push(entry);
        Ok(())
    }

    fn search(&self, query: &str, limit: usize) -> Result<Vec<MemoryEntry>, Error> {
        let query_lower = query.to_lowercase();
        let mut results: Vec<(f64, &MemoryEntry)> = self
            .entries
            .iter()
            .map(|e| {
                let relevance = if e.content.to_lowercase().contains(&query_lower) {
                    1.0
                } else if let Some(meta) = &e.metadata {
                    if meta.to_string().to_lowercase().contains(&query_lower) {
                        0.5
                    } else {
                        0.0
                    }
                } else {
                    0.0
                };
                (relevance, e)
            })
            .filter(|(r, _)| *r > 0.0)
            .collect();

        results.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        Ok(results
            .into_iter()
            .take(limit)
            .map(|(_, e)| e.clone())
            .collect())
    }

    fn recent(&self, limit: usize) -> Result<Vec<MemoryEntry>, Error> {
        let mut entries = self.entries.clone();
        entries.sort_by_key(|b| std::cmp::Reverse(b.timestamp));
        entries.truncate(limit);
        Ok(entries)
    }

    fn clear(&mut self) -> Result<(), Error> {
        self.entries.clear();
        Ok(())
    }
}

impl Default for InMemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Persistent file-based memory store.
///
/// Each memory entry is stored as an individual JSON file
/// in a directory, making it durable across sessions.
pub struct FileStore {
    dir: PathBuf,
    entries: Vec<MemoryEntry>,
    dirty: bool,
}

impl FileStore {
    pub fn new(dir: PathBuf) -> Self {
        let mut store = Self {
            dir,
            entries: Vec::new(),
            dirty: false,
        };
        let _ = store.load_from_disk();
        store
    }

    pub fn default_dir() -> PathBuf {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("pleiades")
            .join("memory")
    }

    fn store_path(&self) -> PathBuf {
        self.dir.join("store.json")
    }

    fn load_from_disk(&mut self) -> Result<(), Error> {
        let path = self.store_path();
        if !path.exists() {
            return Ok(());
        }
        let json = std::fs::read_to_string(&path)
            .map_err(|e| Error::Io(format!("Failed to read memory store: {}", e)))?;
        let entries: Vec<MemoryEntry> = serde_json::from_str(&json)
            .map_err(|e| Error::Serialization(format!("Failed to parse memory store: {}", e)))?;
        self.entries = entries;
        self.dirty = false;
        Ok(())
    }

    fn flush(&mut self) -> Result<(), Error> {
        if !self.dirty {
            return Ok(());
        }
        std::fs::create_dir_all(&self.dir)
            .map_err(|e| Error::Io(format!("Failed to create memory dir: {}", e)))?;
        let json = serde_json::to_string_pretty(&self.entries)
            .map_err(|e| Error::Serialization(e.to_string()))?;
        std::fs::write(self.store_path(), &json)
            .map_err(|e| Error::Io(format!("Failed to write memory store: {}", e)))?;
        self.dirty = false;
        Ok(())
    }
}

impl MemoryStore for FileStore {
    fn insert(&mut self, entry: MemoryEntry) -> Result<(), Error> {
        self.entries.push(entry);
        self.dirty = true;
        self.flush()?;
        Ok(())
    }

    fn search(&self, query: &str, limit: usize) -> Result<Vec<MemoryEntry>, Error> {
        let query_lower = query.to_lowercase();
        let mut results: Vec<(f64, &MemoryEntry)> = self
            .entries
            .iter()
            .map(|e| {
                let relevance = if e.content.to_lowercase().contains(&query_lower) {
                    1.0
                } else if let Some(meta) = &e.metadata {
                    if meta.to_string().to_lowercase().contains(&query_lower) {
                        0.5
                    } else {
                        0.0
                    }
                } else {
                    0.0
                };
                (relevance, e)
            })
            .filter(|(r, _)| *r > 0.0)
            .collect();

        results.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        Ok(results
            .into_iter()
            .take(limit)
            .map(|(_, e)| e.clone())
            .collect())
    }

    fn recent(&self, limit: usize) -> Result<Vec<MemoryEntry>, Error> {
        let mut entries = self.entries.clone();
        entries.sort_by_key(|b| std::cmp::Reverse(b.timestamp));
        entries.truncate(limit);
        Ok(entries)
    }

    fn clear(&mut self) -> Result<(), Error> {
        self.entries.clear();
        self.dirty = true;
        self.flush()?;
        Ok(())
    }
}

impl Drop for FileStore {
    fn drop(&mut self) {
        if self.dirty {
            let _ = self.flush();
        }
    }
}
