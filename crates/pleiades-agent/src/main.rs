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

    /// Agent mode: plan, agent, auto, or yolo
    #[arg(
        long,
        global = true,
        hide_possible_values = true,
        value_parser = ["plan", "agent", "auto", "yolo", "unrestricted", "read-only", "workspace-write", "danger-full-access"]
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

    /// Manage MCP servers
    #[command(subcommand)]
    Mcp(McpCommand),

    /// Manage reusable skills
    #[command(subcommand)]
    Skills(SkillsCommand),

    /// Manage permission rules
    #[command(subcommand)]
    Permissions(PermissionsCommand),

    /// Manage and render prompt templates
    #[command(subcommand)]
    Prompt(PromptCommand),

    /// Manage and run workflows
    #[command(subcommand)]
    Workflow(WorkflowCommand),

    /// AI-assisted Git operations
    #[command(subcommand)]
    Git(GitCommand),

    /// Inspect language-service diagnostics and symbols
    #[command(subcommand)]
    Lsp(LspCommand),

    /// Manage live workspace background processes
    #[command(subcommand)]
    Process(ProcessCommand),

    /// Run browser verification from the live workspace
    #[command(subcommand)]
    Browser(BrowserCommand),

    /// Detect and run project command recipes
    #[command(subcommand)]
    Project(ProjectCommand),

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

    /// Add or remove a model from favorites
    Favorite {
        /// Model identifier
        name: String,
    },

    /// Show favorite models and reasoning preference
    Favorites,

    /// Set preferred reasoning effort
    Reasoning {
        /// One of minimal, low, medium, or high
        level: String,
    },

    /// Discover models from configured providers
    Discover,
}

#[derive(Subcommand)]
enum SessionCommand {
    /// List saved sessions
    List,

    /// Search saved sessions
    Search {
        /// Query matched against title, provider, model, tags, and messages
        query: String,
    },

    /// Show session details
    Show {
        /// Session ID
        id: String,
    },

    /// Rename a session
    Rename {
        /// Session ID or unique prefix
        id: String,
        /// New session title
        name: String,
    },

    /// Fork a session into a new session ID
    Fork {
        /// Session ID or unique prefix
        id: Option<String>,
    },

    /// Print how to resume a session in the live workspace
    Resume {
        /// Session ID or unique prefix
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

    /// Use `/session ephemeral on|off` in the live workspace
    Ephemeral {
        /// on or off
        state: String,
    },
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

    /// Show plugin details and requested permissions
    Info {
        /// Plugin ID
        id: String,
    },

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

    /// Update an external plugin from its original local source
    Update {
        /// Plugin ID
        id: String,
    },

    /// Accept a plugin trust report
    Trust {
        /// Plugin ID
        id: String,
    },

    /// Revoke trust and disable a plugin
    Untrust {
        /// Plugin ID
        id: String,
    },
}

#[derive(Subcommand)]
enum McpCommand {
    /// List configured MCP servers
    List,

    /// Show MCP server details
    Info {
        /// MCP server ID
        id: String,
    },

    /// Open the live MCP manager for interactive setup
    Add,

    /// Remove an MCP server from project configuration
    Remove {
        /// MCP server ID
        id: String,
    },

    /// Enable an MCP server
    Enable {
        /// MCP server ID
        id: String,
    },

    /// Disable an MCP server
    Disable {
        /// MCP server ID
        id: String,
    },

    /// Show configured tool exposure filters
    Tools {
        /// MCP server ID
        id: String,
    },

    /// Inspect one configured tool exposure decision
    ToolInfo {
        /// MCP server ID
        server: String,
        /// MCP tool name
        tool: String,
    },

    /// Reload extension sources
    Reload,

    /// Show live-workspace-only MCP auth guidance
    Auth {
        /// Optional MCP server ID
        id: Option<String>,
    },

    /// Show live-workspace-only MCP logout guidance
    Logout {
        /// Optional MCP server ID
        id: Option<String>,
    },

    /// Show live-workspace-only restart guidance
    Restart {
        /// Optional MCP server ID
        id: Option<String>,
    },

    /// Show live-workspace-only logs guidance
    Logs {
        /// Optional MCP server ID
        id: Option<String>,
    },

    /// Show live-workspace-only debug guidance
    Debug {
        /// Optional MCP server ID
        id: Option<String>,
    },
}

#[derive(Subcommand)]
enum SkillsCommand {
    /// List skills
    List,
    /// Show a skill
    Show { name: String },
    /// Create a project-local skill
    Create { name: String },
    /// Print the file path to edit
    Edit { name: String },
    /// Enable a skill
    Enable { name: String },
    /// Disable a skill
    Disable { name: String },
    /// Reload skill definitions
    Reload,
}

#[derive(Subcommand)]
enum PermissionsCommand {
    /// Show configured permission rules
    Show,

    /// Allow matching bash commands without prompting
    Allow {
        /// Glob pattern matched against each shell command clause
        #[arg(allow_hyphen_values = true, trailing_var_arg = true)]
        pattern: Vec<String>,
    },

    /// Ask before running matching bash commands
    Ask {
        /// Glob pattern matched against each shell command clause
        #[arg(allow_hyphen_values = true, trailing_var_arg = true)]
        pattern: Vec<String>,
    },

    /// Deny matching bash commands
    Deny {
        /// Glob pattern matched against each shell command clause
        #[arg(allow_hyphen_values = true, trailing_var_arg = true)]
        pattern: Vec<String>,
    },

    /// Clear structured and legacy allow/deny permission rules
    Reset,

