//! Default builtin commands shipped with the live workspace.
//!
//! This module seeds a [`crate::CommandRegistry`] with the slash commands the live
//! workspace already understands today (see
//! `crates/pleiades-agent-tui/src/state.rs`), plus a handful of CLI/twin
//! commands described in the directive.  Adding a new builtin command is a
//! one-liner: write a small handler, [`CommandSpec::builder`] it, and
//! `register` it here.
//!
//! The handlers in this module are deliberately lightweight: they emit typed
//! [`AppEffect`]s, request [`OverlayKind`]s, or call application services from
//! the invocation context. They perform no terminal IO and make no direct
//! runtime calls.

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

struct ProviderListHandler;

#[async_trait]
impl CommandHandler for ProviderListHandler {
    async fn handle(&self, context: &CommandContext, _: &[String]) -> HandlerResult {
        let providers = context.services().provider().list()?;
        let mut document = crate::result::RenderableDocument::new("Providers");
        if providers.is_empty() {
            document.push_section(
                "No providers configured",
                "Run /provider add to configure one.",
            );
        }
        for provider in providers {
            document.push_section(
                provider.name,
                format!(
                    "Authentication: {}\nAPI key: {}\nBase URL: {}\nRetries: {} · Timeout: {}s",
                    provider.authentication,
                    provider.api_key_display,
                    provider.base_url,
                    provider.max_retries,
                    provider.timeout_secs
                ),
            );
        }
        Ok(CommandResult::RenderDocument(document))
    }
}

struct ProviderInfoHandler;

#[async_trait]
impl CommandHandler for ProviderInfoHandler {
    async fn handle(&self, context: &CommandContext, args: &[String]) -> HandlerResult {
        let name = args
            .first()
            .ok_or_else(|| Error::invalid_input("usage: /provider info <name>"))?;
        let provider = context.services().provider().info(name)?;
        Ok(CommandResult::RenderDocument(
            crate::result::RenderableDocument::new(format!("Provider: {}", provider.name))
                .section("Authentication", provider.authentication)
                .section("API key", provider.api_key_display)
                .section("Base URL", provider.base_url)
                .section(
                    "Expected environment variable",
                    provider.expected_env_var.as_deref().unwrap_or("(none)"),
                )
                .section(
                    "Request policy",
                    format!(
                        "{} retries · {} second timeout",
                        provider.max_retries, provider.timeout_secs
                    ),
                ),
        ))
    }
}

struct ProviderRemoveHandler;

#[async_trait]
impl CommandHandler for ProviderRemoveHandler {
    async fn handle(&self, context: &CommandContext, args: &[String]) -> HandlerResult {
        let name = args
            .first()
            .ok_or_else(|| Error::invalid_input("usage: /provider remove <name>"))?;
        context.services().provider().remove(name)?;
        Ok(CommandResult::notification(
            crate::result::NotificationLevel::Success,
            format!("Provider `{name}` removed"),
        ))
    }
}

struct ProviderTestHandler;

#[async_trait]
impl CommandHandler for ProviderTestHandler {
    async fn handle(&self, context: &CommandContext, args: &[String]) -> HandlerResult {
        let name = args
            .first()
            .ok_or_else(|| Error::invalid_input("usage: /provider test <name> [model]"))?;
        let report = context
            .services()
            .provider()
            .test(name, args.get(1).map(String::as_str))
            .await?;
        Ok(CommandResult::RenderDocument(
            crate::result::RenderableDocument::new("Provider connection successful")
                .section("Provider", report.provider)
                .section("Model", report.model)
                .section("Response", report.response)
                .section("Finish reason", report.finish_reason),
        ))
    }
}

struct ModelListHandler;

#[async_trait]
impl CommandHandler for ModelListHandler {
    async fn handle(&self, context: &CommandContext, args: &[String]) -> HandlerResult {
        let report = context
            .services()
            .model()
            .list(
                args.first().map(String::as_str),
                args.get(1).map(String::as_str),
            )
            .await?;
        let mut document = crate::result::RenderableDocument::new("Models");
        for model in report.models {
            document.push_section(
                model.display_name.as_deref().unwrap_or(&model.id),
                format!(
                    "ID: {}\nProvider: {}\nContext: {} tokens\nTools: {} · Vision: {} · Reasoning: {}",
                    model.id,
                    model.provider,
                    model.capabilities.max_context_length,
                    model.capabilities.supports_tools,
                    model.capabilities.supports_vision,
                    model.capabilities.supports_thinking
                ),
            );
        }
        if document.sections.is_empty() {
            document.push_section(
                "No models discovered",
                "Check provider connectivity with /provider test.",
            );
        }
        Ok(CommandResult::RenderDocument(document))
    }
}

struct ModelInfoHandler;

