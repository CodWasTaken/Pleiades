//! Default builtin commands shipped with the live workspace.
//!
//! This module seeds a [`crate::CommandRegistry`] with the slash commands the live
//! workspace already understands today (see
//! `crates/pleiades-agent-tui/src/state.rs`), plus a handful of CLI/twin
//! commands described in the directive.  Adding a new builtin command is a
//! one-liner: write a small handler, [`CommandSpec::builder`] it, and
//! `register` it here.
//!
//! The handlers in this module are deliberately lightweight: they emit
//! typed [`AppEffect`]s or request [`OverlayKind`]s.  No business logic,
//! no terminal IO, no runtime calls — that all lives in application
//! services (issue 2.1 unify-CLI-TUI) and the runtime itself.

use async_trait::async_trait;
use pleiades_agent_core::Error;

use crate::context::CommandContext;
use crate::handler::{CommandHandler, HandlerResult};
use crate::result::{AppEffect, CommandResult, OverlayKind};
use crate::spec::{
    ArgumentSpec, CommandAvailability, CommandCategory, CommandSpec, CompletionSource,
    PermissionRequirement, Shortcut,
};

/// Sync function backed by an `async_trait` adapter, so handlers are
/// ergonomic.  `F` receives the positional `args` and returns a typed
/// result or an [`Error`].
struct FnHandler<F>
where
    F: Fn(&[String]) -> HandlerResult + Send + Sync + 'static,
{
    f: F,
}

#[async_trait]
impl<F> CommandHandler for FnHandler<F>
where
    F: Fn(&[String]) -> HandlerResult + Send + Sync + 'static,
{
    async fn handle(&self, _: &CommandContext, args: &[String]) -> HandlerResult {
        (self.f)(args)
    }
}

fn handler<F>(f: F) -> FnHandler<F>
where
    F: Fn(&[String]) -> HandlerResult + Send + Sync + 'static,
{
    FnHandler { f }
}

struct StatusHandler;

#[async_trait]
impl CommandHandler for StatusHandler {
    async fn handle(&self, context: &CommandContext, _: &[String]) -> HandlerResult {
        Ok(CommandResult::RenderDocument(
            crate::result::RenderableDocument::new("Pleiades Status")
                .section("Provider", context.provider())
                .section("Model", context.model())
                .section("Mode", context.mode()),
        ))
    }
}

/// Build and populate a fresh [`crate::CommandRegistry`] with the default live
/// workspace commands and return it.
///
/// Every command registered here corresponds to a current behaviour in
/// `state.rs` so the migration in issue 2.1 ("wire TUI through the
/// registry") can drop the hand-maintained slash dispatcher one-to-one.
pub fn default_registry() -> crate::registry::CommandRegistry {
    let mut r = crate::registry::CommandRegistry::new();
    register_workspace(&mut r);
    register_help(&mut r);
    register_provider_family(&mut r);
    register_model_family(&mut r);
    register_mode_family(&mut r);
    r
}

