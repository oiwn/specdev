//! The specdev-owned `## Log` section of a task file: one dated event per line,
//! human-readable and parseable, e.g. `- 2026-10-04 advance ready → in-progress/implement`.

use std::fmt;
use std::str::FromStr;

use chrono::NaiveDate;

use super::Position;
use super::frontmatter::{DATE_FORMAT, ParseError};

/// How a `set` to an empty value (a cleared field) is written.
const EMPTY: &str = "\"\"";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogEntry {
    pub date: NaiveDate,
    pub event: LogEvent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogEvent {
    Created {
        title: String,
    },
    Advance {
        from: Position,
        to: Position,
        attempts: Option<u32>,
    },
    ScopeApproved(Vec<String>),
    ScopeAdd {
        path: String,
        reason: String,
    },
    ScopeRemove {
        path: String,
    },
    Blocked {
        reason: String,
    },
    Set {
        field: String,
        value: String,
    },
}

impl fmt::Display for LogEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.date.format(DATE_FORMAT), self.event)
    }
}

impl fmt::Display for LogEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Created { title } => write!(f, "created: {title}"),
            Self::Advance { from, to, attempts } => {
                write!(f, "advance {from} → {to}")?;
                if let Some(n) = attempts {
                    write!(f, " (attempts {n})")?;
                }
                Ok(())
            }
            Self::ScopeApproved(paths) if paths.is_empty() => {
                write!(f, "scope approved:")
            }
            Self::ScopeApproved(paths) => {
                write!(f, "scope approved: {}", paths.join(", "))
            }
            Self::ScopeAdd { path, reason } => {
                write!(f, "scope add {path} — {reason}")
            }
            Self::ScopeRemove { path } => write!(f, "scope rm {path}"),
            Self::Blocked { reason } => write!(f, "blocked: {reason}"),
            Self::Set { field, value } if value.is_empty() => {
                write!(f, "set {field} {EMPTY}")
            }
            Self::Set { field, value } => write!(f, "set {field} {value}"),
        }
    }
}

impl FromStr for LogEntry {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, String> {
        let (date, event) = s
            .split_once(' ')
            .ok_or_else(|| format!("expected `YYYY-MM-DD <event>`, got `{s}`"))?;
        let date = NaiveDate::parse_from_str(date, DATE_FORMAT)
            .map_err(|_| format!("invalid date `{date}`, expected YYYY-MM-DD"))?;
        Ok(Self {
            date,
            event: event.parse()?,
        })
    }
}

impl FromStr for LogEvent {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, String> {
        let missing = |what: &str| format!("`{s}`: missing {what}");
        if let Some(title) = s.strip_prefix("created: ") {
            return Ok(Self::Created {
                title: title.to_string(),
            });
        }
        if let Some(rest) = s.strip_prefix("advance ") {
            let (moves, attempts) = match rest
                .strip_suffix(')')
                .and_then(|r| r.rsplit_once(" (attempts "))
            {
                Some((moves, n)) => {
                    let n = n
                        .parse()
                        .map_err(|_| format!("`{s}`: invalid attempts `{n}`"))?;
                    (moves, Some(n))
                }
                None => (rest, None),
            };
            let (from, to) =
                moves.split_once(" → ").ok_or_else(|| missing("` → `"))?;
            return Ok(Self::Advance {
                from: from.parse()?,
                to: to.parse()?,
                attempts,
            });
        }
        if let Some(rest) = s.strip_prefix("scope approved:") {
            let rest = rest.trim();
            let paths = if rest.is_empty() {
                Vec::new()
            } else {
                rest.split(", ").map(str::to_string).collect()
            };
            return Ok(Self::ScopeApproved(paths));
        }
        if let Some(rest) = s.strip_prefix("scope add ") {
            let (path, reason) = rest
                .split_once(" — ")
                .ok_or_else(|| missing("` — <reason>`"))?;
            return Ok(Self::ScopeAdd {
                path: path.to_string(),
                reason: reason.to_string(),
            });
        }
        if let Some(path) = s.strip_prefix("scope rm ") {
            return Ok(Self::ScopeRemove {
                path: path.to_string(),
            });
        }
        if let Some(reason) = s.strip_prefix("blocked: ") {
            return Ok(Self::Blocked {
                reason: reason.to_string(),
            });
        }
        if let Some(rest) = s.strip_prefix("set ") {
            let (field, value) =
                rest.split_once(' ').ok_or_else(|| missing("value"))?;
            let value = if value == EMPTY { "" } else { value };
            return Ok(Self::Set {
                field: field.to_string(),
                value: value.to_string(),
            });
        }
        Err(format!("unknown log event `{s}`"))
    }
}

