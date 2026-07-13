use std::path::PathBuf;
use std::sync::Arc;

use pleiades_core::error::Error;
use pleiades_core::provider::Provider;
use pleiades_prompts::BuiltinPrompts;

use crate::common::{generate_from_diff, git_diff, git_output};

/// Generate a pull-request description from commits and a diff.
pub struct PrSummaryGenerator {
    provider: Arc<dyn Provider>,
    model: String,
    repository: PathBuf,
    base: String,
    title: Option<String>,
}

impl PrSummaryGenerator {
    pub fn new(provider: Arc<dyn Provider>, model: impl Into<String>) -> Self {
        Self {
            provider,
            model: model.into(),
            repository: PathBuf::from("."),
            base: "HEAD~1".to_string(),
            title: None,
        }
    }

    pub fn with_repository(mut self, repository: impl Into<PathBuf>) -> Self {
        self.repository = repository.into();
        self
    }

    pub fn base(mut self, base: impl Into<String>) -> Self {
        self.base = base.into();
        self
    }

    pub fn title(mut self, title: Option<String>) -> Self {
        self.title = title;
        self
    }

    pub async fn generate(&self) -> Result<String, Error> {
        let range = format!("{}..HEAD", self.base);
        let diff = git_diff(&self.repository, &["diff", "--no-ext-diff", &range]).await?;
        let commits = git_output(&self.repository, &["log", "--format=- %s", &range]).await?;
        let context = format!("Commits:\n{commits}\n\nDiff:\n{diff}");
        generate_from_diff(
            self.provider.as_ref(),
            &self.model,
            BuiltinPrompts::pr_summary(),
            &context,
            self.title.as_deref(),
        )
        .await
    }
}
