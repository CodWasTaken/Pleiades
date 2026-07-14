//! Command descriptor types.
//!
//! A [`CommandSpec`] is a pure description of a user-invocable command.  It
//! carries the human-facing metadata (path, aliases, description, usage,
//! examples, category) and machine-facing metadata (availability,
//! permission requirement, shortcut, argument completers), plus an
//! [`CommandHandler`] responsible for producing a typed [`crate::CommandResult`].
//!
//! Specs are intentionally owned structures rather than `&'static` constants
//! so that plugin-provided and user-defined commands can be constructed at
//! load time from manifests and TOML files and inserted into the registry on
//! equal footing with builtin commands.

use std::sync::Arc;

use crate::handler::CommandHandler;

/// High-level group a command belongs to.
///
/// Used to render help in sections, to filter the command palette, and to
/// give plugins and custom commands a stable home in the listing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandCategory {
    /// `/help`, `/status`, `/doctor`.
    Help,
    /// `/files`, `/diff`, `/output`, `/clear`, `/save`, `/quit`.
    Workspace,
    /// `/provider`, `/model`.
    Provider,
    /// `/plugins`, `/mcp`, `/tools`, `/agents`, `/skills`, `/prompts`, `/workflows`.
    Extension,
    /// `/memory`, `/session`.
    Memory,
    /// `/git`, `/lsp`, `/process`, `/browser`, `/project`, `/workspace`.
    Project,
    /// `/config`, `/profile`, `/theme`, `/mode`, `/permissions`, `/context`, `/budget`.
    Configuration,
    /// `/checkpoint`, `/undo`, `/redo`, `/rewind`.
    History,
    /// `/run`, `/test`, `/verify`, `/review`.
    Verification,
    /// Dynamically contributed by plugins.
    Plugin,
    /// Dynamically contributed by MCP servers.
    Mcp,
    /// User-defined custom commands from `.pleiades/commands/*.toml`.
    Custom,
    /// Registered but excluded from suggestions and palette listings.
    Hidden,
}

impl CommandCategory {
    /// Human-readable label used by the help document generator.
    pub fn label(self) -> &'static str {
        match self {
            CommandCategory::Help => "Help & Status",
            CommandCategory::Workspace => "Workspace",
            CommandCategory::Provider => "Provider & Model",
            CommandCategory::Extension => "Extensions",
            CommandCategory::Memory => "Memory & Sessions",
            CommandCategory::Project => "Project Tooling",
            CommandCategory::Configuration => "Configuration",
            CommandCategory::History => "History & Checkpoints",
            CommandCategory::Verification => "Verification",
            CommandCategory::Plugin => "Plugin Commands",
            CommandCategory::Mcp => "MCP Commands",
            CommandCategory::Custom => "Custom Commands",
            CommandCategory::Hidden => "Hidden",
        }
    }
}

/// Where a command is allowed to run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandAvailability {
    /// Only the interactive live workspace (TUI / REPL).
    Interactive,
    /// Only the headless CLI / RPC clients.
    Headless,
    /// Available in both modes.
    Both,
}

impl CommandAvailability {
    /// Whether this availability permits the given mode.
    pub fn allows(self, interactive: bool) -> bool {
        match self {
            CommandAvailability::Both => true,
            CommandAvailability::Interactive => interactive,
            CommandAvailability::Headless => !interactive,
        }
    }
}

/// Minimum permission a caller must hold to invoke the command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PermissionRequirement {
    /// No mutation, no network, no secrets — purely informational.
    None,
    /// Reads workspace state (files, git, configs).
    Read,
    /// Mutates workspace state (writes files, runs build commands, edits config).
    Write,
    /// Potentially destructive or unrestricted (full network, host-wide
    /// filesystem access, plugin shell execution).  Always asks in plan /
    /// agent / auto modes unless explicitly allowed by a rule.
    Dangerous,
}

/// Source of autocomplete suggestions for a positional argument.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompletionSource {
    Provider,
    Model,
    Session,
    File,
    Plugin,
    McpServer,
    Agent,
    Skill,
    Prompt,
    Workflow,
    Mode,
    Theme,
    /// The handler supplies completions dynamically, e.g. by reading live
    /// state not known to the registry.
    Handler,
    /// No completion offered.
    None,
}

/// Description of one positional argument.
#[derive(Debug, Clone)]
pub struct ArgumentSpec {
    pub name: &'static str,
    pub description: &'static str,
    pub required: bool,
    pub completer: CompletionSource,
}

impl ArgumentSpec {
    pub const fn required(name: &'static str, description: &'static str) -> Self {
        Self {
            name,
            description,
            required: true,
            completer: CompletionSource::None,
        }
    }

    pub const fn optional(name: &'static str, description: &'static str) -> Self {
        Self {
            name,
            description,
            required: false,
            completer: CompletionSource::None,
        }
    }

