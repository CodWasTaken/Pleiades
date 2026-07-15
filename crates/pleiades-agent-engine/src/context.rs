//! Conversation context accounting for the live workspace.
//!
//! This module is deliberately provider-independent.  It uses the same rough
//! four-characters-per-token heuristic already used by the engine until exact
//! provider tokenizers are added.

use std::collections::BTreeSet;

use pleiades_agent_core::conversation::{ContentBlock, Conversation, MessageRole};
use serde::{Deserialize, Serialize};

/// A pinned context entry retained by the user.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextPin {
    pub id: String,
    pub target: String,
    pub tokens: usize,
}

/// Per-source token accounting for the active conversation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextReport {
    pub total_tokens: usize,
    pub provider_context_limit: usize,
    pub percent_used: u8,
    pub conversation_tokens: usize,
    pub tool_output_tokens: usize,
    pub memory_tokens: usize,
    pub compression_tokens: usize,
    pub pinned_tokens: usize,
    pub message_count: usize,
    pub tool_result_count: usize,
    pub compression_summary_count: usize,
    pub pinned: Vec<ContextPin>,
    pub sources: Vec<ContextSource>,
    pub compression_history: Vec<CompressionRecord>,
}

/// File or tool source currently represented in context.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextSource {
    pub kind: String,
    pub label: String,
    pub tokens: usize,
}

/// A compact record of a manual or automatic compaction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompressionRecord {
    pub before_tokens: usize,
    pub after_tokens: usize,
    pub summary_tokens: usize,
    pub message: String,
}

/// Builds context reports from a conversation plus runtime-maintained pins and
/// compression history.
#[derive(Debug, Clone)]
pub struct ContextAccountant {
    provider_context_limit: usize,
}

impl ContextAccountant {
    pub fn new(provider_context_limit: usize) -> Self {
        Self {
            provider_context_limit: provider_context_limit.max(1),
        }
    }

    pub fn report(
        &self,
        conversation: &Conversation,
        pinned: &[ContextPin],
        compression_history: &[CompressionRecord],
    ) -> ContextReport {
        let mut conversation_tokens = 0usize;
        let mut tool_output_tokens = 0usize;
        let mut memory_tokens = 0usize;
        let mut compression_tokens = 0usize;
        let mut tool_result_count = 0usize;
        let mut compression_summary_count = 0usize;
        let mut sources = Vec::new();
        let mut seen_sources = BTreeSet::<String>::new();

        for message in &conversation.messages {
            let text = message.text_content();
            let text_tokens = estimate_tokens(&text);
            match message.role {
                MessageRole::Tool => {
                    tool_output_tokens += text_tokens;
                    tool_result_count += 1;
                }
                MessageRole::System if text.starts_with("[Previous Session Context]") => {
                    memory_tokens += text_tokens;
                }
                MessageRole::System if text.starts_with("[Conversation History Summary]") => {
                    compression_tokens += text_tokens;
                    compression_summary_count += 1;
                }
                _ => {
                    conversation_tokens += text_tokens;
                }
            }

            for block in &message.content {
                match block {
                    ContentBlock::ToolResult { content, .. } => {
                        tool_output_tokens += estimate_tokens(content);
                        tool_result_count += 1;
                    }
                    ContentBlock::ToolUse { name, input, .. } => {
                        collect_tool_source(name, input, &mut seen_sources, &mut sources);
                    }
                    ContentBlock::ImageUrl { url, .. } => {
                        collect_source("image", url, 0, &mut seen_sources, &mut sources);
                    }
                    ContentBlock::ImageData { mime_type, data } => {
                        collect_source(
                            "image",
                            &format!("{mime_type} data, {} bytes", data.len()),
                            data.len() / 16,
                            &mut seen_sources,
                            &mut sources,
                        );
                    }
                    ContentBlock::Text(_) => {}
                }
            }
        }

        let pinned_tokens = pinned.iter().map(|pin| pin.tokens).sum::<usize>();
        let source_tokens = sources.iter().map(|source| source.tokens).sum::<usize>();
        let total_tokens = conversation_tokens
            + tool_output_tokens
            + memory_tokens
            + compression_tokens
            + pinned_tokens
            + source_tokens;
        let percent_used =
            ((total_tokens.saturating_mul(100)) / self.provider_context_limit).min(100) as u8;

        ContextReport {
            total_tokens,
            provider_context_limit: self.provider_context_limit,
            percent_used,
            conversation_tokens,
            tool_output_tokens,
            memory_tokens,
            compression_tokens,
            pinned_tokens,
            message_count: conversation.messages.len(),
            tool_result_count,
            compression_summary_count,
            pinned: pinned.to_vec(),
            sources,
            compression_history: compression_history.to_vec(),
        }
    }
}

