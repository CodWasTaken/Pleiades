//! Live, full-screen Ratatui application loop.

use std::collections::BTreeSet;
use std::io::{self, IsTerminal, Read};
use std::path::PathBuf;
use std::time::Duration;

use crossterm::event::{Event, EventStream, KeyEventKind};
use futures_util::StreamExt;
use pleiades_agent_config::Config;
use pleiades_agent_core::conversation::Conversation;
use pleiades_agent_core::error::Error;
use pleiades_agent_engine::{AgentCommand, AgentMode, AgentRuntime};
use tokio::time;

use crate::state::{AppState, Effect};
use crate::terminal::TerminalGuard;
use crate::theme::Theme;
use crate::ui;

/// Terminal workspace application. Providers and tools live in the engine
/// runtime; this type owns only frontend state and channels.
pub struct TuiApp {
    config: Config,
    conversation: Conversation,
    provider_name: String,
    model_name: String,
    mode: AgentMode,
}

impl TuiApp {
    pub fn new(config: Config) -> Result<Self, Error> {
        let provider_name = config
            .core
            .default_provider
            .clone()
            .unwrap_or_else(|| "openai".to_string());
        let model_name = config
            .core
            .default_model
            .clone()
            .unwrap_or_else(|| "gpt-4o".to_string());
        let conversation = Conversation::new(format!("tui_{}", chrono::Utc::now().timestamp()));
        Ok(Self {
            config,
            conversation,
            provider_name,
            model_name,
            mode: AgentMode::Agent,
        })
    }

    /// Override the autonomous access boundary for this session.
    pub fn with_permission_mode(mut self, mode: &str) -> Self {
        self.mode = AgentMode::parse(mode);
        self
    }

    pub fn with_session(&mut self, session_id: &str) -> Result<(), Error> {
        self.conversation =
            pleiades_agent_engine::SessionStore::from_config(&self.config).load(session_id)?;
        Ok(())
    }

    pub fn conversation(&self) -> &Conversation {
        &self.conversation
    }

    pub fn provider_name(&self) -> &str {
        &self.provider_name
    }

    pub fn model_name(&self) -> &str {
        &self.model_name
    }

    /// Run the persistent asynchronous terminal event loop.
    pub async fn run(&mut self) -> Result<(), Error> {
        if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
            return self.run_headless().await;
        }

        let workspace = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let theme_name = self.config.core.theme.as_deref().unwrap_or("seven-sisters");
        let theme = Theme::load(theme_name).unwrap_or_default();
        let runtime = AgentRuntime::new(
            self.config.clone(),
            self.conversation.clone(),
            self.provider_name.clone(),
            self.model_name.clone(),
            self.mode,
        );
        let mut agent = runtime.spawn();
        let mut app = AppState::new(
            theme,
            workspace,
            self.provider_name.clone(),
            self.model_name.clone(),
            self.mode,
        );
        let mut providers = self
            .config
            .providers
            .keys()
            .cloned()
            .collect::<BTreeSet<_>>();
        providers.insert(self.provider_name.clone());
        let mut models = self
            .config
            .models
            .aliases
            .values()
            .cloned()
            .collect::<BTreeSet<_>>();
        models.insert(self.model_name.clone());
        if let Some(model) = &self.config.models.default {
            models.insert(model.clone());
        }
        let sessions = pleiades_agent_engine::SessionStore::from_config(&self.config)
            .list()
            .unwrap_or_default()
            .into_iter()
            .map(|session| session.id)
            .collect();
        let (file_sender, mut file_receiver) = tokio::sync::mpsc::channel(1);
        let file_workspace = app.workspace.clone();
        tokio::task::spawn_blocking(move || {
            let _ = file_sender.blocking_send(workspace_files(&file_workspace, 2_000));
        });
        app.set_picker_options(
            providers.into_iter().collect(),
            models.into_iter().collect(),
            Vec::new(),
            sessions,
        );
        let mut terminal = TerminalGuard::enter()?;
        terminal.terminal_mut().clear()?;
        let mut terminal_events = EventStream::new();
        let mut render_tick = time::interval(Duration::from_millis(50));
        render_tick.set_missed_tick_behavior(time::MissedTickBehavior::Skip);

