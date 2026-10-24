//! `specdev.toml`: the per-project task contract, pipeline limits, acceptance
//! defaults, and quality thresholds. Missing file or keys fall back to defaults.

use std::collections::BTreeMap;
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
optional_sections = ["Summary", "Manual checks", "Context", "Findings", "Review", "Spec updates"]
# Project-specific optional frontmatter fields.
extra_fields = []
# Tasks are created only when the user asks for one; small fixes are direct
# edits. While true, working-tree out-of-scope findings are warnings.
# false restores task-per-small-change and makes out-of-scope an error.
explicit_only = true

[pipeline]
# Failed verifies (verify → fix) before a task is blocked; revisions don't count.
max_attempts = 3

[acceptance]
# Acceptance items every task gets, e.g. ["cargo test"]. specdev never runs them.
default = []

[scope]
# Globs never flagged as out of scope, e.g. ["Cargo.lock"]. specs/ is always allowed.
always_allowed = []

[quality]
# Threshold violations are warnings; true makes them errors.
errors = false
# Flag Markdown tables and ASCII box diagrams in specs/ and task files.
forbid_tables = false

# Per file kind. Keys: max_lines, max_words, max_plan_steps, max_scope.
# Omitted keys keep these defaults; 0 turns a limit off.
# max_words catches soft-wrapped files whose line count stays low.
[quality.task]
max_lines = 150
max_words = 1000
max_plan_steps = 8
max_scope = 6

# Optional size grades a task can declare (`specdev task set <id> grade <name>`).
# Keys set here win; the rest inherit from [quality.task] above; 0 turns a limit off.
[quality.grade.tiny]
max_lines = 80
max_words = 500
max_plan_steps = 5
max_scope = 3

[quality.grade.huge]
max_lines = 300
max_words = 2500
max_plan_steps = 12
max_scope = 12

[quality.ctx]
max_lines = 120
max_words = 600

[quality.overview]
max_lines = 200
max_words = 1500

