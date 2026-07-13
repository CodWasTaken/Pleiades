use std::sync::Arc;

use pleiades_agent_config::Config;
use pleiades_agent_core::conversation::{Conversation, Message};
use pleiades_agent_core::error::Error;
use pleiades_agent_core::provider::StreamEvent;
use pleiades_agent_engine::Engine;
use pleiades_agent_engine::session::SessionStore;
use rustyline::completion::Completer;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::history::FileHistory;
use rustyline::validate::Validator;
use rustyline::{Editor, Helper};

/// REPL state for interactive chat sessions.
pub struct Repl {
    config: Arc<Config>,
    conversation: Conversation,
    session_store: SessionStore,
    provider_name: String,
    model_name: String,
}

impl Repl {
    pub fn new(config: Config) -> Repl {
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
        let conversation = Conversation::new(format!("conv_{}", chrono::Utc::now().timestamp()));

        Repl {
            config: Arc::new(config),
            conversation,
            session_store,
            provider_name,
            model_name,
        }
    }

    pub fn with_session(&mut self, session_id: &str) -> Result<(), Error> {
        let loaded = self.session_store.load(session_id)?;
        self.conversation = loaded;
        Ok(())
    }

    pub async fn run(&mut self) -> Result<(), Error> {
        let mut engine = self.build_engine()?;

        let mut rl = Editor::<ReplHelper, FileHistory>::new()
            .map_err(|e| Error::io(format!("Failed to create REPL editor: {}", e)))?;
        rl.set_helper(Some(ReplHelper));
        rl.load_history(".pleiades_history").ok();

        self.print_welcome();

        loop {
            let readline = rl.readline(">> ");
            match readline {
                Ok(input) => {
                    let trimmed = input.trim();
                    if trimmed.is_empty() {
                        continue;
                    }

                    let _ = rl.add_history_entry(trimmed);
                    rl.save_history(".pleiades_history").ok();

                    if trimmed.starts_with('/') {
                        if !self.handle_command(&mut engine, trimmed).await {
                            break;
                        }
                        continue;
                    }

                    self.handle_user_input(&mut engine, trimmed).await?;
                }
                Err(rustyline::error::ReadlineError::Interrupted) => {
                    break;
                }
                Err(rustyline::error::ReadlineError::Eof) => {
                    break;
                }
                Err(e) => {
                    eprintln!("Readline error: {}", e);
                    break;
                }
            }
        }

        self.save_on_exit(&mut engine);
        Ok(())
    }

    /// Run one prompt through the same agent and tool loop as the interactive REPL.
    pub async fn run_once(&mut self, input: &str) -> Result<(), Error> {
        let mut engine = self.build_engine()?;
        self.handle_user_input(&mut engine, input).await
    }

    fn build_engine(&self) -> Result<Engine, Error> {
        let mut engine = Engine::new((*self.config).clone());

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
        let model = &self.model_name;
        let provider = &self.provider_name;
        println!("\x1b[1;36m╭──────────────────────────────────────────╮\x1b[0m");
        println!(
            "\x1b[1;36m│\x1b[0m  \x1b[1;33mPleiades\x1b[0m - Terminal AI Assistant      \x1b[1;36m│\x1b[0m"
        );
        println!(
            "\x1b[1;36m│\x1b[0m  Model: {}                        \x1b[1;36m│\x1b[0m",
            model
        );
        println!(
            "\x1b[1;36m│\x1b[0m  Provider: {}                      \x1b[1;36m│\x1b[0m",
            provider
        );
        println!(
            "\x1b[1;36m│\x1b[0m  Messages: {}                            \x1b[1;36m│\x1b[0m",
            self.conversation.len()
        );
        println!(
            "\x1b[1;36m│\x1b[0m  Type \x1b[1;32m/help\x1b[0m for commands              \x1b[1;36m│\x1b[0m"
        );
        println!("\x1b[1;36m╰──────────────────────────────────────────╯\x1b[0m");
        println!();
    }

