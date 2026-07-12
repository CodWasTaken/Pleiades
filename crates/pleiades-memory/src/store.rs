use pleiades_core::error::Error;

/// A memory entry with text and metadata.
#[derive(Debug, Clone)]
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
        let mut results: Vec<(f64, &MemoryEntry)> = self.entries
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

        Ok(results.into_iter().take(limit).map(|(_, e)| e.clone()).collect())
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
