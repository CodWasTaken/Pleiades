//! Prompt template library for Pleiades.
//!
//! Provides a small, dependency-light template engine with `{{variable}}`
//! substitution and optional defaults (`{{variable|default}}`), a built-in
//! set of prompts (assistant system prompt, summarizer, code reviewer, commit
//! generator, etc.), and a per-user library for registering custom prompts.

pub mod builtin;
pub mod error;
pub mod library;
pub mod template;

pub use builtin::BuiltinPrompts;
pub use error::PromptError;
pub use library::{PromptLibrary, PromptSummary, StoredPrompt};
pub use template::PromptTemplate;
