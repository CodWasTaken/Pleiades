use std::path::{Path, PathBuf};
use std::sync::Arc;

use pleiades_core::error::Error;
use pleiades_core::provider::Provider;
use pleiades_prompts::BuiltinPrompts;

use crate::common::{generate_from_diff, git_diff};

/// Generate conventional commit messages from staged changes.
pub struct CommitGenerator {
    provider: Arc<dyn Provider>,
    model: String,
    repository: PathBuf,
}

impl CommitGenerator {
    pub fn new(provider: Arc<dyn Provider>, model: impl Into<String>) -> Self {
        Self {
            provider,
            model: model.into(),
            repository: PathBuf::from("."),
        }
    }

    pub fn with_repository(mut self, repository: impl Into<PathBuf>) -> Self {
        self.repository = repository.into();
        self
    }

    /// Generate a commit message from `git diff --staged`.
    pub async fn generate(&self) -> Result<String, Error> {
        let diff = staged_diff(&self.repository).await?;
        generate_from_diff(
            self.provider.as_ref(),
            &self.model,
            BuiltinPrompts::commit_message(),
            &diff,
            None,
        )
        .await
    }
}

pub async fn staged_diff(repository: &Path) -> Result<String, Error> {
    git_diff(repository, &["diff", "--staged", "--no-ext-diff"]).await
}

pub async fn working_diff(repository: &Path, staged: bool) -> Result<String, Error> {
    if staged {
        staged_diff(repository).await
    } else {
        git_diff(repository, &["diff", "--no-ext-diff"]).await
    }
}
