use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

use pleiades_agent_config::loader::ConfigLoader;
use pleiades_agent_config::profile::ProfileManager;
use pleiades_agent_config::validate;

mod repl;

#[derive(Parser)]
#[command(
    name = "pleiades",
    version = env!("CARGO_PKG_VERSION"),
    about = "A next-generation, provider-agnostic terminal AI assistant",
    long_about = "Pleiades is a terminal AI assistant that supports multiple AI providers, \
                  extensible plugins, and a beautiful terminal interface.",
    subcommand_required = false
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Start the interactive autonomous terminal agent (legacy flag; prefer `pleiades chat`)
    #[arg(short, long)]
    chat: bool,

    /// Session ID to resume (for --chat mode)
    #[arg(short = 'S', long)]
    session: Option<String>,

    /// One-shot prompt mode
    #[arg(allow_hyphen_values = true, trailing_var_arg = true)]
    prompt: Option<Vec<String>>,

    /// Model to use
    #[arg(short, long, global = true)]
    model: Option<String>,

    /// Provider to use
    #[arg(short = 'P', long, global = true)]
    provider: Option<String>,

    /// Agent mode: plan, agent, or unrestricted
    #[arg(
        long,
        global = true,
        hide_possible_values = true,
        value_parser = ["plan", "agent", "unrestricted", "read-only", "workspace-write", "danger-full-access"]
    )]
    permission_mode: Option<String>,

    /// Verbose output
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the interactive autonomous terminal agent
    Chat {
        /// Session ID to load
        #[arg(short, long)]
        session: Option<String>,
    },

    /// Configure Pleiades with a guided authentication flow
    Setup {
        /// Authentication method (omit for an interactive choice)
        #[arg(long, value_enum)]
        auth: Option<SetupAuth>,

        /// Use Codex device-code authentication instead of a browser callback
        #[arg(long)]
        device: bool,
    },

    /// Manage OpenAI subscription authentication through the official Codex CLI
    #[command(subcommand)]
    Auth(AuthCommand),

    /// Diagnose configuration and authentication problems
    Doctor,

    /// Manage configuration
    #[command(subcommand)]
    Config(ConfigCommand),

    /// Manage profiles
    #[command(subcommand)]
    Profile(ProfileCommand),

    /// Manage providers
    #[command(subcommand)]
    Provider(ProviderCommand),

    /// Manage models
    #[command(subcommand)]
    Model(ModelCommand),

    /// Manage chat sessions
    #[command(subcommand)]
    Session(SessionCommand),

    /// Manage and execute tools
    #[command(subcommand)]
    Tool(ToolCommand),

    /// Manage plugins
    #[command(subcommand)]
    Plugin(PluginCommand),

    /// Manage and render prompt templates
    #[command(subcommand)]
    Prompt(PromptCommand),

    /// Manage and run workflows
    #[command(subcommand)]
    Workflow(WorkflowCommand),

    /// AI-assisted Git operations
    #[command(subcommand)]
    Git(GitCommand),

    /// Start an interactive REPL session
    Repl {
        /// Session ID to load
        #[arg(short, long)]
        session: Option<String>,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum SetupAuth {
    /// Use an included ChatGPT subscription through the official Codex CLI
    Chatgpt,
    /// Use usage-based OpenAI Platform API billing
    ApiKey,
}

#[derive(Subcommand)]
enum AuthCommand {
    /// Sign in with ChatGPT and configure the subscription provider
    Login {
        /// Use device-code authentication for remote or headless terminals
        #[arg(long)]
        device: bool,
    },
    /// Show the current Codex authentication status
    Status,
    /// Sign out of the Codex session
    Logout,
}

#[derive(Subcommand)]
enum ConfigCommand {
    /// Get a config value by key (e.g., "core.default_provider")
    Get {
        /// Config key path (e.g., "core.default_provider")
        key: String,
    },

    /// Set a config value by key
    Set {
        /// Config key path (e.g., "core.default_provider")
        key: String,
        /// Value to set
        value: String,
    },

    /// Edit config in $EDITOR
    Edit,

    /// Validate the current configuration
    Validate,

    /// Show the current configuration
    Show {
        /// Show raw config (including secrets)
        #[arg(short, long)]
        raw: bool,
    },

    /// Show config file location
    Path,

    /// Initialize a default config file
    Init {
        /// Force overwrite of existing config
        #[arg(short, long)]
        force: bool,
        /// Config format (toml, json, yaml)
        #[arg(long, default_value = "toml")]
        format: String,
    },

    /// Reset config to defaults
    Reset {
        /// Confirm reset
        #[arg(short, long)]
        yes: bool,
    },
}

#[derive(Subcommand)]
enum ProfileCommand {
    /// List all profiles
    List,

    /// Create or update a profile
    Save {
        /// Profile name
        name: String,
    },

    /// Load a profile
    Load {
        /// Profile name
        name: String,
    },

    /// Delete a profile
    Delete {
        /// Profile name
        name: String,
    },

    /// Show active profile
    Active,
}

#[derive(Subcommand)]
enum ProviderCommand {
    /// List all available providers
    List,

    /// Test a provider connection
    Test {
        /// Provider name to test
        name: String,

        /// Model to use for the test (defaults to provider's default)
        #[arg(short, long)]
        model: Option<String>,
    },

    /// Show provider details
    Info {
        /// Provider name
        name: String,
    },

    /// Remove a provider configuration
    Remove {
        /// Provider name to remove
        name: String,
    },
}

#[derive(Subcommand)]
enum ModelCommand {
    /// List available models
    List {
        /// Filter by provider
        #[arg(short, long)]
        provider: Option<String>,

        /// Search string
        #[arg(short, long)]
        search: Option<String>,
    },

    /// Show model details
    Info {
        /// Model name or alias
        name: String,
    },

    /// Set the default model
    SetDefault {
        /// Model name
        model: String,
    },

    /// Create or remove a model alias
    Alias {
        /// Alias name
        alias: String,
        /// Model ID to point to
        model: String,
    },

    /// Remove an alias
    Unalias {
        /// Alias name to remove
        alias: String,
    },

    /// Discover models from configured providers
    Discover,
}

#[derive(Subcommand)]
enum SessionCommand {
    /// List saved sessions
    List,

    /// Show session details
    Show {
        /// Session ID
        id: String,
    },

    /// Delete a saved session
    Delete {
        /// Session ID
        id: String,
    },

    /// Export a session to a file
    Export {
        /// Session ID
        id: String,
        /// Output format (markdown, json)
        #[arg(short, long, default_value = "markdown")]
        format: String,
        /// Output file path (defaults to `ID.FORMAT`)
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Show session store location
    Path,
}

#[derive(Subcommand)]
enum ToolCommand {
    /// List all available tools
    List,

    /// Show tool details
    Info {
        /// Tool name
        name: String,
    },

    /// Execute a tool directly
    Call {
        /// Tool name
        name: String,
        /// JSON input for the tool
        input: String,
    },
}

#[derive(Subcommand)]
enum PluginCommand {
    /// List installed and discoverable plugins
    List,

    /// Install a plugin from a directory
    Install {
        /// Path to the plugin directory
        path: String,
    },

    /// Uninstall a plugin by ID
    Uninstall {
        /// Plugin ID (e.g., "my-plugin-external")
        id: String,
    },

    /// Enable a plugin
    Enable {
        /// Plugin ID
        id: String,
    },

    /// Disable a plugin
    Disable {
        /// Plugin ID
        id: String,
    },
}

#[derive(Subcommand)]
enum PromptCommand {
    /// List available prompt templates
    List,

    /// Show a prompt template's raw text and variables
    Show {
        /// Prompt name
        name: String,
    },

    /// Render a prompt template with variables (key=value pairs)
    Render {
        /// Prompt name
        name: String,
        /// Variables as key=value
        #[arg(long = "var", num_args = 0..)]
        vars: Vec<String>,
    },

    /// Save a custom prompt template
    Save {
        /// Prompt name
        name: String,
        /// Prompt description
        description: String,
        /// Template body (use quotes; {{var}} for substitution)
        template: String,
    },
}

#[derive(Subcommand)]
enum WorkflowCommand {
    /// List available workflow definitions
    List,
    /// Run a workflow
    Run {
        /// Workflow name or path
        name: String,
        /// Variables as key=value
        #[arg(long = "var")]
        vars: Vec<String>,
    },
    /// Show a workflow definition
    Show { name: String },
    /// Validate a workflow definition
    Validate { name: String },
    /// Create a starter workflow in .pleiades/workflows
    Create {
        name: String,
        #[arg(short, long)]
        description: Option<String>,
    },
}

#[derive(Subcommand)]
enum GitCommand {
    /// Generate a conventional commit message from staged changes
    Commit,
    /// Review working-tree or staged changes
    Review {
        #[arg(long)]
        staged: bool,
    },
    /// Generate a pull-request summary
    Summary {
        /// Base revision used for the comparison
        #[arg(long, default_value = "HEAD~1")]
        base: String,
        #[arg(long)]
        title: Option<String>,
    },
    /// Print the current diff
    Diff {
        #[arg(long)]
        staged: bool,
    },
}

fn get_config_dirs() -> (PathBuf, PathBuf) {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("pleiades");
    let project_dir = PathBuf::from(".pleiades");
    (config_dir, project_dir)
}

fn handle_config_get(loader: &ConfigLoader, key: &str) {
    let config = match loader.load_with_interpolation() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    let value = get_nested_value(&config, key);
    match value {
        Some(v) => println!("{}", v),
        None => {
            eprintln!("Error: key '{}' not found in config", key);
            std::process::exit(1);
        }
    }
}

fn handle_config_set(loader: &ConfigLoader, key: &str, value: &str) {
    if let Err(e) = validate::validate_field(key, value) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    let mut config = match loader.load_with_interpolation() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Warning: no existing config ({}), using defaults", e);
            pleiades_agent_config::Config::default()
        }
    };

    set_nested_value(&mut config, key, value);

    match loader.save_project(&config) {
        Ok(_) => println!("Set {} = {}", key, value),
        Err(e) => {
            eprintln!("Error saving config: {}", e);
            std::process::exit(1);
        }
    }
}