fn register_workspace(r: &mut crate::registry::CommandRegistry) {
    // /clear — clear the live conversation.
    r.register(
        CommandSpec::builder(
            vec!["clear"],
            "Clear the live conversation transcript",
            handler(|_| Ok(CommandResult::Effects(vec![AppEffect::ClearConversation]))),
        )
        .aliases(vec!["cls"])
        .category(CommandCategory::Workspace)
        .availability(CommandAvailability::Interactive)
        .shortcut(Shortcut::None)
        .build(),
    )
    .ok();

    // /save — persist the current session.
    r.register(
        CommandSpec::builder(
            vec!["save"],
            "Save the current session",
            handler(|_| Ok(CommandResult::Effects(vec![AppEffect::SaveSession]))),
        )
        .aliases(vec!["w"])
        .arguments(vec![ArgumentSpec::optional(
            "name",
            "Optional name to label the saved session.",
        )])
        .category(CommandCategory::Workspace)
        .availability(CommandAvailability::Both)
        .build(),
    )
    .ok();

    // /load <id> — load a named session.
    r.register(
        CommandSpec::builder(
            vec!["load"],
            "Load a previously saved session by id",
            handler(|args| match args.first() {
                Some(id) => Ok(CommandResult::Effects(vec![AppEffect::LoadSession(
                    id.clone(),
                )])),
                None => Err(Error::invalid_input("usage: /load <id>")),
            }),
        )
        .arguments(vec![
            ArgumentSpec::required("id", "Identifier of the session to load.")
                .with_completer(CompletionSource::Session),
        ])
        .category(CommandCategory::Workspace)
        .availability(CommandAvailability::Interactive)
        .build(),
    )
    .ok();

    // /diff — open the diff overlay.
    r.register(
        CommandSpec::builder(
            vec!["diff"],
            "Review the current working-tree diff",
            handler(|_| Ok(CommandResult::overlay(OverlayKind::Diff))),
        )
        .aliases(vec!["d"])
        .category(CommandCategory::Project)
        .availability(CommandAvailability::Interactive)
        .shortcut(Shortcut::Ctrl('d'))
        .build(),
    )
    .ok();

    // /output [activity_id] — open the tool-output overlay.
    r.register(
        CommandSpec::builder(
            vec!["output"],
            "Inspect the output of a tool activity",
            handler(|_| Ok(CommandResult::overlay(OverlayKind::ToolOutput))),
        )
        .aliases(vec!["o"])
        .arguments(vec![ArgumentSpec::optional(
            "activity_id",
            "Filter to a specific tool activity by id.",
        )])
        .category(CommandCategory::Workspace)
        .availability(CommandAvailability::Interactive)
        .shortcut(Shortcut::Ctrl('o'))
        .build(),
    )
    .ok();

    // /doctor — open the diagnostics overlay.
    r.register(
        CommandSpec::builder(
            vec!["doctor"],
            "Run workspace diagnostics",
            handler(|_| Ok(CommandResult::overlay(OverlayKind::Diagnostics))),
        )
        .aliases(vec!["diag"])
        .category(CommandCategory::Help)
        .availability(CommandAvailability::Both)
        .build(),
    )
    .ok();

    // /config — open the configuration overlay.
    r.register(
        CommandSpec::builder(
            vec!["config"],
            "Inspect or edit live configuration",
            handler(|_| Ok(CommandResult::overlay(OverlayKind::Configuration))),
        )
        .category(CommandCategory::Configuration)
        .availability(CommandAvailability::Interactive)
        .build(),
    )
    .ok();

    // /files — open the file picker.
    r.register(
        CommandSpec::builder(
            vec!["files"],
            "Search and open a workspace file",
            handler(|_| Ok(CommandResult::overlay(OverlayKind::FilePicker))),
        )
        .aliases(vec!["f"])
        .category(CommandCategory::Workspace)
        .availability(CommandAvailability::Interactive)
        .shortcut(Shortcut::Ctrl('f'))
        .build(),
    )
    .ok();

    // /sessions — open the session picker.
    r.register(
        CommandSpec::builder(
            vec!["sessions"],
            "Browse saved sessions",
            handler(|_| Ok(CommandResult::overlay(OverlayKind::SessionPicker))),
        )
        .category(CommandCategory::Memory)
        .availability(CommandAvailability::Interactive)
        .shortcut(Shortcut::Ctrl('l'))
        .build(),
    )
    .ok();

    // /quit, /exit — leave the live workspace.
    r.register(
        CommandSpec::builder(
            vec!["quit"],
            "Quit the Pleiades live workspace",
            handler(|_| {
                Ok(CommandResult::Effects(vec![
                    AppEffect::Quit,
                    AppEffect::Shutdown,
                ]))
            }),
        )
        .aliases(vec!["exit", "q"])
        .category(CommandCategory::Workspace)
        .availability(CommandAvailability::Both)
        .build(),
    )
    .ok();
}

