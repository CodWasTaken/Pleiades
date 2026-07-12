use crate::template::PromptTemplate;

/// Built-in prompt templates shipped with Pleiades.
pub struct BuiltinPrompts;

impl BuiltinPrompts {
    /// All built-in templates.
    pub fn all() -> Vec<PromptTemplate> {
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
    }

    /// Default assistant system prompt.
    pub fn default_assistant() -> PromptTemplate {
        PromptTemplate::new(
            "default-assistant",
            "Default system prompt for the Pleiades assistant",
            "You are Pleiades, a next-generation, provider-agnostic terminal AI assistant.\n\
             You help the user with software engineering tasks directly in their terminal.\n\n\
             Guidelines:\n\
             - Be concise and precise.\n\
             - Prefer editing existing files over creating new ones.\n\
             - Use the available tools to read, search, and modify the codebase.\n\
             - When you propose a change, show the exact diff or edit.\n\
             - If a task is ambiguous, ask one focused clarifying question.\n\
             - Operating system context: {{os|linux}}. Project root: {{cwd|./}}.",
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
