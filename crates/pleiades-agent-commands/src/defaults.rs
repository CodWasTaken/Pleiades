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
use pleiades_agent_permissions::{DecisionKind, PermissionAction};

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

struct ModelFavoriteHandler;

#[async_trait]
impl CommandHandler for ModelFavoriteHandler {
    async fn handle(&self, context: &CommandContext, args: &[String]) -> HandlerResult {
        let model = args
            .first()
            .ok_or_else(|| Error::invalid_input("usage: /model favorite <name>"))?;
        let added = context.services().model().favorite(model)?;
        Ok(CommandResult::notification(
            crate::result::NotificationLevel::Success,
            format!(
                "Model `{model}` {} favorites",
                if added { "added to" } else { "removed from" }
            ),
        ))
    }
}

struct ModelFavoritesHandler;

#[async_trait]
impl CommandHandler for ModelFavoritesHandler {
    async fn handle(&self, context: &CommandContext, _: &[String]) -> HandlerResult {
        let preferences = context.services().model().preferences()?;
        let favorites = if preferences.favorites.is_empty() {
            "No favorite models yet.".to_string()
        } else {
            preferences.favorites.join("\n")
        };
        Ok(CommandResult::RenderDocument(
            crate::result::RenderableDocument::new("Model preferences")
                .section("Favorites", favorites)
                .section(
                    "Reasoning effort",
                    preferences
                        .reasoning
                        .as_deref()
                        .unwrap_or("provider default"),
                ),
        ))
    }
}

struct ModelReasoningHandler;

#[async_trait]
impl CommandHandler for ModelReasoningHandler {
    async fn handle(&self, context: &CommandContext, args: &[String]) -> HandlerResult {
        let level = args.first().ok_or_else(|| {
            Error::invalid_input("usage: /model reasoning <minimal|low|medium|high>")
        })?;
        context.services().model().set_reasoning(level)?;
        Ok(CommandResult::notification(
            crate::result::NotificationLevel::Success,
            format!("Reasoning effort set to `{}`", level.to_ascii_lowercase()),
        ))
    }
}

struct PluginListHandler;

#[async_trait]
impl CommandHandler for PluginListHandler {
    async fn handle(&self, context: &CommandContext, _: &[String]) -> HandlerResult {
        let plugins = context.services().plugin().list()?;
        let mut document = crate::result::RenderableDocument::new("Plugins");
        for plugin in plugins {
            document.push_section(
                plugin.id,
                format!(
                    "{} · {} · v{}\n{}\nTools: {} · Hooks: {}",
                    plugin.kind,
                    if plugin.enabled {
                        "enabled"
                    } else {
                        "disabled"
                    },
                    plugin.version,
                    plugin.description,
                    plugin.tool_count,
                    plugin.has_hooks
                ),
            );
        }
        Ok(CommandResult::RenderDocument(document))
    }
}

struct PluginInfoHandler;

#[async_trait]
impl CommandHandler for PluginInfoHandler {
    async fn handle(&self, context: &CommandContext, args: &[String]) -> HandlerResult {
        let id = args
            .first()
            .ok_or_else(|| Error::invalid_input("usage: /plugins info <id>"))?;
        let plugin = context.services().plugin().info(id)?;
        Ok(CommandResult::RenderDocument(
            crate::result::RenderableDocument::new(plugin.name)
                .section("ID", plugin.id)
                .section("Version", plugin.version)
                .section("Source", plugin.source)
                .section("Description", plugin.description)
                .section(
                    "Permissions",
                    if plugin.permissions.is_empty() {
                        "No executable hooks or tools requested".to_string()
                    } else {
                        plugin.permissions.join("\n")
                    },
                ),
        ))
    }
}

enum PluginMutation {
    Install,
    Uninstall,
    Enable,
    Disable,
    Update,
}

struct PluginMutationHandler(PluginMutation);

