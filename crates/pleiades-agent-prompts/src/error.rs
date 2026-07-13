use thiserror::Error;

#[derive(Debug, Error)]
pub enum PromptError {
    #[error("Prompt template '{template}' has an unterminated variable")]
    Unterminated { template: String },

    #[error(
        "Prompt template '{template}' is missing variable '{name}' and no default was supplied"
    )]
    MissingVariable { name: String, template: String },

    #[error("Prompt '{0}' not found in library")]
    NotFound(String),

    #[error("Prompt error: {0}")]
    Other(String),
}

impl From<PromptError> for pleiades_agent_core::error::Error {
    fn from(e: PromptError) -> Self {
        pleiades_agent_core::error::Error::plugin(e.to_string())
    }
}
