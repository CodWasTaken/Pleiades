//! The command registry: dispatch, lookup, suggestion, help, and palette.
//!
//! The [`CommandRegistry`] is the single source of truth for every user
//! command in Pleiades.  Slash commands, CLI subcommands, and palette
//! entries all live here; plugins, MCP servers, and custom user command
//! files extend it via [`CommandRegistry::register`].  Help text, the
//! command palette, and slash autocompletion are computed from the
//! registry rather than maintained separately.

use std::collections::HashMap;

use pleiades_agent_core::Error;
use thiserror::Error;

use crate::context::CommandContext;
use crate::handler::HandlerResult;
use crate::parser::{ParseError, tokenize};
use crate::result::{CommandResult, RenderableDocument};
use crate::spec::{CommandCategory, CommandSpec, CompletionSource, Shortcut};

/// Errors raised when registering a command.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RegistrationError {
    #[error("command path must have at least one segment")]
    EmptyPath,
    #[error("duplicate command path: `/{path}`")]
    DuplicatePath { path: String },
    #[error("alias `{alias}` already bound to `/{existing}`")]
    AliasCollision { alias: String, existing: String },
}

/// Errors raised when resolving a token stream against the registry.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ResolveError {
    #[error("unknown command `{0}`")]
    NotFound(String),
    #[error("`{0}` requires a subcommand; available: {1}")]
    NeedsSubcommand(String, String),
}

