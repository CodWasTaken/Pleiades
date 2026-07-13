use std::fs;

use assert_cmd::Command;
use predicates::prelude::*;

fn command(home: &std::path::Path) -> Command {
    let mut command = Command::cargo_bin("pleiades").unwrap();
    command
        .env("HOME", home)
        .env("XDG_CONFIG_HOME", home.join("config"));
    command
}

#[test]
fn top_level_and_workflow_help_snapshots() {
    let home = tempfile::tempdir().unwrap();
    let output = command(home.path()).arg("--help").output().unwrap();
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("Pleiades is a terminal AI assistant"));
    assert!(text.contains("workflow"));
    assert!(text.contains("git"));

    let output = command(home.path())
        .args(["workflow", "--help"])
        .output()
        .unwrap();
    let text = String::from_utf8(output.stdout).unwrap();
    insta::assert_snapshot!(text, @r###"
Manage and run workflows

Usage: pleiades workflow [OPTIONS] <COMMAND>

Commands:
  list      List available workflow definitions
  run       Run a workflow
  show      Show a workflow definition
  validate  Validate a workflow definition
  create    Create a starter workflow in .pleiades/workflows
  help      Print this message or the help of the given subcommand(s)

Options:
  -m, --model <MODEL>                      Model to use
  -P, --provider <PROVIDER>                Provider to use
      --permission-mode <PERMISSION_MODE>  Permission mode
  -v, --verbose                            Verbose output
  -h, --help                               Print help
"###);
}

#[test]
fn workflow_create_validate_list_and_run() {
    let workspace = tempfile::tempdir().unwrap();
    command(workspace.path())
        .current_dir(workspace.path())
        .args([
            "workflow",
            "create",
            "smoke",
            "--description",
            "integration test",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Created"));
    command(workspace.path())
        .current_dir(workspace.path())
        .args(["workflow", "validate", "smoke"])
        .assert()
        .success()
        .stdout(predicate::str::contains("is valid"));
    command(workspace.path())
        .current_dir(workspace.path())
        .args(["workflow", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("smoke"));
    command(workspace.path())
        .current_dir(workspace.path())
        .args(["workflow", "run", "smoke", "--var", "name=CLI"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello, CLI!"));
}

#[test]
fn prompt_render_and_provider_override_parse() {
    let home = tempfile::tempdir().unwrap();
    let output = command(home.path())
        .args([
            "--provider",
            "openai",
            "prompt",
            "render",
            "commit-message",
            "--var",
            "diff=example",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    insta::assert_snapshot!(text, @r###"
Write a git commit message for the staged changes below.
Use the Conventional Commits format (type: subject). Keep the subject under 72 chars.
Add a short body only when it adds real context.

Diff:
example
"###);
}

#[test]
fn tool_execution_reads_a_file() {
    let workspace = tempfile::tempdir().unwrap();
    let file = workspace.path().join("sample.txt");
    fs::write(&file, "integration-content").unwrap();
    let input = serde_json::json!({ "path": file }).to_string();
    command(workspace.path())
        .current_dir(workspace.path())
        .args(["tool", "call", "read", &input])
        .assert()
        .success()
        .stdout(predicate::str::contains("integration-content"));
}

#[test]
fn repl_can_exit_cleanly() {
    let home = tempfile::tempdir().unwrap();
    command(home.path())
        .arg("repl")
        .write_stdin("/exit\n")
        .assert()
        .success();
}