    pub const fn with_completer(mut self, source: CompletionSource) -> Self {
        self.completer = source;
        self
    }
}

/// Keyboard shortcut associated with a command (for palette display).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shortcut {
    /// e.g. `Ctrl+R`, `Ctrl+P`.
    Ctrl(char),
    /// e.g. `F1`.
    F(u8),
    /// No shortcut.
    None,
}

impl Shortcut {
    pub const NONE: Shortcut = Shortcut::None;

    /// Render the shortcut as a compact label, e.g. `^P` or `F1`.
    pub fn label(self) -> String {
        match self {
            Shortcut::Ctrl(c) => format!("^{}", c),
            Shortcut::F(n) => format!("F{}", n),
            Shortcut::None => String::new(),
        }
    }
}

/// Pure descriptor for one command.
///
/// Construct builtin specs via [`CommandSpec::new`] and plugin/custom specs
/// via [`CommandSpec::builder`].
#[derive(Clone)]
pub struct CommandSpec {
    pub path: Vec<&'static str>,
    pub aliases: Vec<&'static str>,
    pub description: &'static str,
    pub usage: String,
    pub examples: &'static [&'static str],
    pub category: CommandCategory,
    pub availability: CommandAvailability,
    pub permission: PermissionRequirement,
    pub arguments: Vec<ArgumentSpec>,
    pub shortcut: Shortcut,
    pub handler: Arc<dyn CommandHandler + Send + Sync>,
}

impl std::fmt::Debug for CommandSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommandSpec")
            .field("path", &self.path)
            .field("aliases", &self.aliases)
            .field("category", &self.category)
            .field("availability", &self.availability)
            .field("permission", &self.permission)
            .field("shortcut", &self.shortcut)
            .finish_non_exhaustive()
    }
}

impl CommandSpec {
    /// Construct a spec from the essential fields.  Use the builder for
    /// richer metadata.
    pub fn new(
        path: Vec<&'static str>,
        description: &'static str,
        handler: impl CommandHandler + 'static,
    ) -> Self {
        Self {
            path,
            aliases: Vec::new(),
            description,
            usage: String::new(),
            examples: &[],
            category: CommandCategory::Workspace,
            availability: CommandAvailability::Both,
            permission: PermissionRequirement::None,
            arguments: Vec::new(),
            shortcut: Shortcut::None,
            handler: Arc::new(handler),
        }
    }

    /// Start a builder for richer metadata.
    pub fn builder(
        path: Vec<&'static str>,
        description: &'static str,
        handler: impl CommandHandler + 'static,
    ) -> CommandSpecBuilder {
        CommandSpecBuilder {
            spec: Self::new(path, description, handler),
        }
    }

    /// The canonical name, e.g. `provider use`, with path segments joined
    /// by single spaces.
    pub fn canonical(&self) -> String {
        self.path.join(" ")
    }

    /// How this command is invoked with its leading slash, e.g. `/provider use`.
    pub fn slash(&self) -> String {
        format!("/{}", self.canonical())
    }

    /// Minimum depth of the command path (1 = root command).
    pub fn depth(&self) -> usize {
        self.path.len()
    }

    /// Whether this is a direct (depth+1) child of the given parent path
    /// tokens.  Grandchildren are not considered children.
    pub fn is_child_of(&self, parent: &[&str]) -> bool {
        if parent.len() + 1 != self.path.len() {
            return false;
        }
        self.path[..parent.len()].iter().eq(parent.iter())
    }
}

/// Builder for [`CommandSpec`].
pub struct CommandSpecBuilder {
    spec: CommandSpec,
}

impl CommandSpecBuilder {
    pub fn aliases(mut self, aliases: Vec<&'static str>) -> Self {
        self.spec.aliases = aliases;
        self
    }
    pub fn usage(mut self, usage: impl Into<String>) -> Self {
        self.spec.usage = usage.into();
        self
    }
    pub fn examples(mut self, examples: &'static [&'static str]) -> Self {
        self.spec.examples = examples;
        self
    }
    pub fn category(mut self, category: CommandCategory) -> Self {
        self.spec.category = category;
        self
    }
    pub fn availability(mut self, availability: CommandAvailability) -> Self {
        self.spec.availability = availability;
        self
    }
    pub fn permission(mut self, permission: PermissionRequirement) -> Self {
        self.spec.permission = permission;
        self
    }
    pub fn arguments(mut self, arguments: Vec<ArgumentSpec>) -> Self {
        self.spec.arguments = arguments;
        self
    }
    pub fn shortcut(mut self, shortcut: Shortcut) -> Self {
        self.spec.shortcut = shortcut;
        self
    }
    pub fn build(self) -> CommandSpec {
        self.spec
    }
}
