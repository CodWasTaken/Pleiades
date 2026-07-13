use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::{Duration, Instant};

use pleiades_core::error::Error;
use tokio::process::Command;
use tokio::time::timeout;

use crate::workflow::{Workflow, WorkflowStep};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepStatus {
    Succeeded,
    Failed,
    Skipped,
}

#[derive(Debug, Clone)]
pub struct StepResult {
    pub name: String,
    pub status: StepStatus,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub attempts: u32,
    pub duration: Duration,
}

/// Workflow execution engine.
pub struct WorkflowExecutor {
    variables: HashMap<String, String>,
    working_dir: Option<PathBuf>,
}

impl WorkflowExecutor {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            working_dir: None,
        }
    }

    pub fn with_variables(mut self, variables: HashMap<String, String>) -> Self {
        self.variables = variables;
        self
    }

    pub fn with_working_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.working_dir = Some(path.into());
        self
    }

    /// Execute a workflow.
    ///
    /// This compatibility method returns each step's standard output. Use
    /// [`execute_detailed`](Self::execute_detailed) for statuses and diagnostics.
    pub async fn execute(&self, workflow: &Workflow) -> Result<Vec<String>, Error> {
        Ok(self
            .execute_detailed(workflow)
            .await?
            .into_iter()
            .map(|r| r.stdout)
            .collect())
    }

    pub async fn execute_detailed(&self, workflow: &Workflow) -> Result<Vec<StepResult>, Error> {
        workflow
            .validate()
            .map_err(|e| Error::invalid_input(e.join("; ")))?;
        let variables = self.resolve_variables(workflow)?;
        let mut results = Vec::new();
        let mut index = 0;

        while index < workflow.steps.len() {
            if workflow.steps[index].is_parallel() {
                let start = index;
                while index < workflow.steps.len() && workflow.steps[index].is_parallel() {
                    index += 1;
                }
                let futures = workflow.steps[start..index]
                    .iter()
                    .map(|step| self.execute_step(step, &variables));
                let batch = futures::future::join_all(futures).await;
                for result in batch {
                    results.push(result?);
                }
            } else {
                let result = self
                    .execute_step(&workflow.steps[index], &variables)
                    .await?;
                results.push(result);
                index += 1;
            }

            if let Some(failed) = results.iter().find(|r| r.status == StepStatus::Failed) {
                return Err(Error::tool(format!(
                    "workflow step '{}' failed after {} attempt(s): {}",
                    failed.name,
                    failed.attempts,
                    failed.stderr.trim()
                )));
            }
        }
        Ok(results)
    }

    fn resolve_variables(&self, workflow: &Workflow) -> Result<HashMap<String, String>, Error> {
        let mut values = self.variables.clone();
        for declaration in workflow.variables.as_deref().unwrap_or_default() {
            let (name, default) = declaration.split_once('=').unwrap_or((declaration, ""));
            let name = name.trim();
            if name.is_empty() {
                return Err(Error::invalid_input(
                    "workflow variable name cannot be empty",
                ));
            }
            if !values.contains_key(name) {
                if let Ok(value) = std::env::var(name) {
                    values.insert(name.to_string(), value);
                } else if declaration.contains('=') {
                    values.insert(name.to_string(), default.to_string());
                } else {
                    return Err(Error::invalid_input(format!(
                        "missing workflow variable '{name}'"
                    )));
                }
            }
        }
        Ok(values)
    }

    async fn execute_step(
        &self,
        step: &WorkflowStep,
        variables: &HashMap<String, String>,
    ) -> Result<StepResult, Error> {
        if !evaluate_condition(step.condition.as_deref(), variables)? {
            return Ok(StepResult {
                name: step.name.clone(),
                status: StepStatus::Skipped,
                stdout: String::new(),
                stderr: String::new(),
                exit_code: None,
                attempts: 0,
                duration: Duration::ZERO,
            });
        }

        let command = substitute(&step.command, variables)?;
        let args = step
            .args
            .as_deref()
            .unwrap_or_default()
            .iter()
            .map(|arg| substitute(arg, variables))
            .collect::<Result<Vec<_>, _>>()?;
        let max_attempts = step.retry.unwrap_or(0).saturating_add(1);
        let started = Instant::now();
        let mut last = None;

        for attempt in 1..=max_attempts {
            let mut process = Command::new(&command);
            process
                .args(&args)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .kill_on_drop(true);
            if let Some(dir) = &self.working_dir {
                process.current_dir(dir);
            }
            let run = process.output();
            let output = if let Some(seconds) = step.timeout {
                match timeout(Duration::from_secs(seconds), run).await {
                    Ok(Ok(output)) => output,
                    Ok(Err(error)) => {
                        last = Some(StepResult {
                            name: step.name.clone(),
                            status: StepStatus::Failed,
                            stdout: String::new(),
                            stderr: error.to_string(),
                            exit_code: None,
                            attempts: attempt,
                            duration: started.elapsed(),
                        });
                        continue;
                    }
                    Err(_) => {
                        last = Some(StepResult {
                            name: step.name.clone(),
                            status: StepStatus::Failed,
                            stdout: String::new(),
                            stderr: format!("timed out after {seconds}s"),
                            exit_code: None,
                            attempts: attempt,
                            duration: started.elapsed(),
                        });
                        continue;
                    }
                }
            } else {
                match run.await {
                    Ok(output) => output,
                    Err(error) => {
                        last = Some(StepResult {
                            name: step.name.clone(),
                            status: StepStatus::Failed,
                            stdout: String::new(),
                            stderr: error.to_string(),
                            exit_code: None,
                            attempts: attempt,
                            duration: started.elapsed(),
                        });
                        continue;
                    }
                }
            };

            let result = StepResult {
                name: step.name.clone(),
                status: if output.status.success() {
                    StepStatus::Succeeded
                } else {
                    StepStatus::Failed
                },
                stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
                stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
                exit_code: output.status.code(),
                attempts: attempt,
                duration: started.elapsed(),
            };
            if result.status == StepStatus::Succeeded {
                return Ok(result);
            }
            last = Some(result);
        }
        Ok(last.expect("at least one execution attempt"))
    }
}