fn handle_config_edit() {
    let (config_dir, project_dir) = get_config_dirs();
    let paths = [
        project_dir.join("config.toml"),
        config_dir.join("config.toml"),
    ];

    let config_path = paths
        .iter()
        .find(|p| p.exists())
        .cloned()
        .unwrap_or_else(|| {
            let dir = if PathBuf::from(".pleiades").exists() {
                PathBuf::from(".pleiades")
            } else {
                config_dir.clone()
            };
            dir.join("config.toml")
        });

    let editor = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| "vim".to_string());

    println!("Opening {} with {}", config_path.display(), editor);

    let status = std::process::Command::new(&editor)
        .arg(&config_path)
        .status()
        .expect("Failed to launch editor");

    if !status.success() {
        eprintln!("Editor exited with error");
        std::process::exit(1);
    }

    // Validate after editing
    let loader = ConfigLoader::with_dirs(config_dir.clone(), project_dir.clone());
    match loader.load() {
        Ok(_) => println!("Config is valid"),
        Err(e) => eprintln!("Config validation failed:\n{}", e),
    }
}

fn handle_config_validate(loader: &ConfigLoader) {
    match loader.load() {
        Ok(config) => {
            println!("Config is valid");
            if config.core.verbose {
                println!("Providers configured: {:?}", config.providers.len());
                if let Some(ref provider) = config.core.default_provider {
                    println!("Default provider: {}", provider);
                }
            }
        }
        Err(e) => {
            eprintln!("Config validation failed:\n{}", e);
            std::process::exit(1);
        }
    }
}

fn handle_config_show(loader: &ConfigLoader, raw: bool) {
    let config = match loader.load_with_interpolation() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    let mut display = config.clone();

    if !raw {
        // Mask API keys for display
        for provider in display.providers.values_mut() {
            if let Some(ref key) = provider.api_key {
                provider.api_key = Some(pleiades_agent_config::env_interpolate::mask_secrets(key));
            }
        }
    }

    match toml::to_string_pretty(&display) {
        Ok(s) => println!("{}", s),
        Err(e) => {
            eprintln!("Error serializing config: {}", e);
            std::process::exit(1);
        }
    }
}

fn handle_config_path() {
    let (config_dir, project_dir) = get_config_dirs();
    let paths = [
        ("Project", project_dir.join("config.toml")),
        ("Global", config_dir.join("config.toml")),
    ];

    for (label, path) in &paths {
        let exists = if path.exists() { "exists" } else { "not found" };
        println!("{}: {} ({})", label, path.display(), exists);
    }
}

fn handle_config_init(force: bool, format: &str) {
    let valid_formats = ["toml", "json", "yaml"];
    if !valid_formats.contains(&format) {
        eprintln!(
            "Invalid format '{}'. Must be one of: {}",
            format,
            valid_formats.join(", ")
        );
        std::process::exit(1);
    }

    let (config_dir, project_dir) = get_config_dirs();
    let dir = if project_dir.exists() || force {
        &project_dir
    } else {
        &config_dir
    };

    let config_path = dir.join(format!("config.{}", format));

    if config_path.exists() && !force {
        eprintln!("Config already exists at {}", config_path.display());
        eprintln!("Use --force to overwrite");
        std::process::exit(1);
    }

    std::fs::create_dir_all(dir).expect("Failed to create config directory");
    let config = pleiades_agent_config::Config::default();

    let content: String = match format {
        "json" => serde_json::to_string_pretty(&config).expect("Failed to serialize"),
        "yaml" => serde_yaml::to_string(&config).expect("Failed to serialize"),
        _ => toml::to_string_pretty(&config).expect("Failed to serialize"),
    };

    std::fs::write(&config_path, &content).expect("Failed to write config");
    println!("Initialized config: {}", config_path.display());
}

fn handle_config_reset(loader: &ConfigLoader, yes: bool) {
    if !yes {
        eprintln!("This will reset your config to defaults.");
        eprintln!("Use --yes to confirm.");
        std::process::exit(1);
    }

    match loader.save_project(&pleiades_agent_config::Config::default()) {
        Ok(_) => println!("Config reset to defaults"),
        Err(e) => {
            eprintln!("Error resetting config: {}", e);
            std::process::exit(1);
        }
    }
}

fn handle_profile_list(loader: &ConfigLoader) {
    let manager = ProfileManager::new(loader.global_dir());
    match manager.list() {
        Ok(profiles) => {
            if profiles.is_empty() {
                println!("No profiles found");
            } else {
                println!("Available profiles:");
                for profile in &profiles {
                    let active = match manager.active() {
                        Some(a) if a == profile => " (active)",
                        _ => "",
                    };
                    println!("  - {}{}", profile, active);
                }
            }
        }
        Err(e) => {
            eprintln!("Error listing profiles: {}", e);
            std::process::exit(1);
        }
    }
}

fn handle_profile_save(loader: &ConfigLoader, name: &str) {
    let config = match loader.load_with_interpolation() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error loading config: {}", e);
            std::process::exit(1);
        }
    };

    let manager = ProfileManager::new(loader.global_dir());
    match manager.save(name, &config) {
        Ok(_) => println!("Profile '{}' saved", name),
        Err(e) => {
            eprintln!("Error saving profile: {}", e);
            std::process::exit(1);
        }
    }
}

fn handle_profile_load(loader: &ConfigLoader, name: &str) {
    let manager = ProfileManager::new(loader.global_dir());
    let profile = match manager.load(name) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error loading profile '{}': {}", name, e);
            std::process::exit(1);
        }
    };

    match loader.save_project(&profile) {
        Ok(_) => println!("Profile '{}' applied", name),
        Err(e) => {
            eprintln!("Error applying profile: {}", e);
            std::process::exit(1);
        }
    }
}

fn handle_profile_delete(loader: &ConfigLoader, name: &str) {
    let manager = ProfileManager::new(loader.global_dir());
    match manager.delete(name) {
        Ok(_) => println!("Profile '{}' deleted", name),
        Err(e) => {
            eprintln!("Error deleting profile: {}", e);
            std::process::exit(1);
        }
    }
}

fn handle_profile_active(loader: &ConfigLoader) {
    let manager = ProfileManager::new(loader.global_dir());
    match manager.active() {
        Some(name) => println!("{}", name),
        None => println!("No active profile"),
    }
}

fn handle_provider_list(loader: &ConfigLoader) {
    let config = match loader.load_with_interpolation() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error loading config: {}", e);
            std::process::exit(1);
        }
    };

    if config.providers.is_empty() {
        println!("No providers configured.");
        println!("Run 'pleiades setup' for guided configuration.");
        return;
    }

    for (name, pc) in &config.providers {
        let has_key = pc.api_key.is_some() && !pc.api_key.as_deref().unwrap_or("").is_empty();
        let key_display = if has_key {
            if let Some(ref key) = pc.api_key {
                pleiades_agent_config::env_interpolate::mask_secrets(key)
            } else {
                "not set".to_string()
            }
        } else {
            "not set".to_string()
        };

        let base_url = pc.base_url.as_deref().unwrap_or("(default)");
        println!("  {}:", name);
        if name == "openai-subscription" {
            println!("    Authentication: ChatGPT subscription via Codex CLI");
        } else {
            println!("    API Key: {}", key_display);
        }
        println!("    Base URL: {}", base_url);
        println!();
    }
}

fn handle_provider_info(loader: &ConfigLoader, name: &str) {
    let config = match loader.load_with_interpolation() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error loading config: {}", e);
            std::process::exit(1);
        }
    };

    let pc = match config.providers.get(name) {
        Some(p) => p,
        None => {
            eprintln!("Provider '{}' not found in config", name);
            std::process::exit(1);
        }
    };

    let secret_manager = pleiades_agent_config::SecretManager::new();
    let env_var = secret_manager.expected_env_var(name).unwrap_or("(none)");

    println!("Provider: {}", name);
    if name == "openai-subscription" {
        println!("  Authentication: ChatGPT subscription via the official Codex CLI");
        println!("  Credentials: managed by Codex (Pleiades never reads them)");
        println!("  Status command: pleiades auth status");
        return;
    }
    println!(
        "  API Key: {}",
        pc.api_key
            .as_ref()
            .map(|k| pleiades_agent_config::env_interpolate::mask_secrets(k))
            .unwrap_or_else(|| "not set".to_string())
    );
    println!(
        "  Base URL: {}",
        pc.base_url.as_deref().unwrap_or("(default)")
    );
    println!("  Expected Env Var: {}", env_var);
    println!("  Max Retries: {}", pc.max_retries);
    println!("  Timeout: {}s", pc.timeout_secs);
    if !pc.headers.is_empty() {
        println!("  Custom Headers: {:?}", pc.headers);
    }
}

fn handle_provider_remove(loader: &ConfigLoader, name: &str) {
    let mut config = match loader.load_with_interpolation() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error loading config: {}", e);
            std::process::exit(1);
        }
    };

    if config.providers.remove(name).is_none() {
        eprintln!("Provider '{}' not found in config", name);
        std::process::exit(1);
    }

    match loader.save_project(&config) {
        Ok(_) => println!("Provider '{}' removed", name),
        Err(e) => {
            eprintln!("Error saving config: {}", e);
            std::process::exit(1);
        }
    }
}

