use std::io::{self, Write};

use pleiades_agent_config::Config;
use pleiades_agent_core::conversation::Conversation;
use pleiades_agent_core::error::Error;
use pleiades_agent_core::provider::StreamEvent;
use pleiades_agent_engine::Engine;
use pleiades_agent_engine::session::SessionStore;

use crate::input::{LineEditor, ReadOutcome};
use crate::render::{Spinner, TerminalRenderer};

/// Terminal UI application.
pub struct TuiApp {
    config: Config,
    renderer: TerminalRenderer,
    conversation: Conversation,
    session_store: SessionStore,
    provider_name: String,
    model_name: String,
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

        let session_store = SessionStore::from_config(&config);
        let conversation = Conversation::new(format!("tui_{}", chrono::Utc::now().timestamp()));

        Ok(Self {
            config,
            renderer: TerminalRenderer::new(),
            conversation,
            session_store,
            provider_name,
            model_name,
        })
    }

    pub fn with_session(&mut self, session_id: &str) -> Result<(), Error> {
        self.conversation = self.session_store.load(session_id)?;
        Ok(())
    }

    pub fn renderer(&self) -> &TerminalRenderer {
        &self.renderer
    }

    pub fn conversation(&self) -> &Conversation {
        &self.conversation
    }

    pub fn conversation_mut(&mut self) -> &mut Conversation {
        &mut self.conversation
    }

    pub fn provider_name(&self) -> &str {
        &self.provider_name
    }

    pub fn model_name(&self) -> &str {
        &self.model_name
    }

    /// Run the interactive TUI session.
    pub async fn run(&mut self) -> Result<(), Error> {
        let mut engine = self.build_engine()?;

        let mut rl = LineEditor::new(
            "\x1b[1;32m>>\x1b[0m ".to_string(),
            vec![
                "/help".into(),
                "/clear".into(),
                "/save".into(),
                "/load".into(),
                "/model".into(),
                "/provider".into(),
                "/info".into(),
                "/tokens".into(),
                "/export".into(),
                "/exit".into(),
                "/quit".into(),
            ],
        );

        self.print_welcome();

        loop {
            match rl.read_line() {
                Ok(ReadOutcome::Submit(input)) => {
                    let trimmed = input.trim();
                    if trimmed.is_empty() {
                        continue;
                    }

                    rl.push_history(trimmed);

                    if trimmed.starts_with('/') {
                        let should_continue =
                            self.handle_command(&mut engine, trimmed, &mut rl).await;
                        if !should_continue {
                            break;
                        }
                        continue;
                    }

                    self.handle_user_input(&mut engine, trimmed).await?;
                }
                Ok(ReadOutcome::Cancel) => {
                    continue;
                }
                Ok(ReadOutcome::Exit) => {
                    break;
                }
                Err(e) => {
                    eprintln!("Input error: {e}");
                    break;
                }
            }
        }

        self.save_on_exit(&mut engine);
        Ok(())
    }

    fn build_engine(&self) -> Result<Engine, Error> {
        let mut engine = Engine::new(self.config.clone());

        for (name, pc) in &self.config.providers {
            let api_key = pc.api_key.as_deref().unwrap_or("");
            let base_url = pc.base_url.as_deref().unwrap_or("");
            if name == "openai-subscription" {
                engine.register_provider(Box::new(
                    pleiades_agent_providers::codex::CodexCliProvider::new(),
                ));
                continue;
            }
            if api_key.is_empty() {
                continue;
            }
            let provider: Box<dyn pleiades_agent_core::Provider> = match name.as_str() {
                "anthropic" => {
                    if base_url.is_empty() {
                        Box::new(pleiades_agent_providers::anthropic::AnthropicProvider::new(
                            api_key.to_string(),
                        ))
                    } else {
                        Box::new(
                            pleiades_agent_providers::anthropic::AnthropicProvider::with_base_url(
                                api_key.to_string(),
                                base_url.to_string(),
                            ),
                        )
                    }
                }
                "openai" => {
                    if base_url.is_empty() {
                        Box::new(pleiades_agent_providers::openai::OpenAIProvider::new(
                            api_key.to_string(),
                        ))
                    } else {
                        Box::new(
                            pleiades_agent_providers::openai::OpenAIProvider::with_base_url(
                                api_key.to_string(),
                                base_url.to_string(),
                            ),
                        )
                    }
                }
                _ => {
                    let model = self.model_name.clone();
                    Box::new(
                        pleiades_agent_providers::openai_compat::OpenAICompatibleProvider::new(
                            name,
                            name,
                            api_key.to_string(),
                            base_url.to_string(),
                            model,
                        ),
                    )
                }
            };
            engine.register_provider(provider);
        }

        let tools_to_register: Vec<Box<dyn pleiades_agent_core::Tool>> = vec![
            Box::new(pleiades_agent_tools::read::ReadTool),
            Box::new(pleiades_agent_tools::write::WriteTool),
            Box::new(pleiades_agent_tools::edit::EditTool),
            Box::new(pleiades_agent_tools::bash::BashTool),
            Box::new(pleiades_agent_tools::glob_tool::GlobTool),
            Box::new(pleiades_agent_tools::grep_tool::GrepTool),
            Box::new(pleiades_agent_tools::diff::DiffTool),
            Box::new(pleiades_agent_tools::search::SearchTool::new()),
            Box::new(pleiades_agent_tools::fetch::FetchTool::new()),
        ];
        for t in tools_to_register {
            engine.register_tool(t);
        }

        Ok(engine)
    }

    fn print_welcome(&self) {
        let stdout = io::stdout();
        let mut out = stdout.lock();
        let _ = writeln!(
            out,
            "\x1b[1;36m╭──────────────────────────────────────────╮\x1b[0m"
        );
        let _ = writeln!(
            out,
            "\x1b[1;36m│\x1b[0m  \x1b[1;33mPleiades\x1b[0m - Terminal AI Assistant      \x1b[1;36m│\x1b[0m"
        );
        let _ = writeln!(
            out,
            "\x1b[1;36m│\x1b[0m  Model: {:<31} \x1b[1;36m│\x1b[0m",
            self.model_name
        );
        let _ = writeln!(
            out,
            "\x1b[1;36m│\x1b[0m  Provider: {:<28} \x1b[1;36m│\x1b[0m",
            self.provider_name
        );
        let _ = writeln!(
            out,
            "\x1b[1;36m│\x1b[0m  Messages: {:<26} \x1b[1;36m│\x1b[0m",
            self.conversation.len()
        );
        let _ = writeln!(
            out,
            "\x1b[1;36m│\x1b[0m  Type \x1b[1;32m/help\x1b[0m for commands              \x1b[1;36m│\x1b[0m"
        );
        let _ = writeln!(
            out,
            "\x1b[1;36m╰──────────────────────────────────────────╯\x1b[0m"
        );
        let _ = writeln!(out);
    }

    async fn handle_command(
        &mut self,
        engine: &mut Engine,
        input: &str,
        _rl: &mut LineEditor,
    ) -> bool {
        let parts: Vec<&str> = input.splitn(2, ' ').collect();
        let cmd = parts[0];
        let arg = parts.get(1).copied().unwrap_or("");

        match cmd {
            "/help" => {
                println!("\x1b[1;33mCommands:\x1b[0m");
                println!("  \x1b[1;32m/help\x1b[0m             Show this help message");
                println!("  \x1b[1;32m/clear\x1b[0m            Clear the conversation");
                println!("  \x1b[1;32m/save [name]\x1b[0m      Save the current session");
                println!("  \x1b[1;32m/load <id>\x1b[0m        Load a session by ID");
                println!("  \x1b[1;32m/model <name>\x1b[0m     Switch model");
                println!("  \x1b[1;32m/provider <name>\x1b[0m  Switch provider");
                println!("  \x1b[1;32m/info\x1b[0m             Show session info");
                println!("  \x1b[1;32m/tokens\x1b[0m           Show estimated token count");
                println!("  \x1b[1;32m/export [fmt]\x1b[0m     Export session (markdown/json)");
                println!("  \x1b[1;32m/exit\x1b[0m             Exit");
            }
            "/clear" => {
                self.conversation.clear();
                println!("\x1b[1;32m✓\x1b[0m Conversation cleared");
            }
            "/save" => {
                let title = if arg.is_empty() {
                    let first = self
                        .conversation
                        .messages
                        .first()
                        .map(|m| m.text_content())
                        .unwrap_or_default();
                    let preview: String = first.chars().take(40).collect();
                    if preview.is_empty() {
                        "Untitled".to_string()
                    } else {
                        preview
                    }
                } else {
                    arg.to_string()
                };
                self.conversation.metadata.title = Some(title);
                match self.session_store.save(&self.conversation) {
                    Ok(_) => println!("\x1b[1;32m✓\x1b[0m Session saved: {}", self.conversation.id),
                    Err(e) => eprintln!("\x1b[1;31m✗\x1b[0m Save failed: {e}"),
                }
            }
            "/load" => {
                if arg.is_empty() {
                    eprintln!("Usage: /load <session_id>");
                    return true;
                }
                match self.session_store.load(arg) {
                    Ok(conv) => {
                        self.conversation = conv;
                        println!(
                            "\x1b[1;32m✓\x1b[0m Session '{arg}' loaded ({} messages)",
                            self.conversation.len()
                        );
                        self.print_welcome();
                    }
                    Err(e) => eprintln!("\x1b[1;31m✗\x1b[0m Load failed: {e}"),
                }
            }
            "/model" => {
                if arg.is_empty() {
                    println!("Current model: \x1b[1;33m{}\x1b[0m", self.model_name);
                } else {
                    self.model_name = arg.to_string();
                    println!(
                        "\x1b[1;32m✓\x1b[0m Switched to model: \x1b[1;33m{}\x1b[0m",
                        self.model_name
                    );
                    *engine = match self.build_engine() {
                        Ok(e) => e,
                        Err(e) => {
                            eprintln!("\x1b[1;31m✗\x1b[0m Failed to rebuild engine: {e}");
                            return true;
                        }
                    };
                }
            }
            "/provider" => {
                if arg.is_empty() {
                    println!("Current provider: \x1b[1;33m{}\x1b[0m", self.provider_name);
                } else {
                    self.provider_name = arg.to_string();
                    println!(
                        "\x1b[1;32m✓\x1b[0m Switched to provider: \x1b[1;33m{}\x1b[0m",
                        self.provider_name
                    );
                    *engine = match self.build_engine() {
                        Ok(e) => e,
                        Err(e) => {
                            eprintln!("\x1b[1;31m✗\x1b[0m Failed to rebuild engine: {e}");
                            return true;
                        }
                    };
                }
            }
            "/info" => {
                let tokens = self.conversation.estimated_tokens();
                println!("\x1b[1;33mSession Info:\x1b[0m");
                println!("  ID:       {}", self.conversation.id);
                if let Some(ref title) = self.conversation.metadata.title {
                    println!("  Title:    {title}");
                }
                println!("  Messages: {}", self.conversation.len());
                println!("  Tokens:   ~{tokens}");
                println!("  Model:    {}", self.model_name);
                println!("  Provider: {}", self.provider_name);
            }
            "/tokens" => {
                let tokens = self.conversation.estimated_tokens();
                println!("Estimated tokens: \x1b[1;33m{tokens}\x1b[0m");
            }
            "/export" => {
                let fmt = if arg.is_empty() { "markdown" } else { arg };
                let result = match fmt {
                    "json" => self.session_store.export_json(&self.conversation.id),
                    _ => self.session_store.export_markdown(&self.conversation.id),
                };
                match result {
                    Ok(content) => {
                        let ext = if fmt == "json" { "json" } else { "md" };
                        let path = format!("{}.{ext}", self.conversation.id);
                        std::fs::write(&path, &content).ok();
                        println!("\x1b[1;32m✓\x1b[0m Exported to {path}");
                    }
                    Err(e) => eprintln!("\x1b[1;31m✗\x1b[0m Export failed: {e}"),
                }
            }
            "/exit" | "/quit" => return false,
            _ => eprintln!("Unknown command: {cmd}. Type /help for available commands."),
        }
        true
    }

    async fn handle_user_input(&mut self, engine: &mut Engine, input: &str) -> Result<(), Error> {
        let user_msg = pleiades_agent_core::conversation::Message::user(input);
        self.conversation.add_message(user_msg);

        let max_iterations = self.config.agent.max_tool_iterations;

        for iteration in 0..max_iterations {
            println!("\x1b[1;36m─── response ───\x1b[0m");

            let (text_response, tool_calls, had_error) = self.stream_response(engine).await?;

            if had_error {
                return Ok(());
            }

            if tool_calls.is_empty() {
                if !text_response.is_empty() {
                    let assistant_msg =
                        pleiades_agent_core::conversation::Message::assistant(text_response);
                    self.conversation.add_message(assistant_msg);
                }
                break;
            }

            let mut content_blocks = Vec::new();
            if !text_response.is_empty() {
                content_blocks.push(pleiades_agent_core::conversation::ContentBlock::Text(
                    text_response,
                ));
            }
            for tc in &tool_calls {
                content_blocks.push(pleiades_agent_core::conversation::ContentBlock::ToolUse {
                    id: tc.id.clone(),
                    name: tc.name.clone(),
                    input: tc.input.clone(),
                });
            }

            let assistant_msg = pleiades_agent_core::conversation::Message {
                role: pleiades_agent_core::conversation::MessageRole::Assistant,
                content: content_blocks,
                reasoning: None,
                metadata: None,
            };
            self.conversation.add_message(assistant_msg);

            println!(
                "\n\x1b[1;33m─── executing {} tool(s) ───\x1b[0m",
                tool_calls.len()
            );

            for tc in &tool_calls {
                let allowed = self.check_permission(tc).await;
                if !allowed {
                    self.conversation
                        .add_message(pleiades_agent_core::conversation::Message {
                            role: pleiades_agent_core::conversation::MessageRole::Tool,
                            content: vec![
                                pleiades_agent_core::conversation::ContentBlock::ToolResult {
                                    id: tc.id.clone(),
                                    content: "Tool use blocked by user".to_string(),
                                    is_error: true,
                                },
                            ],
                            reasoning: None,
                            metadata: None,
                        });
                    println!("\x1b[1;33m  ⛔ {} blocked\x1b[0m", tc.name);
                    continue;
                }

                println!(
                    "\x1b[1;32m  🔧 {} ({})...\x1b[0m",
                    tc.name,
                    &tc.id[..tc.id.len().min(8)]
                );
                match engine.execute_tool(&tc.name, tc.input.clone()).await {
                    Ok(result) => {
                        let content = if result.content.len() > 2000 {
                            format!(
                                "{}...(truncated, {} chars)",
                                &result.content[..2000],
                                result.content.len()
                            )
                        } else {
                            result.content.clone()
                        };
                        self.conversation
                            .add_message(pleiades_agent_core::conversation::Message {
                                role: pleiades_agent_core::conversation::MessageRole::Tool,
                                content: vec![
                                    pleiades_agent_core::conversation::ContentBlock::ToolResult {
                                        id: tc.id.clone(),
                                        content,
                                        is_error: !result.success,
                                    },
                                ],
                                reasoning: None,
                                metadata: None,
                            });
                        if result.success {
                            println!("\x1b[1;32m  ✓ {} completed\x1b[0m", tc.name);
                        } else {
                            println!(
                                "\x1b[1;31m  ✗ {} failed: {}\x1b[0m",
                                tc.name,
                                result.error.unwrap_or_default()
                            );
                        }
                    }
                    Err(e) => {
                        self.conversation
                            .add_message(pleiades_agent_core::conversation::Message {
                                role: pleiades_agent_core::conversation::MessageRole::Tool,
                                content: vec![
                                    pleiades_agent_core::conversation::ContentBlock::ToolResult {
                                        id: tc.id.clone(),
                                        content: format!("Error: {e}"),
                                        is_error: true,
                                    },
                                ],
                                reasoning: None,
                                metadata: None,
                            });
                        println!("\x1b[1;31m  ✗ {} error: {e}\x1b[0m", tc.name);
                    }
                }
            }

            self.session_store.save(&self.conversation).ok();

            if iteration == max_iterations - 1 {
                println!("\x1b[1;33m⚠ Max tool iterations ({max_iterations}) reached\x1b[0m");
            }

            println!();
        }

        self.session_store.save(&self.conversation).ok();
        Ok(())
    }

    async fn stream_response(
        &mut self,
        engine: &mut Engine,
    ) -> Result<(String, Vec<ToolCallInfo>, bool), Error> {
        let mut text_response = String::new();
        let mut tool_calls: Vec<ToolCallInfo> = Vec::new();
        let mut stream_state = crate::render::MarkdownStreamState::new();
        let mut spinner = Spinner::new();

        let theme = *self.renderer.color_theme();
        let mut stdout = io::stdout();

        match engine
            .chat_stream(&mut self.conversation, &self.provider_name)
            .await
        {
            Ok(mut rx) => {
                while let Some(event) = rx.recv().await {
                    match event {
                        StreamEvent::Token(token) => {
                            text_response.push_str(&token);
                            if let Some(rendered) = stream_state.push(&self.renderer, &token) {
                                let _ = write!(stdout, "{rendered}");
                                let _ = stdout.flush();
                            }
                        }
                        StreamEvent::ReasoningToken(token) => {
                            let _ = write!(stdout, "\x1b[2m{token}\x1b[0m");
                            let _ = stdout.flush();
                        }
                        StreamEvent::ToolCall { id, name, input } => {
                            tool_calls.push(ToolCallInfo { id, name, input });
                        }
                        StreamEvent::Done { .. } => {
                            if let Some(remaining) = stream_state.flush(&self.renderer) {
                                let _ = write!(stdout, "{remaining}");
                            }
                            let _ = writeln!(stdout);
                            break;
                        }
                        StreamEvent::Error { message, code } => {
                            let _ = spinner.fail(&message, &theme, &mut stdout);
                            eprintln!(
                                "\x1b[1;31mError\x1b[0m: {message} ({})",
                                code.as_deref().unwrap_or("unknown")
                            );
                            return Ok((String::new(), Vec::new(), true));
                        }
                        _ => {}
                    }
                }
            }
            Err(e) => {
                eprintln!("\x1b[1;31mError\x1b[0m: {e}");
                return Ok((String::new(), Vec::new(), true));
            }
        }

        Ok((text_response, tool_calls, false))
    }

    async fn check_permission(&self, tc: &ToolCallInfo) -> bool {
        let config_permissions = &self.config.permissions;

        if config_permissions.always_deny.iter().any(|p| p == &tc.name) {
            return false;
        }
        if config_permissions
            .always_allow
            .iter()
            .any(|p| p == &tc.name)
        {
            return true;
        }
        if !config_permissions.ask_always {
            return true;
        }

        let tool_name = &tc.name;
        let input_str = tc.input.to_string();

        eprintln!("\x1b[1;33m┌─ Tool Permission ─────────────────────────────┐\x1b[0m");
        eprintln!("\x1b[1;33m│\x1b[0m Tool: \x1b[1;37m{tool_name}\x1b[0m");
        if input_str.len() < 200 {
            eprintln!("\x1b[1;33m│\x1b[0m Input: {input_str}");
        }
        eprintln!(
            "\x1b[1;33m│\x1b[0m                                           \x1b[1;33m│\x1b[0m"
        );
        eprintln!(
            "\x1b[1;33m│\x1b[0m \x1b[1;36mAllow?\x1b[0m  \x1b[1;32m(y)es\x1b[0m / \x1b[1;31m(n)o\x1b[0m / \x1b[1;34m(a)lways\x1b[0m / \x1b[1;33m(n)ever\x1b[0m  \x1b[1;33m│\x1b[0m"
        );
        eprintln!("\x1b[1;33m└────────────────────────────────────────────────┘\x1b[0m");
        eprint!("> ");

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).ok();
        let input = input.trim().to_lowercase();

        matches!(input.as_str(), "y" | "yes")
    }

    fn save_on_exit(&mut self, _engine: &mut Engine) {
        if !self.conversation.is_empty() {
            self.session_store.save(&self.conversation).ok();
            println!(
                "\n\x1b[1;32m✓\x1b[0m Session saved: {}",
                self.conversation.id
            );
        }
        println!("\x1b[1;33mGoodbye!\x1b[0m");
    }
}

struct ToolCallInfo {
    id: String,
    name: String,
    input: serde_json::Value,
}