        let mut quit = false;
        while !quit {
            tokio::select! {
                terminal_event = terminal_events.next() => {
                    match terminal_event {
                        Some(Ok(Event::Key(key))) if key.kind == KeyEventKind::Press => {
                            quit = apply_effects(app.handle_key(key), &agent.commands).await?;
                        }
                        Some(Ok(Event::Mouse(mouse))) => app.handle_mouse(mouse),
                        Some(Ok(Event::Paste(text))) => { app.composer.insert_str(text); }
                        Some(Ok(Event::Resize(_, _))) => {}
                        Some(Ok(_)) => {}
                        Some(Err(error)) => app.status = format!("Terminal input error: {error}"),
                        None => quit = true,
                    }
                }
                Some(event) = agent.events.recv() => app.apply_agent(event),
                Some(files) = file_receiver.recv() => app.file_options = files,
                _ = render_tick.tick() => {}
            }
            terminal
                .terminal_mut()
                .draw(|frame| ui::render(frame, &mut app))?;
        }

        let _ = agent.commands.send(AgentCommand::Shutdown).await;
        Ok(())
    }

    /// A deterministic pipe-friendly path used by shell automation. The
    /// interactive experience always uses the full-screen event loop above.
    async fn run_headless(&self) -> Result<(), Error> {
        println!(
            "P L E I A D E S · terminal agent · workspace {}",
            std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .display()
        );
        let mut input = String::new();
        io::stdin().read_to_string(&mut input)?;
        if input
            .lines()
            .all(|line| matches!(line.trim(), "" | "/exit" | "/quit"))
        {
            return Ok(());
        }
        Err(Error::invalid_input(
            "interactive agent input requires a terminal; use `pleiades chat` for piped prompts",
        ))
    }
}

fn workspace_files(root: &std::path::Path, limit: usize) -> Vec<String> {
    let mut directories = vec![root.to_path_buf()];
    let mut files = Vec::new();
    while let Some(directory) = directories.pop() {
        let Ok(entries) = std::fs::read_dir(directory) else {
            continue;
        };
        for entry in entries.flatten() {
            if files.len() >= limit {
                break;
            }
            let path = entry.path();
            let name = entry.file_name();
            let name = name.to_string_lossy();
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if file_type.is_symlink() {
                continue;
            }
            if file_type.is_dir() {
                if !matches!(name.as_ref(), ".git" | "target" | "node_modules" | ".next") {
                    directories.push(path);
                }
            } else if file_type.is_file() {
                files.push(
                    path.strip_prefix(root)
                        .unwrap_or(&path)
                        .to_string_lossy()
                        .to_string(),
                );
            }
        }
    }
    files.sort();
    files
}

async fn apply_effects(
    effects: Vec<Effect>,
    commands: &tokio::sync::mpsc::Sender<AgentCommand>,
) -> Result<bool, Error> {
    let mut quit = false;
    for effect in effects {
        match effect {
            Effect::Command(command) => commands
                .send(command)
                .await
                .map_err(|_| Error::internal("agent runtime stopped unexpectedly"))?,
            Effect::Quit => quit = true,
        }
    }
    Ok(quit)
}

#[cfg(test)]
mod tests {
    use super::TuiApp;
    use pleiades_agent_config::Config;
    use pleiades_agent_engine::AgentMode;

    #[test]
    fn permission_modes_map_to_runtime_boundaries() {
        assert_eq!(
            TuiApp::new(Config::default())
                .unwrap()
                .with_permission_mode("plan")
                .mode,
            AgentMode::Plan
        );
        assert_eq!(
            TuiApp::new(Config::default())
                .unwrap()
                .with_permission_mode("unrestricted")
                .mode,
            AgentMode::Unrestricted
        );
    }
}