/// Estimate tokens with a simple, deterministic heuristic.
pub fn estimate_tokens(value: &str) -> usize {
    let chars = value.chars().count();
    if chars == 0 { 0 } else { (chars / 4).max(1) }
}

pub fn make_pin(id: impl Into<String>, target: impl Into<String>) -> ContextPin {
    let target = target.into();
    ContextPin {
        id: id.into(),
        tokens: estimate_tokens(&target),
        target,
    }
}

fn collect_tool_source(
    name: &str,
    input: &serde_json::Value,
    seen: &mut BTreeSet<String>,
    sources: &mut Vec<ContextSource>,
) {
    for key in [
        "path",
        "file_path",
        "target_file",
        "url",
        "query",
        "pattern",
    ] {
        if let Some(value) = input.get(key).and_then(serde_json::Value::as_str) {
            collect_source(name, value, estimate_tokens(value), seen, sources);
        }
    }
}

fn collect_source(
    kind: &str,
    label: &str,
    tokens: usize,
    seen: &mut BTreeSet<String>,
    sources: &mut Vec<ContextSource>,
) {
    let identity = format!("{kind}:{label}");
    if seen.insert(identity) {
        sources.push(ContextSource {
            kind: kind.to_string(),
            label: label.to_string(),
            tokens,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pleiades_agent_core::conversation::{ContentBlock, Message};

    #[test]
    fn report_splits_conversation_memory_tool_and_pins() {
        let mut conversation = Conversation::new("ctx");
        conversation.add_message(Message::user("hello world"));
        conversation.add_message(Message::system("[Previous Session Context]\nremember this"));
        conversation.add_message(Message::system(
            "[Conversation History Summary]\nolder work",
        ));
        conversation.add_message(Message {
            role: MessageRole::Assistant,
            content: vec![ContentBlock::ToolUse {
                id: "read-1".to_string(),
                name: "read".to_string(),
                input: serde_json::json!({"path": "src/lib.rs"}),
            }],
            reasoning: None,
            metadata: None,
        });
        conversation.add_message(Message {
            role: MessageRole::Tool,
            content: vec![ContentBlock::ToolResult {
                id: "read-1".to_string(),
                content: "file contents".to_string(),
                is_error: false,
            }],
            reasoning: None,
            metadata: None,
        });

        let report = ContextAccountant::new(100).report(
            &conversation,
            &[make_pin("pin-1", "src/main.rs")],
            &[CompressionRecord {
                before_tokens: 80,
                after_tokens: 40,
                summary_tokens: 10,
                message: "manual".to_string(),
            }],
        );

        assert!(report.total_tokens > 0);
        assert!(report.memory_tokens > 0);
        assert!(report.tool_output_tokens > 0);
        assert!(report.compression_tokens > 0);
        assert!(report.pinned_tokens > 0);
        assert_eq!(report.compression_summary_count, 1);
        assert!(
            report
                .sources
                .iter()
                .any(|source| source.label == "src/lib.rs")
        );
        assert_eq!(report.compression_history.len(), 1);
    }
}
