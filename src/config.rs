//! `specdev.toml`: the per-project task contract, pipeline limits, acceptance
//! defaults, and quality thresholds. Missing file or keys fall back to defaults.

use std::fs;
use std::path::Path;

use serde::Deserialize;

use crate::Error;

pub const CONFIG_FILE: &str = "specdev.toml";

/// Written by `specdev init`. Must parse to exactly `Config::default()`.
pub const DEFAULT_TOML: &str = r#"# specdev project config.

[task]
# Sections every task file must have; `task new` creates them.
required_sections = ["Plan", "Acceptance"]
optional_sections = ["Context", "Findings", "Review", "Spec updates"]
# Project-specific optional frontmatter fields.
extra_fields = []

[pipeline]
# Entries into the `fix` stage before a task is blocked.
max_attempts = 3

[acceptance]
# Commands `specdev task verify` runs for every task, e.g. ["cargo test"].
default = []

[quality.task]
max_plan_steps = 8
max_scope = 6
max_lines = 150

[quality.changelog]
max_lines = 400
"#;

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub task: TaskDef,
    pub pipeline: Pipeline,
    pub acceptance: Acceptance,
    pub quality: Quality,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct TaskDef {
    pub required_sections: Vec<String>,
    pub optional_sections: Vec<String>,
    pub extra_fields: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Pipeline {
    pub max_attempts: u32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Acceptance {
    pub default: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Quality {
    pub task: Limits,
    pub changelog: Limits,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Limits {
    pub max_plan_steps: Option<u32>,
    pub max_scope: Option<u32>,
    pub max_lines: Option<u32>,
}

impl Default for TaskDef {
    fn default() -> Self {
        let strings =
            |items: &[&str]| items.iter().map(|s| s.to_string()).collect();
        Self {
            required_sections: strings(&["Plan", "Acceptance"]),
            optional_sections: strings(&[
                "Context",
                "Findings",
                "Review",
                "Spec updates",
            ]),
            extra_fields: Vec::new(),
        }
    }
}

impl Default for Pipeline {
    fn default() -> Self {
        Self { max_attempts: 3 }
    }
}

impl Default for Quality {
    fn default() -> Self {
        Self {
            task: Limits {
                max_plan_steps: Some(8),
                max_scope: Some(6),
                max_lines: Some(150),
            },
            changelog: Limits {
                max_lines: Some(400),
                ..Limits::default()
            },
        }
    }
}

impl Config {
    /// Load `<root>/specdev.toml`, or defaults when the file is absent.
    pub fn load(root: &Path) -> Result<Self, Error> {
        let path = root.join(CONFIG_FILE);
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(&path)?;
        Self::parse(&content)
            .map_err(|e| Error::Config(format!("{}: {e}", path.display())))
    }

    pub fn parse(content: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn default_toml_matches_default_config() {
        assert_eq!(Config::parse(DEFAULT_TOML).unwrap(), Config::default());
    }

    #[test]
    fn missing_keys_fall_back_to_defaults() {
        let c = Config::parse("[pipeline]\nmax_attempts = 5\n").unwrap();
        assert_eq!(c.pipeline.max_attempts, 5);
        assert_eq!(c.task, TaskDef::default());
        assert_eq!(c.quality, Quality::default());
    }

    #[test]
    fn unknown_keys_are_rejected() {
        assert!(Config::parse("[pipeline]\nstages = [\"x\"]\n").is_err());
    }

    #[test]
    fn load_without_file_gives_defaults() {
        let tmp = TempDir::new().unwrap();
        assert_eq!(Config::load(tmp.path()).unwrap(), Config::default());
    }

    #[test]
    fn load_reports_path_on_error() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join(CONFIG_FILE), "[task\n").unwrap();
        let err = Config::load(tmp.path()).unwrap_err().to_string();
        assert!(err.contains(CONFIG_FILE), "got {err}");
    }
}