#[async_trait]
impl CommandHandler for ModelInfoHandler {
    async fn handle(&self, context: &CommandContext, args: &[String]) -> HandlerResult {
        let name = args
            .first()
            .ok_or_else(|| Error::invalid_input("usage: /model info <name>"))?;
        let model = context.services().model().info(name).await?;
        Ok(CommandResult::RenderDocument(
            crate::result::RenderableDocument::new(
                model.display_name.as_deref().unwrap_or(&model.id),
            )
            .section("Identifier", model.id)
            .section("Provider", model.provider)
            .section(
                "Capabilities",
                format!(
                    "Context: {} · Output: {}\nTools: {} · Vision: {} · Streaming: {} · Reasoning: {} · JSON: {}",
                    model.capabilities.max_context_length,
                    model.capabilities.max_output_tokens,
                    model.capabilities.supports_tools,
                    model.capabilities.supports_vision,
                    model.capabilities.supports_streaming,
                    model.capabilities.supports_thinking,
                    model.capabilities.supports_json_mode
                ),
            ),
        ))
    }
}

struct ModelAliasHandler;

#[async_trait]
impl CommandHandler for ModelAliasHandler {
    async fn handle(&self, context: &CommandContext, args: &[String]) -> HandlerResult {
        let alias = args
            .first()
            .ok_or_else(|| Error::invalid_input("usage: /model alias <alias> <model>"))?;
        let model = args
            .get(1)
            .ok_or_else(|| Error::invalid_input("usage: /model alias <alias> <model>"))?;
        context.services().model().alias(alias, model)?;
        Ok(CommandResult::notification(
            crate::result::NotificationLevel::Success,
            format!("Model alias `{alias}` now resolves to `{model}`"),
        ))
    }
}

struct ModelUnaliasHandler;