fn handle_provider_test(loader: &ConfigLoader, name: &str, model: Option<String>) {
    let config = match loader.load_with_interpolation() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error loading config: {}", e);
            std::process::exit(1);
        }
    };

    let pc = match config.providers.get(name) {
        Some(p) => p,
        None => {
            eprintln!("Provider '{}' not found in config", name);
            std::process::exit(1);
        }
    };

    let api_key = if name == "openai-subscription" {
        String::new()
    } else {
        match pc.api_key.as_deref() {
            Some(k) if !k.is_empty() => k.to_string(),
            _ => {
                eprintln!("No API key configured for '{}'", name);
                std::process::exit(1);
            }
        }
    };
    let base_url = pc.base_url.clone().unwrap_or_default();

    let provider: Box<dyn pleiades_agent_core::Provider> = match name {
        "openai-subscription" => Box::new(pleiades_agent_providers::codex::CodexCliProvider::new()),
        "anthropic" => {
            if base_url.is_empty() {
                Box::new(pleiades_agent_providers::anthropic::AnthropicProvider::new(
                    api_key,
                ))
            } else {
                Box::new(
                    pleiades_agent_providers::anthropic::AnthropicProvider::with_base_url(
                        api_key, base_url,
                    ),
                )
            }
        }
        "openai" => {
            if base_url.is_empty() {
                Box::new(pleiades_agent_providers::openai::OpenAIProvider::new(
                    api_key,
                ))
            } else {
                Box::new(
                    pleiades_agent_providers::openai::OpenAIProvider::with_base_url(
                        api_key, base_url,
                    ),
                )
            }
        }
        _ => {
            let display = name;
            let model_name = model.clone().unwrap_or_else(|| "gpt-4o".to_string());
            Box::new(
                pleiades_agent_providers::openai_compat::OpenAICompatibleProvider::new(
                    name, display, api_key, base_url, model_name,
                ),
            )
        }
    };

    let model_name = model
        .clone()
        .unwrap_or_else(|| provider.default_model().to_string());
    println!("Testing provider '{}' with model '{}'...", name, model_name);

    let rt = match tokio::runtime::Runtime::new() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to create runtime: {}", e);
            std::process::exit(1);
        }
    };

    rt.block_on(async {
        match provider
            .chat_stream(pleiades_agent_core::provider::ChatRequest {
                model: model_name,
                messages: vec![pleiades_agent_core::conversation::Message::user(
                    "Respond with exactly: Hello from Pleiades!",
                )],
                system_prompt: None,
                temperature: Some(0.0),
                top_p: None,
                max_tokens: Some(50),
                stop: None,
                tools: None,
            })
            .await
        {
            Ok(mut rx) => {
                println!("  Connection successful! Response:");
                print!("  ");
                while let Some(event) = rx.recv().await {
                    match event {
                        pleiades_agent_core::provider::StreamEvent::Token(t) => print!("{}", t),
                        pleiades_agent_core::provider::StreamEvent::Done { .. } => {
                            println!();
                            println!("  ✓ Streaming works");
                            break;
                        }
                        pleiades_agent_core::provider::StreamEvent::Error { message, .. } => {
                            eprintln!("\n  ✗ Stream error: {}", message);
                            break;
                        }
                        _ => {}
                    }
                }
            }
            Err(e) => {
                eprintln!("  ✗ Connection failed: {}", e);
                std::process::exit(1);
            }
        }
    });
}

fn handle_model_list(
    loader: &ConfigLoader,
    provider_filter: Option<String>,
    search: Option<String>,
) {
    let config = match loader.load_with_interpolation() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error loading config: {}", e);
            std::process::exit(1);
        }
    };

    let mut registry = pleiades_agent_core::ModelRegistry::new();
    let providers = match provider_filter.as_deref() {
        Some(name) => {
            let pc = match config.providers.get(name) {
                Some(p) => p,
                None => {
                    eprintln!("Provider '{}' not found in config", name);
                    std::process::exit(1);
                }
            };
            let api_key = pc.api_key.as_deref().unwrap_or("");
            let base_url = pc.base_url.as_deref().unwrap_or("");
            if api_key.is_empty() && name != "openai-subscription" {
                eprintln!("No API key configured for '{}'", name);
                std::process::exit(1);
            }
            vec![build_test_provider(name, api_key, base_url)]
        }
        None => build_providers_from_config(&config),
    };

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let provider_refs: Vec<&dyn pleiades_agent_core::Provider> =
            providers.iter().map(|p| p.as_ref()).collect();
        let results = registry.discover_from_providers(&provider_refs).await;
        for (name, result) in &results {
            match result {
                Ok(count) => {
                    if *count == 0 {
                        eprintln!("Warning: {} returned 0 models", name);
                    }
                }
                Err(e) => {
                    eprintln!("Warning: {} discovery failed: {}", name, e);
                }
            }
        }
    });

    let models = match (provider_filter, search) {
        (Some(provider), None) => registry.list_by_provider(&provider),
        (_, Some(query)) => registry.search(&query),
        (None, None) => registry.list(),
    };

    if models.is_empty() {
        println!("No models found. Use 'pleiades model discover' to query providers.");
        return;
    }

    for model in &models {
        let ctx = model.capabilities.max_context_length;
        let ctx_str = pleiades_agent_core::model::format_context_length(ctx);
        let pricing = model
            .pricing
            .as_ref()
            .map(|p| format!("${:.2}i/${:.2}o", p.input_per_million, p.output_per_million))
            .unwrap_or_else(|| "pricing N/A".to_string());

        let name = model.display_name.as_deref().unwrap_or(&model.id);
        println!(
            "  {:<30}  {:<18}  ctx={:<8}  {}",
            name, model.provider, ctx_str, pricing
        );
    }
}

fn handle_model_info(loader: &ConfigLoader, name: &str) {
    let config = match loader.load_with_interpolation() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error loading config: {}", e);
            std::process::exit(1);
        }
    };

    let providers = build_providers_from_config(&config);
    let mut registry = pleiades_agent_core::ModelRegistry::new();

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let provider_refs: Vec<&dyn pleiades_agent_core::Provider> =
            providers.iter().map(|p| p.as_ref()).collect();
        let _ = registry.discover_from_providers(&provider_refs).await;
    });

    let model = match registry.resolve(name) {
        Some(m) => m,
        None => {
            eprintln!("Model '{}' not found", name);
            std::process::exit(1);
        }
    };

    println!(
        "Model: {}",
        model.display_name.as_deref().unwrap_or(&model.id)
    );
    println!("  ID:       {}", model.id);
    println!("  Provider: {}", model.provider);
    if let Some(ref desc) = model.description {
        println!("  Description: {}", desc);
    }
    println!("  Capabilities:");
    println!(
        "    Context:  {} tokens",
        pleiades_agent_core::model::format_context_length(model.capabilities.max_context_length)
    );
    println!(
        "    Output:   {} tokens",
        model.capabilities.max_output_tokens
    );
    println!("    Tools:    {}", yesno(model.capabilities.supports_tools));
    println!(
        "    Vision:   {}",
        yesno(model.capabilities.supports_vision)
    );
    println!(
        "    Streaming: {}",
        yesno(model.capabilities.supports_streaming)
    );
    println!(
        "    Thinking: {}",
        yesno(model.capabilities.supports_thinking)
    );
    println!(
        "    JSON mode: {}",
        yesno(model.capabilities.supports_json_mode)
    );

    if let Some(ref pricing) = model.pricing {
        println!("  Pricing (per million tokens):");
        println!(
            "    Input:  {}",
            pleiades_agent_core::model::format_price(pricing.input_per_million)
        );
        println!(
            "    Output: {}",
            pleiades_agent_core::model::format_price(pricing.output_per_million)
        );
        if let Some(cr) = pricing.cache_read_per_million {
            println!(
                "    Cache Read:  {}",
                pleiades_agent_core::model::format_price(cr)
            );
        }
        if let Some(cw) = pricing.cache_write_per_million {
            println!(
                "    Cache Write: {}",
                pleiades_agent_core::model::format_price(cw)
            );
        }
    }

    let aliases: Vec<String> = registry
        .aliases()
        .iter()
        .filter(|(_, v)| *v == &model.id)
        .map(|(k, _)| k.clone())
        .collect();

    if !aliases.is_empty() {
        println!("  Aliases: {}", aliases.join(", "));
    }
}

fn handle_model_set_default(loader: &ConfigLoader, model: &str) {
    let mut config = match loader.load_with_interpolation() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error loading config: {}", e);
            std::process::exit(1);
        }
    };

    config.core.default_model = Some(model.to_string());
    match loader.save_project(&config) {
        Ok(_) => println!("Default model set to '{}'", model),
        Err(e) => {
            eprintln!("Error saving config: {}", e);
            std::process::exit(1);
        }
    }
}

fn handle_model_alias(loader: &ConfigLoader, alias: &str, model: &str) {
    let config = match loader.load_with_interpolation() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error loading config: {}", e);
            std::process::exit(1);
        }
    };

    let mut registry = pleiades_agent_core::ModelRegistry::new();
    let providers = build_providers_from_config(&config);
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let provider_refs: Vec<&dyn pleiades_agent_core::Provider> =
            providers.iter().map(|p| p.as_ref()).collect();
        let _ = registry.discover_from_providers(&provider_refs).await;
    });

    match registry.add_alias(alias, model) {
        Ok(_) => {
            let mut config = config.clone();
            config
                .models
                .aliases
                .insert(alias.to_string(), model.to_string());
            match loader.save_project(&config) {
                Ok(_) => println!("Alias '{}' -> '{}' created", alias, model),
                Err(e) => eprintln!("Error saving config: {}", e),
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn handle_model_unalias(loader: &ConfigLoader, alias: &str) {
    let mut config = match loader.load_with_interpolation() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error loading config: {}", e);
            std::process::exit(1);
        }
    };

    if config.models.aliases.remove(alias).is_none() {
        eprintln!("Alias '{}' not found", alias);
        std::process::exit(1);
    }

    match loader.save_project(&config) {
        Ok(_) => println!("Alias '{}' removed", alias),
        Err(e) => eprintln!("Error saving config: {}", e),
    }
}

fn handle_model_discover(loader: &ConfigLoader) {
    let config = match loader.load_with_interpolation() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error loading config: {}", e);
            std::process::exit(1);
        }
    };

    let providers = build_providers_from_config(&config);
    if providers.is_empty() {
        println!(
            "No providers configured. Use 'pleiades config set providers.<name>.api_key <key>' first."
        );
        return;
    }

    let mut registry = pleiades_agent_core::ModelRegistry::new();
    let rt = tokio::runtime::Runtime::new().unwrap();

    println!("Discovering models from {} provider(s)...", providers.len());
    rt.block_on(async {
        let provider_refs: Vec<&dyn pleiades_agent_core::Provider> =
            providers.iter().map(|p| p.as_ref()).collect();
        let results = registry.discover_from_providers(&provider_refs).await;

        for (name, result) in &results {
            match result {
                Ok(count) => println!("  ✓ {}: {} models", name, count),
                Err(e) => println!("  ✗ {}: {}", name, e),
            }
        }
    });

    if !registry.is_empty() {
        let total = registry.len();
        println!(
            "\nTotal: {} models across {} provider(s)",
            total,
            registry.summary_by_provider().len()
        );
    }
}