#[async_trait]
impl CommandHandler for PluginMutationHandler {
    async fn handle(&self, context: &CommandContext, args: &[String]) -> HandlerResult {
        let target = args
            .first()
            .ok_or_else(|| Error::invalid_input("plugin path or identifier is required"))?;
        let service = context.services().plugin();
        let message = match self.0 {
            PluginMutation::Install => {
                let report = service.install(target)?;
                format!("Installed `{}` v{}", report.id, report.version)
            }
            PluginMutation::Uninstall => {
                service.uninstall(target)?;
                format!("Uninstalled `{target}`")
            }
            PluginMutation::Enable => {
                service.enable(target)?;
                format!("Enabled `{target}`")
            }
            PluginMutation::Disable => {
                service.disable(target)?;
                format!("Disabled `{target}`")
            }
            PluginMutation::Update => {
                let report = service.update(target)?;
                format!(
                    "Updated `{}` from v{} to v{}",
                    report.id, report.old_version, report.new_version
                )
            }
        };
        Ok(CommandResult::notification(
            crate::result::NotificationLevel::Success,
            message,
        ))
    }
}

struct PermissionsShowHandler;

#[async_trait]
impl CommandHandler for PermissionsShowHandler {
    async fn handle(&self, context: &CommandContext, _: &[String]) -> HandlerResult {
        let report = context.services().permission().show()?;
        let mut document = crate::result::RenderableDocument::new("Permissions");
        if report.rules.is_empty() {
            document.push_section("Rules", "No structured permission rules configured.");
        } else {
            for item in report.rules {
                document.push_section(
                    format!("Rule {}", item.index),
                    format!(
                        "{} {} `{}`",
                        permission_action_label(item.rule.action),
                        item.rule.tool,
                        item.rule.pattern
                    ),
                );
            }
        }
        document.push_section(
            "Legacy always allow",
            if report.always_allow.is_empty() {
                "(none)".to_string()
            } else {
                report.always_allow.join("\n")
            },
        );
        document.push_section(
            "Legacy always deny",
            if report.always_deny.is_empty() {
                "(none)".to_string()
            } else {
                report.always_deny.join("\n")
            },
        );
        Ok(CommandResult::RenderDocument(document))
    }
}

struct PermissionAddHandler(PermissionAction);

#[async_trait]
impl CommandHandler for PermissionAddHandler {
    async fn handle(&self, context: &CommandContext, args: &[String]) -> HandlerResult {
        if args.is_empty() {
            return Err(Error::invalid_input(
                "usage: /permissions <allow|ask|deny> <pattern>",
            ));
        }
        let pattern = args.join(" ");
        context
            .services()
            .permission()
            .add_bash_rule(self.0, &pattern)?;
        Ok(CommandResult::notification(
            crate::result::NotificationLevel::Success,
            format!(
                "Permission rule added: {} bash `{pattern}`",
                permission_action_label(self.0)
            ),
        ))
    }
}

struct PermissionResetHandler;

#[async_trait]
impl CommandHandler for PermissionResetHandler {
    async fn handle(&self, context: &CommandContext, _: &[String]) -> HandlerResult {
        context.services().permission().reset()?;
        Ok(CommandResult::notification(
            crate::result::NotificationLevel::Success,
            "Permission rules reset",
        ))
    }
}

struct PermissionTestHandler;

