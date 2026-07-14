//! Structured permission matching for tool and shell invocations.
//!
//! This crate is terminal- and runtime-independent. It parses shell commands
//! into clauses, validates paths against a workspace boundary, and evaluates
//! configured rules with deny-first precedence.

use std::path::{Component, Path, PathBuf};

use globset::Glob;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PermissionAction {
    Allow,
    Ask,
    Deny,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PermissionRule {
    pub tool: String,
    pub pattern: String,
    pub action: PermissionAction,
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub network: Option<String>,
    #[serde(default)]
    pub mcp_server: Option<String>,
    #[serde(default)]
    pub mcp_tool: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ToolInvocation {
    pub tool: String,
    pub command: Option<String>,
    pub cwd: PathBuf,
    pub workspace_root: PathBuf,
    pub target_paths: Vec<PathBuf>,
    pub network_destinations: Vec<String>,
    pub plugin_source: Option<String>,
    pub mcp_server: Option<String>,
    pub mcp_tool: Option<String>,
    pub env_vars: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecisionKind {
    Allow,
    Ask,
    Deny,
    Default,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decision {
    pub kind: DecisionKind,
    pub matched_rule: Option<usize>,
    pub reason: String,
    pub clauses: Vec<String>,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ParseError {
    #[error("unterminated shell quote")]
    UnterminatedQuote,
    #[error("empty command clause around `{0}`")]
    EmptyClause(String),
    #[error("invalid permission pattern `{pattern}`: {message}")]
    InvalidPattern { pattern: String, message: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellProgram {
    pub clauses: Vec<ShellClause>,
    pub command_substitution: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellClause {
    pub words: Vec<String>,
    pub redirections: Vec<Redirection>,
    pub next_operator: Option<ShellOperator>,
}

impl ShellClause {
    pub fn normalized(&self) -> String {
        self.words.join(" ")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Redirection {
    pub operator: String,
    pub target: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellOperator {
    And,
    Or,
    Pipe,
    Sequence,
    Background,
}

pub struct PermissionEngine {
    rules: Vec<CompiledRule>,
}

struct CompiledRule {
    rule: PermissionRule,
    tool: globset::GlobMatcher,
    pattern: globset::GlobMatcher,
    cwd: Option<globset::GlobMatcher>,
    network: Option<globset::GlobMatcher>,
}

impl PermissionEngine {
    pub fn new(rules: Vec<PermissionRule>) -> Result<Self, ParseError> {
        let rules = rules
            .into_iter()
            .map(CompiledRule::new)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { rules })
    }

    pub fn evaluate(&self, invocation: &ToolInvocation) -> Decision {
        if let Some(reason) = validate_invocation_paths(invocation) {
            return Decision {
                kind: DecisionKind::Deny,
                matched_rule: None,
                reason,
                clauses: Vec::new(),
            };
        }

        let program = match invocation.command.as_deref() {
            Some(command) => match parse_shell(command) {
                Ok(program) => Some(program),
                Err(error) => {
                    return Decision {
                        kind: DecisionKind::Deny,
                        matched_rule: None,
                        reason: format!("shell command could not be parsed safely: {error}"),
                        clauses: Vec::new(),
                    };
                }
            },
            None => None,
        };
        let clauses = program
            .as_ref()
            .map(|program| {
                program
                    .clauses
                    .iter()
                    .map(ShellClause::normalized)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(|| vec![invocation_pattern(invocation)]);

        let mut outcomes = Vec::with_capacity(clauses.len());
        for clause in &clauses {
            outcomes.push(self.evaluate_clause(invocation, clause));
        }
        if let Some((index, action)) = outcomes
            .iter()
            .find_map(|outcome| outcome.filter(|(_, action)| *action == PermissionAction::Deny))
        {
            return decision_from_rule(DecisionKind::Deny, index, action, clauses);
        }
        if let Some((index, action)) = outcomes
            .iter()
            .find_map(|outcome| outcome.filter(|(_, action)| *action == PermissionAction::Ask))
        {
            return decision_from_rule(DecisionKind::Ask, index, action, clauses);
        }
        if program
            .as_ref()
            .is_some_and(|value| value.command_substitution)
        {
            return Decision {
                kind: DecisionKind::Ask,
                matched_rule: None,
                reason: "shell command substitution requires explicit review".to_string(),
                clauses,
            };
        }
        if !outcomes.is_empty() && outcomes.iter().all(Option::is_some) {
            let (index, action) = outcomes[0].expect("checked above");
            return decision_from_rule(DecisionKind::Allow, index, action, clauses);
        }
        Decision {
            kind: DecisionKind::Default,
            matched_rule: None,
            reason: "no permission rule matched every command clause".to_string(),
            clauses,
        }
    }

    fn evaluate_clause(
        &self,
        invocation: &ToolInvocation,
        clause: &str,
    ) -> Option<(usize, PermissionAction)> {
        let mut allow = None;
        let mut ask = None;
        for (index, compiled) in self.rules.iter().enumerate() {
            if !compiled.matches(invocation, clause) {
                continue;
            }
            match compiled.rule.action {
                PermissionAction::Deny => return Some((index, PermissionAction::Deny)),
                PermissionAction::Ask => ask.get_or_insert((index, PermissionAction::Ask)),
                PermissionAction::Allow => allow.get_or_insert((index, PermissionAction::Allow)),
            };
        }
        ask.or(allow)
    }
}

impl CompiledRule {
    fn new(rule: PermissionRule) -> Result<Self, ParseError> {
        let tool = compile_glob(&rule.tool)?;
        let pattern = compile_glob(&rule.pattern)?;
        let cwd = rule.cwd.as_deref().map(compile_glob).transpose()?;
        let network = rule.network.as_deref().map(compile_glob).transpose()?;
        Ok(Self {
            rule,
            tool,
            pattern,
            cwd,
            network,
        })
    }

    fn matches(&self, invocation: &ToolInvocation, clause: &str) -> bool {
        self.tool.is_match(&invocation.tool)
            && self.pattern.is_match(clause)
            && self
                .cwd
                .as_ref()
                .is_none_or(|matcher| matcher.is_match(invocation.cwd.to_string_lossy().as_ref()))
            && self.network.as_ref().is_none_or(|matcher| {
                !invocation.network_destinations.is_empty()
                    && invocation
                        .network_destinations
                        .iter()
                        .all(|destination| matcher.is_match(destination))
            })
            && self
                .rule
                .mcp_server
                .as_ref()
                .is_none_or(|server| invocation.mcp_server.as_ref() == Some(server))
            && self
                .rule
                .mcp_tool
                .as_ref()
                .is_none_or(|tool| invocation.mcp_tool.as_ref() == Some(tool))
    }
}

fn compile_glob(pattern: &str) -> Result<globset::GlobMatcher, ParseError> {
    Glob::new(pattern)
        .map(|glob| glob.compile_matcher())
        .map_err(|error| ParseError::InvalidPattern {
            pattern: pattern.to_string(),
            message: error.to_string(),
        })
}

fn decision_from_rule(
    kind: DecisionKind,
    index: usize,
    action: PermissionAction,
    clauses: Vec<String>,
) -> Decision {
    Decision {
        kind,
        matched_rule: Some(index),
        reason: format!("matched rule {} ({action:?})", index + 1),
        clauses,
    }
}

fn invocation_pattern(invocation: &ToolInvocation) -> String {
    invocation
        .target_paths
        .first()
        .map(|path| path.to_string_lossy().to_string())
        .or_else(|| invocation.plugin_source.clone())
        .or_else(|| invocation.mcp_tool.clone())
        .unwrap_or_else(|| "*".to_string())
}

fn validate_invocation_paths(invocation: &ToolInvocation) -> Option<String> {
    if invocation.workspace_root.as_os_str().is_empty() {
        return None;
    }
    let root = invocation
        .workspace_root
        .canonicalize()
        .unwrap_or_else(|_| lexical_normalize(&invocation.workspace_root));
    let cwd = invocation
        .cwd
        .canonicalize()
        .unwrap_or_else(|_| lexical_normalize(&invocation.cwd));
    if !cwd.starts_with(&root) {
        return Some(format!(
            "working directory `{}` escapes workspace `{}`",
            cwd.display(),
            root.display()
        ));
    }
    let mut paths = invocation.target_paths.clone();
    if let Some(program) = invocation
        .command
        .as_deref()
        .and_then(|command| parse_shell(command).ok())
    {
        paths.extend(program.clauses.into_iter().flat_map(|clause| {
            clause
                .redirections
                .into_iter()
                .map(|redirection| PathBuf::from(redirection.target))
        }));
    }
    for path in paths {
        let absolute = if path.is_absolute() {
            path
        } else {
            invocation.cwd.join(path)
        };
        let resolved =
            canonicalize_existing_prefix(&absolute).unwrap_or_else(|| lexical_normalize(&absolute));
        if !resolved.starts_with(&root) {
            return Some(format!(
                "target path `{}` escapes workspace `{}`",
                resolved.display(),
                root.display()
            ));
        }
    }
    None
}

fn canonicalize_existing_prefix(path: &Path) -> Option<PathBuf> {
    let mut existing = path.to_path_buf();
    let mut suffix = Vec::new();
    while !existing.exists() {
        suffix.push(existing.file_name()?.to_os_string());
        existing = existing.parent()?.to_path_buf();
    }
    let mut resolved = existing.canonicalize().ok()?;
    for part in suffix.into_iter().rev() {
        resolved.push(part);
    }
    Some(lexical_normalize(&resolved))
}

fn lexical_normalize(path: &Path) -> PathBuf {
    let mut result = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                result.pop();
            }
            other => result.push(other.as_os_str()),
        }
    }
    result
}

pub fn parse_shell(command: &str) -> Result<ShellProgram, ParseError> {
    let tokens = tokenize(command)?;
    let command_substitution = command.contains("$(") || command.contains('`');
    let mut clauses = Vec::new();
    let mut words = Vec::new();
    let mut redirections = Vec::new();
    let mut index = 0;
    while index < tokens.len() {
        let token = &tokens[index];
        if let Some(operator) = parse_operator(token) {
            if words.is_empty() {
                return Err(ParseError::EmptyClause(token.clone()));
            }
            clauses.push(ShellClause {
                words: std::mem::take(&mut words),
                redirections: std::mem::take(&mut redirections),
                next_operator: Some(operator),
            });
        } else if is_redirection(token) {
            index += 1;
            let target = tokens
                .get(index)
                .filter(|target| parse_operator(target).is_none() && !is_redirection(target))
                .ok_or_else(|| ParseError::EmptyClause(token.clone()))?;
            redirections.push(Redirection {
                operator: token.clone(),
                target: target.clone(),
            });
        } else {
            words.push(token.clone());
        }
        index += 1;
    }
    if words.is_empty() {
        if let Some(last) = tokens.last() {
            if parse_operator(last).is_some() {
                return Err(ParseError::EmptyClause(last.clone()));
            }
        }
    } else {
        clauses.push(ShellClause {
            words,
            redirections,
            next_operator: None,
        });
    }
    Ok(ShellProgram {
        clauses,
        command_substitution,
    })
}

fn tokenize(command: &str) -> Result<Vec<String>, ParseError> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut chars = command.chars().peekable();
    let mut quote = None;
    while let Some(character) = chars.next() {
        if let Some(active_quote) = quote {
            if character == active_quote {
                quote = None;
            } else if character == '\\' && active_quote == '"' {
                current.push(chars.next().unwrap_or(character));
            } else {
                current.push(character);
            }
            continue;
        }
        match character {
            '\'' | '"' => quote = Some(character),
            '\\' => current.push(chars.next().unwrap_or(character)),
            ' ' | '\t' | '\n' => push_token(&mut tokens, &mut current),
            '&' | '|' | ';' | '<' | '>' => {
                push_token(&mut tokens, &mut current);
                let mut operator = character.to_string();
                if chars.peek() == Some(&character) && character != ';' {
                    operator.push(chars.next().expect("peeked"));
                }
                tokens.push(operator);
            }
            '0'..='9' if current.is_empty() && matches!(chars.peek(), Some('>' | '<')) => {
                let mut operator = character.to_string();
                operator.push(chars.next().expect("peeked"));
                if chars.peek() == Some(&'>') {
                    operator.push(chars.next().expect("peeked"));
                }
                tokens.push(operator);
            }
            _ => current.push(character),
        }
    }
    if quote.is_some() {
        return Err(ParseError::UnterminatedQuote);
    }
    push_token(&mut tokens, &mut current);
    Ok(tokens)
}

fn push_token(tokens: &mut Vec<String>, current: &mut String) {
    if !current.is_empty() {
        tokens.push(std::mem::take(current));
    }
}

fn parse_operator(token: &str) -> Option<ShellOperator> {
    match token {
        "&&" => Some(ShellOperator::And),
        "||" => Some(ShellOperator::Or),
        "|" => Some(ShellOperator::Pipe),
        ";" => Some(ShellOperator::Sequence),
        "&" => Some(ShellOperator::Background),
        _ => None,
    }
}

fn is_redirection(token: &str) -> bool {
    matches!(token, ">" | ">>" | "<" | "<<")
        || token
            .strip_suffix('>')
            .is_some_and(|prefix| prefix.chars().all(|character| character.is_ascii_digit()))
        || token
            .strip_suffix(">>")
            .is_some_and(|prefix| prefix.chars().all(|character| character.is_ascii_digit()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(pattern: &str, action: PermissionAction) -> PermissionRule {
        PermissionRule {
            tool: "bash".to_string(),
            pattern: pattern.to_string(),
            action,
            cwd: None,
            network: None,
            mcp_server: None,
            mcp_tool: None,
        }
    }

    fn invocation(root: &Path, command: &str) -> ToolInvocation {
        ToolInvocation {
            tool: "bash".to_string(),
            command: Some(command.to_string()),
            cwd: root.to_path_buf(),
            workspace_root: root.to_path_buf(),
            ..ToolInvocation::default()
        }
    }

    #[test]
    fn compound_commands_are_evaluated_independently() {
        let temp = tempfile::tempdir().unwrap();
        let engine = PermissionEngine::new(vec![
            rule("cargo test*", PermissionAction::Allow),
            rule("rm -rf*", PermissionAction::Deny),
        ])
        .unwrap();
        let result = engine.evaluate(&invocation(temp.path(), "cargo test && rm -rf target"));
        assert_eq!(result.kind, DecisionKind::Deny);
        assert_eq!(result.clauses, ["cargo test", "rm -rf target"]);
    }

    #[test]
    fn every_clause_must_be_allowed() {
        let temp = tempfile::tempdir().unwrap();
        let engine =
            PermissionEngine::new(vec![rule("cargo test*", PermissionAction::Allow)]).unwrap();
        let result = engine.evaluate(&invocation(temp.path(), "cargo test | curl example.com"));
        assert_eq!(result.kind, DecisionKind::Default);
    }

    #[test]
    fn redirection_outside_workspace_is_denied() {
        let temp = tempfile::tempdir().unwrap();
        let engine = PermissionEngine::new(vec![rule("*", PermissionAction::Allow)]).unwrap();
        let result = engine.evaluate(&invocation(temp.path(), "printf secret > ../../outside"));
        assert_eq!(result.kind, DecisionKind::Deny);
        assert!(result.reason.contains("escapes workspace"));
    }

    #[test]
    fn cwd_outside_workspace_is_denied() {
        let temp = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let engine = PermissionEngine::new(vec![rule("*", PermissionAction::Allow)]).unwrap();
        let mut invocation = invocation(temp.path(), "cargo test");
        invocation.cwd = outside.path().to_path_buf();
        let result = engine.evaluate(&invocation);
        assert_eq!(result.kind, DecisionKind::Deny);
        assert!(result.reason.contains("working directory"));
    }

    #[cfg(unix)]
    #[test]
    fn symlink_escape_is_denied() {
        use std::os::unix::fs::symlink;
        let temp = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        symlink(outside.path(), temp.path().join("escape")).unwrap();
        let engine = PermissionEngine::new(vec![rule("*", PermissionAction::Allow)]).unwrap();
        let mut invocation = invocation(temp.path(), "write escape/secret");
        invocation.target_paths.push(PathBuf::from("escape/secret"));
        assert_eq!(engine.evaluate(&invocation).kind, DecisionKind::Deny);
    }

    #[test]
    fn command_substitution_requires_review() {
        let temp = tempfile::tempdir().unwrap();
        let engine = PermissionEngine::new(vec![rule("*", PermissionAction::Allow)]).unwrap();
        assert_eq!(
            engine
                .evaluate(&invocation(temp.path(), "echo $(cat .env)"))
                .kind,
            DecisionKind::Ask
        );
    }

    #[test]
    fn deny_rules_take_precedence_over_command_substitution_review() {
        let temp = tempfile::tempdir().unwrap();
        let engine = PermissionEngine::new(vec![rule("echo *", PermissionAction::Deny)]).unwrap();
        assert_eq!(
            engine
                .evaluate(&invocation(temp.path(), "echo $(cat .env)"))
                .kind,
            DecisionKind::Deny
        );
    }

    #[test]
    fn quotes_preserve_operators_as_arguments() {
        let parsed = parse_shell("printf '%s | %s' one two && cargo test").unwrap();
        assert_eq!(parsed.clauses.len(), 2);
        assert_eq!(parsed.clauses[0].words[1], "%s | %s");
    }
}