impl Default for WorkflowExecutor {
    fn default() -> Self {
        Self::new()
    }
}

fn substitute(input: &str, variables: &HashMap<String, String>) -> Result<String, Error> {
    let mut output = String::new();
    let mut rest = input;
    while let Some(start) = rest.find("{{") {
        output.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        let end = after
            .find("}}")
            .ok_or_else(|| Error::invalid_input(format!("unclosed variable in '{input}'")))?;
        let name = after[..end].trim();
        let value = variables
            .get(name)
            .cloned()
            .or_else(|| std::env::var(name).ok())
            .ok_or_else(|| Error::invalid_input(format!("unknown workflow variable '{name}'")))?;
        output.push_str(&value);
        rest = &after[end + 2..];
    }
    output.push_str(rest);
    Ok(output)
}

fn evaluate_condition(
    condition: Option<&str>,
    variables: &HashMap<String, String>,
) -> Result<bool, Error> {
    let Some(expression) = condition.map(str::trim).filter(|s| !s.is_empty()) else {
        return Ok(true);
    };
    if let Some((left, right)) = expression.split_once("!=") {
        return Ok(resolve_operand(left, variables) != resolve_operand(right, variables));
    }
    if let Some((left, right)) = expression.split_once("==") {
        return Ok(resolve_operand(left, variables) == resolve_operand(right, variables));
    }
    let (negated, operand) = expression
        .strip_prefix('!')
        .map_or((false, expression), |s| (true, s));
    let value = resolve_operand(operand, variables);
    let truthy = !value.is_empty()
        && !matches!(
            value.to_ascii_lowercase().as_str(),
            "false" | "0" | "no" | "off"
        );
    Ok(if negated { !truthy } else { truthy })
}

fn resolve_operand(operand: &str, variables: &HashMap<String, String>) -> String {
    let trimmed = operand.trim();
    if (trimmed.starts_with('"') && trimmed.ends_with('"'))
        || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
    {
        return trimmed[1..trimmed.len() - 1].to_string();
    }
    let name = trimmed
        .strip_prefix("{{")
        .and_then(|s| s.strip_suffix("}}"))
        .unwrap_or(trimmed)
        .trim();
    variables
        .get(name)
        .cloned()
        .or_else(|| std::env::var(name).ok())
        .unwrap_or_else(|| name.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn step(name: &str, command: &str, args: &[&str]) -> WorkflowStep {
        WorkflowStep {
            name: name.into(),
            command: command.into(),
            args: Some(args.iter().map(|s| s.to_string()).collect()),
            condition: None,
            parallel: None,
            timeout: Some(2),
            retry: None,
        }
    }

    #[tokio::test]
    async fn substitutes_variables_and_runs_sequentially() {
        let workflow = Workflow {
            name: "test".into(),
            description: None,
            variables: Some(vec!["who=world".into()]),
            steps: vec![step("hello", "printf", &["hello {{who}}"])],
        };
        let results = WorkflowExecutor::new()
            .execute_detailed(&workflow)
            .await
            .unwrap();
        assert_eq!(results[0].stdout, "hello world");
        assert_eq!(results[0].status, StepStatus::Succeeded);
    }

    #[tokio::test]
    async fn skips_false_condition() {
        let mut conditional = step("skip", "printf", &["bad"]);
        conditional.condition = Some("enabled == true".into());
        let workflow = Workflow {
            name: "test".into(),
            description: None,
            variables: Some(vec!["enabled=false".into()]),
            steps: vec![conditional],
        };
        let results = WorkflowExecutor::new()
            .execute_detailed(&workflow)
            .await
            .unwrap();
        assert_eq!(results[0].status, StepStatus::Skipped);
    }

    #[tokio::test]
    async fn retries_failed_steps() {
        let mut retrying = step("fail", "sh", &["-c", "exit 2"]);
        retrying.retry = Some(2);
        let workflow = Workflow {
            name: "test".into(),
            description: None,
            variables: None,
            steps: vec![retrying],
        };
        let error = WorkflowExecutor::new()
            .execute_detailed(&workflow)
            .await
            .unwrap_err();
        assert!(error.to_string().contains("3 attempt(s)"));
    }

    #[test]
    fn validation_rejects_duplicate_steps() {
        let workflow = Workflow {
            name: "test".into(),
            description: None,
            variables: None,
            steps: vec![step("same", "true", &[]), step("same", "true", &[])],
        };
        assert!(workflow.validate().unwrap_err()[0].contains("duplicate"));
    }
}