fn register_help(r: &mut crate::registry::CommandRegistry) {
    // /help [name]
    r.register(
        CommandSpec::builder(
            vec!["help"],
            "Show the live-workspace command reference",
            handler(|_| Ok(CommandResult::overlay(OverlayKind::Help))),
        )
        .aliases(vec!["?"])
        .arguments(vec![ArgumentSpec::optional(
            "command",
            "Deep-link to a specific command's documentation.",
        )])
        .category(CommandCategory::Help)
        .availability(CommandAvailability::Both)
        .shortcut(Shortcut::F(1))
        .build(),
    )
    .ok();

    // /status — emit a transient status.
    r.register(
        CommandSpec::builder(
            vec!["status"],
            "Show the current workspace status snapshot",
            StatusHandler,
        )
        .category(CommandCategory::Help)
        .availability(CommandAvailability::Both)
        .build(),
    )
    .ok();
}

fn register_provider_family(r: &mut crate::registry::CommandRegistry) {
    // /provider [name]
    r.register(
        CommandSpec::builder(
            vec!["provider"],
            "Switch provider or open the provider picker",
            handler(|args| match args.first() {
                Some(name) => Ok(CommandResult::Effects(vec![AppEffect::SetProvider(
                    name.clone(),
                )])),
                None => Ok(CommandResult::overlay(OverlayKind::ProviderPicker)),
            }),
        )
        .aliases(vec!["p"])
        .arguments(vec![
            ArgumentSpec::optional(
                "name",
                "Provider to switch to; if absent, opens the picker.",
            )
            .with_completer(CompletionSource::Provider),
        ])
        .category(CommandCategory::Provider)
        .availability(CommandAvailability::Both)
        .shortcut(Shortcut::Ctrl('r'))
        .permission(PermissionRequirement::Read)
        .build(),
    )
    .ok();
}

fn register_model_family(r: &mut crate::registry::CommandRegistry) {
    // /model [name]
    r.register(
        CommandSpec::builder(
            vec!["model"],
            "Switch model or open the model picker",
            handler(|args| match args.first() {
                Some(name) => Ok(CommandResult::Effects(vec![AppEffect::SetModel(
                    name.clone(),
                )])),
                None => Ok(CommandResult::overlay(OverlayKind::ModelPicker)),
            }),
        )
        .aliases(vec!["m"])
        .arguments(vec![
            ArgumentSpec::optional(
                "name",
                "Model identifier to switch to; if absent, opens the picker.",
            )
            .with_completer(CompletionSource::Model),
        ])
        .category(CommandCategory::Provider)
        .availability(CommandAvailability::Both)
        .shortcut(Shortcut::Ctrl('m'))
        .permission(PermissionRequirement::Read)
        .build(),
    )
    .ok();
}

fn register_mode_family(r: &mut crate::registry::CommandRegistry) {
    // /mode [preset]
    r.register(
        CommandSpec::builder(
            vec!["mode"],
            "Switch to a mode preset (plan, agent, unrestricted)",
            handler(|args| match args.first() {
                Some(preset) => {
                    if is_known_preset(preset) {
                        Ok(CommandResult::Effects(vec![AppEffect::SetMode(
                            preset.clone(),
                        )]))
                    } else {
                        Err(Error::invalid_input(format!(
                            "unknown mode preset `{preset}`; valid: plan, agent, unrestricted"
                        )))
                    }
                }
                None => Ok(CommandResult::overlay(OverlayKind::ModePicker)),
            }),
        )
        .aliases(vec!["mo"])
        .arguments(vec![
            ArgumentSpec::optional("preset", "Mode preset (`plan`, `agent`, `unrestricted`).")
                .with_completer(CompletionSource::Mode),
        ])
        .category(CommandCategory::Configuration)
        .availability(CommandAvailability::Both)
        .permission(PermissionRequirement::Dangerous)
        .build(),
    )
    .ok();

    // /mode <preset> — explicit subcommands for autocomplete / help.
    for preset in ["plan", "agent", "unrestricted"] {
        r.register(
            CommandSpec::builder(
                vec!["mode", preset],
                "Switch to the {preset} mode preset",
                handler(move |args| {
                    let _ = args;
                    Ok(CommandResult::Effects(vec![AppEffect::SetMode(
                        preset.to_string(),
                    )]))
                }),
            )
            .category(CommandCategory::Configuration)
            .availability(CommandAvailability::Both)
            .permission(PermissionRequirement::Dangerous)
            .build(),
        )
        .ok();
    }
}

