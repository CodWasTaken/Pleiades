use pleiades_core::error::Error;

/// Generate code reviews from diffs.
pub struct ReviewGenerator;

impl Default for ReviewGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl ReviewGenerator {
    pub fn new() -> Self {
        Self
    }

    /// Generate a code review from the current staged diff.
    pub async fn generate(&self) -> Result<String, Error> {
        // Review generation will be implemented in Milestone 14
        Err(Error::NotImplemented("Review generation not yet implemented".to_string()))
    }
}
