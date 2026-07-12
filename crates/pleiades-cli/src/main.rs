use std::path::PathBuf;

use clap::{Parser, Subcommand};

use pleiades_config::loader::ConfigLoader;
use pleiades_config::profile::ProfileManager;
use pleiades_config::validate;

#[derive(Parser)]
#[command(
    name = "pleiades",
    version = "0.1.0",
    about = "A next-generation, provider-agnostic terminal AI assistant",
    long_about = "Pleiades is a terminal AI assistant that supports multiple AI providers, \
                  extensible plugins, and a beautiful terminal interface.",
    subcommand_required = false,
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Start an interactive chat session
    #[arg(short, long)]
    chat: bool,

    /// One-shot prompt mode
    #[arg(allow_hyphen_values = true, trailing_var_arg = true)]
    prompt: Option<Vec<String>>,

    /// Model to use
    #[arg(short, long, global = true)]
    model: Option<String>,

    /// Provider to use
    #[arg(short = 'P', long, global = true)]
    provider: Option<String>,

    /// Permission mode
    #[arg(long, global = true)]
    permission_mode: Option<String>,

    /// Verbose output
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage configuration
    #[command(subcommand)]
    Config(ConfigCommand),

    /// Manage profiles
    #[command(subcommand)]
    Profile(ProfileCommand),

    /// Manage providers
    #[command(subcommand)]
    Provider(ProviderCommand),
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
            pleiades_config::Config::default()
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

    let config_path = paths.iter().find(|p| p.exists()).cloned()
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
                provider.api_key = Some(pleiades_config::env_interpolate::mask_secrets(key));
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
        eprintln!("Invalid format '{}'. Must be one of: {}", format, valid_formats.join(", "));
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
    let config = pleiades_config::Config::default();

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

    match loader.save_project(&pleiades_config::Config::default()) {
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
        println!("Use 'pleiades config set providers.<name>.api_key <key>' to add one.");
        return;
    }

    for (name, pc) in &config.providers {
        let has_key = pc.api_key.is_some() && !pc.api_key.as_deref().unwrap_or("").is_empty();
        let key_display = if has_key {
            if let Some(ref key) = pc.api_key {
                pleiades_config::env_interpolate::mask_secrets(key)
            } else {
                "not set".to_string()
            }
        } else {
            "not set".to_string()
        };

        let base_url = pc.base_url.as_deref().unwrap_or("(default)");
        println!("  {}:", name);
        println!("    API Key: {}", key_display);
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

    let secret_manager = pleiades_config::SecretManager::new();
    let env_var = secret_manager.expected_env_var(name).unwrap_or("(none)");

    println!("Provider: {}", name);
    println!("  API Key: {}", pc.api_key.as_ref().map(|k| pleiades_config::env_interpolate::mask_secrets(k)).unwrap_or_else(|| "not set".to_string()));
    println!("  Base URL: {}", pc.base_url.as_deref().unwrap_or("(default)"));
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

    let api_key = match pc.api_key.as_deref() {
        Some(k) if !k.is_empty() => k.to_string(),
        _ => {
            eprintln!("No API key configured for '{}'", name);
            std::process::exit(1);
        }
    };
    let base_url = pc.base_url.clone().unwrap_or_default();

    let provider: Box<dyn pleiades_core::Provider> = match name {
        "anthropic" => {
            if base_url.is_empty() {
                Box::new(pleiades_providers::anthropic::AnthropicProvider::new(api_key))
            } else {
                Box::new(pleiades_providers::anthropic::AnthropicProvider::with_base_url(api_key, base_url))
            }
        }
        "openai" => {
            if base_url.is_empty() {
                Box::new(pleiades_providers::openai::OpenAIProvider::new(api_key))
            } else {
                Box::new(pleiades_providers::openai::OpenAIProvider::with_base_url(api_key, base_url))
            }
        }
        _ => {
            let display = name;
            let model_name = model.clone().unwrap_or_else(|| "gpt-4o".to_string());
            Box::new(pleiades_providers::openai_compat::OpenAICompatibleProvider::new(
                name, display, api_key, base_url, model_name,
            ))
        }
    };

    let model_name = model.clone().unwrap_or_else(|| provider.default_model().to_string());
    println!("Testing provider '{}' with model '{}'...", name, model_name);

    let rt = match tokio::runtime::Runtime::new() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to create runtime: {}", e);
            std::process::exit(1);
        }
    };

    rt.block_on(async {
        match provider.chat_stream(pleiades_core::provider::ChatRequest {
            model: model_name,
            messages: vec![pleiades_core::conversation::Message::user("Respond with exactly: Hello from Pleiades!")],
            system_prompt: None,
            temperature: Some(0.0),
            top_p: None,
            max_tokens: Some(50),
            stop: None,
            tools: None,
        }).await {
            Ok(mut rx) => {
                println!("  Connection successful! Response:");
                print!("  ");
                while let Some(event) = rx.recv().await {
                    match event {
                        pleiades_core::provider::StreamEvent::Token(t) => print!("{}", t),
                        pleiades_core::provider::StreamEvent::Done { .. } => {
                            println!();
                            println!("  ✓ Streaming works");
                            break;
                        }
                        pleiades_core::provider::StreamEvent::Error { message, .. } => {
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

fn main() {
    let cli = Cli::parse();
    let (config_dir, project_dir) = get_config_dirs();
    let loader = ConfigLoader::with_dirs(config_dir, project_dir);

    match cli.command {
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
        None => {
            if cli.chat {
                println!("Chat mode will be available in Milestone 5");
                return;
            }

            if let Some(args) = cli.prompt {
                let prompt = args.join(" ");
                println!("Processing: {}", prompt);
                println!("Prompt execution will be available in Milestone 5");
                return;
            }

            println!();
            println!("Usage: pleiades [OPTIONS] [PROMPT]...");
            println!();
            println!("Run 'pleiades --help' for more information.");
        }
    }
}

/// Get a nested config value by dot-separated key path.
fn get_nested_value(config: &pleiades_config::Config, key: &str) -> Option<String> {
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

fn get_core_field(config: &pleiades_config::Config, field: &str) -> Option<String> {
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

fn get_provider_field(config: &pleiades_config::Config, name: &str, field: &str) -> Option<String> {
    let provider = config.providers.get(name)?;
    match field {
        "api_key" => provider.api_key.clone().map(|v| pleiades_config::env_interpolate::mask_secrets(&v)),
        "base_url" => provider.base_url.clone(),
        "organization_id" => provider.organization_id.clone(),
        "max_retries" => Some(provider.max_retries.to_string()),
        "timeout_secs" => Some(provider.timeout_secs.to_string()),
        _ => None,
    }
}

fn get_models_field(config: &pleiades_config::Config, field: &str) -> Option<String> {
    match field {
        "default" => config.models.default.clone(),
        _ => None,
    }
}

fn get_session_field(config: &pleiades_config::Config, field: &str) -> Option<String> {
    match field {
        "context_size" => Some(config.session.context_size.to_string()),
        "auto_save_interval_secs" => config.session.auto_save_interval_secs.map(|v| v.to_string()),
        "max_concurrent" => Some(config.session.max_concurrent.to_string()),
        "compress_history" => Some(config.session.compress_history.to_string()),
        _ => None,
    }
}

fn get_display_field(config: &pleiades_config::Config, field: &str) -> Option<String> {
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

fn get_agent_field(config: &pleiades_config::Config, field: &str) -> Option<String> {
    match field {
        "default_persona" => config.agent.default_persona.clone(),
        "max_tool_iterations" => Some(config.agent.max_tool_iterations.to_string()),
        "auto_edit" => Some(config.agent.auto_edit.to_string()),
        _ => None,
    }
}

fn get_plugins_field(config: &pleiades_config::Config, field: &str) -> Option<String> {
    match field {
        "sandbox" => Some(config.plugins.sandbox.to_string()),
        _ => None,
    }
}

fn get_permissions_field(config: &pleiades_config::Config, field: &str) -> Option<String> {
    match field {
        "ask_always" => Some(config.permissions.ask_always.to_string()),
        "grant_duration_minutes" => Some(config.permissions.grant_duration_minutes.to_string()),
        _ => None,
    }
}

/// Set a nested config value by dot-separated key path.
fn set_nested_value(config: &mut pleiades_config::Config, key: &str, value: &str) {
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

fn set_core_field(config: &mut pleiades_config::Config, field: &str, value: &str) {
    match field {
        "default_provider" => config.core.default_provider = Some(value.to_string()),
        "default_model" => config.core.default_model = Some(value.to_string()),
        "theme" => config.core.theme = Some(value.to_string()),
        "verbose" => config.core.verbose = value == "true" || value == "1",
        "debug" => config.core.debug = value == "true" || value == "1",
        "max_tokens" => { config.core.max_tokens = value.parse().ok(); }
        "temperature" => { config.core.temperature = value.parse().ok(); }
        "auto_update" => { config.core.auto_update = value == "true" || value == "1"; }
        "log_level" => { config.core.log_level = value.to_string(); }
        _ => eprintln!("Warning: unknown core field '{}'", field),
    }
}

fn set_provider_field(config: &mut pleiades_config::Config, name: &str, field: &str, value: &str) {
    let provider = config.providers.entry(name.to_string())
        .or_default();
    match field {
        "api_key" => provider.api_key = Some(value.to_string()),
        "base_url" => provider.base_url = Some(value.to_string()),
        "organization_id" => provider.organization_id = Some(value.to_string()),
        "max_retries" => { provider.max_retries = value.parse().unwrap_or(3); }
        "timeout_secs" => { provider.timeout_secs = value.parse().unwrap_or(120); }
        _ => eprintln!("Warning: unknown provider field '{}'", field),
    }
}

fn set_session_field(config: &mut pleiades_config::Config, field: &str, value: &str) {
    match field {
        "context_size" => { config.session.context_size = value.parse().unwrap_or(100); }
        "auto_save_interval_secs" => { config.session.auto_save_interval_secs = value.parse().ok(); }
        "max_concurrent" => { config.session.max_concurrent = value.parse().unwrap_or(10); }
        "compress_history" => { config.session.compress_history = value == "true"; }
        _ => eprintln!("Warning: unknown session field '{}'", field),
    }
}

fn set_display_field(config: &mut pleiades_config::Config, field: &str, value: &str) {
    match field {
        "style" => { config.display.style = value.to_string(); }
        "syntax_highlighting" => { config.display.syntax_highlighting = value == "true"; }
        "show_token_usage" => { config.display.show_token_usage = value == "true"; }
        "show_timing" => { config.display.show_timing = value == "true"; }
        "output_width" => { config.display.output_width = value.parse().unwrap_or(0); }
        "show_progress" => { config.display.show_progress = value == "true"; }
        _ => eprintln!("Warning: unknown display field '{}'", field),
    }
}

fn set_agent_field(config: &mut pleiades_config::Config, field: &str, value: &str) {
    match field {
        "default_persona" => { config.agent.default_persona = Some(value.to_string()); }
        "max_tool_iterations" => { config.agent.max_tool_iterations = value.parse().unwrap_or(25); }
        "auto_edit" => { config.agent.auto_edit = value == "true"; }
        _ => eprintln!("Warning: unknown agent field '{}'", field),
    }
}

fn set_permissions_field(config: &mut pleiades_config::Config, field: &str, value: &str) {
    match field {
        "ask_always" => { config.permissions.ask_always = value == "true"; }
        "grant_duration_minutes" => { config.permissions.grant_duration_minutes = value.parse().unwrap_or(60); }
        _ => eprintln!("Warning: unknown permissions field '{}'", field),
    }
}

fn set_models_field(config: &mut pleiades_config::Config, field: &str, value: &str) {
    match field {
        "default" => { config.models.default = Some(value.to_string()); }
        _ => eprintln!("Warning: unknown models field '{}'", field),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_core_value() {
        let config = pleiades_config::Config::default();
        assert_eq!(get_nested_value(&config, "core.max_tokens"), Some("4096".to_string()));
        assert_eq!(get_nested_value(&config, "core.verbose"), Some("false".to_string()));
    }

    #[test]
    fn test_get_nonexistent_key() {
        let config = pleiades_config::Config::default();
        assert_eq!(get_nested_value(&config, "nonexistent.key"), None);
    }

    #[test]
    fn test_set_core_value() {
        let mut config = pleiades_config::Config::default();
        set_nested_value(&mut config, "core.max_tokens", "8192");
        assert_eq!(config.core.max_tokens, Some(8192));
    }

    #[test]
    fn test_set_provider_value() {
        let mut config = pleiades_config::Config::default();
        set_nested_value(&mut config, "providers.anthropic.api_key", "sk-test");
        assert_eq!(
            config.providers.get("anthropic").unwrap().api_key,
            Some("sk-test".to_string())
        );
    }

    #[test]
    fn test_roundtrip() {
        let mut config = pleiades_config::Config::default();
        set_nested_value(&mut config, "core.temperature", "0.8");
        let got = get_nested_value(&config, "core.temperature");
        assert_eq!(got, Some("0.8".to_string()));
    }
}
