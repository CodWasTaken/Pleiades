//! Doom-loop detection for repeated autonomous failures.

use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

/// A repeated agent behavior that should be bounded.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LoopSignal {
    ToolFailure {
        tool: String,
        target: String,
        error: String,
    },
    StreamError {
        message: String,
    },
    RepeatedRead {
        target: String,
    },
    RepeatedCommand {
        command: String,
    },
}

impl LoopSignal {
    fn label(&self) -> String {
        match self {
            Self::ToolFailure {
                tool,
                target,
                error,
            } => {
                format!("identical failure for `{tool}` on `{target}`: {error}")
            }
            Self::StreamError { message } => format!("identical stream error: {message}"),
            Self::RepeatedRead { target } => format!("repeated read without progress: {target}"),
            Self::RepeatedCommand { command } => format!("repeated command: {command}"),
        }
    }
}

/// Sliding-window repeat detector.
#[derive(Debug, Clone)]
pub struct DoomLoopDetector {
    max_repeats: usize,
    window: VecDeque<LoopSignal>,
}

impl DoomLoopDetector {
    pub fn new(max_repeats: usize) -> Self {
        Self {
            max_repeats: max_repeats.max(2),
            window: VecDeque::with_capacity(max_repeats.max(2) * 4),
        }
    }

    /// Record a signal. Returns a stop reason when the configured cap is hit.
    pub fn record(&mut self, signal: LoopSignal) -> Option<String> {
        self.window.push_back(signal.clone());
        while self.window.len() > self.max_repeats * 4 {
            self.window.pop_front();
        }
        let count = self
            .window
            .iter()
            .filter(|existing| *existing == &signal)
            .count();
        if count >= self.max_repeats {
            Some(format!(
                "Stopping: {} repeated {} times",
                signal.label(),
                count
            ))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stops_after_identical_failure_repeats() {
        let mut detector = DoomLoopDetector::new(3);
        let signal = LoopSignal::ToolFailure {
            tool: "bash".to_string(),
            target: "cargo test".to_string(),
            error: "failed".to_string(),
        };
        assert!(detector.record(signal.clone()).is_none());
        assert!(detector.record(signal.clone()).is_none());
        let reason = detector.record(signal).unwrap();
        assert!(reason.contains("repeated 3 times"));
    }
}
