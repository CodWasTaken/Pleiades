//! Language-service integration primitives.
//!
//! This crate owns the provider-neutral language-service data model used by
//! CLI, TUI, runtime, and future JSON-RPC language-server clients. The first
//! implementation exposes Rust diagnostics through `cargo check
//! --message-format=json` and maps compiler messages into LSP-compatible
//! diagnostics so the rest of Pleiades can consume one stable shape.

use std::path::{Path, PathBuf};

use lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};
use pleiades_agent_core::Error;
use serde::Deserialize;
use tokio::process::Command;

/// Workspace language-service status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LspStatusReport {
    pub workspace: PathBuf,
    pub servers: Vec<LspServerReport>,
}

/// One configured or detected language-service backend.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LspServerReport {
    pub id: String,
    pub language: String,
    pub command: String,
    pub status: LspServerStatus,
    pub transport: String,
    pub notes: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LspServerStatus {
    Available,
    Missing,
    Disabled,
}

impl LspServerStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Available => "available",
            Self::Missing => "missing",
            Self::Disabled => "disabled",
        }
    }
}

/// Diagnostics grouped by source file.
#[derive(Debug, Clone, PartialEq)]
pub struct DiagnosticReport {
    pub workspace: PathBuf,
    pub command: Option<String>,
    pub diagnostics: Vec<FileDiagnostics>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FileDiagnostics {
    pub path: PathBuf,
    pub diagnostics: Vec<Diagnostic>,
}

/// Workspace symbol search result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolSearchReport {
    pub query: String,
    pub symbols: Vec<WorkspaceSymbol>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceSymbol {
    pub name: String,
    pub kind: String,
    pub location: PathBuf,
    pub line: usize,
}

/// Stateless service entry point.
#[derive(Debug, Clone)]
pub struct LspService {
    workspace: PathBuf,
}

impl LspService {
    pub fn new(workspace: PathBuf) -> Self {
        Self { workspace }
    }

    pub fn workspace(&self) -> &Path {
        &self.workspace
    }

    pub async fn status(&self) -> Result<LspStatusReport, Error> {
        let mut servers = Vec::new();
        if self.workspace.join("Cargo.toml").exists() {
            servers.push(LspServerReport {
                id: "rust-analyzer".to_string(),
                language: "rust".to_string(),
                command: "rust-analyzer".to_string(),
                status: if command_exists("rust-analyzer").await {
                    LspServerStatus::Available
                } else {
                    LspServerStatus::Missing
                },
                transport: "stdio".to_string(),
                notes: "Detected Rust workspace. Diagnostics currently use cargo check fallback; persistent JSON-RPC transport is a follow-up slice.".to_string(),
            });
        }
        Ok(LspStatusReport {
            workspace: self.workspace.clone(),
            servers,
        })
    }