/// Aggregate error for [`CommandRegistry::dispatch`]: parse, resolution,
/// or handler failure.
#[derive(Debug, Error)]
pub enum DispatchError {
    #[error(transparent)]
    Parse(#[from] ParseError),
    #[error(transparent)]
    Resolve(#[from] ResolveError),
    #[error(transparent)]
    Handler(#[from] Error),
}

/// What kind of suggestion a [`Suggestion`] represents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SuggestionKind {
    /// A top-level (root) command path.
    Command,
    /// A child path of a parent command.
    Subcommand,
    /// An alias that resolves to a registered command.
    Alias,
    /// A positional argument to be completed by the frontend using the
    /// supplied completion source; the registry itself does not enumerate
    /// the candidate values.
    Argument {
        source: CompletionSource,
        name: String,
        description: String,
    },
}

/// One autocomplete suggestion produced by [`CommandRegistry::suggest`].
#[derive(Debug, Clone)]
pub struct Suggestion {
    /// Human-readable label, e.g. `provider use` or `p` (for an alias).
    pub label: String,
    /// Text the frontend should insert at the cursor when the user accepts
    /// this suggestion, e.g. `provider use ` (without leading slash).
    pub insertion: String,
    /// Human-readable description copied from the underlying command.
    pub description: String,
    /// Alias hint when this suggestion represents an alias mapping.
    pub alias_hint: Option<String>,
    /// Keyboard shortcut associated with the underlying command, if any.
    pub shortcut: Shortcut,
    /// The variety of suggestion.
    pub kind: SuggestionKind,
}

/// The registry.
#[derive(Default)]
pub struct CommandRegistry {
    specs: Vec<CommandSpec>,
    /// Maps canonical path (space-joined) to index in `specs`.
    path_index: HashMap<String, usize>,
    /// Maps alias (space-joined, no leading slash) to index in `specs`.
    alias_index: HashMap<String, usize>,
}

impl CommandRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a command spec.  Returns an error if the path is empty, the
    /// path is already registered, or any of the spec's aliases collide.
    pub fn register(&mut self, spec: CommandSpec) -> Result<(), RegistrationError> {
        if spec.path.is_empty() {
            return Err(RegistrationError::EmptyPath);
        }
        let canonical = spec.canonical();
        if self.path_index.contains_key(&canonical) {
            return Err(RegistrationError::DuplicatePath { path: canonical });
        }
        let incoming_aliases: Vec<String> = spec
            .aliases
            .iter()
            .map(|a| a.trim_start_matches('/').to_string())
            .collect();
        for alias in &incoming_aliases {
            if let Some(idx) = self.alias_index.get(alias) {
                return Err(RegistrationError::AliasCollision {
                    alias: alias.clone(),
                    existing: self.specs[*idx].canonical(),
                });
            }
        }
        let idx = self.specs.len();
        self.path_index.insert(canonical, idx);
        for alias in incoming_aliases {
            self.alias_index.insert(alias, idx);
        }
        self.specs.push(spec);
        Ok(())
    }

    /// Iterate over all registered commands in insertion order.
    pub fn iter(&self) -> impl Iterator<Item = &CommandSpec> {
        self.specs.iter()
    }

    /// All registered command specs, in insertion order.
    pub fn commands(&self) -> &[CommandSpec] {
        &self.specs
    }

    /// Count of registered commands.
    pub fn len(&self) -> usize {
        self.specs.len()
    }

    /// Is the registry empty?
    pub fn is_empty(&self) -> bool {
        self.specs.is_empty()
    }

    /// Lookup command by canonical path or alias (`"provider use"` or
    /// `"p"`), without the leading slash.  Aliases may be multi-segment
    /// (e.g. `"p u"` maps to `provider use`).
    pub fn get(&self, path_or_alias: &str) -> Option<&CommandSpec> {
        let key = path_or_alias.trim_start_matches('/');
        if let Some(idx) = self.path_index.get(key) {
            return Some(&self.specs[*idx]);
        }
        self.alias_index.get(key).map(|idx| &self.specs[*idx])
    }

    /// Lookup by tokens (path or alias), e.g. `["provider", "use"]` or
    /// `["p", "u"]`.
    pub fn find_path(&self, tokens: &[&str]) -> Option<&CommandSpec> {
        let joined = tokens.join(" ");
        self.get(&joined)
    }

    /// Find the deepest registered spec covering `tokens`, or `None` if
    /// none matches.  Aliases are honoured: a multi-segment alias that
    /// covers the leading tokens resolves just like a canonical path.
    pub fn resolve_deepest(&self, tokens: &[&str]) -> Option<&CommandSpec> {
        for n in (1..=tokens.len()).rev() {
            if let Some(spec) = self.find_path(&tokens[..n]) {
                if spec.path.len() == n {
                    return Some(spec);
                }
            }
        }
        None
    }

    /// Direct child commands of the given parent path, ordered by canonical
    /// name.  Grandchildren are not included.
    pub fn children(&self, parent: &[&str]) -> Vec<&CommandSpec> {
        let mut out: Vec<&CommandSpec> = self
            .specs
            .iter()
            .filter(|s| s.is_child_of(parent))
            .collect();
        out.sort_by_key(|c| c.canonical());
        out
    }

    /// Distinct categories present in the registry, in canonical order.
    pub fn categories(&self) -> Vec<CommandCategory> {
        let order = [
            CommandCategory::Help,
            CommandCategory::Workspace,
            CommandCategory::Provider,
            CommandCategory::Extension,
            CommandCategory::Memory,
            CommandCategory::Project,
            CommandCategory::Configuration,
            CommandCategory::History,
            CommandCategory::Verification,
            CommandCategory::Plugin,
            CommandCategory::Mcp,
            CommandCategory::Custom,
            CommandCategory::Hidden,
        ];
        order
            .into_iter()
            .filter(|c| self.specs.iter().any(|s| s.category == *c))
            .collect()
    }

    /// Palette filter: every non-hidden command whose path, aliases, or
    /// description matches `query` (case-insensitive substring), and whose
    /// availability permits the caller mode.
    pub fn filter(&self, query: &str, interactive: bool) -> Vec<&CommandSpec> {
        let q = query.trim().to_lowercase();
        let q = q.strip_prefix('/').unwrap_or(&q);
        let mut out: Vec<&CommandSpec> = self
            .specs
            .iter()
            .filter(|s| s.category != CommandCategory::Hidden)
            .filter(|s| s.availability.allows(interactive))
            .filter(|s| q.is_empty() || matches_query(s, q))
            .collect();
        out.sort_by(|a, b| {
            a.category
                .label()
                .cmp(b.category.label())
                .then_with(|| a.canonical().cmp(&b.canonical()))
        });
        out
    }

    /// Slash-command autocompletion.  `partial` may include a leading slash
    /// and a trailing partial token.  Returns ranked suggestions across
    /// commands, subcommands, and aliases.
    pub fn suggest(&self, partial: &str, interactive: bool) -> Vec<Suggestion> {
        let stripped = partial
            .trim()
            .strip_prefix('/')
            .unwrap_or_else(|| partial.trim());
        if stripped.is_empty() {
            return self.root_suggestions(interactive);
        }
        let (prefix, last) = split_partial(stripped);
        let mut suggestions: Vec<Suggestion> = Vec::new();
        for spec in self.specs.iter() {
            if spec.category == CommandCategory::Hidden {
                continue;
            }
            if !spec.availability.allows(interactive) {
                continue;
            }
            if let Some(s) = spec_match(spec, &prefix, last) {
                suggestions.push(s);
            }
            for alias in &spec.aliases {
                if let Some(s) = alias_match(spec, alias, &prefix, last) {
                    suggestions.push(s);
                }
            }
        }
        if suggestions.is_empty() {
            // Path fully resolved but mid-argument: surface argument slot so
            // the frontend knows which completion source to consult.
            if let Some(spec) = self.resolve_deepest(&prefix) {
                if let Some(arg_suggest) = argument_suggestions(spec, &prefix, last) {
                    suggestions.push(arg_suggest);
                }
            }
        }
        suggestions.sort_by(|a, b| a.label.cmp(&b.label));
        suggestions.dedup_by(|a, b| a.label == b.label && a.kind == b.kind);
        suggestions
    }

    fn root_suggestions(&self, interactive: bool) -> Vec<Suggestion> {
        let mut out: Vec<Suggestion> = self
            .specs
            .iter()
            .filter(|s| s.path.len() == 1 && s.category != CommandCategory::Hidden)
            .filter(|s| s.availability.allows(interactive))
            .map(|s| Suggestion {
                label: s.canonical(),
                insertion: s.canonical(),
                description: s.description.to_string(),
                alias_hint: s.aliases.first().map(|a| a.to_string()),
                shortcut: s.shortcut,
                kind: SuggestionKind::Command,
            })
            .collect();
        out.sort_by(|a, b| a.label.cmp(&b.label));
        out
    }

    /// Generate the help document.  If `category` is supplied, the document
    /// contains only that category's commands.  `interactive` controls
    /// availability filtering (e.g. headless-only commands are hidden in
    /// the live workspace).
    pub fn help_document(
        &self,
        category: Option<CommandCategory>,
        interactive: bool,
    ) -> RenderableDocument {
        let title = match category {
            Some(c) => format!("Pleiades Commands · {}", c.label()),
            None => "Pleiades Commands".to_string(),
        };
        let mut doc = RenderableDocument::new(title);
        for c in self.categories() {
            if c == CommandCategory::Hidden {
                continue;
            }
            if let Some(filter) = category {
                if filter != c {
                    continue;
                }
            }
            let mut body = String::new();
            for spec in self.specs.iter().filter(|s| s.category == c) {
                if !spec.availability.allows(interactive) {
                    continue;
                }
                let aliases = if spec.aliases.is_empty() {
                    String::new()
                } else {
                    format!(" [aliases: {}]", spec.aliases.join(", "))
                };
                let usage = if spec.usage.is_empty() {
                    String::new()
                } else {
                    format!("\n      usage: {}", spec.usage)
                };
                let shortcut = match spec.shortcut {
                    Shortcut::None => String::new(),
                    _ => format!(" ({})", spec.shortcut.label()),
                };
                body.push_str(&format!(
                    "  /{}{} — {}{}{}\n",
                    spec.canonical(),
                    aliases,
                    spec.description,
                    shortcut,
                    usage
                ));
            }
            if body.is_empty() {
                continue;
            }
            doc.push_section(c.label(), body.trim_end().to_string());
        }
        doc
    }

    /// Parse `input` (with or without leading slash), resolve the deepest
    /// matching spec, validate arguments, and invoke its handler.  On
    /// ambiguity (path matches no spec but children exist), returns
    /// [`ResolveError::NeedsSubcommand`] with the available subcommands.
    pub async fn dispatch(
        &self,
        input: &str,
        ctx: &CommandContext,
        interactive: bool,
    ) -> Result<CommandResult, DispatchError> {
        let tokens = tokenize(input).map_err(DispatchError::Parse)?;
        if tokens.is_empty() {
            return Err(DispatchError::Parse(ParseError::Empty));
        }
        let tokens_str: Vec<&str> = tokens.iter().map(|s| s.as_str()).collect();
        let spec = match self.resolve_deepest(&tokens_str) {
            Some(s) => s,
            None => {
                // If the leading tokens form a parent of registered
                // subcommands, surface them so the UI can guide the user.
                let parent: Vec<&str> = tokens.iter().map(|s| s.as_str()).collect();
                let kids = self.children(&parent);
                if !kids.is_empty() {
                    let names = kids
                        .iter()
                        .map(|c| format!("/{}", c.canonical()))
                        .collect::<Vec<_>>()
                        .join(", ");
                    return Err(DispatchError::Resolve(ResolveError::NeedsSubcommand(
                        tokens.join(" "),
                        names,
                    )));
                }
                return Err(DispatchError::Resolve(ResolveError::NotFound(
                    tokens.join(" "),
                )));
            }
        };
        if !spec.availability.allows(interactive) {
            return Err(DispatchError::Resolve(ResolveError::NotFound(
                spec.canonical(),
            )));
        }
        let args: Vec<String> = tokens
            .get(spec.path.len()..)
            .map(|slice| slice.to_vec())
            .unwrap_or_default();
        let result: HandlerResult = spec.handler.handle(ctx, &args).await;
        result.map_err(DispatchError::Handler)
    }
}

impl std::fmt::Debug for CommandRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommandRegistry")
            .field("commands", &self.specs.len())
            .finish_non_exhaustive()
    }
}

