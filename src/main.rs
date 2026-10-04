use std::path::Path;

use clap::{Parser, Subcommand};

mod check;
mod config;
mod diag;
mod init;
mod list;
mod md;
mod output;
mod scan;
mod skill;
mod status;
mod task;

use output::Format;

#[derive(thiserror::Error, Debug)]
enum Error {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Config(String),
    #[error("{0}")]
    Output(String),
    #[error("{0}")]
    Usage(String),
    #[error("{0}")]
    Diagnostic(#[from] diag::Diagnostic),
    #[error("check failed: {0} error(s)")]
    CheckFailed(usize),
}

#[derive(Parser)]
#[command(
    name = "specdev",
    version,
    about = "Specification-driven development toolkit"
)]
struct Cli {
    /// Output format
    #[arg(long, global = true, value_enum, default_value_t = Format::Text)]
    format: Format,

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
    /// Validate task files, their logs, the task index, and spec rules
    Check,
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
    /// Manage task files in specs/tasks/
    Task {
        #[command(subcommand)]
        command: TaskCommands,
    },
}

#[derive(Subcommand, Debug, PartialEq)]
enum TaskCommands {
    /// Create a draft task file from the specdev.toml task definition
    New {
        /// Lowercase words joined by dashes, e.g. status-freshness
        slug: String,
    },
    /// List open tasks in queue order (done tasks excluded)
    List,
    /// Show a task: state, plan progress, log, and the next step
    Show {
        /// Task id, sequence number, or slug
        id: String,
    },
    /// Regenerate specs/tasks/_index.md
    Index,
    /// Move a task to its next status/stage, or to one named with --to
    Advance {
        /// Task id, sequence number, or slug
        id: String,
        /// Target: draft, ready, approval, or a stage (implement, verify, review, fix)
        #[arg(long)]
        to: Option<String>,
    },
    /// Block a task; the human decides where it goes next
    Block {
        /// Task id, sequence number, or slug
        id: String,
        /// Why the task can't proceed
        #[arg(long)]
        reason: String,
    },
    /// Set a plain frontmatter field: source, depends, or a configured extra field
    Set {
        /// Task id, sequence number, or slug
        id: String,
        field: String,
        /// New value; "" clears. depends takes comma-separated task ids
        value: String,
    },
    /// Add or remove a scope entry; every change is logged
    Scope {
        /// Task id, sequence number, or slug
        id: String,
        #[command(subcommand)]
        action: ScopeAction,
    },
}

#[derive(Subcommand, Debug, PartialEq)]
enum ScopeAction {
    /// Add a file or glob to the task's scope
    Add {
        path: String,
        /// Why the task needs it
        #[arg(long)]
        reason: String,
    },
    /// Remove a scope entry
    Rm { path: String },
}

#[derive(Subcommand, Debug, PartialEq)]
enum SkillCommands {
    /// Install the specdev skill to an agents skills directory
    Install {
        /// Install locally to .agents/skills/ instead of globally
        #[arg(long)]
        local: bool,
    },
    /// Check whether the installed skill is up to date, stale, or modified
    Check,
}

fn main() {
    let cli = Cli::parse();
    let format = cli.format;
    let result = match cli.command {
        Commands::Init => {
            output::require_text("init", format).and_then(|()| init::run())
        }
        Commands::Scan => scan::run(Path::new(""), format),
        Commands::Status => status::run(Path::new(""), format),
        Commands::Check => check::run(Path::new(""), format),
        Commands::List { stats } => list::run(stats, format),
        Commands::Skill { command } => output::require_text("skill", format)
            .and_then(|()| match command {
                SkillCommands::Install { local } => skill::install(local),
                SkillCommands::Check => skill::check(),
            }),
        Commands::Task { command } => {
            let root = Path::new("");
            match command {
                TaskCommands::New { slug } => task::cmd::new(root, &slug, format),
                TaskCommands::List => task::cmd::list(root, format),
                TaskCommands::Show { id } => task::cmd::show(root, &id, format),
                TaskCommands::Index => task::cmd::index(root, format),
                TaskCommands::Advance { id, to } => {
                    task::state::advance(root, &id, to.as_deref(), format)
                }
                TaskCommands::Block { id, reason } => {
                    task::state::block(root, &id, &reason, format)
                }
                TaskCommands::Set { id, field, value } => {
                    task::state::set(root, &id, &field, &value, format)
                }
                TaskCommands::Scope { id, action } => match action {
                    ScopeAction::Add { path, reason } => {
                        task::state::scope_add(root, &id, &path, &reason, format)
                    }
                    ScopeAction::Rm { path } => {
                        task::state::scope_rm(root, &id, &path, format)
                    }
                },
            }
        }
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
    fn parses_check() {
        assert!(matches!(cli(&["check"]).unwrap(), Commands::Check));
    }

    #[test]
    fn parses_task_state_commands() {
        let task = |args: &[&str]| match cli(args).unwrap() {
            Commands::Task { command } => command,
            other => panic!("{other:?}"),
        };
        assert_eq!(
            task(&["task", "advance", "3", "--to", "fix"]),
            TaskCommands::Advance {
                id: "3".into(),
                to: Some("fix".into())
            }
        );
        assert_eq!(
            task(&["task", "scope", "3", "add", "src/a.rs", "--reason", "r"]),
            TaskCommands::Scope {
                id: "3".into(),
                action: ScopeAction::Add {
                    path: "src/a.rs".into(),
                    reason: "r".into()
                }
            }
        );
        assert_eq!(
            task(&["task", "set", "3", "source", ""]),
            TaskCommands::Set {
                id: "3".into(),
                field: "source".into(),
                value: String::new()
            }
        );
        assert!(
            cli(&["task", "block", "3"]).is_err(),
            "--reason is required"
        );
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
    fn parses_skill_check() {
        assert_eq!(
            cli(&["skill", "check"]).unwrap(),
            Commands::Skill {
                command: SkillCommands::Check
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