#[async_trait]
impl CommandHandler for PermissionTestHandler {
    async fn handle(&self, context: &CommandContext, args: &[String]) -> HandlerResult {
        if args.is_empty() {
            return Err(Error::invalid_input("usage: /permissions test <command>"));
        }
        let command = args.join(" ");
        let report = context
            .services()
            .permission()
            .test_bash_command(&command)?;
        Ok(CommandResult::RenderDocument(
            crate::result::RenderableDocument::new("Permission test")
                .section("Command", report.command)
                .section("Decision", permission_decision_label(report.decision.kind))
                .section("Reason", report.decision.reason)
                .section(
                    "Clauses",
                    if report.decision.clauses.is_empty() {
                        "(none)".to_string()
                    } else {
                        report.decision.clauses.join("\n")
                    },
                ),
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
    register_plugin_family(&mut r);
    register_mode_family(&mut r);
    register_permissions_family(&mut r);
    register_checkpoint_family(&mut r);
    register_context_family(&mut r);
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
    r.register(
        CommandSpec::builder(
            vec!["model", "favorite"],
            "Add or remove a model from favorites",
            ModelFavoriteHandler,
        )
        .arguments(vec![
            ArgumentSpec::required("name", "Model identifier")
                .with_completer(CompletionSource::Model),
        ])
        .category(CommandCategory::Provider)
        .availability(CommandAvailability::Both)
        .permission(PermissionRequirement::Write)
        .build(),
    )
    .ok();
    r.register(
        CommandSpec::builder(
            vec!["model", "favorites"],
            "Show favorite models and reasoning preference",
            ModelFavoritesHandler,
        )
        .category(CommandCategory::Provider)
        .availability(CommandAvailability::Both)
        .permission(PermissionRequirement::Read)
        .build(),
    )
    .ok();
    r.register(
        CommandSpec::builder(
            vec!["model", "reasoning"],
            "Set preferred reasoning effort",
            ModelReasoningHandler,
        )
        .arguments(vec![ArgumentSpec::required(
            "level",
            "minimal, low, medium, or high",
        )])
        .category(CommandCategory::Provider)
        .availability(CommandAvailability::Both)
        .permission(PermissionRequirement::Write)
        .build(),
    )
    .ok();
}

fn register_plugin_family(r: &mut crate::registry::CommandRegistry) {
    r.register(
        CommandSpec::builder(
            vec!["plugins"],
            "Manage installed plugins",
            PluginListHandler,
        )
        .aliases(vec!["plugin"])
        .category(CommandCategory::Extension)
        .availability(CommandAvailability::Both)
        .build(),
    )
    .ok();
    r.register(
        CommandSpec::builder(
            vec!["plugins", "list"],
            "List installed plugins",
            PluginListHandler,
        )
        .category(CommandCategory::Extension)
        .availability(CommandAvailability::Both)
        .permission(PermissionRequirement::Read)
        .build(),
    )
    .ok();
    r.register(
        CommandSpec::builder(
            vec!["plugins", "info"],
            "Inspect a plugin and its requested permissions",
            PluginInfoHandler,
        )
        .arguments(vec![
            ArgumentSpec::required("id", "Plugin identifier")
                .with_completer(CompletionSource::Plugin),
        ])
        .category(CommandCategory::Extension)
        .availability(CommandAvailability::Both)
        .permission(PermissionRequirement::Read)
        .build(),
    )
    .ok();
    for (name, description, mutation, argument) in [
        (
            "install",
            "Install a plugin from a local directory",
            PluginMutation::Install,
            "path",
        ),
        (
            "uninstall",
            "Uninstall a plugin",
            PluginMutation::Uninstall,
            "id",
        ),
        ("enable", "Enable a plugin", PluginMutation::Enable, "id"),
        ("disable", "Disable a plugin", PluginMutation::Disable, "id"),
        (
            "update",
            "Update a plugin from its source",
            PluginMutation::Update,
            "id",
        ),
    ] {
        r.register(
            CommandSpec::builder(
                vec!["plugins", name],
                description,
                PluginMutationHandler(mutation),
            )
            .arguments(vec![
                ArgumentSpec::required(argument, "Plugin path or identifier")
                    .with_completer(CompletionSource::Plugin),
            ])
            .category(CommandCategory::Extension)
            .availability(CommandAvailability::Both)
            .permission(PermissionRequirement::Dangerous)
            .build(),
        )
        .ok();
    }
    r.register(
        CommandSpec::builder(
            vec!["plugins", "permissions"],
            "Inspect plugin permissions",
            PluginInfoHandler,
        )
        .arguments(vec![
            ArgumentSpec::required("id", "Plugin identifier")
                .with_completer(CompletionSource::Plugin),
        ])
        .category(CommandCategory::Extension)
        .availability(CommandAvailability::Both)
        .permission(PermissionRequirement::Read)
        .build(),
    )
    .ok();
    r.register(
        CommandSpec::builder(
            vec!["plugins", "reload"],
            "Reload installed plugins",
            handler(|_| Ok(CommandResult::effects([AppEffect::ReloadExtensions]))),
        )
        .category(CommandCategory::Extension)
        .availability(CommandAvailability::Both)
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
            "Switch to a mode preset (plan, agent, auto, yolo)",
            handler(|args| match args.first() {
                Some(preset) if preset == "yolo" => {
                    Ok(CommandResult::overlay(OverlayKind::YoloWarning))
                }
                Some(preset) => {
                    if is_known_preset(preset) {
                        Ok(CommandResult::Effects(vec![AppEffect::SetMode(
                            preset.clone(),
                        )]))
                    } else {
                        Err(Error::invalid_input(format!(
                            "unknown mode preset `{preset}`; valid: plan, agent, auto, yolo"
                        )))
                    }
                }
                None => Ok(CommandResult::overlay(OverlayKind::ModePicker)),
            }),
        )
        .aliases(vec!["mo"])
        .arguments(vec![
            ArgumentSpec::optional("preset", "Mode preset (`plan`, `agent`, `auto`, `yolo`).")
                .with_completer(CompletionSource::Mode),
        ])
        .category(CommandCategory::Configuration)
        .availability(CommandAvailability::Both)
        .permission(PermissionRequirement::Dangerous)
        .build(),
    )
    .ok();

    // /mode <preset> — explicit subcommands for autocomplete / help.
    for preset in ["plan", "agent", "auto"] {
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
    r.register(
        CommandSpec::builder(
            vec!["mode", "yolo"],
            "Enable unapproved full-host access after confirmation",
            handler(|_| Ok(CommandResult::overlay(OverlayKind::YoloWarning))),
        )
        .category(CommandCategory::Configuration)
        .availability(CommandAvailability::Interactive)
        .permission(PermissionRequirement::Dangerous)
        .build(),
    )
    .ok();
}

fn is_known_preset(preset: &str) -> bool {
    matches!(preset, "plan" | "agent" | "auto" | "yolo")
}

fn register_permissions_family(r: &mut crate::registry::CommandRegistry) {
    r.register(
        CommandSpec::builder(
            vec!["permissions"],
            "Inspect and edit permission rules",
            PermissionsShowHandler,
        )
        .category(CommandCategory::Configuration)
        .availability(CommandAvailability::Both)
        .permission(PermissionRequirement::Read)
        .build(),
    )
    .ok();
    r.register(
        CommandSpec::builder(
            vec!["permissions", "show"],
            "Show permission rules",
            PermissionsShowHandler,
        )
        .category(CommandCategory::Configuration)
        .availability(CommandAvailability::Both)
        .permission(PermissionRequirement::Read)
        .build(),
    )
    .ok();
    for (name, action) in [
        ("allow", PermissionAction::Allow),
        ("ask", PermissionAction::Ask),
        ("deny", PermissionAction::Deny),
    ] {
        r.register(
            CommandSpec::builder(
                vec!["permissions", name],
                "Add a bash permission rule",
                PermissionAddHandler(action),
            )
            .arguments(vec![ArgumentSpec::required(
                "pattern",
                "Glob pattern matched against each shell command clause.",
            )])
            .category(CommandCategory::Configuration)
            .availability(CommandAvailability::Both)
            .permission(PermissionRequirement::Write)
            .build(),
        )
        .ok();
    }
    r.register(
        CommandSpec::builder(
            vec!["permissions", "reset"],
            "Remove configured permission rules",
            PermissionResetHandler,
        )
        .category(CommandCategory::Configuration)
        .availability(CommandAvailability::Both)
        .permission(PermissionRequirement::Write)
        .build(),
    )
    .ok();
    r.register(
        CommandSpec::builder(
            vec!["permissions", "test"],
            "Evaluate a bash command against configured rules",
            PermissionTestHandler,
        )
        .arguments(vec![ArgumentSpec::required(
            "command",
            "Shell command to evaluate.",
        )])
        .category(CommandCategory::Configuration)
        .availability(CommandAvailability::Both)
        .permission(PermissionRequirement::Read)
        .build(),
    )
    .ok();
}

fn permission_action_label(action: PermissionAction) -> &'static str {
    match action {
        PermissionAction::Allow => "allow",
        PermissionAction::Ask => "ask",
        PermissionAction::Deny => "deny",
    }
}

fn permission_decision_label(kind: DecisionKind) -> &'static str {
    match kind {
        DecisionKind::Allow => "allow",
        DecisionKind::Ask => "ask",
        DecisionKind::Deny => "deny",
        DecisionKind::Default => "default",
    }
}

fn register_checkpoint_family(r: &mut crate::registry::CommandRegistry) {
    r.register(
        CommandSpec::builder(
            vec!["checkpoint"],
            "Create or manage checkpoints",
            handler(|_| Ok(CommandResult::RenderDocument(checkpoint_help_document()))),
        )
        .category(CommandCategory::History)
        .availability(CommandAvailability::Both)
        .build(),
    )
    .ok();
    r.register(
        CommandSpec::builder(
            vec!["checkpoint", "create"],
            "Create a checkpoint",
            handler(|args| {
                let name = if args.is_empty() {
                    None
                } else {
                    Some(args.join(" "))
                };
                Ok(CommandResult::effects([AppEffect::CreateCheckpoint(name)]))
            }),
        )
        .arguments(vec![ArgumentSpec::optional(
            "name",
            "Optional checkpoint label.",
        )])
        .category(CommandCategory::History)
        .availability(CommandAvailability::Both)
        .permission(PermissionRequirement::Write)
        .build(),
    )
    .ok();
    r.register(
        CommandSpec::builder(
            vec!["checkpoint", "list"],
            "List checkpoints",
            handler(|_| Ok(CommandResult::effects([AppEffect::ListCheckpoints]))),
        )
        .category(CommandCategory::History)
        .availability(CommandAvailability::Both)
        .permission(PermissionRequirement::Read)
        .build(),
    )
    .ok();
    r.register(
        CommandSpec::builder(
            vec!["checkpoint", "show"],
            "Show checkpoint details",
            handler(|args| match args.first() {
                Some(id) => Ok(CommandResult::effects([AppEffect::ShowCheckpoint(
                    id.clone(),
                )])),
                None => Err(Error::invalid_input("usage: /checkpoint show <id>")),
            }),
        )
        .arguments(vec![ArgumentSpec::required("id", "Checkpoint id")])
        .category(CommandCategory::History)
        .availability(CommandAvailability::Both)
        .permission(PermissionRequirement::Read)
        .build(),
    )
    .ok();
    r.register(
        CommandSpec::builder(
            vec!["checkpoint", "restore"],
            "Preview or restore a checkpoint",
            handler(|args| match args.first() {
                Some(id) => Ok(CommandResult::effects([AppEffect::RestoreCheckpoint {
                    id: id.clone(),
                    confirm: args
                        .iter()
                        .any(|arg| arg == "--confirm" || arg == "confirm"),
                }])),
                None => Err(Error::invalid_input(
                    "usage: /checkpoint restore <id> [--confirm]",
                )),
            }),
        )
        .arguments(vec![
            ArgumentSpec::required("id", "Checkpoint id"),
            ArgumentSpec::optional("--confirm", "Apply the restore after previewing."),
        ])
        .category(CommandCategory::History)
        .availability(CommandAvailability::Both)
        .permission(PermissionRequirement::Write)
        .build(),
    )
    .ok();
    r.register(
        CommandSpec::builder(
            vec!["checkpoint", "delete"],
            "Delete a checkpoint",
            handler(|args| match args.first() {
                Some(id) => Ok(CommandResult::effects([AppEffect::DeleteCheckpoint(
                    id.clone(),
                )])),
                None => Err(Error::invalid_input("usage: /checkpoint delete <id>")),
            }),
        )
        .arguments(vec![ArgumentSpec::required("id", "Checkpoint id")])
        .category(CommandCategory::History)
        .availability(CommandAvailability::Both)
        .permission(PermissionRequirement::Write)
        .build(),
    )
    .ok();
    for name in ["undo", "redo", "rewind"] {
        r.register(
            CommandSpec::builder(
                vec![name],
                "Open checkpoint history",
                handler(|_| Ok(CommandResult::RenderDocument(checkpoint_help_document()))),
            )
            .category(CommandCategory::History)
            .availability(CommandAvailability::Interactive)
            .permission(PermissionRequirement::Read)
            .build(),
        )
        .ok();
    }
}

fn checkpoint_help_document() -> crate::result::RenderableDocument {
    crate::result::RenderableDocument::new("Checkpoints")
        .section("Create", "/checkpoint create [name]")
        .section("List", "/checkpoint list")
        .section("Inspect", "/checkpoint show <id>")
        .section(
            "Restore",
            "/checkpoint restore <id> previews; /checkpoint restore <id> --confirm applies.",
        )
        .section("Delete", "/checkpoint delete <id>")
}

fn register_context_family(r: &mut crate::registry::CommandRegistry) {
    r.register(
        CommandSpec::builder(
            vec!["context"],
            "Inspect and compact model context",
            handler(|_| Ok(CommandResult::effects([AppEffect::ContextStatus]))),
        )
        .category(CommandCategory::Workspace)
        .availability(CommandAvailability::Both)
        .permission(PermissionRequirement::Read)
        .build(),
    )
    .ok();
    r.register(
        CommandSpec::builder(
            vec!["context", "status"],
            "Show context usage summary",
            handler(|_| Ok(CommandResult::effects([AppEffect::ContextStatus]))),
        )
        .category(CommandCategory::Workspace)
        .availability(CommandAvailability::Both)
        .permission(PermissionRequirement::Read)
        .build(),
    )
    .ok();
    r.register(
        CommandSpec::builder(
            vec!["context", "inspect"],
            "Show detailed context accounting",
            handler(|_| Ok(CommandResult::effects([AppEffect::ContextInspect]))),
        )
        .category(CommandCategory::Workspace)
        .availability(CommandAvailability::Both)
        .permission(PermissionRequirement::Read)
        .build(),
    )
    .ok();
    r.register(
        CommandSpec::builder(
            vec!["context", "compact"],
            "Compact older conversation context into a summary",
            handler(|_| Ok(CommandResult::effects([AppEffect::ContextCompact]))),
        )
        .category(CommandCategory::Workspace)
        .availability(CommandAvailability::Both)
        .permission(PermissionRequirement::Dangerous)
        .build(),
    )
    .ok();
    r.register(
        CommandSpec::builder(
            vec!["context", "pin"],
            "Pin a file path, message id, or note into the context report",
            handler(|args| {
                if args.is_empty() {
                    return Err(Error::invalid_input(
                        "usage: /context pin <file-or-message>",
                    ));
                }
                Ok(CommandResult::effects([AppEffect::ContextPin(
                    args.join(" "),
                )]))
            }),
        )
        .arguments(vec![ArgumentSpec::required(
            "file-or-message",
            "File path, message id, or short note to pin.",
        )])
        .category(CommandCategory::Workspace)
        .availability(CommandAvailability::Both)
        .permission(PermissionRequirement::Write)
        .build(),
    )
    .ok();
    r.register(
        CommandSpec::builder(
            vec!["context", "unpin"],
            "Remove a pinned context item",
            handler(|args| match args.first() {
                Some(id) => Ok(CommandResult::effects([AppEffect::ContextUnpin(
                    id.clone(),
                )])),
                None => Err(Error::invalid_input("usage: /context unpin <id>")),
            }),
        )
        .arguments(vec![ArgumentSpec::required("id", "Pinned context id")])
        .category(CommandCategory::Workspace)
        .availability(CommandAvailability::Both)
        .permission(PermissionRequirement::Write)
        .build(),
    )
    .ok();
    r.register(
        CommandSpec::builder(
            vec!["context", "sources"],
            "Show files and tools represented in context",
            handler(|_| Ok(CommandResult::effects([AppEffect::ContextSources]))),
        )
        .category(CommandCategory::Workspace)
        .availability(CommandAvailability::Both)
        .permission(PermissionRequirement::Read)
        .build(),
    )
    .ok();
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
            "plugins",
            "plugins list",
            "plugins info",
            "plugins install",
            "plugins uninstall",
            "plugins enable",
            "plugins disable",
            "plugins permissions",
            "plugins reload",
            "mode",
            "mode plan",
            "mode agent",
            "mode auto",
            "mode yolo",
            "permissions",
            "permissions show",
            "permissions allow",
            "permissions ask",
            "permissions deny",
            "permissions reset",
            "permissions test",
            "checkpoint",
            "checkpoint create",
            "checkpoint list",
            "checkpoint show",
            "checkpoint restore",
            "checkpoint delete",
            "undo",
            "redo",
            "rewind",
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
    async fn dispatch_yolo_requires_the_warning_overlay() {
        let registry = default_registry();
        let context = CommandContextBuilder::default().build();
        let result = registry
            .dispatch("/mode yolo", &context, true)
            .await
            .unwrap();
        assert!(matches!(
            result,
            CommandResult::OpenOverlay(OverlayKind::YoloWarning)
        ));
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
    async fn permissions_commands_use_the_injected_application_services() {
        let temp = tempfile::tempdir().unwrap();
        let services = pleiades_agent_services::ApplicationServices::with_config_dirs(
            temp.path().join("global"),
            temp.path().join("project"),
        );
        let ctx = CommandContextBuilder::default()
            .services(services.clone())
            .build();

        let result = default_registry()
            .dispatch("/permissions deny git push *", &ctx, true)
            .await
            .unwrap();
        assert!(matches!(result, CommandResult::Notification(_)));

        let result = default_registry()
            .dispatch("/permissions test git push origin main", &ctx, true)
            .await
            .unwrap();
        let CommandResult::RenderDocument(document) = result else {
            panic!("permission test should render a document");
        };
        assert!(
            document
                .sections
                .iter()
                .any(|section| section.heading == "Decision" && section.body == "deny")
        );
    }

    #[tokio::test]
    async fn checkpoint_commands_emit_typed_effects() {
        let context = CommandContextBuilder::default().build();
        let result = default_registry()
            .dispatch("/checkpoint create before edit", &context, true)
            .await
            .unwrap();
        let CommandResult::Effects(effects) = result else {
            panic!("checkpoint create should emit an effect");
        };
        assert_eq!(
            effects,
            vec![AppEffect::CreateCheckpoint(Some("before edit".to_string()))]
        );

        let result = default_registry()
            .dispatch("/checkpoint restore abc --confirm", &context, true)
            .await
            .unwrap();
        let CommandResult::Effects(effects) = result else {
            panic!("checkpoint restore should emit an effect");
        };
        assert_eq!(
            effects,
            vec![AppEffect::RestoreCheckpoint {
                id: "abc".to_string(),
                confirm: true
            }]
        );
    }

    #[tokio::test]
    async fn context_commands_emit_typed_effects() {
        let context = CommandContextBuilder::default().build();
        let result = default_registry()
            .dispatch("/context status", &context, true)
            .await
            .unwrap();
        let CommandResult::Effects(effects) = result else {
            panic!("context status should emit an effect");
        };
        assert_eq!(effects, vec![AppEffect::ContextStatus]);

        let result = default_registry()
            .dispatch(
                "/context pin crates/pleiades-agent-engine/src/runtime.rs",
                &context,
                true,
            )
            .await
            .unwrap();
        let CommandResult::Effects(effects) = result else {
            panic!("context pin should emit an effect");
        };
        assert_eq!(
            effects,
            vec![AppEffect::ContextPin(
                "crates/pleiades-agent-engine/src/runtime.rs".to_string()
            )]
        );

        let suggestions = default_registry().suggest("/context ", true);
        assert!(
            suggestions
                .iter()
                .any(|suggestion| suggestion.label == "context compact")
        );
    }

    #[tokio::test]
    async fn plugin_list_uses_the_injected_application_services() {
        let temp = tempfile::tempdir().unwrap();
        let services = pleiades_agent_services::ApplicationServices::with_config_dirs(
            temp.path().join("global"),
            temp.path().join("project"),
        );
        let ctx = CommandContextBuilder::default().services(services).build();
        let result = default_registry()
            .dispatch("/plugins list", &ctx, true)
            .await
            .unwrap();
        let CommandResult::RenderDocument(document) = result else {
            panic!("plugin list should render a document");
        };
        assert!(document.sections.iter().any(|section| {
            section.heading == "pleiades-agent-core-builtin" && section.body.contains("enabled")
        }));
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
