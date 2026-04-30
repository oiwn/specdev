use clap::{Parser, Subcommand};

mod init;
mod scan;
mod skill;
mod status;

#[derive(thiserror::Error, Debug)]
enum Error {
    #[error("{0}")]
    Io(#[from] std::io::Error),
}

#[derive(Parser)]
#[command(name = "specdev")]
#[command(about = "Specification-driven development toolkit")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scaffold specs/ directory with core spec files
    Init,
    /// Scan specs/ for unresolved ^^^ markers
    Scan,
    /// Show spec directory health and status
    Status,
    /// Manage the specdev agent skill
    Skill {
        #[command(subcommand)]
        command: SkillCommands,
    },
}

#[derive(Subcommand)]
enum SkillCommands {
    /// Install the specdev skill to an agents skills directory
    Install {
        /// Install locally to .agents/skills/ instead of globally
        #[arg(long)]
        local: bool,
    },
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Commands::Init => init::run(),
        Commands::Scan => scan::run(),
        Commands::Status => status::run(),
        Commands::Skill { command } => match command {
            SkillCommands::Install { local } => skill::install(local),
        },
    };
    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