fn matches_query(spec: &CommandSpec, q: &str) -> bool {
    if spec.canonical().to_lowercase().contains(q) {
        return true;
    }
    if spec.description.to_lowercase().contains(q) {
        return true;
    }
    spec.aliases.iter().any(|a| a.to_lowercase().contains(q))
}

/// Split `partial` into the full-token prefix and the (possibly empty)
/// trailing partial token.  A trailing whitespace means the user has
/// finished typing the last token and started the next (empty) one.
fn split_partial(partial: &str) -> (Vec<&str>, &str) {
    let raw = partial.trim_start();
    let rstripped = raw.trim_end();
    if raw.len() > rstripped.len() {
        // Trailing whitespace present: everything typed so far is a
        // complete command path, and the partial token is empty.
        let prefix: Vec<&str> = rstripped.split_whitespace().collect();
        return (prefix, "");
    }
    if let Some(idx) = rstripped.rfind(char::is_whitespace) {
        let head = &rstripped[..idx];
        let last = rstripped[idx..].trim_start();
        let prefix: Vec<&str> = head.split_whitespace().collect();
        (prefix, last)
    } else {
        (Vec::new(), rstripped)
    }
}

fn spec_match(spec: &CommandSpec, prefix: &[&str], last: &str) -> Option<Suggestion> {
    let path = &spec.path;
    if prefix.len() > path.len() {
        return None;
    }
    for (i, seg) in prefix.iter().enumerate() {
        if path[i] != *seg {
            return None;
        }
    }
    let next_idx = prefix.len();
    let kind = if path.len() > prefix.len() {
        SuggestionKind::Subcommand
    } else {
        SuggestionKind::Command
    };
    if next_idx == path.len() {
        if !last.is_empty() {
            return None;
        }
        return Some(Suggestion {
            label: spec.canonical(),
            insertion: spec.canonical(),
            description: spec.description.to_string(),
            alias_hint: spec.aliases.first().map(|a| a.to_string()),
            shortcut: spec.shortcut,
            kind,
        });
    }
    let next_seg = path[next_idx];
    if !next_seg.starts_with(last) {
        return None;
    }
    Some(Suggestion {
        label: spec.canonical(),
        insertion: spec.canonical(),
        description: spec.description.to_string(),
        alias_hint: spec.aliases.first().map(|a| a.to_string()),
        shortcut: spec.shortcut,
        kind,
    })
}