[quality.changelog]
max_lines = 400
max_words = 3000
"#;

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub task: TaskDef,
    pub pipeline: Pipeline,
    pub acceptance: Acceptance,
    pub scope: ScopeConfig,
    pub quality: Quality,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct TaskDef {
    pub required_sections: Vec<String>,
    pub optional_sections: Vec<String>,
    pub extra_fields: Vec<String>,
    /// Tasks exist only when the user asks for one (#19). While true, a
    /// working-tree `out-of-scope` finding is a warning, so a small direct
    /// fix doesn't pressure the agent into creating a task.
    pub explicit_only: bool,
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

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ScopeConfig {
    /// Globs that are never out of scope (on top of `specs/`).
    pub always_allowed: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(from = "RawQuality")]
pub struct Quality {
    /// Report threshold violations as errors instead of warnings.
    pub errors: bool,
    /// Flag Markdown tables and ASCII box diagrams.
    pub forbid_tables: bool,
    pub task: Limits,
    pub ctx: Limits,
    pub overview: Limits,
    pub changelog: Limits,
    /// Size grades (`[quality.grade.<name>]`); unset tasks use `task`.
    /// A file's table replaces the default of that name; unused names are inert.
    pub grade: BTreeMap<String, Limits>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Limits {
    pub max_lines: Option<usize>,
    pub max_words: Option<usize>,
    pub max_plan_steps: Option<usize>,
    pub max_scope: Option<usize>,
}

impl Limits {
    fn size(max_lines: usize, max_words: usize) -> Self {
        Self {
            max_lines: Some(max_lines),
            max_words: Some(max_words),
            ..Self::default()
        }
    }

    /// Keys set here win; the rest come from `defaults`.
    fn or(self, defaults: Self) -> Self {
        Self {
            max_lines: self.max_lines.or(defaults.max_lines),
            max_words: self.max_words.or(defaults.max_words),
            max_plan_steps: self.max_plan_steps.or(defaults.max_plan_steps),
            max_scope: self.max_scope.or(defaults.max_scope),
        }
    }
}

/// `[quality]` as written: a kind's table may set only some keys, and the
/// others keep their defaults (instead of becoming "no limit").
#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct RawQuality {
    errors: bool,
    forbid_tables: bool,
    task: Limits,
    ctx: Limits,
    overview: Limits,
    changelog: Limits,
    grade: BTreeMap<String, Limits>,
}

impl From<RawQuality> for Quality {
    fn from(raw: RawQuality) -> Self {
        let d = Quality::default();
        // Grade tables merge over the defaults by name: a project that
        // defines none keeps `tiny`/`huge`, and defining one replaces that
        // name entirely (its keys still inherit from `[quality.task]`).
        let mut grade = d.grade;
        grade.extend(raw.grade);
        Self {
            errors: raw.errors,
            forbid_tables: raw.forbid_tables,
            task: raw.task.or(d.task),
            ctx: raw.ctx.or(d.ctx),
            overview: raw.overview.or(d.overview),
            changelog: raw.changelog.or(d.changelog),
            grade,
        }
    }
}

impl Default for TaskDef {
    fn default() -> Self {
        let strings =
            |items: &[&str]| items.iter().map(|s| s.to_string()).collect();
        Self {
            required_sections: strings(&["Plan", "Acceptance"]),
            optional_sections: strings(&[
                "Summary",
                "Manual checks",
                "Context",
                "Findings",
                "Review",
                "Spec updates",
            ]),
            extra_fields: Vec::new(),
            explicit_only: true,
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
        let grade = |lines, words, steps, scope: usize| Limits {
            max_lines: Some(lines),
            max_words: Some(words),
            max_plan_steps: Some(steps),
            max_scope: Some(scope),
        };
        Self {
            errors: false,
            forbid_tables: false,
            task: grade(150, 1000, 8, 6),
            ctx: Limits::size(120, 600),
            overview: Limits::size(200, 1500),
            changelog: Limits::size(400, 3000),
            grade: BTreeMap::from([
                ("tiny".to_string(), grade(80, 500, 5, 3)),
                ("huge".to_string(), grade(300, 2500, 12, 12)),
            ]),
        }
    }
}

impl Quality {
    /// Effective limits for a task file: the declared grade's keys win,
    /// the rest inherit from `[quality.task]` (the default grade).
    pub fn task_limits(&self, grade: Option<&str>) -> Limits {
        match grade.and_then(|g| self.grade.get(g)) {
            Some(grade) => grade.clone().or(self.task.clone()),
            None => self.task.clone(),
        }
    }

    /// Configured grade names, sorted for messages.
    pub fn grade_names(&self) -> Vec<&str> {
        self.grade.keys().map(String::as_str).collect()
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
    fn explicit_only_defaults_true_and_can_be_turned_off() {
        assert!(Config::default().task.explicit_only);
        let c = Config::parse("[task]\nexplicit_only = false\n").unwrap();
        assert!(!c.task.explicit_only);
        assert_eq!(c.pipeline, Pipeline::default());
    }

    #[test]
    fn grades_default_merge_and_inherit() {
        let q = Quality::default();
        assert_eq!(q.grade_names(), ["huge", "tiny"]);
        // Unset task: the default grade.
        assert_eq!(q.task_limits(None), q.task);
        // Declared grade: its keys win, the rest inherit from [quality.task].
        let tiny = q.task_limits(Some("tiny"));
        assert_eq!(tiny.max_lines, Some(80));
        assert_eq!(tiny.max_words, Some(500));
        assert_eq!(tiny.max_plan_steps, Some(5));
        assert_eq!(tiny.max_scope, Some(3));
        // A file that defines a grade replaces that name; the built-ins stay.
        let c = Config::parse(
            "[quality.grade.big]\nmax_lines = 30\n\n[quality.grade.tiny]\nmax_lines = 0\n",
        )
        .unwrap();
        let q = &c.quality;
        assert_eq!(q.grade_names(), ["big", "huge", "tiny"]);
        let big = q.task_limits(Some("big"));
        assert_eq!(big.max_lines, Some(30));
        assert_eq!(big.max_words, Some(1000), "inherited from [quality.task]");
        // A whole limit turned off inside a grade.
        assert_eq!(q.task_limits(Some("tiny")).max_lines, Some(0));
    }

    #[test]
    fn unknown_keys_are_rejected() {
        assert!(Config::parse("[pipeline]\nstages = [\"x\"]\n").is_err());
        assert!(Config::parse("[scope]\nallow = [\"x\"]\n").is_err());
    }

    #[test]
    fn quality_tables_keep_defaults_for_omitted_keys() {
        let c = Config::parse(
            "[quality.task]\nmax_scope = 1\n\n[quality.ctx]\nmax_lines = 0\n",
        )
        .unwrap();
        assert_eq!(c.quality.task.max_scope, Some(1));
        assert_eq!(c.quality.task.max_plan_steps, Some(8), "default kept");
        assert_eq!(c.quality.ctx.max_lines, Some(0), "0 turns it off");
        assert_eq!(c.quality.changelog, Quality::default().changelog);
        assert!(Config::parse("[quality]\nstrict = true\n").is_err());
    }

    #[test]
    fn scope_allowlist_parses() {
        let c =
            Config::parse("[scope]\nalways_allowed = [\"Cargo.lock\"]\n").unwrap();
        assert_eq!(c.scope.always_allowed, ["Cargo.lock"]);
        assert!(Config::default().scope.always_allowed.is_empty());
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