fn is_known_preset(preset: &str) -> bool {
    matches!(preset, "plan" | "agent" | "unrestricted")
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::context::CommandContextBuilder;

    #[test]
    fn default_registry_is_populated() {
        let r = default_registry();
        assert!(
            r.len() >= 15,
            "default registry should seed builtin slash commands, got {}",
            r.len()
        );
        for path in &[
            "help",
            "status",
            "doctor",
            "diff",
            "output",
            "config",
            "files",
            "sessions",
            "clear",
            "save",
            "load",
            "quit",
            "provider",
            "model",
            "mode",
            "mode plan",
            "mode agent",
            "mode unrestricted",
        ] {
            assert!(r.get(path).is_some(), "expected `/{path}` to be registered");
        }
    }

    #[test]
    fn help_document_lists_builtin_categories() {
        let r = default_registry();
        let doc = r.help_document(None, true);
        assert_eq!(doc.title, "Pleiades Commands");
        let joined = doc
            .sections
            .iter()
            .map(|s| s.heading.clone())
            .collect::<Vec<_>>()
            .join(" · ");
        assert!(joined.contains("Help & Status"));
        assert!(joined.contains("Provider & Model"));
    }

    #[tokio::test]
    async fn dispatch_clear_emits_effect() {
        let r = default_registry();
        let ctx = CommandContextBuilder::default().build();
        let res = r.dispatch("/clear", &ctx, true).await.unwrap();
        match res {
            CommandResult::Effects(effects) => {
                assert_eq!(effects, vec![AppEffect::ClearConversation]);
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[tokio::test]
    async fn dispatch_mode_subcommand_routes_effect() {
        let r = default_registry();
        let ctx = CommandContextBuilder::default().build();
        let res = r.dispatch("/mode plan", &ctx, true).await.unwrap();
        match res {
            CommandResult::Effects(effects) => {
                assert_eq!(effects, vec![AppEffect::SetMode("plan".to_string())]);
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[tokio::test]
    async fn dispatch_provider_with_arg_routes_effect() {
        let r = default_registry();
        let ctx = CommandContextBuilder::default().build();
        let res = r
            .dispatch("/provider bogus extra", &ctx, true)
            .await
            .unwrap();
        match res {
            CommandResult::Effects(effects) => {
                assert_eq!(effects, vec![AppEffect::SetProvider("bogus".to_string())]);
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[tokio::test]
    async fn dispatch_provider_no_arg_opens_picker() {
        let r = default_registry();
        let ctx = CommandContextBuilder::default().build();
        let res = r.dispatch("/provider", &ctx, true).await.unwrap();
        assert!(matches!(
            res,
            CommandResult::OpenOverlay(OverlayKind::ProviderPicker)
        ));
    }

    #[tokio::test]
    async fn status_document_uses_the_invocation_context() {
        let r = default_registry();
        let ctx = CommandContextBuilder::default()
            .provider("anthropic")
            .model("claude-test")
            .mode("plan")
            .build();
        let res = r.dispatch("/status", &ctx, true).await.unwrap();
        let CommandResult::RenderDocument(document) = res else {
            panic!("status should return a structured document");
        };
        assert_eq!(document.title, "Pleiades Status");
        assert!(
            document
                .sections
                .iter()
                .any(|section| { section.heading == "Provider" && section.body == "anthropic" })
        );
        assert!(
            document
                .sections
                .iter()
                .any(|section| section.heading == "Model" && section.body == "claude-test")
        );
        assert!(
            document
                .sections
                .iter()
                .any(|section| section.heading == "Mode" && section.body == "plan")
        );
    }
}
