//! Task files: `specs/tasks/<id>.md` = flat-YAML frontmatter, a prose body kept
//! byte-for-byte, and a specdev-owned `## Log`. Commands rewrite only the
//! frontmatter and the log; everything between them is the agent's.

pub mod cmd;
pub mod done;
mod frontmatter;
mod id;
mod log;
pub mod state;
mod store;
pub mod transition;

use std::fmt;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use serde::Serialize;

use crate::diag::Diagnostic;
use crate::md;

pub use frontmatter::Frontmatter;
pub use id::{TaskId, validate_slug};
pub use log::{Fix, LogEntry, LogEvent};
pub use store::{INDEX_FILE, TASKS_DIR, TaskStore};

/// Defines a hardcoded string enum with `as_str`, `FromStr`, `Display`, and
/// kebab-case serialization.
macro_rules! str_enum {
    ($name:ident, $what:literal { $($variant:ident => $s:literal),+ $(,)? }) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
        #[serde(rename_all = "kebab-case")]
        pub enum $name { $($variant),+ }

        impl $name {
            pub fn as_str(self) -> &'static str {
                match self { $(Self::$variant => $s),+ }
            }
        }

        impl FromStr for $name {
            type Err = String;
            fn from_str(s: &str) -> Result<Self, String> {
                match s {
                    $($s => Ok(Self::$variant),)+
                    _ => Err(format!(
                        concat!("unknown ", $what, " `{}` (expected {})"),
                        s,
                        [$($s),+].join(", ")
                    )),
                }
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(self.as_str())
            }
        }
    };
}

str_enum!(Status, "status" {
    Draft => "draft",
    Ready => "ready",
    InProgress => "in-progress",
    Approval => "approval",
    Done => "done",
    Blocked => "blocked",
});

impl Status {
    /// Current work: in progress, or waiting for the user's acceptance.
    pub fn is_active(self) -> bool {
        matches!(self, Self::InProgress | Self::Approval)
    }

    /// The one exclusive slot: only an in-progress task holds it. Tasks in
    /// approval wait for the user and don't stop the next task starting.
    pub fn holds_slot(self) -> bool {
        self == Self::InProgress
    }
}

str_enum!(Stage, "stage" {
    Implement => "implement",
    Verify => "verify",
    Review => "review",
    Fix => "fix",
});

/// Where a task stands: status plus stage (stage only while in progress).
/// Renders as `ready` or `in-progress/verify`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub status: Status,
    pub stage: Option<Stage>,
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.stage {
            Some(stage) => write!(f, "{}/{}", self.status, stage),
            None => write!(f, "{}", self.status),
        }
    }
}

impl FromStr for Position {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, String> {
        let (status, stage) = match s.split_once('/') {
            Some((status, stage)) => (status, Some(stage.parse()?)),
            None => (s, None),
        };
        Ok(Self {
            status: status.parse()?,
            stage,
        })
    }
}

const TITLE_PREFIX: &str = "Task:";
const LOG_HEADING: &str = "Log";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Task {
    pub path: PathBuf,
    pub front: Frontmatter,
    /// From the first `# Task: <title>` heading.
    pub title: String,
    /// Everything between the frontmatter and `## Log`, verbatim.
    pub body: String,
    pub log: Vec<LogEntry>,
}

impl Task {
    pub fn parse(path: &Path, content: &str) -> Result<Self, Diagnostic> {
        let error = |line: Option<usize>, code: &'static str, msg: String| {
            Diagnostic::error(path, line, code, msg)
        };

        let rest = content.strip_prefix("---\n").ok_or_else(|| {
            error(
                Some(1),
                "frontmatter",
                "file must start with a `---` frontmatter line".into(),
            )
        })?;
        let mut offset = 0;
        let mut close = None;
        for line in rest.split_inclusive('\n') {
            if line.trim_end() == "---" {
                close = Some((offset, offset + line.len()));
                break;
            }
            offset += line.len();
        }
        let (fm_end, body_start) = close.ok_or_else(|| {
            error(
                Some(1),
                "frontmatter",
                "unterminated frontmatter: no closing `---`".into(),
            )
        })?;
        let fm_text = &rest[..fm_end];
        let front = Frontmatter::parse(fm_text)
            .map_err(|(line, msg)| error(Some(line + 1), "frontmatter", msg))?;

        // File line number of the first body line, minus one.
        let body_line_offset = fm_text.lines().count() + 2;
        let after = &rest[body_start..];
        let outline = md::outline(after);