fn yesno(v: bool) -> &'static str {
    if v { "yes" } else { "no" }
}

/// Build provider instances from configuration (without starting a runtime).
fn build_providers_from_config(
    config: &pleiades_agent_config::Config,
) -> Vec<Box<dyn pleiades_agent_core::Provider>> {
    let mut providers: Vec<Box<dyn pleiades_agent_core::Provider>> = Vec::new();

    for (name, pc) in &config.providers {
        let api_key = pc.api_key.as_deref().unwrap_or("");
        let base_url = pc.base_url.as_deref().unwrap_or("");
        if name == "openai-subscription" {
            providers.push(Box::new(
                pleiades_agent_providers::codex::CodexCliProvider::new(),
            ));
            continue;
        }
        if api_key.is_empty() {
            continue;
        }
        providers.push(build_test_provider(name, api_key, base_url));
    }

    providers
}

/// Build a single provider instance by name.
fn build_test_provider(
    name: &str,
    api_key: &str,
    base_url: &str,
) -> Box<dyn pleiades_agent_core::Provider> {
    match name {
        "openai-subscription" => Box::new(pleiades_agent_providers::codex::CodexCliProvider::new()),
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
        _ => Box::new(
            pleiades_agent_providers::openai_compat::OpenAICompatibleProvider::new(
                name,
                name,
                api_key.to_string(),
                base_url.to_string(),
                "gpt-4o".to_string(),
            ),
        ),
    }
}

fn handle_session_list(loader: &ConfigLoader) {
    let config = match loader.load() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error loading config: {}", e);
            std::process::exit(1);
        }
    };

    let store = pleiades_agent_engine::SessionStore::from_config(&config);

    match store.list() {
        Ok(sessions) => {
            if sessions.is_empty() {
                println!("No saved sessions.");
                println!("Sessions are saved to: {}", store.dir().display());
                return;
            }
            println!("Sessions ({} total):", sessions.len());
            println!();
            for session in &sessions {
                let title = session.metadata.title.as_deref().unwrap_or("Untitled");
                let created = session.metadata.created_at.format("%Y-%m-%d %H:%M");
                let model = session.metadata.model.as_deref().unwrap_or("?");
                let count = session
                    .metadata
                    .total_tokens
                    .map(|t| t.to_string())
                    .unwrap_or_else(|| "?".to_string());
                println!("  {}  {}  {}", &session.id[..8], created, title);
                println!("      model: {} | tokens: {}", model, count);
                println!();
            }
        }
        Err(e) => {
            eprintln!("Error listing sessions: {}", e);
            std::process::exit(1);
        }
    }
}

fn handle_session_show(loader: &ConfigLoader, id: &str) {
    let config = match loader.load() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error loading config: {}", e);
            std::process::exit(1);
        }
    };

    let store = pleiades_agent_engine::SessionStore::from_config(&config);

    match store.load(id) {
        Ok(conv) => {
            println!("Session: {}", conv.id);
            println!(
                "  Title:    {}",
                conv.metadata.title.as_deref().unwrap_or("Untitled")
            );
            println!(
                "  Created:  {}",
                conv.metadata.created_at.format("%Y-%m-%d %H:%M UTC")
            );
            println!(
                "  Updated:  {}",
                conv.metadata.updated_at.format("%Y-%m-%d %H:%M UTC")
            );
            if let Some(ref model) = conv.metadata.model {
                println!("  Model:    {}", model);
            }
            if let Some(ref provider) = conv.metadata.provider {
                println!("  Provider: {}", provider);
            }
            if let Some(tokens) = conv.metadata.total_tokens {
                println!("  Tokens:   {}", tokens);
            }
            if !conv.metadata.tags.is_empty() {
                println!("  Tags:     {}", conv.metadata.tags.join(", "));
            }
            println!("  Messages: {}", conv.messages.len());
            println!();
            for msg in &conv.messages {
                let role = format!("{:?}", msg.role).to_lowercase();
                let preview = &msg.text_content()[..msg.text_content().len().min(100)];
                println!("  [{}] {}", role, preview);
                if msg.text_content().len() > 100 {
                    println!("    ... ({} more chars)", msg.text_content().len() - 100);
                }
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn handle_session_delete(loader: &ConfigLoader, id: &str) {
    let config = match loader.load() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error loading config: {}", e);
            std::process::exit(1);
        }
    };

    let store = pleiades_agent_engine::SessionStore::from_config(&config);

    match store.delete(id) {
        Ok(_) => println!("Session '{}' deleted", id),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn handle_session_export(loader: &ConfigLoader, id: &str, format: &str, output: Option<String>) {
    let config = match loader.load() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error loading config: {}", e);
            std::process::exit(1);
        }
    };

    let store = pleiades_agent_engine::SessionStore::from_config(&config);

    let content = match format {
        "json" => store.export_json(id),
        _ => store.export_markdown(id),
    };

    match content {
        Ok(data) => {
            let path = output.unwrap_or_else(|| {
                format!("{}.{}", id, if format == "json" { "json" } else { "md" })
            });
            match std::fs::write(&path, &data) {
                Ok(_) => println!("Exported to {}", path),
                Err(e) => {
                    eprintln!("Error writing file: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Error exporting session: {}", e);
            std::process::exit(1);
        }
    }
}

fn handle_tool_list(loader: &ConfigLoader) {
    if let Err(e) = loader.load() {
        eprintln!("Warning: {}", e);
    }

    let mut tool_registry = pleiades_agent_tools::ToolRegistry::new();
    tool_registry.register_defaults();
    let tools = tool_registry.list();

    if tools.is_empty() {
        println!("No tools available.");
        return;
    }

    println!("Available tools ({}):", tools.len());
    println!();
    for tool in &tools {
        let ro = if tool.is_readonly() {
            "readonly"
        } else {
            "modifies"
        };
        println!("  {:<12}  {}  [{}]", tool.name(), tool.description(), ro);
    }
}

fn handle_tool_info(loader: &ConfigLoader, name: &str) {
    if let Err(e) = loader.load() {
        eprintln!("Warning: {}", e);
    }

    let mut tool_registry = pleiades_agent_tools::ToolRegistry::new();
    tool_registry.register_defaults();

    let tool = match tool_registry.get(name) {
        Some(t) => t,
        None => {
            eprintln!("Tool '{}' not found", name);
            std::process::exit(1);
        }
    };

    println!("Tool: {}", tool.name());
    println!("  Description: {}", tool.description());
    println!("  Readonly:    {}", yesno(tool.is_readonly()));
    println!("  Concurrency: {}", yesno(tool.is_concurrency_safe()));
    println!("  Permission:  {:?}", tool.permission_level());
    println!("  Input Schema:");
    let schema = tool.input_schema();
    if let Ok(formatted) = serde_json::to_string_pretty(&schema) {
        for line in formatted.lines() {
            println!("    {}", line);
        }
    } else {
        println!("    {}", schema);
    }
}

fn handle_tool_call(loader: &ConfigLoader, name: &str, input_str: &str) {
    let config = match loader.load() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error loading config: {}", e);
            std::process::exit(1);
        }
    };

    let input: serde_json::Value = match serde_json::from_str(input_str) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Invalid JSON input: {}", e);
            std::process::exit(1);
        }
    };

    let mut tool_registry = pleiades_agent_tools::ToolRegistry::new();
    tool_registry.register_defaults();

    let tool = match tool_registry.get(name) {
        Some(t) => t,
        None => {
            eprintln!("Tool '{}' not found", name);
            std::process::exit(1);
        }
    };

    let ctx = pleiades_agent_core::tool::ToolContext {
        cwd: std::env::current_dir().unwrap_or_default(),
        working_directory: std::env::current_dir().unwrap_or_default(),
        permission_mode: pleiades_agent_core::tool::PermissionMode::Allow,
        sandbox_mode: "workspace-write".to_string(),
        config: std::sync::Arc::new(serde_json::to_value(&config).unwrap_or_default()),
    };

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        match tool.execute(input, &ctx).await {
            Ok(result) => {
                if result.success {
                    println!("{}", result.content);
                } else {
                    eprintln!(
                        "Tool failed: {}",
                        result.error.unwrap_or_else(|| "unknown error".to_string())
                    );
                    std::process::exit(1);
                }
            }
            Err(e) => {
                eprintln!("Tool execution error: {}", e);
                std::process::exit(1);
            }
        }
    });
}

fn handle_session_path(loader: &ConfigLoader) {
    let config = match loader.load() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error loading config: {}", e);
            std::process::exit(1);
        }
    };

    let store = pleiades_agent_engine::SessionStore::from_config(&config);
    println!("{}", store.dir().display());
}

fn plugin_config_home() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("pleiades")
}

fn build_plugin_manager() -> pleiades_agent_plugins::PluginManager {
    pleiades_agent_plugins::PluginManager::new(plugin_config_home())
}

fn handle_plugin_list(_loader: &ConfigLoader) {
    let manager = build_plugin_manager();
    match manager.list_plugins() {
        Ok(plugins) => {
            if plugins.is_empty() {
                println!("No plugins found.");
                return;
            }
            for p in &plugins {
                let status = if p.enabled {
                    "\x1b[1;32menabled\x1b[0m"
                } else {
                    "\x1b[2mdisabled\x1b[0m"
                };
                println!(
                    "  {:<30}  {:<8}  {}  {} tools  {}",
                    p.id, p.version, status, p.tool_count, p.description
                );
            }
        }
        Err(e) => {
            eprintln!("Error listing plugins: {}", e);
            std::process::exit(1);
        }
    }
}

fn handle_plugin_install(_loader: &ConfigLoader, path: &str) {
    let mut manager = build_plugin_manager();
    match manager.install(path) {
        Ok(outcome) => {
            println!(
                "\x1b[1;32m✓\x1b[0m Plugin installed: {} v{}",
                outcome.plugin_id, outcome.version
            );
        }
        Err(e) => {
            eprintln!("\x1b[1;31m✗\x1b[0m Install failed: {}", e);
            std::process::exit(1);
        }
    }
}

