use std::path::PathBuf;
use std::sync::Arc;

use pleiades_core::error::Error;
use pleiades_core::provider::Provider;
use pleiades_prompts::BuiltinPrompts;

use crate::common::{generate_from_diff, git_diff};

/// Generate structured code reviews from repository diffs.
pub struct ReviewGenerator {
    provider: Arc<dyn Provider>,
    model: String,
    repository: PathBuf,
    staged: bool,
}

impl ReviewGenerator {
    pub fn new(provider: Arc<dyn Provider>, model: impl Into<String>) -> Self {
        Self {
            provider,
            model: model.into(),
            repository: PathBuf::from("."),
            staged: false,
        }
    }

    pub fn with_repository(mut self, repository: impl Into<PathBuf>) -> Self {
        self.repository = repository.into();
        self
    }

    pub fn staged(mut self, staged: bool) -> Self {
        self.staged = staged;
        self
    }

    pub async fn generate(&self) -> Result<String, Error> {
        let arguments: &[&str] = if self.staged {
            &["diff", "--staged", "--no-ext-diff"]
        } else {
            &["diff", "--no-ext-diff"]
        };
        let diff = git_diff(&self.repository, arguments).await?;
        generate_from_diff(
            self.provider.as_ref(),
            &self.model,
            BuiltinPrompts::code_reviewer(),
            &diff,
            None,
        )
        .await
    }
}