/// Parse a `## Log` section (heading line included). `first_line` is the
/// file line number of the heading, so errors point into the file.
pub fn parse_section(
    text: &str,
    first_line: usize,
) -> Result<Vec<LogEntry>, ParseError> {
    let mut entries = Vec::new();
    for (i, line) in text.lines().enumerate().skip(1) {
        let line_no = first_line + i;
        let line = line.trim_end();
        if line.trim().is_empty() {
            continue;
        }
        let entry = line.strip_prefix("- ").ok_or_else(|| {
            (
                line_no,
                format!("log lines must start with `- `, got `{line}`"),
            )
        })?;
        entries.push(entry.parse().map_err(|e| (line_no, e))?);
    }
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::{Stage, Status};

    fn pos(status: Status, stage: Option<Stage>) -> Position {
        Position { status, stage }
    }

    #[test]
    fn every_event_round_trips() {
        let date = NaiveDate::from_ymd_opt(2026, 10, 4).unwrap();
        let events = vec![
            LogEvent::Created {
                title: "warn when ctx.md is stale".to_string(),
            },
            LogEvent::Advance {
                from: pos(Status::Draft, None),
                to: pos(Status::Ready, None),
                attempts: None,
            },
            LogEvent::Advance {
                from: pos(Status::InProgress, Some(Stage::Verify)),
                to: pos(Status::InProgress, Some(Stage::Fix)),
                attempts: Some(1),
            },
            LogEvent::ScopeApproved(vec![
                "src/status.rs".to_string(),
                "tests/cli.rs".to_string(),
            ]),
            LogEvent::ScopeApproved(vec![]),
            LogEvent::ScopeAdd {
                path: "src/scan.rs".to_string(),
                reason: "needs the shared remark parser".to_string(),
            },
            LogEvent::ScopeRemove {
                path: "src/old.rs".to_string(),
            },
            LogEvent::Blocked {
                reason: "plan was wrong".to_string(),
            },
            LogEvent::Set {
                field: "source".to_string(),
                value: "gh issue 12".to_string(),
            },
            LogEvent::Set {
                field: "depends".to_string(),
                value: String::new(),
            },
        ];
        for event in events {
            let entry = LogEntry { date, event };
            let line = entry.to_string();
            assert_eq!(line.parse::<LogEntry>(), Ok(entry), "{line}");
        }
    }

    #[test]
    fn documented_examples_parse() {
        for line in [
            "2026-10-03 created: warn when ctx.md is stale while in progress",
            "2026-10-04 advance ready → in-progress/implement",
            "2026-10-04 advance in-progress/verify → in-progress/fix (attempts 1)",
        ] {
            assert!(line.parse::<LogEntry>().is_ok(), "{line}");
        }
    }

    #[test]
    fn section_errors_point_at_file_lines() {
        let text = "## Log\n\n- 2026-10-03 created: x\nnot a bullet\n";
        assert_eq!(parse_section(text, 10).unwrap_err().0, 13);
        let text = "## Log\n- 2026-10-03 frobnicated\n";
        assert_eq!(parse_section(text, 1).unwrap_err().0, 2);
    }

    #[test]
    fn bad_positions_are_rejected() {
        assert!(
            "2026-10-04 advance ready → doing"
                .parse::<LogEntry>()
                .is_err()
        );
        assert!(
            "2026-10-04 advance ready → in-progress/coding"
                .parse::<LogEntry>()
                .is_err()
        );
    }
}
