use std::fs;

use assert_cmd::Command;
use predicates::prelude::*;

fn command(home: &std::path::Path) -> Command {
    let mut command = Command::cargo_bin("pleiades").unwrap();
    command
        .current_dir(home)
        .env("HOME", home)
        .env("USERPROFILE", home)
        .env("APPDATA", home.join("config"))
        .env("LOCALAPPDATA", home.join("local"))
        .env("XDG_CONFIG_HOME", home.join("config"));
    command
}

fn find_config(root: &std::path::Path) -> std::path::PathBuf {
    let mut directories = vec![root.to_path_buf()];
    while let Some(directory) = directories.pop() {
        for entry in fs::read_dir(&directory).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                directories.push(path);
            } else if path.file_name().is_some_and(|name| name == "config.toml") {
                return path;
            }
        }
    }
    panic!("config.toml was not created below {}", root.display());
}

#[test]
fn top_level_and_workflow_help_snapshots() {
    let home = tempfile::tempdir().unwrap();
    let output = command(home.path()).arg("--help").output().unwrap();
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout)
        .unwrap()
        .replace("pleiades.exe", "pleiades");
    assert!(text.contains("Pleiades is a terminal AI assistant"));
    assert!(text.contains("workflow"));
    assert!(text.contains("git"));
    assert!(text.contains("setup"));
    assert!(text.contains("doctor"));

    let output = command(home.path())
        .args(["workflow", "--help"])
        .output()
        .unwrap();
    let text = String::from_utf8(output.stdout)
        .unwrap()
        .replace("pleiades.exe", "pleiades");
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
      --permission-mode <PERMISSION_MODE>  Agent mode: plan, agent, or unrestricted
  -v, --verbose                            Verbose output
  -h, --help                               Print help
"###);
}

#[test]
fn guided_api_setup_and_doctor_use_environment_reference() {
    let home = tempfile::tempdir().unwrap();
    command(home.path())
        .args(["setup", "--auth", "api-key"])
        .assert()
        .success()
        .stdout(predicate::str::contains("usage-based OpenAI API access"));

    let config = fs::read_to_string(find_config(home.path())).unwrap();
    assert!(config.contains("${OPENAI_API_KEY}"));
    assert!(!config.contains("sk-test"));

    command(home.path())
        .env("OPENAI_API_KEY", "sk-test")
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains("API credential is available"))
        .stdout(predicate::str::contains("sk-test").not());
}

#[test]
fn provider_service_reports_never_print_resolved_secrets() {
    let home = tempfile::tempdir().unwrap();
    command(home.path())
        .args(["config", "init"])
        .assert()
        .success();
    command(home.path())
        .args([
            "config",
            "set",
            "providers.openai.api_key",
            "${OPENAI_API_KEY}",
        ])
        .assert()
        .success();
    command(home.path())
        .env("OPENAI_API_KEY", "sk-never-print-this-secret")
        .args(["provider", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("openai"))
        .stdout(predicate::str::contains("sk-never-print-this-secret").not());
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

#[test]
fn chat_command_can_exit_cleanly() {
    let home = tempfile::tempdir().unwrap();
    command(home.path())
        .env("OPENAI_API_KEY", "test-key")
        .write_stdin("/exit\n")
        .args(["chat", "--provider", "openai"])
        .assert()
        .success()
        .stdout(predicate::str::contains("P L E I A D E S"))
        .stdout(predicate::str::contains("workspace"));
}

#[test]
fn one_shot_prompt_uses_the_engine() {
    let temp = tempfile::tempdir().unwrap();
    let mut command = Command::cargo_bin("pleiades").unwrap();
    command
        .current_dir(temp.path())
        .env("XDG_CONFIG_HOME", temp.path().join("config"))
        .env_remove("OPENAI_API_KEY")
        .arg("hello");

    command
        .assert()
        .failure()
        .stderr(predicate::str::contains("Provider 'openai' not found"))
        .stderr(predicate::str::contains("Milestone 5").not());
}