fn handle_plugin_uninstall(_loader: &ConfigLoader, id: &str) {
    let mut manager = build_plugin_manager();
    match manager.uninstall(id) {
        Ok(_) => println!("\x1b[1;32m✓\x1b[0m Plugin uninstalled: {}", id),
        Err(e) => {
            eprintln!("\x1b[1;31m✗\x1b[0m Uninstall failed: {}", e);
            std::process::exit(1);
        }
    }
}

fn handle_plugin_enable(_loader: &ConfigLoader, id: &str) {
    let mut manager = build_plugin_manager();
    match manager.enable(id) {
        Ok(_) => println!("\x1b[1;32m✓\x1b[0m Plugin enabled: {}", id),
        Err(e) => {
            eprintln!("\x1b[1;31m✗\x1b[0m Enable failed: {}", e);
            std::process::exit(1);
        }
    }
}

fn handle_plugin_disable(_loader: &ConfigLoader, id: &str) {
    let mut manager = build_plugin_manager();
    match manager.disable(id) {
        Ok(_) => println!("\x1b[1;32m✓\x1b[0m Plugin disabled: {}", id),
        Err(e) => {
            eprintln!("\x1b[1;31m✗\x1b[0m Disable failed: {}", e);
            std::process::exit(1);
        }
    }
}

fn handle_prompt_list() {
    let lib = pleiades_agent_prompts::PromptLibrary::with_builtins();
    let summaries = lib.list();
    if summaries.is_empty() {
        println!("No prompt templates found.");
        return;
    }
    println!("Prompt templates ({}):\n", summaries.len());
    for s in &summaries {
        println!(
            "  \x1b[1;32m{:<22}\x1b[0m [\x1b[2m{}\x1b[0m] {}",
            s.name, s.source, s.description
        );
        if !s.variables.is_empty() {
            println!("      variables: {}", s.variables.join(", "));
        }
    }
}

fn handle_prompt_show(name: &str) {
    let lib = pleiades_agent_prompts::PromptLibrary::with_builtins();
    match lib.get(name) {
        Some(tpl) => {
            println!("Name:        {}", tpl.name());
            println!("Description: {}", tpl.description());
            println!("Variables:   {}", tpl.variable_names().join(", "));
            println!();
            println!("{}", tpl.raw());
        }
        None => {
            eprintln!("\x1b[1;31m✗\x1b[0m Prompt '{}' not found", name);
            std::process::exit(1);
        }
    }
}

fn handle_prompt_render(name: &str, vars: &[String]) {
    let lib = pleiades_agent_prompts::PromptLibrary::with_builtins();
    let mut map = std::collections::HashMap::new();
    for kv in vars {
        if let Some((k, v)) = kv.split_once('=') {
            map.insert(k.to_string(), v.to_string());
        } else {
            eprintln!(
                "\x1b[1;31m✗\x1b[0m Invalid variable '{}', expected key=value",
                kv
            );
            std::process::exit(1);
        }
    }
    match lib.render(name, &map) {
        Ok(rendered) => println!("{}", rendered),
        Err(e) => {
            eprintln!("\x1b[1;31m✗\x1b[0m {}", e);
            std::process::exit(1);
        }
    }
}

fn handle_prompt_save(name: &str, description: &str, template: &str) {
    let lib = pleiades_agent_prompts::PromptLibrary::with_builtins();
    let stored = pleiades_agent_prompts::StoredPrompt {
        name: name.to_string(),
        description: description.to_string(),
        template: template.to_string(),
    };
    match lib.save_custom(&stored) {
        Ok(_) => println!("\x1b[1;32m✓\x1b[0m Saved prompt '{}'", name),
        Err(e) => {
            eprintln!("\x1b[1;31m✗\x1b[0m Failed to save prompt: {}", e);
            std::process::exit(1);
        }
    }
}

fn workflow_dirs() -> Vec<PathBuf> {
    let mut dirs = vec![PathBuf::from(".pleiades/workflows")];
    if let Some(config) = dirs::config_dir() {
        dirs.push(config.join("pleiades/workflows"));
    }
    dirs
}

fn find_workflow(name: &str) -> Result<PathBuf, String> {
    let supplied = PathBuf::from(name);
    if supplied.is_file() {
        return Ok(supplied);
    }
    for dir in workflow_dirs() {
        for extension in ["toml", "yaml", "yml", "json"] {
            let candidate = dir.join(format!("{name}.{extension}"));
            if candidate.is_file() {
                return Ok(candidate);
            }
        }
    }
    Err(format!("workflow '{name}' not found"))
}

fn load_workflow(path: &std::path::Path) -> Result<pleiades_agent_workflow::Workflow, String> {
    let contents = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    match path.extension().and_then(|e| e.to_str()).unwrap_or("") {
        "toml" => toml::from_str(&contents).map_err(|e| e.to_string()),
        "yaml" | "yml" => serde_yaml::from_str(&contents).map_err(|e| e.to_string()),
        "json" => serde_json::from_str(&contents).map_err(|e| e.to_string()),
        extension => Err(format!("unsupported workflow format '{extension}'")),
    }
}

fn resolve_workflow(name: &str) -> Result<(PathBuf, pleiades_agent_workflow::Workflow), String> {
    let path = find_workflow(name)?;
    let workflow = load_workflow(&path)?;
    Ok((path, workflow))
}

fn workflow_or_exit(name: &str) -> (PathBuf, pleiades_agent_workflow::Workflow) {
    resolve_workflow(name).unwrap_or_else(|e| {
        eprintln!("\x1b[1;31m✗\x1b[0m {e}");
        std::process::exit(1);
    })
}

fn handle_workflow_list() {
    let mut workflows = Vec::new();
    for dir in workflow_dirs() {
        let Ok(entries) = std::fs::read_dir(dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if matches!(
                path.extension().and_then(|e| e.to_str()),
                Some("toml" | "yaml" | "yml" | "json")
            ) {
                workflows.push(path);
            }
        }
    }
    workflows.sort();
    if workflows.is_empty() {
        println!("No workflows found.");
        return;
    }
    for path in workflows {
        match load_workflow(&path) {
            Ok(workflow) => println!(
                "  \x1b[1;32m{:<24}\x1b[0m {}",
                workflow.name,
                workflow.description.as_deref().unwrap_or("")
            ),
            Err(error) => println!(
                "  \x1b[1;31m{:<24}\x1b[0m invalid: {}",
                path.display(),
                error
            ),
        }
    }
}

fn handle_workflow_show(name: &str) {
    let (path, workflow) = workflow_or_exit(name);
    println!("Name:        {}", workflow.name);
    println!(
        "Description: {}",
        workflow.description.as_deref().unwrap_or("")
    );
    println!("Path:        {}", path.display());
    if let Some(variables) = &workflow.variables {
        println!("Variables:   {}", variables.join(", "));
    }
    println!("Steps:");
    for (index, step) in workflow.steps.iter().enumerate() {
        let mode = if step.is_parallel() {
            "parallel"
        } else {
            "sequential"
        };
        println!(
            "  {}. {} [{}] — {}",
            index + 1,
            step.name,
            mode,
            step.command
        );
    }
}

fn handle_workflow_validate(name: &str) {
    let (path, workflow) = workflow_or_exit(name);
    match workflow.validate() {
        Ok(()) => println!("\x1b[1;32m✓\x1b[0m {} is valid", path.display()),
        Err(errors) => {
            for error in errors {
                eprintln!("\x1b[1;31m✗\x1b[0m {error}");
            }
            std::process::exit(1);
        }
    }
}

fn parse_workflow_vars(
    vars: &[String],
) -> Result<std::collections::HashMap<String, String>, String> {
    vars.iter()
        .map(|item| {
            item.split_once('=')
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .ok_or_else(|| format!("invalid variable '{item}', expected key=value"))
        })
        .collect()
}

fn handle_workflow_run(name: &str, vars: &[String]) {
    let (_, workflow) = workflow_or_exit(name);
    let variables = parse_workflow_vars(vars).unwrap_or_else(|e| {
        eprintln!("\x1b[1;31m✗\x1b[0m {e}");
        std::process::exit(1);
    });
    let executor = pleiades_agent_workflow::WorkflowExecutor::new().with_variables(variables);
    let runtime = tokio::runtime::Runtime::new().expect("Tokio runtime");
    match runtime.block_on(executor.execute_detailed(&workflow)) {
        Ok(results) => {
            for result in results {
                let marker = match result.status {
                    pleiades_agent_workflow::StepStatus::Succeeded => "\x1b[1;32m✓\x1b[0m",
                    pleiades_agent_workflow::StepStatus::Skipped => "\x1b[1;33m−\x1b[0m",
                    pleiades_agent_workflow::StepStatus::Failed => "\x1b[1;31m✗\x1b[0m",
                };
                println!("{marker} {} ({:.2?})", result.name, result.duration);
                if !result.stdout.is_empty() {
                    print!("{}", result.stdout);
                }
                if !result.stderr.is_empty() {
                    eprint!("{}", result.stderr);
                }
            }
        }
        Err(error) => {
            eprintln!("\x1b[1;31m✗\x1b[0m {error}");
            std::process::exit(1);
        }
    }
}

fn handle_workflow_create(name: &str, description: Option<String>) {
    if name.is_empty() || name.contains(['/', '\\']) {
        eprintln!("\x1b[1;31m✗\x1b[0m workflow name must be a non-empty file name");
        std::process::exit(1);
    }
    let dir = PathBuf::from(".pleiades/workflows");
    if let Err(error) = std::fs::create_dir_all(&dir) {
        eprintln!("\x1b[1;31m✗\x1b[0m {error}");
        std::process::exit(1);
    }
    let path = dir.join(format!("{name}.toml"));
    let workflow = pleiades_agent_workflow::Workflow {
        name: name.to_string(),
        description,
        variables: Some(vec!["name=world".to_string()]),
        steps: vec![pleiades_agent_workflow::WorkflowStep {
            name: "hello".to_string(),
            command: "printf".to_string(),
            args: Some(vec!["Hello, {{name}}!\\n".to_string()]),
            condition: None,
            parallel: None,
            timeout: Some(30),
            retry: Some(0),
        }],
    };
    let serialized = toml::to_string_pretty(&workflow).expect("workflow serialization");
    let write = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .and_then(|mut file| std::io::Write::write_all(&mut file, serialized.as_bytes()));
    match write {
        Ok(()) => println!("\x1b[1;32m✓\x1b[0m Created {}", path.display()),
        Err(error) => {
            eprintln!(
                "\x1b[1;31m✗\x1b[0m could not create {}: {error}",
                path.display()
            );
            std::process::exit(1);
        }
    }
}

