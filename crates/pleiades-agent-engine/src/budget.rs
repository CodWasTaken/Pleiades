//! Process-local usage accounting and task budgets.
//!
//! Budgets are enforced by the runtime, not the TUI.  This keeps
//! cancellation behavior identical for interactive and headless frontends.

use std::time::{Duration, Instant};

use pleiades_agent_core::provider::Usage;
use serde::{Deserialize, Serialize};

/// Optional limits for one live runtime process.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct BudgetLimits {
    pub token_limit: Option<u64>,
    pub cost_limit_usd: Option<f64>,
    pub time_limit_secs: Option<u64>,
    pub tool_limit: Option<u64>,
}

/// Accumulated usage for one live runtime process.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct UsageTotals {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_write_tokens: u64,
    pub tool_calls: u64,
    pub provider_latency_ms: u64,
    pub tool_time_ms: u64,
}

impl UsageTotals {
    pub fn total_tokens(&self) -> u64 {
        self.input_tokens + self.output_tokens + self.cache_read_tokens + self.cache_write_tokens
    }
}

/// Human-readable runtime usage and budget report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BudgetReport {
    pub totals: UsageTotals,
    pub limits: BudgetLimits,
    pub elapsed_secs: u64,
    pub estimated_cost_usd: Option<f64>,
    pub rate_limit: Option<String>,
}

/// A limit breach that should stop the active task.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BudgetBreach {
    pub message: String,
}

/// Mutable budget state owned by the runtime context.
#[derive(Debug, Clone)]
pub struct BudgetService {
    limits: BudgetLimits,
    totals: UsageTotals,
    started: Instant,
    rate_limit: Option<String>,
}

impl Default for BudgetService {
    fn default() -> Self {
        Self::new()
    }
}

impl BudgetService {
    pub fn new() -> Self {
        Self {
            limits: BudgetLimits::default(),
            totals: UsageTotals::default(),
            started: Instant::now(),
            rate_limit: None,
        }
    }

    pub fn limits(&self) -> &BudgetLimits {
        &self.limits
    }

    pub fn set_token_limit(&mut self, limit: u64) {
        self.limits.token_limit = Some(limit);
    }

    pub fn set_cost_limit_usd(&mut self, limit: f64) {
        self.limits.cost_limit_usd = Some(limit);
    }

    pub fn set_time_limit(&mut self, duration: Duration) {
        self.limits.time_limit_secs = Some(duration.as_secs().max(1));
    }

    pub fn set_tool_limit(&mut self, limit: u64) {
        self.limits.tool_limit = Some(limit);
    }

    pub fn reset(&mut self) {
        self.limits = BudgetLimits::default();
        self.totals = UsageTotals::default();
        self.started = Instant::now();
        self.rate_limit = None;
    }

    pub fn record_usage(&mut self, usage: &Usage) -> Result<(), BudgetBreach> {
        self.totals.input_tokens = self.totals.input_tokens.saturating_add(usage.input_tokens);
        self.totals.output_tokens = self
            .totals
            .output_tokens
            .saturating_add(usage.output_tokens);
        self.totals.cache_read_tokens = self
            .totals
            .cache_read_tokens
            .saturating_add(usage.cache_read_tokens.unwrap_or(0));
        self.totals.cache_write_tokens = self
            .totals
            .cache_write_tokens
            .saturating_add(usage.cache_write_tokens.unwrap_or(0));
        self.check()
    }

    pub fn record_provider_latency(&mut self, duration: Duration) -> Result<(), BudgetBreach> {
        self.totals.provider_latency_ms = self
            .totals
            .provider_latency_ms
            .saturating_add(duration.as_millis() as u64);
        self.check()
    }

    pub fn record_tool_call(&mut self, duration: Duration) -> Result<(), BudgetBreach> {
        self.totals.tool_calls = self.totals.tool_calls.saturating_add(1);
        self.totals.tool_time_ms = self
            .totals
            .tool_time_ms
            .saturating_add(duration.as_millis() as u64);
        self.check()
    }

    pub fn check(&self) -> Result<(), BudgetBreach> {
        if let Some(limit) = self.limits.token_limit {
            let used = self.totals.total_tokens();
            if used > limit {
                return Err(BudgetBreach {
                    message: format!("Token budget exceeded: used {used}, limit {limit}"),
                });
            }
        }
        if let Some(limit) = self.limits.time_limit_secs {
            let used = self.started.elapsed().as_secs();
            if used > limit {
                return Err(BudgetBreach {
                    message: format!("Time budget exceeded: used {used}s, limit {limit}s"),
                });
            }
        }
        if let Some(limit) = self.limits.tool_limit {
            let used = self.totals.tool_calls;
            if used > limit {
                return Err(BudgetBreach {
                    message: format!("Tool-call budget exceeded: used {used}, limit {limit}"),
                });
            }
        }
        Ok(())
    }

    pub fn report(&self) -> BudgetReport {
        BudgetReport {
            totals: self.totals.clone(),
            limits: self.limits.clone(),
            elapsed_secs: self.started.elapsed().as_secs(),
            estimated_cost_usd: None,
            rate_limit: self.rate_limit.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_budget_detects_breach() {
        let mut budget = BudgetService::new();
        budget.set_token_limit(10);
        let result = budget.record_usage(&Usage {
            input_tokens: 9,
            output_tokens: 2,
            cache_read_tokens: None,
            cache_write_tokens: None,
        });
        assert!(
            result
                .unwrap_err()
                .message
                .contains("Token budget exceeded")
        );
    }

    #[test]
    fn tool_budget_detects_breach_after_recording_call() {
        let mut budget = BudgetService::new();
        budget.set_tool_limit(0);
        let result = budget.record_tool_call(Duration::from_millis(5));
        assert!(
            result
                .unwrap_err()
                .message
                .contains("Tool-call budget exceeded")
        );
    }
}
