use clap::{Parser, Subcommand};

mod init;
mod list;
mod scan;
mod skill;
mod status;

#[derive(thiserror::Error, Debug)]
enum Error {
    #[error("{0}")]
    Io(#[from] std::io::Error),
}

#[derive(Parser)]
#[command(
    name = "specdev",
    version,
    about = "Specification-driven development toolkit"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug, PartialEq)]
enum Commands {
    /// Scaffold specs/ directory with core spec files
    Init,
    /// Scan specs/ for unresolved ^^^ markers
    Scan,
    /// Show spec directory health and status
    Status,
    /// List spec files; use --stats for a structural breakdown
    List {
        /// Show per-file heading/checkbox/word stats instead of the simple list
        #[arg(long)]
        stats: bool,
    },
    /// Manage the specdev agent skill
    Skill {
        #[command(subcommand)]
        command: SkillCommands,
    },
}

#[derive(Subcommand, Debug, PartialEq)]
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
        Commands::List { stats } => list::run(stats),
        Commands::Skill { command } => match command {
            SkillCommands::Install { local } => skill::install(local),
        },
    };
    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    fn cli(args: &[&str]) -> Result<Commands, clap::error::Error> {
        let mut full = vec!["specdev"];
        full.extend_from_slice(args);
        Ok(Cli::try_parse_from(full)?.command)
    }

    #[test]
    fn parses_init() {
        assert!(matches!(cli(&["init"]).unwrap(), Commands::Init));
    }

    #[test]
    fn parses_scan() {
        assert!(matches!(cli(&["scan"]).unwrap(), Commands::Scan));
    }

    #[test]
    fn parses_status() {
        assert!(matches!(cli(&["status"]).unwrap(), Commands::Status));
    }

    #[test]
    fn parses_list_default() {
        assert_eq!(cli(&["list"]).unwrap(), Commands::List { stats: false });
    }

    #[test]
    fn parses_list_stats_flag() {
        assert_eq!(
            cli(&["list", "--stats"]).unwrap(),
            Commands::List { stats: true }
        );
    }

    #[test]
    fn parses_skill_install_global() {
        assert_eq!(
            cli(&["skill", "install"]).unwrap(),
            Commands::Skill {
                command: SkillCommands::Install { local: false }
            }
        );
    }

    #[test]
    fn parses_skill_install_local() {
        assert_eq!(
            cli(&["skill", "install", "--local"]).unwrap(),
            Commands::Skill {
                command: SkillCommands::Install { local: true }
            }
        );
    }

    #[test]
    fn rejects_unknown_command() {
        assert!(cli(&["frobnicate"]).is_err());
    }

    #[test]
    fn rejects_unknown_flag() {
        assert!(cli(&["list", "--bogus"]).is_err());
    }

    #[test]
    fn rejects_no_subcommand() {
        assert!(cli(&[]).is_err());
    }
}