    async fn handle_command(&mut self, engine: &mut Engine, input: &str) -> bool {
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
                println!(
                    "  \x1b[1;32m/model <name>\x1b[0m     Switch model (e.g., /model claude-sonnet-4-20250514)"
                );
                println!("  \x1b[1;32m/provider <name>\x1b[0m  Switch provider");
                println!("  \x1b[1;32m/info\x1b[0m             Show session info");
                println!("  \x1b[1;32m/tokens\x1b[0m           Show estimated token count");
                println!("  \x1b[1;32m/export [fmt]\x1b[0m     Export session (markdown/json)");
                println!("  \x1b[1;32m/exit\x1b[0m             Exit the REPL");
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
                    Err(e) => eprintln!("\x1b[1;31m✗\x1b[0m Save failed: {}", e),
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
                            "\x1b[1;32m✓\x1b[0m Session '{}' loaded ({} messages)",
                            arg,
                            self.conversation.len()
                        );
                        self.print_welcome();
                    }
                    Err(e) => eprintln!("\x1b[1;31m✗\x1b[0m Load failed: {}", e),
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
                    *engine = self.build_engine().expect("Failed to build engine");
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
                    *engine = self.build_engine().expect("Failed to build engine");
                }
            }
            "/info" => {
                let tokens = self.conversation.estimated_tokens();
                println!("\x1b[1;33mSession Info:\x1b[0m");
                println!("  ID:       {}", self.conversation.id);
                if let Some(ref title) = self.conversation.metadata.title {
                    println!("  Title:    {}", title);
                }
                println!("  Messages: {}", self.conversation.len());
                println!("  Tokens:   ~{}", tokens);
                println!("  Model:    {}", self.model_name);
                println!("  Provider: {}", self.provider_name);
            }
            "/tokens" => {
                let tokens = self.conversation.estimated_tokens();
                println!("Estimated tokens: \x1b[1;33m{}\x1b[0m", tokens);
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
                        let path = format!("{}.{}", self.conversation.id, ext);
                        std::fs::write(&path, &content).ok();
                        println!("\x1b[1;32m✓\x1b[0m Exported to {}", path);
                    }
                    Err(e) => eprintln!("\x1b[1;31m✗\x1b[0m Export failed: {}", e),
                }
            }
            "/exit" | "/quit" => {
                return false;
            }
            _ => {
                eprintln!(
                    "Unknown command: {}. Type /help for available commands.",
                    cmd
                );
            }
        }
        true
    }

    async fn handle_user_input(&mut self, engine: &mut Engine, input: &str) -> Result<(), Error> {
        let user_msg = Message::user(input);
        self.conversation.add_message(user_msg);

        let max_iterations = self.config.agent.max_tool_iterations;

        for iteration in 0..max_iterations {
            println!("\x1b[1;36m─── response ───\x1b[0m");

            let (text_response, tool_calls, had_error) = self.stream_response(engine).await?;

            if had_error {
                return Ok(());
            }

            let text_content = text_response.clone();

            if tool_calls.is_empty() {
                if !text_content.is_empty() {
                    let assistant_msg = Message::assistant(text_content);
                    self.conversation.add_message(assistant_msg);
                }
                break;
            }

            let mut content_blocks = Vec::new();
            if !text_content.is_empty() {
                content_blocks.push(pleiades_agent_core::conversation::ContentBlock::Text(
                    text_content,
                ));
            }
            for tc in &tool_calls {
                content_blocks.push(pleiades_agent_core::conversation::ContentBlock::ToolUse {
                    id: tc.id.clone(),
                    name: tc.name.clone(),
                    input: tc.input.clone(),
                });
            }

            let assistant_msg = Message {
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

            let mut all_succeeded = true;
            for tc in &tool_calls {
                let allowed = self.check_permission(tc).await;
                if !allowed {
                    self.conversation.add_message(Message {
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
                        self.conversation.add_message(Message {
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
                            all_succeeded = false;
                        }
                    }
                    Err(e) => {
                        self.conversation.add_message(Message {
                            role: pleiades_agent_core::conversation::MessageRole::Tool,
                            content: vec![
                                pleiades_agent_core::conversation::ContentBlock::ToolResult {
                                    id: tc.id.clone(),
                                    content: format!("Error: {}", e),
                                    is_error: true,
                                },
                            ],
                            reasoning: None,
                            metadata: None,
                        });
                        println!("\x1b[1;31m  ✗ {} error: {}\x1b[0m", tc.name, e);
                        all_succeeded = false;
                    }
                }
            }

            self.session_store.save(&self.conversation).ok();

            if iteration == max_iterations - 1 {
                println!(
                    "\x1b[1;33m⚠ Max tool iterations ({}) reached\x1b[0m",
                    max_iterations
                );
            }

            if !all_succeeded {
                println!("\x1b[1;33m⚠ Some tools failed, continuing...\x1b[0m");
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

        match engine
            .chat_stream(&mut self.conversation, &self.provider_name)
            .await
        {
            Ok(mut rx) => {
                while let Some(event) = rx.recv().await {
                    match event {
                        StreamEvent::Token(token) => {
                            print!("{}", token);
                            text_response.push_str(&token);
                        }
                        StreamEvent::ReasoningToken(token) => {
                            print!("\x1b[2m{}\x1b[0m", token);
                        }
                        StreamEvent::ToolCall { id, name, input } => {
                            tool_calls.push(ToolCallInfo { id, name, input });
                        }
                        StreamEvent::Done { .. } => {
                            println!();
                            break;
                        }
                        StreamEvent::Error { message, code } => {
                            eprintln!(
                                "\n\x1b[1;31mError\x1b[0m: {} ({})",
                                message,
                                code.as_deref().unwrap_or("unknown")
                            );
                            return Ok((String::new(), Vec::new(), true));
                        }
                        _ => {}
                    }
                }
            }
            Err(e) => return Err(e),
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
        eprintln!("\x1b[1;33m│\x1b[0m Tool: \x1b[1;37m{}\x1b[0m", tool_name);
        if input_str.len() < 200 {
            eprintln!("\x1b[1;33m│\x1b[0m Input: {}", input_str);
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

        match input.as_str() {
            "y" | "yes" => true,
            "n" | "no" => false,
            _ => true,
        }
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

#[derive(Clone)]
struct ReplHelper;

impl Completer for ReplHelper {
    type Candidate = String;
}

impl Hinter for ReplHelper {
    type Hint = String;
}

impl Highlighter for ReplHelper {}

impl Validator for ReplHelper {}

impl Helper for ReplHelper {}
