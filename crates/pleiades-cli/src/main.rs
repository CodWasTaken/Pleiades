use clap::Parser;

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

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    println!("Hello from Pleiades");

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