fn configured_git_provider(
    loader: &ConfigLoader,
    provider_override: Option<&str>,
    model_override: Option<&str>,
) -> Result<(std::sync::Arc<dyn pleiades_agent_core::Provider>, String), String> {
    let config = loader
        .load_with_interpolation()
        .map_err(|error| error.to_string())?;
    let requested = provider_override
        .map(str::to_string)
        .or_else(|| config.core.default_provider.clone());
    let mut providers = build_providers_from_config(&config);
    let index = match requested.as_deref() {
        Some(name) => providers
            .iter()
            .position(|provider| provider.name() == name)
            .ok_or_else(|| format!("provider '{name}' is not configured"))?,
        None => {
            if providers.len() == 1 {
                0
            } else {
                return Err("set a default provider or pass --provider".to_string());
            }
        }
    };
    let provider: std::sync::Arc<dyn pleiades_agent_core::Provider> =
        std::sync::Arc::from(providers.swap_remove(index));
    let model = model_override
        .map(str::to_string)
        .or(config.core.default_model)
        .unwrap_or_else(|| provider.default_model().to_string());
    Ok((provider, model))
}

fn git_provider_or_exit(
    loader: &ConfigLoader,
    provider: Option<&str>,
    model: Option<&str>,
) -> (std::sync::Arc<dyn pleiades_agent_core::Provider>, String) {
    configured_git_provider(loader, provider, model).unwrap_or_else(|error| {
        eprintln!("\x1b[1;31m✗\x1b[0m {error}");
        std::process::exit(1);
    })
}

fn run_git_generation<F>(future: F)
where
    F: std::future::Future<Output = Result<String, pleiades_agent_core::Error>>,
{
    let runtime = tokio::runtime::Runtime::new().expect("Tokio runtime");
    match runtime.block_on(future) {
        Ok(output) => println!("{output}"),
        Err(error) => {
            eprintln!("\x1b[1;31m✗\x1b[0m {error}");
            std::process::exit(1);
        }
    }
}

fn handle_git_command(
    loader: &ConfigLoader,
    command: GitCommand,
    provider_name: Option<&str>,
    model_name: Option<&str>,
) {
    if let GitCommand::Diff { staged } = command {
        return run_git_generation(pleiades_agent_git::working_diff(
            std::path::Path::new("."),
            staged,
        ));
    }

    let (provider, model) = git_provider_or_exit(loader, provider_name, model_name);
    match command {
        GitCommand::Commit => {
            run_git_generation(pleiades_agent_git::CommitGenerator::new(provider, model).generate())
        }
        GitCommand::Review { staged } => run_git_generation(
            pleiades_agent_git::ReviewGenerator::new(provider, model)
                .staged(staged)
                .generate(),
        ),
        GitCommand::Summary { base, title } => run_git_generation(
            pleiades_agent_git::PrSummaryGenerator::new(provider, model)
                .base(base)
                .title(title)
                .generate(),
        ),
        GitCommand::Diff { .. } => unreachable!(),
    }
}

fn codex_binary() -> String {
    std::env::var("PLEIADES_CODEX_BIN").unwrap_or_else(|_| "codex".to_string())
}

fn save_setup_config(loader: &ConfigLoader, config: &pleiades_agent_config::Config) {
    let result = if loader.project_dir().exists() {
        loader.save_project(config)
    } else {
        loader.save_global(config)
    };
    if let Err(error) = result {
        eprintln!("\x1b[1;31m✗\x1b[0m Could not save configuration: {error}");
        std::process::exit(1);
    }
}

fn configure_subscription_provider(loader: &ConfigLoader) {
    let mut config = loader.load().unwrap_or_default();
    config.providers.insert(
        "openai-subscription".to_string(),
        pleiades_agent_config::ProviderConfig::default(),
    );
    config.core.default_provider = Some("openai-subscription".to_string());
    config.core.default_model = Some("codex-default".to_string());
    save_setup_config(loader, &config);
    println!("\x1b[1;32m✓\x1b[0m Default provider: openai-subscription");
    println!("\x1b[1;32m✓\x1b[0m Credentials remain managed by the official Codex CLI");
}

fn codex_login_status() -> Result<Option<String>, String> {
    let output = std::process::Command::new(codex_binary())
        .args(["login", "status"])
        .output()
        .map_err(|error| {
            format!("The official Codex CLI is required for ChatGPT subscription access: {error}")
        })?;
    if !output.status.success() {
        return Ok(None);
    }
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    Ok(Some(if stdout.is_empty() { stderr } else { stdout }))
}

fn handle_auth_login(loader: &ConfigLoader, device: bool) {
    match codex_login_status() {
        Ok(Some(status)) if status.to_lowercase().contains("chatgpt") => {
            println!("\x1b[1;32m✓\x1b[0m Codex is already authenticated with ChatGPT")
        }
        Ok(status) => {
            if let Some(status) = status {
                println!("Codex is currently using a different login method: {status}");
                println!("Switching to ChatGPT subscription sign-in...");
                match std::process::Command::new(codex_binary())
                    .arg("logout")
                    .status()
                {
                    Ok(status) if status.success() => {}
                    Ok(_) => {
                        eprintln!(
                            "\x1b[1;31m✗\x1b[0m Could not sign out of the current Codex session"
                        );
                        std::process::exit(1);
                    }
                    Err(error) => {
                        eprintln!("\x1b[1;31m✗\x1b[0m Could not run Codex logout: {error}");
                        std::process::exit(1);
                    }
                }
            }
            println!("Opening the official OpenAI sign-in flow...");
            let mut command = std::process::Command::new(codex_binary());
            command.arg("login");
            if device {
                command.arg("--device-auth");
            }
            match command.status() {
                Ok(status) if status.success() => {
                    println!("\x1b[1;32m✓\x1b[0m OpenAI sign-in complete")
                }
                Ok(_) => {
                    eprintln!("\x1b[1;31m✗\x1b[0m OpenAI sign-in did not complete");
                    std::process::exit(1);
                }
                Err(error) => {
                    eprintln!("\x1b[1;31m✗\x1b[0m Could not start Codex: {error}");
                    eprintln!("Install the official Codex CLI, then run `pleiades auth login`.");
                    std::process::exit(1);
                }
            }
        }
        Err(error) => {
            eprintln!("\x1b[1;31m✗\x1b[0m {error}");
            std::process::exit(1);
        }
    }

    configure_subscription_provider(loader);
    println!("Run `pleiades provider test openai-subscription` to verify access.");
}

fn handle_auth_status() {
    match std::process::Command::new(codex_binary())
        .args(["login", "status"])
        .status()
    {
        Ok(status) if status.success() => {}
        Ok(_) => std::process::exit(1),
        Err(error) => {
            eprintln!("Could not run the official Codex CLI: {error}");
            std::process::exit(1);
        }
    }
}

fn handle_auth_logout() {
    match std::process::Command::new(codex_binary())
        .arg("logout")
        .status()
    {
        Ok(status) if status.success() => println!("Signed out of OpenAI subscription access"),
        Ok(_) => std::process::exit(1),
        Err(error) => {
            eprintln!("Could not run the official Codex CLI: {error}");
            std::process::exit(1);
        }
    }
}

fn configure_api_key_provider(loader: &ConfigLoader) {
    let mut config = loader.load().unwrap_or_default();
    let mut provider = config.providers.get("openai").cloned().unwrap_or_default();
    provider.api_key = Some("${OPENAI_API_KEY}".to_string());
    config.providers.insert("openai".to_string(), provider);
    config.core.default_provider = Some("openai".to_string());
    if config.core.default_model.as_deref() == Some("codex-default")
        || config.core.default_model.is_none()
    {
        config.core.default_model = Some("gpt-4o".to_string());
    }
    save_setup_config(loader, &config);

    println!("\x1b[1;32m✓\x1b[0m Configured usage-based OpenAI API access");
    if std::env::var_os("OPENAI_API_KEY").is_none() {
        println!("Set the key in your shell before starting Pleiades:");
        println!("  export OPENAI_API_KEY=\"your-new-key\"");
    }
    println!("OpenAI API billing is separate from ChatGPT subscriptions.");
    println!("Run `pleiades provider test openai` to verify API access.");
}

fn handle_setup(loader: &ConfigLoader, method: Option<SetupAuth>, device: bool) {
    println!("\x1b[1;36mPleiades setup\x1b[0m");
    println!("Credentials are never copied into Pleiades when using ChatGPT sign-in.\n");

    let method = method.unwrap_or_else(|| {
        println!("Choose how to access OpenAI:");
        println!("  1) ChatGPT subscription (browser sign-in through official Codex CLI)");
        println!("  2) OpenAI Platform API key (usage-based billing)");
        print!("Selection [1]: ");
        use std::io::Write;
        let _ = std::io::stdout().flush();
        let mut selection = String::new();
        let _ = std::io::stdin().read_line(&mut selection);
        if selection.trim() == "2" {
            SetupAuth::ApiKey
        } else {
            SetupAuth::Chatgpt
        }
    });

    match method {
        SetupAuth::Chatgpt => handle_auth_login(loader, device),
        SetupAuth::ApiKey => configure_api_key_provider(loader),
    }
}