    /// Evaluate a bash command against configured rules
    Test {
        /// Command to evaluate
        #[arg(allow_hyphen_values = true, trailing_var_arg = true)]
        command: Vec<String>,
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

#[derive(Subcommand)]
enum LspCommand {
    /// Show detected language-service status
    Status,
    /// List detected language-service servers
    Servers,
    /// Restart language-service backends where supported
    Restart,
    /// Run language diagnostics
    Diagnostics,
    /// Search workspace symbols
    Symbols {
        /// Symbol name fragment
        query: String,
    },
}

#[derive(Subcommand)]
enum ProcessCommand {
    /// List live workspace processes
    List,
    /// Start a background process in the live workspace
    Start {
        #[arg(allow_hyphen_values = true, trailing_var_arg = true)]
        command: Vec<String>,
    },
    /// Show captured logs for a process
    Logs { id: String },
    /// Stop a process
    Stop { id: String },
    /// Restart a process
    Restart { id: String },
    /// Attach to process output in the live workspace
    Attach { id: String },
}

#[derive(Subcommand)]
enum BrowserCommand {
    /// Open a URL through the live workspace browser integration
    Open { url: String },
    /// Capture a screenshot of the last opened URL
    Screenshot,
    /// Inspect the last browser report
    Inspect,
    /// Show browser console output
    Console,
    /// Clear browser session state
    Close,
}

#[derive(Subcommand)]
enum ProjectCommand {
    /// Detect likely project commands
    Detect,
    /// List configured and detected project commands
    Commands,
    /// Run one project recipe
    Run { recipe: String },
    /// Run the project verify recipe
    Verify,
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
    let services = pleiades_agent_services::ApplicationServices::with_config_dirs(
        loader.global_dir().to_path_buf(),
        loader.project_dir().to_path_buf(),
    );
    let providers = match services.provider().list() {
        Ok(providers) => providers,
        Err(e) => {
            eprintln!("Error listing providers: {}", e);
            std::process::exit(1);
        }
    };

    if providers.is_empty() {
        println!("No providers configured.");
        println!("Run 'pleiades setup' for guided configuration.");
        return;
    }

    for provider in providers {
        println!("  {}:", provider.name);
        println!("    Authentication: {}", provider.authentication);
        if provider.name != "openai-subscription" {
            println!("    API Key: {}", provider.api_key_display);
        }
        println!("    Base URL: {}", provider.base_url);
        println!();
    }
}

fn handle_provider_info(loader: &ConfigLoader, name: &str) {
    let services = pleiades_agent_services::ApplicationServices::with_config_dirs(
        loader.global_dir().to_path_buf(),
        loader.project_dir().to_path_buf(),
    );
    let provider = match services.provider().info(name) {
        Ok(provider) => provider,
        Err(e) => {
            eprintln!("Error reading provider: {}", e);
            std::process::exit(1);
        }
    };

    println!("Provider: {}", provider.name);
    println!("  Authentication: {}", provider.authentication);
    if provider.name == "openai-subscription" {
        println!("  Credentials: managed by Codex (Pleiades never reads them)");
        println!("  Status command: pleiades auth status");
        return;
    }
    println!("  API Key: {}", provider.api_key_display);
    println!("  Base URL: {}", provider.base_url);
    println!(
        "  Expected Env Var: {}",
        provider.expected_env_var.as_deref().unwrap_or("(none)")
    );
    println!("  Max Retries: {}", provider.max_retries);
    println!("  Timeout: {}s", provider.timeout_secs);
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
    let services = pleiades_agent_services::ApplicationServices::with_config_dirs(
        loader.global_dir().to_path_buf(),
        loader.project_dir().to_path_buf(),
    );
    let runtime = tokio::runtime::Runtime::new().unwrap_or_else(|error| {
        eprintln!("Failed to create runtime: {error}");
        std::process::exit(1);
    });
    let report = runtime
        .block_on(services.provider().test(name, model.as_deref()))
        .unwrap_or_else(|error| {
            eprintln!("Connection failed: {error}");
            std::process::exit(1);
        });
    println!(
        "Provider '{}' connected with model '{}'.",
        report.provider, report.model
    );
    println!("  {}", report.response);
    println!("  ✓ Streaming completed ({})", report.finish_reason);
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

fn model_service(loader: &ConfigLoader) -> pleiades_agent_services::ModelService {
    pleiades_agent_services::ApplicationServices::with_config_dirs(
        loader.global_dir().to_path_buf(),
        loader.project_dir().to_path_buf(),
    )
    .model()
}

fn handle_model_favorite(loader: &ConfigLoader, model: &str) {
    match model_service(loader).favorite(model) {
        Ok(true) => println!("Model '{}' added to favorites", model),
        Ok(false) => println!("Model '{}' removed from favorites", model),
        Err(error) => {
            eprintln!("Error: {error}");
            std::process::exit(1);
        }
    }
}

fn handle_model_favorites(loader: &ConfigLoader) {
    match model_service(loader).preferences() {
        Ok(preferences) => {
            if preferences.favorites.is_empty() {
                println!("No favorite models.");
            } else {
                println!("Favorite models:");
                for model in preferences.favorites {
                    println!("  {model}");
                }
            }
            println!(
                "Reasoning effort: {}",
                preferences
                    .reasoning
                    .as_deref()
                    .unwrap_or("provider default")
            );
        }
        Err(error) => {
            eprintln!("Error: {error}");
            std::process::exit(1);
        }
    }
}

fn handle_model_reasoning(loader: &ConfigLoader, level: &str) {
    match model_service(loader).set_reasoning(level) {
        Ok(()) => println!("Reasoning effort set to '{}'", level.to_ascii_lowercase()),
        Err(error) => {
            eprintln!("Error: {error}");
            std::process::exit(1);
        }
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

fn handle_session_search(loader: &ConfigLoader, query: &str) {
    let config = match loader.load() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error loading config: {}", e);
            std::process::exit(1);
        }
    };

    let store = pleiades_agent_engine::SessionStore::from_config(&config);
    match store.search(query) {
        Ok(sessions) => {
            if sessions.is_empty() {
                println!("No saved sessions matched `{query}`.");
                return;
            }
            println!("Sessions matching `{query}` ({}):", sessions.len());
            for session in sessions {
                let title = session.metadata.title.as_deref().unwrap_or("Untitled");
                let updated = session.metadata.updated_at.format("%Y-%m-%d %H:%M");
                println!("  {}  {}  {}", short_id(&session.id), updated, title);
            }
        }
        Err(e) => {
            eprintln!("Error searching sessions: {}", e);
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

fn handle_session_rename(loader: &ConfigLoader, id: &str, name: &str) {
    let config = match loader.load() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error loading config: {}", e);
            std::process::exit(1);
        }
    };

    let store = pleiades_agent_engine::SessionStore::from_config(&config);
    match store.rename(id, name) {
        Ok(conversation) => println!(
            "Renamed session {} to `{}`",
            short_id(&conversation.id),
            conversation.metadata.title.as_deref().unwrap_or("Untitled")
        ),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn handle_session_fork(loader: &ConfigLoader, id: Option<&str>) {
    let config = match loader.load() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error loading config: {}", e);
            std::process::exit(1);
        }
    };

    let store = pleiades_agent_engine::SessionStore::from_config(&config);
    let source = match id {
        Some(id) => id.to_string(),
        None => match store
            .list()
            .ok()
            .and_then(|sessions| sessions.first().cloned())
        {
            Some(session) => session.id,
            None => {
                eprintln!("Error: no saved session to fork");
                std::process::exit(1);
            }
        },
    };
    match store.fork(&source) {
        Ok(conversation) => println!(
            "Forked session {} from {}",
            short_id(&conversation.id),
            short_id(&source)
        ),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn handle_session_resume(loader: &ConfigLoader, id: &str) {
    let config = match loader.load() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error loading config: {}", e);
            std::process::exit(1);
        }
    };

    let store = pleiades_agent_engine::SessionStore::from_config(&config);
    match store.resolve_id(id) {
        Ok(resolved) => println!("Run `pleiades chat --session {resolved}` to resume."),
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

fn handle_session_ephemeral(state: &str) {
    match state {
        "on" | "true" => {
            println!("Ephemeral mode is process-local.");
            println!("Run `pleiades`, then `/session ephemeral on`.");
        }
        "off" | "false" => {
            println!("Run `pleiades`, then `/session ephemeral off` to re-enable saves.");
        }
        _ => {
            eprintln!("Error: usage: pleiades session ephemeral <on|off>");
            std::process::exit(1);
        }
    }
}

fn short_id(id: &str) -> &str {
    id.get(..8).unwrap_or(id)
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

fn plugin_service(loader: &ConfigLoader) -> pleiades_agent_services::PluginService {
    pleiades_agent_services::ApplicationServices::with_config_dirs(
        loader.global_dir().to_path_buf(),
        loader.project_dir().to_path_buf(),
    )
    .plugin()
}

fn handle_plugin_list(loader: &ConfigLoader) {
    match plugin_service(loader).list() {
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

fn handle_plugin_info(loader: &ConfigLoader, id: &str) {
    match plugin_service(loader).info(id) {
        Ok(plugin) => {
            println!("Plugin: {}", plugin.name);
            println!("  ID:          {}", plugin.id);
            println!("  Version:     {}", plugin.version);
            println!("  Kind:        {}", plugin.kind);
            println!("  Enabled:     {}", plugin.enabled);
            println!("  Source:      {}", plugin.source);
            println!("  Description: {}", plugin.description);
            println!("  Trusted:     {}", plugin.trusted);
            println!("  Trust req.:  {}", plugin.trust_required);
            println!("  Tools:       {}", plugin.tool_count);
            println!("  Hooks:       {}", plugin.has_hooks);
            println!(
                "  Permissions: {}",
                if plugin.permissions.is_empty() {
                    "none".to_string()
                } else {
                    plugin.permissions.join(", ")
                }
            );
            println!(
                "  Exec hooks:  {}",
                cli_list_or_none(&plugin.executable_hooks)
            );
            println!(
                "  Lifecycle:   {}",
                cli_list_or_none(&plugin.lifecycle_commands)
            );
            println!(
                "  Commands:    {}",
                cli_list_or_none(&plugin.custom_commands)
            );
            println!(
                "  Paths:       {}",
                cli_list_or_none(&plugin.requested_paths)
            );
            println!("  Network:     {}", plugin.network_access);
            println!("  Env vars:    {}", cli_list_or_none(&plugin.env_vars));
            println!(
                "  Checksum:    {}",
                plugin.checksum.as_deref().unwrap_or("(none)")
            );
            println!(
                "  Signature:   {}",
                plugin.signature.as_deref().unwrap_or("(none)")
            );
        }
        Err(error) => {
            eprintln!("Error: {error}");
            std::process::exit(1);
        }
    }
}

fn handle_plugin_install(loader: &ConfigLoader, path: &str) {
    match plugin_service(loader).install(path) {
        Ok(outcome) => {
            println!(
                "\x1b[1;32m✓\x1b[0m Plugin installed: {} v{}",
                outcome.id, outcome.version
            );
        }
        Err(e) => {
            eprintln!("\x1b[1;31m✗\x1b[0m Install failed: {}", e);
            std::process::exit(1);
        }
    }
}

fn handle_plugin_uninstall(loader: &ConfigLoader, id: &str) {
    match plugin_service(loader).uninstall(id) {
        Ok(_) => println!("\x1b[1;32m✓\x1b[0m Plugin uninstalled: {}", id),
        Err(e) => {
            eprintln!("\x1b[1;31m✗\x1b[0m Uninstall failed: {}", e);
            std::process::exit(1);
        }
    }
}

fn handle_plugin_enable(loader: &ConfigLoader, id: &str) {
    let service = plugin_service(loader);
    match service.info(id) {
        Ok(plugin) if plugin.trust_required && !plugin.trusted => {
            println!("Plugin `{id}` requires trust before enabling.");
            println!("Review with `pleiades plugin info {id}`.");
            println!("Accept with `pleiades plugin trust {id}`, then run enable again.");
        }
        Ok(_) => match service.enable(id) {
            Ok(_) => println!("\x1b[1;32m✓\x1b[0m Plugin enabled: {}", id),
            Err(e) => {
                eprintln!("\x1b[1;31m✗\x1b[0m Enable failed: {}", e);
                std::process::exit(1);
            }
        },
        Err(e) => {
            eprintln!("\x1b[1;31m✗\x1b[0m Enable failed: {}", e);
            std::process::exit(1);
        }
    }
}

fn handle_plugin_disable(loader: &ConfigLoader, id: &str) {
    match plugin_service(loader).disable(id) {
        Ok(_) => println!("\x1b[1;32m✓\x1b[0m Plugin disabled: {}", id),
        Err(e) => {
            eprintln!("\x1b[1;31m✗\x1b[0m Disable failed: {}", e);
            std::process::exit(1);
        }
    }
}

fn handle_plugin_update(loader: &ConfigLoader, id: &str) {
    match plugin_service(loader).update(id) {
        Ok(outcome) => println!(
            "\x1b[1;32m✓\x1b[0m Plugin updated: {} v{} -> v{}",
            outcome.id, outcome.old_version, outcome.new_version
        ),
        Err(error) => {
            eprintln!("\x1b[1;31m✗\x1b[0m Update failed: {error}");
            std::process::exit(1);
        }
    }
}

fn handle_plugin_trust(loader: &ConfigLoader, id: &str) {
    match plugin_service(loader).trust(id) {
        Ok(_) => println!("\x1b[1;32m✓\x1b[0m Plugin trusted: {}", id),
        Err(error) => {
            eprintln!("\x1b[1;31m✗\x1b[0m Trust failed: {error}");
            std::process::exit(1);
        }
    }
}

fn handle_plugin_untrust(loader: &ConfigLoader, id: &str) {
    match plugin_service(loader).untrust(id) {
        Ok(_) => println!("\x1b[1;32m✓\x1b[0m Plugin untrusted and disabled: {}", id),
        Err(error) => {
            eprintln!("\x1b[1;31m✗\x1b[0m Untrust failed: {error}");
            std::process::exit(1);
        }
    }
}

fn mcp_service(loader: &ConfigLoader) -> pleiades_agent_services::McpService {
    pleiades_agent_services::ApplicationServices::with_config_dirs(
        loader.global_dir().to_path_buf(),
        loader.project_dir().to_path_buf(),
    )
    .mcp()
}

fn handle_mcp_list(loader: &ConfigLoader) {
    match mcp_service(loader).list() {
        Ok(servers) => {
            if servers.is_empty() {
                println!("No MCP servers configured.");
                return;
            }
            for server in servers {
                let status = if server.enabled {
                    "\x1b[1;32menabled\x1b[0m"
                } else {
                    "\x1b[2mdisabled\x1b[0m"
                };
                println!(
                    "  {:<24}  {:<8}  {:<12}  {}",
                    server.id, status, server.health, server.transport
                );
            }
        }
        Err(error) => {
            eprintln!("Error listing MCP servers: {error}");
            std::process::exit(1);
        }
    }
}

fn handle_mcp_info(loader: &ConfigLoader, id: &str) {
    match mcp_service(loader).info(id) {
        Ok(server) => {
            println!("MCP server: {}", server.id);
            println!("  Enabled:   {}", server.enabled);
            println!("  Transport: {}", server.transport);
            println!("  Health:    {}", server.health);
            println!("  Timeout:   {}s", server.timeout_secs);
            println!(
                "  Tools:     {}",
                server
                    .tool_count
                    .map(|count| count.to_string())
                    .unwrap_or_else(|| "not discovered".to_string())
            );
            println!("  Allow:     {}", cli_list_or_all(&server.allowlist));
            println!("  Deny:      {}", cli_list_or_none(&server.denylist));
            println!(
                "  Error:     {}",
                server.last_error.as_deref().unwrap_or("(none)")
            );
        }
        Err(error) => {
            eprintln!("Error: {error}");
            std::process::exit(1);
        }
    }
}

fn handle_mcp_remove(loader: &ConfigLoader, id: &str) {
    match mcp_service(loader).remove(id) {
        Ok(()) => println!("\x1b[1;32m✓\x1b[0m MCP server removed: {id}"),
        Err(error) => {
            eprintln!("\x1b[1;31m✗\x1b[0m Remove failed: {error}");
            std::process::exit(1);
        }
    }
}

fn handle_mcp_enable(loader: &ConfigLoader, id: &str) {
    match mcp_service(loader).enable(id) {
        Ok(()) => println!("\x1b[1;32m✓\x1b[0m MCP server enabled: {id}"),
        Err(error) => {
            eprintln!("\x1b[1;31m✗\x1b[0m Enable failed: {error}");
            std::process::exit(1);
        }
    }
}

fn handle_mcp_disable(loader: &ConfigLoader, id: &str) {
    match mcp_service(loader).disable(id) {
        Ok(()) => println!("\x1b[1;32m✓\x1b[0m MCP server disabled: {id}"),
        Err(error) => {
            eprintln!("\x1b[1;31m✗\x1b[0m Disable failed: {error}");
            std::process::exit(1);
        }
    }
}

fn handle_mcp_tools(loader: &ConfigLoader, id: &str) {
    match mcp_service(loader).tools(id) {
        Ok(tools) => {
            if tools.is_empty() {
                println!("No configured tool filters for MCP server `{id}`.");
                println!("Live schema discovery is not connected yet.");
                return;
            }
            for tool in tools {
                println!(
                    "  {:<30}  exposed={}  schema={}  {}",
                    tool.tool, tool.exposed, tool.schema_available, tool.notes
                );
            }
        }
        Err(error) => {
            eprintln!("Error: {error}");
            std::process::exit(1);
        }
    }
}

fn handle_mcp_tool_info(loader: &ConfigLoader, server: &str, tool: &str) {
    match mcp_service(loader).tool_info(server, tool) {
        Ok(report) => {
            println!("MCP tool: {}/{}", report.server, report.tool);
            println!("  Exposed: {}", report.exposed);
            println!("  Schema:  {}", report.schema_available);
            println!("  Notes:   {}", report.notes);
        }
        Err(error) => {
            eprintln!("Error: {error}");
            std::process::exit(1);
        }
    }
}

fn handle_mcp_reload() {
    println!("MCP configuration will be reloaded by the live workspace runtime.");
    println!("For headless commands, each invocation reads the latest configuration.");
}

fn handle_mcp_live_only(action: &str, id: Option<&str>) {
    match id {
        Some(id) => println!(
            "`pleiades mcp {action} {id}` is managed from the live workspace overlay. Run `pleiades`, then `/mcp {action} {id}`."
        ),
        None => println!(
            "`pleiades mcp {action}` is managed from the live workspace overlay. Run `pleiades`, then `/mcp {action}`."
        ),
    }
}

fn skill_service(loader: &ConfigLoader) -> pleiades_agent_services::SkillService {
    pleiades_agent_services::ApplicationServices::with_config_dirs(
        loader.global_dir().to_path_buf(),
        loader.project_dir().to_path_buf(),
    )
    .skill()
}

fn lsp_service(loader: &ConfigLoader) -> pleiades_agent_lsp::LspService {
    pleiades_agent_services::ApplicationServices::with_config_dirs(
        loader.global_dir().to_path_buf(),
        loader.project_dir().to_path_buf(),
    )
    .lsp()
}

fn handle_skills_list(loader: &ConfigLoader) {
    match skill_service(loader).list() {
        Ok(skills) => {
            if skills.is_empty() {
                println!("No skills configured.");
                println!("Run `pleiades skills create <name>` to create a project-local skill.");
                return;
            }
            for skill in skills {
                println!(
                    "  {:<24}  {:<8}  {:<8}  {}",
                    skill.name,
                    skill.scope,
                    if skill.enabled { "enabled" } else { "disabled" },
                    skill.description
                );
            }
        }
        Err(error) => {
            eprintln!("Error listing skills: {error}");
            std::process::exit(1);
        }
    }
}

fn handle_skills_show(loader: &ConfigLoader, name: &str) {
    match skill_service(loader).show(name) {
        Ok(skill) => {
            println!("Skill: {}", skill.name);
            println!("  Description: {}", skill.description);
            println!("  Scope:       {}", skill.scope);
            println!("  Enabled:     {}", skill.enabled);
            println!("  Permissions: {}", cli_list_or_none(&skill.permissions));
            println!("  Path:        {}", skill.path.display());
            println!();
            println!("{}", skill.instructions);
        }
        Err(error) => {
            eprintln!("Error: {error}");
            std::process::exit(1);
        }
    }
}

fn handle_skills_create(loader: &ConfigLoader, name: &str) {
    match skill_service(loader).create(name) {
        Ok(skill) => {
            println!(
                "\x1b[1;32m✓\x1b[0m Created skill `{}` at {}",
                skill.name,
                skill.path.display()
            );
            println!(
                "Edit the instructions, then run `pleiades skills enable {}`.",
                skill.name
            );
        }
        Err(error) => {
            eprintln!("\x1b[1;31m✗\x1b[0m Create failed: {error}");
            std::process::exit(1);
        }
    }
}

fn handle_skills_edit(loader: &ConfigLoader, name: &str) {
    let service = skill_service(loader);
    let skill = match service.show(name) {
        Ok(skill) => skill,
        Err(error) => {
            eprintln!("Error: {error}");
            std::process::exit(1);
        }
    };

    match std::env::var("EDITOR") {
        Ok(editor) if !editor.trim().is_empty() => {
            match std::process::Command::new(editor).arg(&skill.path).status() {
                Ok(status) if status.success() => println!("Edited skill `{name}`."),
                Ok(status) => {
                    eprintln!("Editor exited with status {status}");
                    std::process::exit(status.code().unwrap_or(1));
                }
                Err(error) => {
                    eprintln!("Could not start editor: {error}");
                    std::process::exit(1);
                }
            }
        }
        _ => {
            println!("Set $EDITOR to edit automatically, or open this file:");
            println!("{}", skill.path.display());
        }
    }
}

fn handle_skills_enable(loader: &ConfigLoader, name: &str) {
    match skill_service(loader).enable(name) {
        Ok(()) => println!("\x1b[1;32m✓\x1b[0m Skill enabled: {name}"),
        Err(error) => {
            eprintln!("\x1b[1;31m✗\x1b[0m Enable failed: {error}");
            std::process::exit(1);
        }
    }
}

fn handle_skills_disable(loader: &ConfigLoader, name: &str) {
    match skill_service(loader).disable(name) {
        Ok(()) => println!("\x1b[1;32m✓\x1b[0m Skill disabled: {name}"),
        Err(error) => {
            eprintln!("\x1b[1;31m✗\x1b[0m Disable failed: {error}");
            std::process::exit(1);
        }
    }
}

fn handle_skills_reload() {
    println!("Skills are reloaded automatically by headless commands.");
    println!("The live workspace reloads skills through `/skills reload`.");
}

fn handle_lsp_command(loader: &ConfigLoader, command: LspCommand) {
    let runtime = tokio::runtime::Runtime::new().expect("Tokio runtime");
    let service = lsp_service(loader);
    match command {
        LspCommand::Status | LspCommand::Servers => match runtime.block_on(service.status()) {
            Ok(report) => print_lsp_status(&report),
            Err(error) => {
                eprintln!("\x1b[1;31m✗\x1b[0m {error}");
                std::process::exit(1);
            }
        },
        LspCommand::Restart => {
            println!("No persistent LSP server process is running in this slice.");
            println!("Diagnostics are executed on demand through `pleiades lsp diagnostics`.");
        }
        LspCommand::Diagnostics => match runtime.block_on(service.diagnostics()) {
            Ok(report) => print_lsp_diagnostics(&report),
            Err(error) => {
                eprintln!("\x1b[1;31m✗\x1b[0m {error}");
                std::process::exit(1);
            }
        },
        LspCommand::Symbols { query } => match runtime.block_on(service.symbols(&query)) {
            Ok(report) => print_lsp_symbols(&report),
            Err(error) => {
                eprintln!("\x1b[1;31m✗\x1b[0m {error}");
                std::process::exit(1);
            }
        },
    }
}

fn handle_process_command(command: ProcessCommand) {
    match command {
        ProcessCommand::List => {
            println!("Background processes are owned by the live workspace runtime.");
            println!("Run `pleiades`, then `/process list`.");
        }
        ProcessCommand::Start { command } => {
            let command = command.join(" ");
            println!(
                "Start background processes from the live workspace so they can be supervised."
            );
            if command.trim().is_empty() {
                println!("Run `pleiades`, then `/process start <command>`.");
            } else {
                println!("Run `pleiades`, then `/process start {command}`.");
            }
        }
        ProcessCommand::Logs { id } => {
            println!("Run `pleiades`, then `/process logs {id}`.");
        }
        ProcessCommand::Stop { id } => {
            println!("Run `pleiades`, then `/process stop {id}`.");
        }
        ProcessCommand::Restart { id } => {
            println!("Run `pleiades`, then `/process restart {id}`.");
        }
        ProcessCommand::Attach { id } => {
            println!("Run `pleiades`, then `/process attach {id}`.");
        }
    }
}

fn handle_browser_command(command: BrowserCommand) {
    match command {
        BrowserCommand::Open { url } => {
            println!(
                "Run browser verification from the live workspace so the browser session can be reused."
            );
            println!("Run `pleiades`, then `/browser open {url}`.");
        }
        BrowserCommand::Screenshot => {
            println!("Run `pleiades`, then `/browser screenshot` after `/browser open <url>`.");
        }
        BrowserCommand::Inspect => {
            println!("Run `pleiades`, then `/browser inspect` after `/browser open <url>`.");
        }
        BrowserCommand::Console => {
            println!("Run `pleiades`, then `/browser console` after `/browser open <url>`.");
        }
        BrowserCommand::Close => {
            println!("Run `pleiades`, then `/browser close`.");
        }
    }
}

fn project_service(loader: &ConfigLoader) -> pleiades_agent_services::ProjectService {
    pleiades_agent_services::ApplicationServices::with_config_dirs(
        loader.global_dir().to_path_buf(),
        loader.project_dir().to_path_buf(),
    )
    .project()
}

fn handle_project_command(loader: &ConfigLoader, command: ProjectCommand) {
    let service = project_service(loader);
    match command {
        ProjectCommand::Detect => {
            let report = service.detect();
            println!("Markers: {}", cli_list_or_none(&report.markers));
            for recipe in report.suggested {
                println!(
                    "  {:<12} {:<18} {}",
                    recipe.name, recipe.source, recipe.command
                );
            }
        }
        ProjectCommand::Commands => match service.commands() {
            Ok(commands) => {
                if commands.is_empty() {
                    println!("No project commands found.");
                }
                for recipe in commands {
                    println!(
                        "  {:<12} {:<24} {}",
                        recipe.name, recipe.source, recipe.command
                    );
                }
            }
            Err(error) => {
                eprintln!("\x1b[1;31m✗\x1b[0m {error}");
                std::process::exit(1);
            }
        },
        ProjectCommand::Run { recipe } => match service.command(&recipe) {
            Ok(recipe) => run_project_recipe(&recipe),
            Err(error) => {
                eprintln!("\x1b[1;31m✗\x1b[0m {error}");
                std::process::exit(1);
            }
        },
        ProjectCommand::Verify => match service.verify_command() {
            Ok(recipe) => run_project_recipe(&recipe),
            Err(error) => {
                eprintln!("\x1b[1;31m✗\x1b[0m {error}");
                std::process::exit(1);
            }
        },
    }
}

fn run_project_recipe(recipe: &pleiades_agent_services::ProjectCommandReport) {
    println!(
        "Running project recipe `{}`: {}",
        recipe.name, recipe.command
    );
    let status = if cfg!(windows) {
        std::process::Command::new("cmd")
            .args(["/C", &recipe.command])
            .status()
    } else {
        std::process::Command::new("sh")
            .args(["-c", &recipe.command])
            .status()
    };
    match status {
        Ok(status) if status.success() => {}
        Ok(status) => std::process::exit(status.code().unwrap_or(1)),
        Err(error) => {
            eprintln!("\x1b[1;31m✗\x1b[0m failed to run recipe: {error}");
            std::process::exit(1);
        }
    }
}

fn print_lsp_status(report: &pleiades_agent_lsp::LspStatusReport) {
    println!("Workspace: {}", report.workspace.display());
    if report.servers.is_empty() {
        println!("No detected language services.");
        return;
    }
    for server in &report.servers {
        println!(
            "  {:<18} {:<8} {:<10} {}",
            server.id,
            server.language,
            server.status.label(),
            server.command
        );
        println!("    {}", server.notes);
    }
}

fn print_lsp_diagnostics(report: &pleiades_agent_lsp::DiagnosticReport) {
    println!("Workspace: {}", report.workspace.display());
    println!(
        "Command: {}",
        report
            .command
            .as_deref()
            .unwrap_or("no diagnostics command run")
    );
    if report.diagnostics.is_empty() {
        println!("No diagnostics reported.");
        return;
    }
    for file in &report.diagnostics {
        for diagnostic in &file.diagnostics {
            println!(
                "{}:{}:{}: {}",
                file.path.display(),
                diagnostic.range.start.line + 1,
                diagnostic.range.start.character + 1,
                diagnostic.message
            );
        }
    }
}

fn print_lsp_symbols(report: &pleiades_agent_lsp::SymbolSearchReport) {
    if report.symbols.is_empty() {
        println!("No symbols matched `{}`.", report.query);
        return;
    }
    for symbol in &report.symbols {
        println!(
            "{} {:<24} {}:{}",
            symbol.kind,
            symbol.name,
            symbol.location.display(),
            symbol.line
        );
    }
}

fn cli_list_or_all(values: &[String]) -> String {
    if values.is_empty() {
        "all non-denied tools".to_string()
    } else {
        values.join(", ")
    }
}

fn cli_list_or_none(values: &[String]) -> String {
    if values.is_empty() {
        "(none)".to_string()
    } else {
        values.join(", ")
    }
}

fn permission_service(loader: &ConfigLoader) -> pleiades_agent_services::PermissionService {
    pleiades_agent_services::ApplicationServices::with_config_dirs(
        loader.global_dir().to_path_buf(),
        loader.project_dir().to_path_buf(),
    )
    .permission()
}

fn handle_permissions_show(loader: &ConfigLoader) {
    match permission_service(loader).show() {
        Ok(report) => {
            println!("Permission rules:");
            if report.rules.is_empty() {
                println!("  (none)");
            } else {
                for item in report.rules {
                    println!(
                        "  {}. {} {} {}",
                        item.index,
                        permission_action_label_cli(item.rule.action),
                        item.rule.tool,
                        item.rule.pattern
                    );
                }
            }
            println!(
                "Legacy always allow: {}",
                if report.always_allow.is_empty() {
                    "(none)".to_string()
                } else {
                    report.always_allow.join(", ")
                }
            );
            println!(
                "Legacy always deny: {}",
                if report.always_deny.is_empty() {
                    "(none)".to_string()
                } else {
                    report.always_deny.join(", ")
                }
            );
        }
        Err(error) => {
            eprintln!("Error: {error}");
            std::process::exit(1);
        }
    }
}

fn handle_permissions_add(
    loader: &ConfigLoader,
    action: pleiades_agent_permissions::PermissionAction,
    pattern: Vec<String>,
) {
    if pattern.is_empty() {
        eprintln!("Error: usage: pleiades permissions <allow|ask|deny> <pattern>");
        std::process::exit(1);
    }
    let pattern = pattern.join(" ");
    match permission_service(loader).add_bash_rule(action, &pattern) {
        Ok(()) => println!(
            "Added permission rule: {} bash {}",
            permission_action_label_cli(action),
            pattern
        ),
        Err(error) => {
            eprintln!("Error: {error}");
            std::process::exit(1);
        }
    }
}

fn handle_permissions_reset(loader: &ConfigLoader) {
    match permission_service(loader).reset() {
        Ok(()) => println!("Permission rules reset"),
        Err(error) => {
            eprintln!("Error: {error}");
            std::process::exit(1);
        }
    }
}

fn handle_permissions_test(loader: &ConfigLoader, command: Vec<String>) {
    if command.is_empty() {
        eprintln!("Error: usage: pleiades permissions test <command>");
        std::process::exit(1);
    }
    let command = command.join(" ");
    match permission_service(loader).test_bash_command(&command) {
        Ok(report) => {
            println!("Command: {}", report.command);
            println!(
                "Decision: {}",
                permission_decision_label_cli(report.decision.kind)
            );
            println!("Reason: {}", report.decision.reason);
            if !report.decision.clauses.is_empty() {
                println!("Clauses:");
                for clause in report.decision.clauses {
                    println!("  - {}", clause);
                }
            }
        }
        Err(error) => {
            eprintln!("Error: {error}");
            std::process::exit(1);
        }
    }
}

fn permission_action_label_cli(
    action: pleiades_agent_permissions::PermissionAction,
) -> &'static str {
    match action {
        pleiades_agent_permissions::PermissionAction::Allow => "allow",
        pleiades_agent_permissions::PermissionAction::Ask => "ask",
        pleiades_agent_permissions::PermissionAction::Deny => "deny",
    }
}

fn permission_decision_label_cli(kind: pleiades_agent_permissions::DecisionKind) -> &'static str {
    match kind {
        pleiades_agent_permissions::DecisionKind::Allow => "allow",
        pleiades_agent_permissions::DecisionKind::Ask => "ask",
        pleiades_agent_permissions::DecisionKind::Deny => "deny",
        pleiades_agent_permissions::DecisionKind::Default => "default",
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
            ModelCommand::Favorite { name } => handle_model_favorite(&loader, &name),
            ModelCommand::Favorites => handle_model_favorites(&loader),
            ModelCommand::Reasoning { level } => handle_model_reasoning(&loader, &level),
        },
        Some(Commands::Session(cmd)) => match cmd {
            SessionCommand::List => handle_session_list(&loader),
            SessionCommand::Search { query } => handle_session_search(&loader, &query),
            SessionCommand::Show { id } => handle_session_show(&loader, &id),
            SessionCommand::Rename { id, name } => handle_session_rename(&loader, &id, &name),
            SessionCommand::Fork { id } => handle_session_fork(&loader, id.as_deref()),
            SessionCommand::Resume { id } => handle_session_resume(&loader, &id),
            SessionCommand::Delete { id } => handle_session_delete(&loader, &id),
            SessionCommand::Export { id, format, output } => {
                handle_session_export(&loader, &id, &format, output)
            }
            SessionCommand::Path => handle_session_path(&loader),
            SessionCommand::Ephemeral { state } => handle_session_ephemeral(&state),
        },
        Some(Commands::Tool(cmd)) => match cmd {
            ToolCommand::List => handle_tool_list(&loader),
            ToolCommand::Info { name } => handle_tool_info(&loader, &name),
            ToolCommand::Call { name, input } => handle_tool_call(&loader, &name, &input),
        },
        Some(Commands::Plugin(cmd)) => match cmd {
            PluginCommand::List => handle_plugin_list(&loader),
            PluginCommand::Info { id } => handle_plugin_info(&loader, &id),
            PluginCommand::Install { path } => handle_plugin_install(&loader, &path),
            PluginCommand::Uninstall { id } => handle_plugin_uninstall(&loader, &id),
            PluginCommand::Enable { id } => handle_plugin_enable(&loader, &id),
            PluginCommand::Disable { id } => handle_plugin_disable(&loader, &id),
            PluginCommand::Update { id } => handle_plugin_update(&loader, &id),
            PluginCommand::Trust { id } => handle_plugin_trust(&loader, &id),
            PluginCommand::Untrust { id } => handle_plugin_untrust(&loader, &id),
        },
        Some(Commands::Mcp(cmd)) => match cmd {
            McpCommand::List => handle_mcp_list(&loader),
            McpCommand::Info { id } => handle_mcp_info(&loader, &id),
            McpCommand::Add => handle_mcp_live_only("add", None),
            McpCommand::Remove { id } => handle_mcp_remove(&loader, &id),
            McpCommand::Enable { id } => handle_mcp_enable(&loader, &id),
            McpCommand::Disable { id } => handle_mcp_disable(&loader, &id),
            McpCommand::Tools { id } => handle_mcp_tools(&loader, &id),
            McpCommand::ToolInfo { server, tool } => handle_mcp_tool_info(&loader, &server, &tool),
            McpCommand::Reload => handle_mcp_reload(),
            McpCommand::Auth { id } => handle_mcp_live_only("auth", id.as_deref()),
            McpCommand::Logout { id } => handle_mcp_live_only("logout", id.as_deref()),
            McpCommand::Restart { id } => handle_mcp_live_only("restart", id.as_deref()),
            McpCommand::Logs { id } => handle_mcp_live_only("logs", id.as_deref()),
            McpCommand::Debug { id } => handle_mcp_live_only("debug", id.as_deref()),
        },
        Some(Commands::Skills(cmd)) => match cmd {
            SkillsCommand::List => handle_skills_list(&loader),
            SkillsCommand::Show { name } => handle_skills_show(&loader, &name),
            SkillsCommand::Create { name } => handle_skills_create(&loader, &name),
            SkillsCommand::Edit { name } => handle_skills_edit(&loader, &name),
            SkillsCommand::Enable { name } => handle_skills_enable(&loader, &name),
            SkillsCommand::Disable { name } => handle_skills_disable(&loader, &name),
            SkillsCommand::Reload => handle_skills_reload(),
        },
        Some(Commands::Permissions(cmd)) => match cmd {
            PermissionsCommand::Show => handle_permissions_show(&loader),
            PermissionsCommand::Allow { pattern } => handle_permissions_add(
                &loader,
                pleiades_agent_permissions::PermissionAction::Allow,
                pattern,
            ),
            PermissionsCommand::Ask { pattern } => handle_permissions_add(
                &loader,
                pleiades_agent_permissions::PermissionAction::Ask,
                pattern,
            ),
            PermissionsCommand::Deny { pattern } => handle_permissions_add(
                &loader,
                pleiades_agent_permissions::PermissionAction::Deny,
                pattern,
            ),
            PermissionsCommand::Reset => handle_permissions_reset(&loader),
            PermissionsCommand::Test { command } => handle_permissions_test(&loader, command),
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
        Some(Commands::Lsp(cmd)) => handle_lsp_command(&loader, cmd),
        Some(Commands::Process(cmd)) => handle_process_command(cmd),
        Some(Commands::Browser(cmd)) => handle_browser_command(cmd),
        Some(Commands::Project(cmd)) => handle_project_command(&loader, cmd),
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
        "history_dir" => config.session.history_dir.clone(),
        "max_concurrent" => Some(config.session.max_concurrent.to_string()),
        "compress_history" => Some(config.session.compress_history.to_string()),
        "ephemeral" => Some(config.session.ephemeral.to_string()),
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
        "history_dir" => {
            config.session.history_dir = Some(value.to_string());
        }
        "max_concurrent" => {
            config.session.max_concurrent = value.parse().unwrap_or(10);
        }
        "compress_history" => {
            config.session.compress_history = value == "true";
        }
        "ephemeral" => {
            config.session.ephemeral = value == "true";
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