        let title = outline
            .headings
            .iter()
            .find(|h| h.level == 1)
            .and_then(|h| h.text.strip_prefix(TITLE_PREFIX))
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty())
            .ok_or_else(|| {
                error(None, "title", "missing `# Task: <title>` heading".into())
            })?;

        let log_heading = outline
            .headings
            .iter()
            .rev()
            .find(|h| h.level == 2 && h.text == LOG_HEADING);
        let (body, log) = match log_heading {
            Some(h) => {
                let split = byte_offset_of_line(after, h.line);
                let log =
                    log::parse_section(&after[split..], body_line_offset + h.line)
                        .map_err(|(line, msg)| error(Some(line), "log", msg))?;
                (after[..split].to_string(), log)
            }
            None => (after.to_string(), Vec::new()),
        };

        Ok(Self {
            path: path.to_path_buf(),
            front,
            title,
            body,
            log,
        })
    }

    pub fn render(&self) -> String {
        let mut out = format!("---\n{}---\n{}", self.front.render(), self.body);
        if !out.ends_with("\n\n") {
            out.push_str(if out.ends_with('\n') { "\n" } else { "\n\n" });
        }
        out.push_str("## Log\n\n");
        for entry in &self.log {
            out.push_str(&format!("- {entry}\n"));
        }
        out
    }

    pub fn position(&self) -> Position {
        Position {
            status: self.front.status,
            stage: self.front.stage,
        }
    }
}

/// Byte offset where 1-based `line` starts.
fn byte_offset_of_line(text: &str, line: usize) -> usize {
    text.split_inclusive('\n')
        .take(line - 1)
        .map(str::len)
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    const SAMPLE: &str = "\
---
id: 0007-status-freshness
status: draft
scope: [src/status.rs]
created: 2026-10-02
---
# Task: warn when ctx.md is stale

## Plan
- [x] parse `State:` line
- [ ] add the warning

```md
## Log
- not the real log
```

Prose mentioning ## Log inline stays prose.

## Acceptance
- `cargo test`

## Log

- 2026-10-02 created: warn when ctx.md is stale
";

    fn sample() -> Task {
        Task::parse(Path::new("specs/tasks/0007-status-freshness.md"), SAMPLE)
            .unwrap()
    }

    #[test]
    fn canonical_file_round_trips_exactly() {
        assert_eq!(sample().render(), SAMPLE);
    }

    #[test]
    fn parses_title_body_and_log() {
        let task = sample();
        assert_eq!(task.title, "warn when ctx.md is stale");
        assert!(task.body.starts_with("# Task:"));
        assert!(task.body.contains("- not the real log"));
        assert!(task.body.ends_with("- `cargo test`\n\n"));
        assert_eq!(task.log.len(), 1);
        assert_eq!(task.position().to_string(), "draft");
    }

    #[test]
    fn state_edits_keep_body_bytes() {
        let original = sample();
        let mut task = original.clone();
        task.front.status = Status::Ready;
        task.front.scope.push("tests/cli.rs".to_string());
        task.log.push(LogEntry {
            date: NaiveDate::from_ymd_opt(2026, 10, 3).unwrap(),
            event: LogEvent::Advance {
                from: original.position(),
                to: Position {
                    status: Status::Ready,
                    stage: None,
                },
                fix: None,
            },
        });
        let reparsed = Task::parse(&task.path, &task.render()).unwrap();
        assert_eq!(reparsed.body, original.body);
        assert_eq!(reparsed.front, task.front);
        assert_eq!(reparsed.log, task.log);
    }

    #[test]
    fn missing_log_section_is_added_on_render() {
        let content = "---\nid: 0001-x\nstatus: draft\nscope: []\ncreated: 2026-10-03\n---\n# Task: x\n";
        let task = Task::parse(Path::new("x.md"), content).unwrap();
        assert!(task.log.is_empty());
        assert!(task.render().ends_with("# Task: x\n\n## Log\n\n"));
    }

    #[test]
    fn errors_point_at_file_lines() {
        let path = Path::new("t.md");
        let no_fm = Task::parse(path, "# Task: x\n").unwrap_err();
        assert_eq!((no_fm.code, no_fm.line), ("frontmatter", Some(1)));

        let bad_field = "---\nid: 0001-x\nstatus: doing\nscope: []\ncreated: 2026-10-03\n---\n# Task: x\n";
        assert_eq!(Task::parse(path, bad_field).unwrap_err().line, Some(3));

        let no_title = "---\nid: 0001-x\nstatus: draft\nscope: []\ncreated: 2026-10-03\n---\n# Notes\n";
        assert_eq!(Task::parse(path, no_title).unwrap_err().code, "title");

        let bad_log = "---\nid: 0001-x\nstatus: draft\nscope: []\ncreated: 2026-10-03\n---\n# Task: x\n\n## Log\n\n- 2026-10-03 frobnicated\n";
        let err = Task::parse(path, bad_log).unwrap_err();
        assert_eq!((err.code, err.line), ("log", Some(11)));
    }

    #[test]
    fn positions_parse_and_display() {
        for s in ["draft", "in-progress/verify", "approval"] {
            assert_eq!(s.parse::<Position>().unwrap().to_string(), s);
        }
        assert!("in-progress/coding".parse::<Position>().is_err());
        assert!("doing".parse::<Position>().is_err());
    }
}