fn handle_doctor(loader: &ConfigLoader) {
    println!("\x1b[1;36mPleiades doctor\x1b[0m");
    let config = match loader.load_with_interpolation() {
        Ok(config) => {
            println!("\x1b[1;32m✓\x1b[0m Configuration parses and validates");
            config
        }
        Err(error) => {
            eprintln!("\x1b[1;31m✗\x1b[0m Configuration: {error}");
            eprintln!("Run `pleiades config path` to locate the active files.");
            std::process::exit(1);
        }
    };

    let Some(provider_name) = config.core.default_provider.as_deref() else {
        eprintln!("\x1b[1;31m✗\x1b[0m No default provider is selected");
        eprintln!("Run `pleiades setup`.");
        std::process::exit(1);
    };
    println!("\x1b[1;32m✓\x1b[0m Default provider: {provider_name}");
    println!(
        "\x1b[1;32m✓\x1b[0m Default model: {}",
        config
            .core
            .default_model
            .as_deref()
            .unwrap_or("provider default")
    );

    let Some(provider) = config.providers.get(provider_name) else {
        eprintln!("\x1b[1;31m✗\x1b[0m Provider '{provider_name}' has no configuration");
        eprintln!("Run `pleiades setup`.");
        std::process::exit(1);
    };

    if provider_name == "openai-subscription" {
        match codex_login_status() {
            Ok(Some(status)) if status.to_lowercase().contains("chatgpt") => {
                println!("\x1b[1;32m✓\x1b[0m ChatGPT subscription login is active")
            }
            Ok(Some(status)) => {
                eprintln!("\x1b[1;31m✗\x1b[0m Codex login is not using ChatGPT: {status}");
                eprintln!("Run `pleiades auth login` to switch authentication methods.");
                std::process::exit(1);
            }
            Ok(None) => {
                eprintln!("\x1b[1;31m✗\x1b[0m Codex is installed but not signed in");
                eprintln!("Run `pleiades auth login`.");
                std::process::exit(1);
            }
            Err(error) => {
                eprintln!("\x1b[1;31m✗\x1b[0m {error}");
                std::process::exit(1);
            }
        }
    } else if provider.api_key.as_deref().unwrap_or("").is_empty() {
        eprintln!("\x1b[1;31m✗\x1b[0m Provider '{provider_name}' has no resolved API key");
        eprintln!("Check its environment variable, then restart your shell.");
        std::process::exit(1);
    } else {
        println!("\x1b[1;32m✓\x1b[0m API credential is available (value hidden)");
    }

    println!("\nConfiguration looks ready.");
    println!("For a live request, run `pleiades provider test {provider_name}`.");
}

fn run_interactive_agent(
    mut config: pleiades_agent_config::Config,
    session: Option<&str>,
    provider: Option<&str>,
    model: Option<&str>,
    permission_mode: Option<&str>,
) {
    if let Some(provider) = provider {
        config.core.default_provider = Some(provider.to_string());
    }
    if let Some(model) = model {
        config.core.default_model = Some(model.to_string());
    }

    let runtime = tokio::runtime::Runtime::new().expect("Tokio runtime");
    runtime.block_on(async move {
        let mut app = match pleiades_agent_tui::TuiApp::new(config) {
            Ok(app) => app.with_permission_mode(permission_mode.unwrap_or("agent")),
            Err(error) => {
                eprintln!("Could not start Pleiades: {error}");
                std::process::exit(1);
            }
        };
        if let Some(session_id) = session {
            if let Err(error) = app.with_session(session_id) {
                eprintln!("Error loading session '{session_id}': {error}");
                std::process::exit(1);
            }
        }
        if let Err(error) = app.run().await {
            eprintln!("Agent error: {error}");
            std::process::exit(1);
        }
    });
}

fn main() {
    let cli = Cli::parse();
    let (config_dir, project_dir) = get_config_dirs();
    let loader = ConfigLoader::with_dirs(config_dir, project_dir);

    match cli.command {
        Some(Commands::Chat { ref session }) => {
            let config = match loader.load_with_interpolation() {
                Ok(config) => config,
                Err(error) => {
                    eprintln!("Error loading config: {error}");
                    std::process::exit(1);
                }
            };
            run_interactive_agent(
                config,
                session.as_deref(),
                cli.provider.as_deref(),
                cli.model.as_deref(),
                cli.permission_mode.as_deref(),
            );
        }
        Some(Commands::Setup { auth, device }) => handle_setup(&loader, auth, device),
        Some(Commands::Auth(command)) => match command {
            AuthCommand::Login { device } => handle_auth_login(&loader, device),
            AuthCommand::Status => handle_auth_status(),
            AuthCommand::Logout => handle_auth_logout(),
        },
        Some(Commands::Doctor) => handle_doctor(&loader),
        Some(Commands::Config(cmd)) => match cmd {
            ConfigCommand::Get { key } => handle_config_get(&loader, &key),
            ConfigCommand::Set { key, value } => handle_config_set(&loader, &key, &value),
            ConfigCommand::Edit => handle_config_edit(),
            ConfigCommand::Validate => handle_config_validate(&loader),
            ConfigCommand::Show { raw } => handle_config_show(&loader, raw),
            ConfigCommand::Path => handle_config_path(),
            ConfigCommand::Init { force, format } => handle_config_init(force, &format),
            ConfigCommand::Reset { yes } => handle_config_reset(&loader, yes),
        },
        Some(Commands::Profile(cmd)) => match cmd {
            ProfileCommand::List => handle_profile_list(&loader),
            ProfileCommand::Save { name } => handle_profile_save(&loader, &name),
            ProfileCommand::Load { name } => handle_profile_load(&loader, &name),
            ProfileCommand::Delete { name } => handle_profile_delete(&loader, &name),
            ProfileCommand::Active => handle_profile_active(&loader),
        },
        Some(Commands::Provider(cmd)) => match cmd {
            ProviderCommand::List => handle_provider_list(&loader),
            ProviderCommand::Info { name } => handle_provider_info(&loader, &name),
            ProviderCommand::Test { name, model } => handle_provider_test(&loader, &name, model),
            ProviderCommand::Remove { name } => handle_provider_remove(&loader, &name),
        },
        Some(Commands::Model(cmd)) => match cmd {
            ModelCommand::List { provider, search } => handle_model_list(&loader, provider, search),
            ModelCommand::Info { name } => handle_model_info(&loader, &name),
            ModelCommand::SetDefault { model } => handle_model_set_default(&loader, &model),
            ModelCommand::Alias { alias, model } => handle_model_alias(&loader, &alias, &model),
            ModelCommand::Unalias { alias } => handle_model_unalias(&loader, &alias),
            ModelCommand::Discover => handle_model_discover(&loader),
        },
        Some(Commands::Session(cmd)) => match cmd {
            SessionCommand::List => handle_session_list(&loader),
            SessionCommand::Show { id } => handle_session_show(&loader, &id),
            SessionCommand::Delete { id } => handle_session_delete(&loader, &id),
            SessionCommand::Export { id, format, output } => {
                handle_session_export(&loader, &id, &format, output)
            }
            SessionCommand::Path => handle_session_path(&loader),
        },
        Some(Commands::Tool(cmd)) => match cmd {
            ToolCommand::List => handle_tool_list(&loader),
            ToolCommand::Info { name } => handle_tool_info(&loader, &name),
            ToolCommand::Call { name, input } => handle_tool_call(&loader, &name, &input),
        },
        Some(Commands::Plugin(cmd)) => match cmd {
            PluginCommand::List => handle_plugin_list(&loader),
            PluginCommand::Install { path } => handle_plugin_install(&loader, &path),
            PluginCommand::Uninstall { id } => handle_plugin_uninstall(&loader, &id),
            PluginCommand::Enable { id } => handle_plugin_enable(&loader, &id),
            PluginCommand::Disable { id } => handle_plugin_disable(&loader, &id),
        },
        Some(Commands::Prompt(cmd)) => match cmd {
            PromptCommand::List => handle_prompt_list(),
            PromptCommand::Show { name } => handle_prompt_show(&name),
            PromptCommand::Render { name, vars } => handle_prompt_render(&name, &vars),
            PromptCommand::Save {
                name,
                description,
                template,
            } => handle_prompt_save(&name, &description, &template),
        },
        Some(Commands::Workflow(cmd)) => match cmd {
            WorkflowCommand::List => handle_workflow_list(),
            WorkflowCommand::Run { name, vars } => handle_workflow_run(&name, &vars),
            WorkflowCommand::Show { name } => handle_workflow_show(&name),
            WorkflowCommand::Validate { name } => handle_workflow_validate(&name),
            WorkflowCommand::Create { name, description } => {
                handle_workflow_create(&name, description)
            }
        },
        Some(Commands::Git(cmd)) => {
            handle_git_command(&loader, cmd, cli.provider.as_deref(), cli.model.as_deref())
        }
        Some(Commands::Repl { session }) => {
            let config = match loader.load_with_interpolation() {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Error loading config: {}", e);
                    std::process::exit(1);
                }
            };
            run_interactive_agent(
                config,
                session.as_deref(),
                cli.provider.as_deref(),
                cli.model.as_deref(),
                cli.permission_mode.as_deref(),
            );
        }
        None => {
            if cli.chat {
                let config = match loader.load_with_interpolation() {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("Error loading config: {}", e);
                        std::process::exit(1);
                    }
                };
                run_interactive_agent(
                    config,
                    cli.session.as_deref(),
                    cli.provider.as_deref(),
                    cli.model.as_deref(),
                    cli.permission_mode.as_deref(),
                );
                return;
            }

            if let Some(args) = cli.prompt {
                let prompt = args.join(" ");
                let mut config = match loader.load_with_interpolation() {
                    Ok(config) => config,
                    Err(error) => {
                        eprintln!("Error loading config: {error}");
                        std::process::exit(1);
                    }
                };
                if let Some(model) = cli.model {
                    config.core.default_model = Some(model);
                }
                if let Some(provider) = cli.provider {
                    config.core.default_provider = Some(provider);
                }

                let runtime = tokio::runtime::Runtime::new().expect("Tokio runtime");
                let mut repl =
                    repl::Repl::new(config).with_permission_mode(cli.permission_mode.as_deref());
                if let Err(error) = runtime.block_on(repl.run_once(&prompt)) {
                    eprintln!("Error: {error}");
                    std::process::exit(1);
                }
                return;
            }

            let config = match loader.load_with_interpolation() {
                Ok(config) => config,
                Err(error) => {
                    eprintln!("Configuration error: {error}");
                    eprintln!("Run `pleiades setup` to repair configuration.");
                    std::process::exit(1);
                }
            };
            let configured = config
                .core
                .default_provider
                .as_ref()
                .is_some_and(|provider| config.providers.contains_key(provider));
            if !configured {
                handle_setup(&loader, None, false);
                println!("\nSetup complete. Run `pleiades` again to start chatting.");
                return;
            }

            run_interactive_agent(
                config,
                cli.session.as_deref(),
                cli.provider.as_deref(),
                cli.model.as_deref(),
                cli.permission_mode.as_deref(),
            );
        }
    }
}