#[async_trait]
impl CommandHandler for ModelUnaliasHandler {
    async fn handle(&self, context: &CommandContext, args: &[String]) -> HandlerResult {
        let alias = args
            .first()
            .ok_or_else(|| Error::invalid_input("usage: /model unalias <alias>"))?;
        context.services().model().unalias(alias)?;
        Ok(CommandResult::notification(
            crate::result::NotificationLevel::Success,
            format!("Model alias `{alias}` removed"),
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

    r.register(
        CommandSpec::builder(
            vec!["provider", "list"],
            "List configured providers",
            ProviderListHandler,
        )
        .category(CommandCategory::Provider)
        .availability(CommandAvailability::Both)
        .permission(PermissionRequirement::Read)
        .build(),
    )
    .ok();
    r.register(
        CommandSpec::builder(
            vec!["provider", "use"],
            "Switch to a configured provider",
            handler(|args| match args.first() {
                Some(name) => Ok(CommandResult::effects([AppEffect::SetProvider(
                    name.clone(),
                )])),
                None => Err(Error::invalid_input("usage: /provider use <name>")),
            }),
        )
        .arguments(vec![
            ArgumentSpec::required("name", "Provider name")
                .with_completer(CompletionSource::Provider),
        ])
        .category(CommandCategory::Provider)
        .availability(CommandAvailability::Both)
        .build(),
    )
    .ok();
    r.register(
        CommandSpec::builder(
            vec!["provider", "info"],
            "Show provider configuration",
            ProviderInfoHandler,
        )
        .arguments(vec![
            ArgumentSpec::required("name", "Provider name")
                .with_completer(CompletionSource::Provider),
        ])
        .category(CommandCategory::Provider)
        .availability(CommandAvailability::Both)
        .permission(PermissionRequirement::Read)
        .build(),
    )
    .ok();
    r.register(
        CommandSpec::builder(
            vec!["provider", "add"],
            "Configure a provider with the secret-safe wizard",
            handler(|_| Ok(CommandResult::overlay(OverlayKind::ProviderWizard))),
        )
        .category(CommandCategory::Provider)
        .availability(CommandAvailability::Interactive)
        .permission(PermissionRequirement::Write)
        .build(),
    )
    .ok();
    r.register(
        CommandSpec::builder(
            vec!["provider", "test"],
            "Test provider connectivity with a live streamed request",
            ProviderTestHandler,
        )
        .arguments(vec![
            ArgumentSpec::required("name", "Provider name")
                .with_completer(CompletionSource::Provider),
            ArgumentSpec::optional("model", "Optional model override")
                .with_completer(CompletionSource::Model),
        ])
        .category(CommandCategory::Provider)
        .availability(CommandAvailability::Both)
        .permission(PermissionRequirement::Dangerous)
        .build(),
    )
    .ok();
    r.register(
        CommandSpec::builder(
            vec!["provider", "remove"],
            "Remove a provider configuration",
            ProviderRemoveHandler,
        )
        .arguments(vec![
            ArgumentSpec::required("name", "Provider name")
                .with_completer(CompletionSource::Provider),
        ])
        .category(CommandCategory::Provider)
        .availability(CommandAvailability::Both)
        .permission(PermissionRequirement::Write)
        .build(),
    )
    .ok();
    r.register(
        CommandSpec::builder(
            vec!["provider", "reload"],
            "Reload provider configuration",
            handler(|_| Ok(CommandResult::effects([AppEffect::ReloadExtensions]))),
        )
        .category(CommandCategory::Provider)
        .availability(CommandAvailability::Both)
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

    r.register(
        CommandSpec::builder(
            vec!["model", "list"],
            "Discover and list available models",
            ModelListHandler,
        )
        .arguments(vec![
            ArgumentSpec::optional("provider", "Optional provider filter")
                .with_completer(CompletionSource::Provider),
            ArgumentSpec::optional("search", "Optional model search text"),
        ])
        .category(CommandCategory::Provider)
        .availability(CommandAvailability::Both)
        .permission(PermissionRequirement::Dangerous)
        .build(),
    )
    .ok();
    r.register(
        CommandSpec::builder(
            vec!["model", "use"],
            "Switch the active model",
            handler(|args| match args.first() {
                Some(model) => Ok(CommandResult::effects([AppEffect::SetModel(model.clone())])),
                None => Err(Error::invalid_input("usage: /model use <name>")),
            }),
        )
        .arguments(vec![
            ArgumentSpec::required("name", "Model identifier")
                .with_completer(CompletionSource::Model),
        ])
        .category(CommandCategory::Provider)
        .availability(CommandAvailability::Both)
        .build(),
    )
    .ok();
    r.register(
        CommandSpec::builder(
            vec!["model", "info"],
            "Show model capabilities",
            ModelInfoHandler,
        )
        .arguments(vec![
            ArgumentSpec::required("name", "Model identifier or alias")
                .with_completer(CompletionSource::Model),
        ])
        .category(CommandCategory::Provider)
        .availability(CommandAvailability::Both)
        .permission(PermissionRequirement::Dangerous)
        .build(),
    )
    .ok();
    r.register(
        CommandSpec::builder(
            vec!["model", "discover"],
            "Query configured providers for models",
            ModelListHandler,
        )
        .category(CommandCategory::Provider)
        .availability(CommandAvailability::Both)
        .permission(PermissionRequirement::Dangerous)
        .build(),
    )
    .ok();
    r.register(
        CommandSpec::builder(
            vec!["model", "alias"],
            "Create a model alias",
            ModelAliasHandler,
        )
        .arguments(vec![
            ArgumentSpec::required("alias", "Alias"),
            ArgumentSpec::required("model", "Model identifier"),
        ])
        .category(CommandCategory::Provider)
        .availability(CommandAvailability::Both)
        .permission(PermissionRequirement::Write)
        .build(),
    )
    .ok();
    r.register(
        CommandSpec::builder(
            vec!["model", "unalias"],
            "Remove a model alias",
            ModelUnaliasHandler,
        )
        .arguments(vec![ArgumentSpec::required("alias", "Alias to remove")])
        .category(CommandCategory::Provider)
        .availability(CommandAvailability::Both)
        .permission(PermissionRequirement::Write)
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
            "provider list",
            "provider use",
            "provider info",
            "provider add",
            "provider test",
            "provider remove",
            "provider reload",
            "model",
            "model list",
            "model use",
            "model info",
            "model discover",
            "model alias",
            "model unalias",
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
    async fn provider_list_uses_the_injected_application_services() {
        let temp = tempfile::tempdir().unwrap();
        let services = pleiades_agent_services::ApplicationServices::with_config_dirs(
            temp.path().join("global"),
            temp.path().join("project"),
        );
        let mut config = pleiades_agent_config::Config::default();
        config.providers.insert(
            "test-provider".to_string(),
            pleiades_agent_config::ProviderConfig::default(),
        );
        services.loader().save_project(&config).unwrap();
        let ctx = CommandContextBuilder::default().services(services).build();

        let result = default_registry()
            .dispatch("/provider list", &ctx, true)
            .await
            .unwrap();
        let CommandResult::RenderDocument(document) = result else {
            panic!("provider list should render a document");
        };
        assert!(document.sections.iter().any(|section| {
            section.heading == "test-provider" && section.body.contains("Authentication")
        }));
    }

    #[tokio::test]
    async fn model_alias_uses_the_injected_application_services() {
        let temp = tempfile::tempdir().unwrap();
        let services = pleiades_agent_services::ApplicationServices::with_config_dirs(
            temp.path().join("global"),
            temp.path().join("project"),
        );
        services
            .loader()
            .save_project(&pleiades_agent_config::Config::default())
            .unwrap();
        let ctx = CommandContextBuilder::default()
            .services(services.clone())
            .build();
        let result = default_registry()
            .dispatch("/model alias fast model-x", &ctx, true)
            .await
            .unwrap();
        assert!(matches!(result, CommandResult::Notification(_)));
        let config = services.loader().load().unwrap();
        assert_eq!(
            config.models.aliases.get("fast").map(String::as_str),
            Some("model-x")
        );
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