    pub async fn diagnostics(&self) -> Result<DiagnosticReport, Error> {
        if !self.workspace.join("Cargo.toml").exists() {
            return Ok(DiagnosticReport {
                workspace: self.workspace.clone(),
                command: None,
                diagnostics: Vec::new(),
            });
        }
        let output = Command::new("cargo")
            .args(["check", "--message-format=json"])
            .current_dir(&self.workspace)
            .output()
            .await?;
        let mut diagnostics = parse_cargo_diagnostics(&self.workspace, &output.stdout)?;
        diagnostics.sort_by(|a, b| a.path.cmp(&b.path));
        if !output.status.success() && diagnostics.is_empty() {
            return Err(Error::tool(format!(
                "cargo check failed without JSON diagnostics: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            )));
        }
        Ok(DiagnosticReport {
            workspace: self.workspace.clone(),
            command: Some("cargo check --message-format=json".to_string()),
            diagnostics,
        })
    }

    pub async fn symbols(&self, query: &str) -> Result<SymbolSearchReport, Error> {
        let query = query.trim();
        if query.is_empty() {
            return Err(Error::invalid_input("usage: /lsp symbols <query>"));
        }
        let mut symbols = Vec::new();
        collect_rust_symbols(&self.workspace, query, &mut symbols)?;
        symbols.sort_by(|a, b| a.location.cmp(&b.location).then(a.line.cmp(&b.line)));
        Ok(SymbolSearchReport {
            query: query.to_string(),
            symbols,
        })
    }
}

async fn command_exists(command: &str) -> bool {
    Command::new(command)
        .arg("--version")
        .output()
        .await
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn parse_cargo_diagnostics(workspace: &Path, output: &[u8]) -> Result<Vec<FileDiagnostics>, Error> {
    let mut grouped = Vec::<FileDiagnostics>::new();
    for line in String::from_utf8_lossy(output).lines() {
        let Ok(message) = serde_json::from_str::<CargoMessage>(line) else {
            continue;
        };
        if message.reason.as_deref() != Some("compiler-message") {
            continue;
        }
        let Some(message) = message.message else {
            continue;
        };
        if message.level == "failure-note" {
            continue;
        }
        let Some(span) = message.spans.iter().find(|span| span.is_primary) else {
            continue;
        };
        let path = workspace.join(&span.file_name);
        let diagnostic = Diagnostic {
            range: Range {
                start: Position::new(
                    span.line_start.saturating_sub(1),
                    span.column_start.saturating_sub(1),
                ),
                end: Position::new(span.line_end.saturating_sub(1), span.column_end),
            },
            severity: cargo_severity(&message.level),
            code: message
                .code
                .map(|code| lsp_types::NumberOrString::String(code.code)),
            code_description: None,
            source: Some("cargo check".to_string()),
            message: message.message,
            related_information: None,
            tags: None,
            data: None,
        };
        match grouped.iter_mut().find(|entry| entry.path == path) {
            Some(entry) => entry.diagnostics.push(diagnostic),
            None => grouped.push(FileDiagnostics {
                path,
                diagnostics: vec![diagnostic],
            }),
        }
    }
    Ok(grouped)
}

fn cargo_severity(level: &str) -> Option<DiagnosticSeverity> {
    match level {
        "error" => Some(DiagnosticSeverity::ERROR),
        "warning" => Some(DiagnosticSeverity::WARNING),
        "note" | "help" => Some(DiagnosticSeverity::INFORMATION),
        _ => None,
    }
}

fn collect_rust_symbols(
    root: &Path,
    query: &str,
    out: &mut Vec<WorkspaceSymbol>,
) -> Result<(), Error> {
    if !root.exists() {
        return Ok(());
    }
    for entry in std::fs::read_dir(root).map_err(Error::from)? {
        let entry = entry.map_err(Error::from)?;
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name == "target" || name == ".git" {
            continue;
        }
        if path.is_dir() {
            collect_rust_symbols(&path, query, out)?;
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            collect_symbols_in_file(&path, query, out)?;
        }
    }
    Ok(())
}

fn collect_symbols_in_file(
    path: &Path,
    query: &str,
    out: &mut Vec<WorkspaceSymbol>,
) -> Result<(), Error> {
    let content = std::fs::read_to_string(path).map_err(Error::from)?;
    for (line_index, line) in content.lines().enumerate() {
        if let Some((kind, name)) = parse_rust_symbol_line(line) {
            if name.contains(query) {
                out.push(WorkspaceSymbol {
                    name,
                    kind,
                    location: path.to_path_buf(),
                    line: line_index + 1,
                });
            }
        }
    }
    Ok(())
}

fn parse_rust_symbol_line(line: &str) -> Option<(String, String)> {
    let trimmed = line.trim_start();
    for prefix in ["pub async fn ", "pub fn ", "async fn ", "fn "] {
        if let Some(rest) = trimmed.strip_prefix(prefix) {
            return Some(("function".to_string(), identifier(rest)?));
        }
    }
    for prefix in ["pub struct ", "struct "] {
        if let Some(rest) = trimmed.strip_prefix(prefix) {
            return Some(("struct".to_string(), identifier(rest)?));
        }
    }
    for prefix in ["pub enum ", "enum "] {
        if let Some(rest) = trimmed.strip_prefix(prefix) {
            return Some(("enum".to_string(), identifier(rest)?));
        }
    }
    for prefix in ["pub trait ", "trait "] {
        if let Some(rest) = trimmed.strip_prefix(prefix) {
            return Some(("trait".to_string(), identifier(rest)?));
        }
    }
    None
}

fn identifier(rest: &str) -> Option<String> {
    let value = rest
        .chars()
        .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '_')
        .collect::<String>();
    (!value.is_empty()).then_some(value)
}

#[derive(Debug, Deserialize)]
struct CargoMessage {
    reason: Option<String>,
    message: Option<CargoCompilerMessage>,
}

#[derive(Debug, Deserialize)]
struct CargoCompilerMessage {
    message: String,
    level: String,
    code: Option<CargoCode>,
    spans: Vec<CargoSpan>,
}

#[derive(Debug, Deserialize)]
struct CargoCode {
    code: String,
}

#[derive(Debug, Deserialize)]
struct CargoSpan {
    file_name: String,
    line_start: u32,
    line_end: u32,
    column_start: u32,
    column_end: u32,
    is_primary: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_rust_symbol_lines() {
        assert_eq!(
            parse_rust_symbol_line("pub async fn inspect_project() {}"),
            Some(("function".to_string(), "inspect_project".to_string()))
        );
        assert_eq!(
            parse_rust_symbol_line("pub struct WorkspaceState;"),
            Some(("struct".to_string(), "WorkspaceState".to_string()))
        );
    }

    #[test]
    fn maps_cargo_json_to_lsp_diagnostics() {
        let dir = tempfile::tempdir().unwrap();
        let json = r#"{"reason":"compiler-message","message":{"message":"cannot find value `x` in this scope","level":"error","code":{"code":"E0425"},"spans":[{"file_name":"src/main.rs","line_start":2,"line_end":2,"column_start":5,"column_end":6,"is_primary":true}]}}"#;
        let diagnostics = parse_cargo_diagnostics(dir.path(), json.as_bytes()).unwrap();
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].diagnostics[0].severity,
            Some(DiagnosticSeverity::ERROR)
        );
        assert_eq!(diagnostics[0].diagnostics[0].range.start.line, 1);
    }

    #[test]
    fn searches_rust_symbols_without_target_directory() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("src");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(src.join("lib.rs"), "pub fn alpha() {}\nfn beta() {}\n").unwrap();
        std::fs::create_dir_all(dir.path().join("target")).unwrap();
        std::fs::write(
            dir.path().join("target").join("ignored.rs"),
            "fn alpha_target() {}",
        )
        .unwrap();
        let mut symbols = Vec::new();
        collect_rust_symbols(dir.path(), "alpha", &mut symbols).unwrap();
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "alpha");
    }
}
