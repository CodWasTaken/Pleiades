use pleiades_core::error::Error;

/// Generate commit messages from staged changes.
pub struct CommitGenerator;

impl Default for CommitGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl CommitGenerator {
    pub fn new() -> Self {
        Self
    }

    /// Generate a commit message from the current git diff.
    pub async fn generate(&self) -> Result<String, Error> {
        // Commit generation will be implemented in Milestone 14
        Err(Error::NotImplemented("Commit generation not yet implemented".to_string()))
    }
}
