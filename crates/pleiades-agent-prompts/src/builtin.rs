use crate::template::PromptTemplate;
use std::sync::OnceLock;

/// Built-in prompt templates shipped with Pleiades.
pub struct BuiltinPrompts;

impl BuiltinPrompts {
    /// All built-in templates.
    pub fn all() -> Vec<PromptTemplate> {
        static BUILTINS: OnceLock<Vec<PromptTemplate>> = OnceLock::new();
        BUILTINS
            .get_or_init(|| {
                vec![
                    Self::default_assistant(),
                    Self::summarizer(),
                    Self::code_reviewer(),
                    Self::commit_message(),
                    Self::pr_summary(),
                    Self::explain_diff(),
                    Self::refactor(),
                    Self::test_generator(),
                ]
            })
            .clone()
    }

    /// Default assistant system prompt.
    pub fn default_assistant() -> PromptTemplate {
        PromptTemplate::new(
            "default-assistant",
            "Default system prompt for the Pleiades assistant",
            "You are Pleiades, a professional autonomous coding agent working inside a selected project.\n\
             Behave like a careful software engineer, not a turn-based chatbot. Continue through the complete task whenever safe and practical.\n\n\
             Development protocol:\n\
             - First understand the request and inspect relevant repository files, instructions, conventions, and existing tests.\n\
             - For substantial work, formulate a focused internal execution plan before changing files. Do not edit blindly.\n\
             - Make the smallest coherent changes that fully solve the underlying problem; preserve unrelated user work.\n\
             - Use read, glob, and grep to investigate; use edit or write for focused changes; use bash for appropriate formatting, linting, tests, and builds.\n\
             - Diagnose and repair failures when practical. Never claim a command, build, test, formatter, linter, or check passed unless you actually ran it and observed a successful result. If verification was skipped, blocked, or failed, say that plainly.\n\
             - Review the final diff for correctness, security, accidental changes, and missing coverage before finishing.\n\
             - End with a concise report covering the underlying cause or goal, files changed, important decisions, exact validation commands and observed outcomes, and any remaining risks.\n\
             - Ask one focused question only when a missing decision would materially change the solution and cannot be learned from the repository.\n\
             - Respect the active permission mode and workspace boundary. Never evade a denial or access files outside the selected workspace.\n\
             - Operating system context: {{os|linux}}. Selected project root: {{cwd|./}}.",
        )
    }

    /// Summarization prompt used by the memory/compression system.
    pub fn summarizer() -> PromptTemplate {
        PromptTemplate::new(
            "summarizer",
            "Concise conversation summarizer",
            "Summarize the following conversation concisely in 2-3 sentences.\n\
             Focus on key decisions, code changes, and important context.\n\n\
             {{conversation}}",
        )
    }

    /// Code review prompt.
    pub fn code_reviewer() -> PromptTemplate {
        PromptTemplate::new(
            "code-reviewer",
            "Generate a code review from a diff",
            "You are a senior engineer performing a code review.\n\
             Review the following diff and report issues by severity (critical, warning, nit).\n\
             For each issue, cite the file and line range and suggest a concrete fix.\n\n\
             Diff:\n\
             {{diff}}",
        )
    }

    /// Commit message generator prompt.
    pub fn commit_message() -> PromptTemplate {
        PromptTemplate::new(
            "commit-message",
            "Generate a conventional commit message from a diff",
            "Write a git commit message for the staged changes below.\n\
             Use the Conventional Commits format (type: subject). Keep the subject under 72 chars.\n\
             Add a short body only when it adds real context.\n\n\
             Diff:\n\
             {{diff}}",
        )
    }

    /// Pull request summary prompt.
    pub fn pr_summary() -> PromptTemplate {
        PromptTemplate::new(
            "pr-summary",
            "Summarize a pull request",
            "Summarize the following pull request changes for a reviewer.\n\
             Include: what changed, why, and any risk areas.\n\n\
             Title: {{title|Untitled}}\n\
             Diff:\n\
             {{diff}}",
        )
    }

    /// Diff explanation prompt.
    pub fn explain_diff() -> PromptTemplate {
        PromptTemplate::new(
            "explain-diff",
            "Explain a diff in plain language",
            "Explain the following diff to a teammate in plain language.\n\
             Describe the intent and impact without repeating the raw lines.\n\n\
             {{diff}}",
        )
    }

    /// Refactor guidance prompt.
    pub fn refactor() -> PromptTemplate {
        PromptTemplate::new(
            "refactor",
            "Refactor a code block with guidance",
            "Refactor the following {{language|rust}} code to improve {{goal|readability}}.\n\
             Preserve behavior. Return the full updated code in a fenced block.\n\n\
             {{code}}",
        )
    }

    /// Test generation prompt.
    pub fn test_generator() -> PromptTemplate {
        PromptTemplate::new(
            "test-generator",
            "Generate tests for a code block",
            "Write unit tests for the following {{language|rust}} code.\n\
             Cover the main behavior and edge cases. Return the test code in a fenced block.\n\n\
             {{code}}",
        )
    }
}

#[cfg(test)]
mod tests {
    use super::BuiltinPrompts;

    #[test]
    fn coding_agent_prompt_requires_inspection_validation_and_evidence() {
        let template = BuiltinPrompts::default_assistant();
        let prompt = template.raw();
        assert!(prompt.contains("inspect relevant repository files"));
        assert!(prompt.contains("Never claim a command, build, test"));
        assert!(prompt.contains("If verification was skipped, blocked, or failed"));
        assert!(prompt.contains("Review the final diff"));
        assert!(prompt.contains("remaining risks"));
    }
}