fn alias_match(spec: &CommandSpec, alias: &str, prefix: &[&str], last: &str) -> Option<Suggestion> {
    let alias_tokens: Vec<&str> = alias.trim_start_matches('/').split_whitespace().collect();
    if alias_tokens.is_empty() {
        return None;
    }
    if prefix.len() > alias_tokens.len() {
        return None;
    }
    for (i, seg) in prefix.iter().enumerate() {
        if alias_tokens[i] != *seg {
            return None;
        }
    }
    if alias_tokens.len() == prefix.len() {
        if !last.is_empty() {
            return None;
        }
        return Some(Suggestion {
            label: alias.to_string(),
            insertion: spec.canonical(),
            description: spec.description.to_string(),
            alias_hint: Some(alias.to_string()),
            shortcut: spec.shortcut,
            kind: SuggestionKind::Alias,
        });
    }
    let next_seg = alias_tokens[prefix.len()];
    if !next_seg.starts_with(last) {
        return None;
    }
    Some(Suggestion {
        label: alias.to_string(),
        insertion: spec.canonical(),
        description: spec.description.to_string(),
        alias_hint: Some(alias.to_string()),
        shortcut: spec.shortcut,
        kind: SuggestionKind::Alias,
    })
}

fn argument_suggestions(spec: &CommandSpec, prefix: &[&str], _last: &str) -> Option<Suggestion> {
    let arg_index = prefix.len().saturating_sub(spec.path.len());
    let arg = spec.arguments.get(arg_index)?;
    if arg.completer == CompletionSource::None {
        return None;
    }
    Some(Suggestion {
        label: format!("<{}>", arg.name),
        insertion: String::new(),
        description: arg.description.to_string(),
        alias_hint: None,
        shortcut: Shortcut::None,
        kind: SuggestionKind::Argument {
            source: arg.completer,
            name: arg.name.to_string(),
            description: arg.description.to_string(),
        },
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::handler::CommandHandler;
    use crate::spec::CommandSpec;
    use async_trait::async_trait;

    struct Echo;
    #[async_trait]
    impl CommandHandler for Echo {
        async fn handle(&self, _: &crate::context::CommandContext, _: &[String]) -> HandlerResult {
            Ok(CommandResult::Noop)
        }
    }

    fn spec(
        path: Vec<&'static str>,
        aliases: Vec<&'static str>,
        category: CommandCategory,
    ) -> CommandSpec {
        let path_string = path
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .join(" ");
        CommandSpec::builder(path, "demo", Echo)
            .aliases(aliases)
            .category(category)
            .usage(format!("usage: /{}", path_string))
            .build()
    }

    #[test]
    fn registers_root_and_nested() {
        let mut r = CommandRegistry::new();
        r.register(spec(vec!["provider"], vec!["p"], CommandCategory::Provider))
            .unwrap();
        r.register(spec(
            vec!["provider", "use"],
            vec![],
            CommandCategory::Provider,
        ))
        .unwrap();
        r.register(spec(vec!["model"], vec!["m"], CommandCategory::Provider))
            .unwrap();
        assert_eq!(r.len(), 3);
        assert!(r.get("provider").is_some());
        assert!(r.get("provider use").is_some());
        assert!(r.get("p").is_some());
        assert!(r.get("m").is_some());
        assert!(r.get("nope").is_none());
    }

    #[test]
    fn rejects_duplicate_path() {
        let mut r = CommandRegistry::new();
        r.register(spec(vec!["provider"], vec![], CommandCategory::Provider))
            .unwrap();
        let err = r
            .register(spec(vec!["provider"], vec![], CommandCategory::Provider))
            .unwrap_err();
        assert_eq!(
            err,
            RegistrationError::DuplicatePath {
                path: "provider".into()
            }
        );
    }

    #[test]
    fn rejects_alias_collision() {
        let mut r = CommandRegistry::new();
        r.register(spec(vec!["provider"], vec!["p"], CommandCategory::Provider))
            .unwrap();
        r.register(spec(
            vec!["provider", "use"],
            vec!["p u"],
            CommandCategory::Provider,
        ))
        .unwrap();
        let err = r
            .register(spec(vec!["model"], vec!["p"], CommandCategory::Provider))
            .unwrap_err();
        assert!(matches!(err, RegistrationError::AliasCollision { .. }));
    }

    #[test]
    fn rejects_empty_path() {
        let mut r = CommandRegistry::new();
        assert!(matches!(
            r.register(spec(vec![], vec![], CommandCategory::Workspace))
                .err(),
            Some(RegistrationError::EmptyPath)
        ));
    }

    #[test]
    fn children_lists_direct_children() {
        let mut r = CommandRegistry::new();
        r.register(spec(vec!["provider"], vec![], CommandCategory::Provider))
            .unwrap();
        r.register(spec(
            vec!["provider", "use"],
            vec![],
            CommandCategory::Provider,
        ))
        .unwrap();
        r.register(spec(
            vec!["provider", "info"],
            vec![],
            CommandCategory::Provider,
        ))
        .unwrap();
        r.register(spec(
            vec!["provider", "use", "openai"],
            vec![],
            CommandCategory::Provider,
        ))
        .unwrap();
        let kids = r.children(&["provider"]);
        let names: Vec<String> = kids.iter().map(|c| c.canonical()).collect();
        assert_eq!(names, vec!["provider info", "provider use"]);
    }

    #[test]
    fn suggest_root_when_empty() {
        let mut r = CommandRegistry::new();
        r.register(spec(vec!["provider"], vec!["p"], CommandCategory::Provider))
            .unwrap();
        r.register(spec(vec!["help"], vec![], CommandCategory::Help))
            .unwrap();
        let s = r.suggest("", true);
        let labels: Vec<&str> = s.iter().map(|x| x.label.as_str()).collect();
        assert_eq!(labels, vec!["help", "provider"]);
    }

    #[test]
    fn suggest_completes_subcommands() {
        let mut r = CommandRegistry::new();
        r.register(spec(vec!["provider"], vec![], CommandCategory::Provider))
            .unwrap();
        r.register(spec(
            vec!["provider", "use"],
            vec![],
            CommandCategory::Provider,
        ))
        .unwrap();
        r.register(spec(
            vec!["provider", "info"],
            vec![],
            CommandCategory::Provider,
        ))
        .unwrap();
        let s = r.suggest("/provider ", true);
        let labels: Vec<&str> = s.iter().map(|x| x.label.as_str()).collect();
        assert!(labels.contains(&"provider use"));
        assert!(labels.contains(&"provider info"));
    }

    #[test]
    fn suggest_filters_by_partial_token() {
        let mut r = CommandRegistry::new();
        r.register(spec(
            vec!["provider", "use"],
            vec![],
            CommandCategory::Provider,
        ))
        .unwrap();
        r.register(spec(
            vec!["provider", "info"],
            vec![],
            CommandCategory::Provider,
        ))
        .unwrap();
        let s = r.suggest("/provider u", true);
        assert_eq!(s.len(), 1);
        assert_eq!(s[0].label, "provider use");
    }

    #[test]
    fn suggest_supports_aliases() {
        let mut r = CommandRegistry::new();
        r.register(spec(vec!["provider"], vec!["p"], CommandCategory::Provider))
            .unwrap();
        r.register(spec(
            vec!["provider", "use"],
            vec!["p u"],
            CommandCategory::Provider,
        ))
        .unwrap();
        let s = r.suggest("/p ", true);
        assert!(s.iter().any(|x| x.kind == SuggestionKind::Subcommand));
        let s = r.suggest("/p u", true);
        assert!(s.iter().any(|x| x.label == "p u"));
    }

    #[test]
    fn filter_substring_case_insensitive() {
        let mut r = CommandRegistry::new();
        r.register(spec(vec!["provider"], vec![], CommandCategory::Provider))
            .unwrap();
        r.register(spec(vec!["help"], vec![], CommandCategory::Help))
            .unwrap();
        let f = r.filter("prov", true);
        let names: Vec<String> = f.iter().map(|c| c.canonical()).collect();
        assert_eq!(names, vec!["provider"]);
        let f = r.filter("HELP", true);
        assert_eq!(
            f.iter().map(|c| c.canonical()).collect::<Vec<_>>(),
            vec!["help"]
        );
    }

    #[test]
    fn help_document_contains_registered_commands() {
        let mut r = CommandRegistry::new();
        r.register(spec(vec!["provider"], vec!["p"], CommandCategory::Provider))
            .unwrap();
        r.register(spec(vec!["help"], vec![], CommandCategory::Help))
            .unwrap();
        let doc = r.help_document(None, true);
        let joined = format!(
            "{}\n{}",
            doc.title,
            doc.sections
                .iter()
                .map(|s| s.body.clone())
                .collect::<Vec<_>>()
                .join("\n")
        );
        assert!(joined.contains("/provider"));
        assert!(joined.contains("[aliases: p]"));
        assert!(joined.contains("/help"));
    }

    #[test]
    fn hidden_excluded_from_suggest_and_filter() {
        let mut r = CommandRegistry::new();
        r.register(spec(vec!["palette"], vec![], CommandCategory::Hidden))
            .unwrap();
        assert!(r.suggest("", true).is_empty());
        assert!(r.filter("", true).is_empty());
    }

    #[tokio::test]
    async fn dispatch_resolves_and_invokes() {
        let mut r = CommandRegistry::new();
        r.register(spec(vec!["provider"], vec!["p"], CommandCategory::Provider))
            .unwrap();
        let ctx = crate::context::CommandContextBuilder::default().build();
        let res = r.dispatch("/provider", &ctx, true).await.unwrap();
        assert!(matches!(res, CommandResult::Noop));
        // alias also resolves.
        let res = r.dispatch("/p", &ctx, true).await.unwrap();
        assert!(matches!(res, CommandResult::Noop));
        // unknown.
        let err = r.dispatch("/nope", &ctx, true).await.unwrap_err();
        assert!(matches!(
            err,
            DispatchError::Resolve(ResolveError::NotFound(_))
        ));
    }

    #[tokio::test]
    async fn dispatch_needs_subcommand_when_only_children_exist() {
        let mut r = CommandRegistry::new();
        r.register(spec(
            vec!["provider", "use"],
            vec![],
            CommandCategory::Provider,
        ))
        .unwrap();
        let ctx = crate::context::CommandContextBuilder::default().build();
        let err = r.dispatch("/provider", &ctx, true).await.unwrap_err();
        match err {
            DispatchError::Resolve(ResolveError::NeedsSubcommand(_, kids)) => {
                assert!(kids.contains("/provider use"));
            }
            other => panic!("unexpected: {:?}", other),
        }
    }

    #[test]
    fn resolve_deepest_prefers_longest_path() {
        let mut r = CommandRegistry::new();
        r.register(spec(vec!["provider"], vec![], CommandCategory::Provider))
            .unwrap();
        r.register(spec(
            vec!["provider", "use"],
            vec![],
            CommandCategory::Provider,
        ))
        .unwrap();
        let resolved = r.resolve_deepest(&["provider", "use", "openai"]).unwrap();
        assert_eq!(resolved.canonical(), "provider use");
    }

    #[test]
    fn resolve_deepest_honours_alias_paths() {
        let mut r = CommandRegistry::new();
        r.register(spec(
            vec!["provider", "use"],
            vec!["p u"],
            CommandCategory::Provider,
        ))
        .unwrap();
        let resolved = r.resolve_deepest(&["p", "u", "openai"]).unwrap();
        assert_eq!(resolved.canonical(), "provider use");
    }

    #[test]
    fn split_partial_basic() {
        assert_eq!(split_partial("provider"), (Vec::<&str>::new(), "provider"));
        assert_eq!(split_partial("provider use"), (vec!["provider"], "use"));
        assert_eq!(
            split_partial("provider use "),
            (vec!["provider", "use"], "")
        );
        assert_eq!(split_partial("provider "), (vec!["provider"], ""));
    }
}