/// Get a nested config value by dot-separated key path.
fn get_nested_value(config: &pleiades_agent_config::Config, key: &str) -> Option<String> {
    let parts: Vec<&str> = key.splitn(3, '.').collect();

    match parts.as_slice() {
        ["core", field] => get_core_field(config, field),
        ["providers", name, field] => get_provider_field(config, name, field),
        ["models", field] => get_models_field(config, field),
        ["session", field] => get_session_field(config, field),
        ["display", field] => get_display_field(config, field),
        ["agent", field] => get_agent_field(config, field),
        ["plugins", field] => get_plugins_field(config, field),
        ["permissions", field] => get_permissions_field(config, field),
        _ => None,
    }
}

fn get_core_field(config: &pleiades_agent_config::Config, field: &str) -> Option<String> {
    match field {
        "default_provider" => config.core.default_provider.clone(),
        "default_model" => config.core.default_model.clone(),
        "theme" => config.core.theme.clone(),
        "verbose" => Some(config.core.verbose.to_string()),
        "debug" => Some(config.core.debug.to_string()),
        "max_tokens" => config.core.max_tokens.map(|v| v.to_string()),
        "temperature" => config.core.temperature.map(|v| v.to_string()),
        "auto_update" => Some(config.core.auto_update.to_string()),
        "log_level" => Some(config.core.log_level.clone()),
        _ => None,
    }
}

fn get_provider_field(
    config: &pleiades_agent_config::Config,
    name: &str,
    field: &str,
) -> Option<String> {
    let provider = config.providers.get(name)?;
    match field {
        "api_key" => provider
            .api_key
            .clone()
            .map(|v| pleiades_agent_config::env_interpolate::mask_secrets(&v)),
        "base_url" => provider.base_url.clone(),
        "organization_id" => provider.organization_id.clone(),
        "max_retries" => Some(provider.max_retries.to_string()),
        "timeout_secs" => Some(provider.timeout_secs.to_string()),
        _ => None,
    }
}

fn get_models_field(config: &pleiades_agent_config::Config, field: &str) -> Option<String> {
    match field {
        "default" => config.models.default.clone(),
        _ => None,
    }
}

fn get_session_field(config: &pleiades_agent_config::Config, field: &str) -> Option<String> {
    match field {
        "context_size" => Some(config.session.context_size.to_string()),
        "auto_save_interval_secs" => config
            .session
            .auto_save_interval_secs
            .map(|v| v.to_string()),
        "max_concurrent" => Some(config.session.max_concurrent.to_string()),
        "compress_history" => Some(config.session.compress_history.to_string()),
        _ => None,
    }
}

fn get_display_field(config: &pleiades_agent_config::Config, field: &str) -> Option<String> {
    match field {
        "style" => Some(config.display.style.clone()),
        "syntax_highlighting" => Some(config.display.syntax_highlighting.to_string()),
        "show_token_usage" => Some(config.display.show_token_usage.to_string()),
        "show_timing" => Some(config.display.show_timing.to_string()),
        "output_width" => Some(config.display.output_width.to_string()),
        "show_progress" => Some(config.display.show_progress.to_string()),
        _ => None,
    }
}

fn get_agent_field(config: &pleiades_agent_config::Config, field: &str) -> Option<String> {
    match field {
        "default_persona" => config.agent.default_persona.clone(),
        "max_tool_iterations" => Some(config.agent.max_tool_iterations.to_string()),
        "auto_edit" => Some(config.agent.auto_edit.to_string()),
        _ => None,
    }
}

fn get_plugins_field(config: &pleiades_agent_config::Config, field: &str) -> Option<String> {
    match field {
        "sandbox" => Some(config.plugins.sandbox.to_string()),
        _ => None,
    }
}

fn get_permissions_field(config: &pleiades_agent_config::Config, field: &str) -> Option<String> {
    match field {
        "ask_always" => Some(config.permissions.ask_always.to_string()),
        "grant_duration_minutes" => Some(config.permissions.grant_duration_minutes.to_string()),
        _ => None,
    }
}

/// Set a nested config value by dot-separated key path.
fn set_nested_value(config: &mut pleiades_agent_config::Config, key: &str, value: &str) {
    let parts: Vec<&str> = key.splitn(3, '.').collect();

    match parts.as_slice() {
        ["core", field] => set_core_field(config, field, value),
        ["providers", name, field] => set_provider_field(config, name, field, value),
        ["session", field] => set_session_field(config, field, value),
        ["display", field] => set_display_field(config, field, value),
        ["agent", field] => set_agent_field(config, field, value),
        ["permissions", field] => set_permissions_field(config, field, value),
        ["models", field] => set_models_field(config, field, value),
        _ => eprintln!("Warning: unknown key '{}'", key),
    }
}

fn set_core_field(config: &mut pleiades_agent_config::Config, field: &str, value: &str) {
    match field {
        "default_provider" => config.core.default_provider = Some(value.to_string()),
        "default_model" => config.core.default_model = Some(value.to_string()),
        "theme" => config.core.theme = Some(value.to_string()),
        "verbose" => config.core.verbose = value == "true" || value == "1",
        "debug" => config.core.debug = value == "true" || value == "1",
        "max_tokens" => {
            config.core.max_tokens = value.parse().ok();
        }
        "temperature" => {
            config.core.temperature = value.parse().ok();
        }
        "auto_update" => {
            config.core.auto_update = value == "true" || value == "1";
        }
        "log_level" => {
            config.core.log_level = value.to_string();
        }
        _ => eprintln!("Warning: unknown core field '{}'", field),
    }
}

fn set_provider_field(
    config: &mut pleiades_agent_config::Config,
    name: &str,
    field: &str,
    value: &str,
) {
    let provider = config.providers.entry(name.to_string()).or_default();
    match field {
        "api_key" => provider.api_key = Some(value.to_string()),
        "base_url" => provider.base_url = Some(value.to_string()),
        "organization_id" => provider.organization_id = Some(value.to_string()),
        "max_retries" => {
            provider.max_retries = value.parse().unwrap_or(3);
        }
        "timeout_secs" => {
            provider.timeout_secs = value.parse().unwrap_or(120);
        }
        _ => eprintln!("Warning: unknown provider field '{}'", field),
    }
}

fn set_session_field(config: &mut pleiades_agent_config::Config, field: &str, value: &str) {
    match field {
        "context_size" => {
            config.session.context_size = value.parse().unwrap_or(100);
        }
        "auto_save_interval_secs" => {
            config.session.auto_save_interval_secs = value.parse().ok();
        }
        "max_concurrent" => {
            config.session.max_concurrent = value.parse().unwrap_or(10);
        }
        "compress_history" => {
            config.session.compress_history = value == "true";
        }
        _ => eprintln!("Warning: unknown session field '{}'", field),
    }
}

fn set_display_field(config: &mut pleiades_agent_config::Config, field: &str, value: &str) {
    match field {
        "style" => {
            config.display.style = value.to_string();
        }
        "syntax_highlighting" => {
            config.display.syntax_highlighting = value == "true";
        }
        "show_token_usage" => {
            config.display.show_token_usage = value == "true";
        }
        "show_timing" => {
            config.display.show_timing = value == "true";
        }
        "output_width" => {
            config.display.output_width = value.parse().unwrap_or(0);
        }
        "show_progress" => {
            config.display.show_progress = value == "true";
        }
        _ => eprintln!("Warning: unknown display field '{}'", field),
    }
}

fn set_agent_field(config: &mut pleiades_agent_config::Config, field: &str, value: &str) {
    match field {
        "default_persona" => {
            config.agent.default_persona = Some(value.to_string());
        }
        "max_tool_iterations" => {
            config.agent.max_tool_iterations = value.parse().unwrap_or(25);
        }
        "auto_edit" => {
            config.agent.auto_edit = value == "true";
        }
        _ => eprintln!("Warning: unknown agent field '{}'", field),
    }
}

fn set_permissions_field(config: &mut pleiades_agent_config::Config, field: &str, value: &str) {
    match field {
        "ask_always" => {
            config.permissions.ask_always = value == "true";
        }
        "grant_duration_minutes" => {
            config.permissions.grant_duration_minutes = value.parse().unwrap_or(60);
        }
        _ => eprintln!("Warning: unknown permissions field '{}'", field),
    }
}

fn set_models_field(config: &mut pleiades_agent_config::Config, field: &str, value: &str) {
    match field {
        "default" => {
            config.models.default = Some(value.to_string());
        }
        _ => eprintln!("Warning: unknown models field '{}'", field),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_core_value() {
        let config = pleiades_agent_config::Config::default();
        assert_eq!(
            get_nested_value(&config, "core.max_tokens"),
            Some("4096".to_string())
        );
        assert_eq!(
            get_nested_value(&config, "core.verbose"),
            Some("false".to_string())
        );
    }

    #[test]
    fn test_get_nonexistent_key() {
        let config = pleiades_agent_config::Config::default();
        assert_eq!(get_nested_value(&config, "nonexistent.key"), None);
    }

    #[test]
    fn test_set_core_value() {
        let mut config = pleiades_agent_config::Config::default();
        set_nested_value(&mut config, "core.max_tokens", "8192");
        assert_eq!(config.core.max_tokens, Some(8192));
    }

    #[test]
    fn test_set_provider_value() {
        let mut config = pleiades_agent_config::Config::default();
        set_nested_value(&mut config, "providers.anthropic.api_key", "sk-test");
        assert_eq!(
            config.providers.get("anthropic").unwrap().api_key,
            Some("sk-test".to_string())
        );
    }

    #[test]
    fn test_roundtrip() {
        let mut config = pleiades_agent_config::Config::default();
        set_nested_value(&mut config, "core.temperature", "0.8");
        let got = get_nested_value(&config, "core.temperature");
        assert_eq!(got, Some("0.8".to_string()));
    }
}
