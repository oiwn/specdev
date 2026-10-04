//! State commands: `task advance|block|set|scope`. Each rewrites only the
//! frontmatter and appends to `## Log` (through `Task::render`), regenerates
//! `_index.md`, and refuses a change that would add `specdev check` errors.

use std::path::Path;

use chrono::Local;
use serde::Serialize;

use crate::Error;
use crate::check;
use crate::config::Config;
use crate::diag::{Diagnostic, Severity};
use crate::md;
use crate::output::{self, Format, Report};
use crate::scan;

use super::frontmatter::Value;
use super::store::TaskStore;
use super::transition;
use super::{LogEntry, LogEvent, Position, Stage, Status, Task, TaskId};

/// What a handler did: the log events to append and an optional note.
struct Change {
    events: Vec<LogEvent>,
    note: Option<String>,
}

impl Change {
    fn log(events: Vec<LogEvent>) -> Self {
        Self { events, note: None }
    }
}

/// An open task loaded for a state change, with its diagnostics before the
/// change (the baseline for the safety net).
pub(super) struct Loaded {
    pub config: Config,
    pub store: TaskStore,
    pub task: Task,
    pub before: Vec<Diagnostic>,
}

/// Load the task matching `query` and refuse if its frontmatter already
/// disagrees with its log: state can't be built on a broken history.
pub(super) fn load(root: &Path, query: &str) -> Result<Loaded, Error> {
    let config = Config::load(root)?;
    let store = TaskStore::open(root)?;
    let task = find_open(&store, query)?.clone();
    let before = check::task_diagnostics(&task, &store, &config, root);
    let broken: Vec<&Diagnostic> = before
        .iter()
        .filter(|d| d.severity == Severity::Error && d.code.starts_with("log-"))
        .collect();
    if !broken.is_empty() {
        return Err(Error::Usage(format!(
            "`{}` disagrees with its own log; repair the file before changing its state:\n{}",
            task.front.id,
            lines(&broken)
        )));
    }
    Ok(Loaded {
        config,
        store,
        task,
        before,
    })
}

/// Append `events` dated today; returns the rendered log lines.
pub(super) fn append(task: &mut Task, events: Vec<LogEvent>) -> Vec<String> {
    let date = Local::now().date_naive();
    let entries: Vec<LogEntry> = events
        .into_iter()
        .map(|event| LogEntry { date, event })
        .collect();
    let lines = entries.iter().map(ToString::to_string).collect();
    task.log.extend(entries);
    lines
}

/// Refuse when the changed `task` has check errors the original didn't.
pub(super) fn refuse_new_errors(
    loaded: &Loaded,
    task: &Task,
    root: &Path,
) -> Result<(), Error> {
    let after = check::task_diagnostics(task, &loaded.store, &loaded.config, root);
    let added: Vec<&Diagnostic> = after
        .iter()
        .filter(|d| d.severity == Severity::Error && !loaded.before.contains(d))
        .collect();
    if added.is_empty() {
        return Ok(());
    }
    Err(Error::Usage(format!(
        "refused: this change would make `specdev check` fail for `{}`:\n{}",
        task.front.id,
        lines(&added)
    )))
}

/// Load the task, run `change` on a copy, refuse if the result adds check
/// errors, then write the task and the index and report.
fn apply(
    root: &Path,
    query: &str,
    format: Format,
    change: impl FnOnce(&mut Task, &TaskStore, &Config) -> Result<Change, Error>,
) -> Result<(), Error> {
    let mut loaded = load(root, query)?;
    let mut task = loaded.task.clone();
    let change = change(&mut task, &loaded.store, &loaded.config)?;
    let log = append(&mut task, change.events);
    refuse_new_errors(&loaded, &task, root)?;

    loaded.store.write_task(&task)?;
    let report = StateReport {
        id: task.front.id.clone(),
        from: loaded.task.position().to_string(),
        to: task.position().to_string(),
        log,
        note: change.note,
        index: String::new(),
    };
    if let Some(slot) = loaded
        .store
        .tasks
        .iter_mut()
        .find(|t| t.front.id == task.front.id)
    {
        *slot = task;
    }
    let index = loaded.store.write_index()?;
    output::emit(
        &StateReport {
            index: index.display().to_string(),
            ..report
        },
        format,
    )
}

/// An open task (not in `done/`) matching `query`.
fn find_open<'a>(store: &'a TaskStore, query: &str) -> Result<&'a Task, Error> {
    let task = store.find(query).ok_or_else(|| {
        Error::Usage(format!("no task matching `{query}` in specs/tasks/"))
    })?;
    if store.done.iter().any(|t| t.front.id == task.front.id) {
        return Err(Error::Usage(format!(
            "`{}` is archived in done/; its state is final",
            task.front.id
        )));
    }
    Ok(task)
}

fn lines(diags: &[&Diagnostic]) -> String {
    diags
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n")
}

pub(super) fn usage(msg: impl Into<String>) -> Error {
    Error::Usage(msg.into())
}

/// A required, single-line text argument (log entries are one line each).
fn one_line<'a>(what: &str, value: &'a str) -> Result<&'a str, Error> {
    let value = value.trim();
    if value.is_empty() {
        return Err(usage(format!("{what} must not be empty")));
    }
    if value.contains(['\n', '\r']) {
        return Err(usage(format!("{what} must be a single line")));
    }
    Ok(value)
}

fn list(positions: &[Position]) -> String {
    if positions.is_empty() {
        return "none".to_string();
    }
    positions
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

/// `--to` value: a status, a stage (meaning `in-progress/<stage>`), or a
/// full `in-progress/<stage>` position.
fn parse_target(s: &str) -> Result<Position, Error> {
    if let Ok(stage) = s.parse::<Stage>() {
        return Ok(Position {
            status: Status::InProgress,
            stage: Some(stage),
        });
    }
    let target: Position = s.parse().map_err(Error::Usage)?;
    match (target.status, target.stage) {
        (Status::Blocked, _) => Err(usage(
            "use `specdev task block <id> --reason \"...\"` to block a task",
        )),
        (Status::Done, _) => Err(usage(
            "finish a task with `specdev task done <id>` (after the merge)",
        )),
        (Status::InProgress, None) => Err(usage(
            "`in-progress` needs a stage: implement, verify, review, or fix",
        )),
        (_, Some(_)) if target.status != Status::InProgress => {
            Err(usage(format!("`{s}`: only in-progress has a stage")))
        }
        _ => Ok(target),
    }
}

pub fn advance(
    root: &Path,
    query: &str,
    to: Option<&str>,
    format: Format,
) -> Result<(), Error> {
    apply(root, query, format, |task, store, config| {
        let id = task.front.id.clone();
        let from = task.position();
        let to = match to {
            Some(s) => parse_target(s)?,
            None => transition::next(from).ok_or_else(|| {
                usage(format!(
                    "`{id}` is `{from}`; choose where it goes with --to (legal: {})",
                    list(&transition::targets(from))
                ))
            })?,
        };
        if !transition::is_legal(from, to) {
            return Err(usage(format!(
                "`{id}` can't move from `{from}` to `{to}` (legal: {})",
                list(&transition::targets(from))
            )));
        }
        gates(task, store, from, to)?;

        let entering_fix = to.stage == Some(Stage::Fix);
        if entering_fix {
            let attempts = task.front.attempts + 1;
            let max = config.pipeline.max_attempts;
            if attempts > max {
                let reason = format!("attempts cap reached ({max})");
                task.front.status = Status::Blocked;
                task.front.stage = None;
                task.front.blocked_reason = Some(reason.clone());
                return Ok(Change {
                    events: vec![LogEvent::Blocked {
                        reason: reason.clone(),
                    }],
                    note: Some(format!(
                        "not moved to fix: {reason}. The task is blocked; the human decides what happens next."
                    )),
                });
            }
            task.front.attempts = attempts;
        }

        let mut events = vec![LogEvent::Advance {
            from,
            to,
            attempts: entering_fix.then_some(task.front.attempts),
        }];
        if from.status == Status::Draft && to.status == Status::Ready {
            events.push(LogEvent::ScopeApproved(task.front.scope.clone()));
        }
        if from.status == Status::Blocked {
            task.front.blocked_reason = None;
        }
        task.front.status = to.status;
        task.front.stage = to.stage;
        Ok(Change::log(events))
    })
}

/// Stage gates beyond the transition table.
fn gates(
    task: &Task,
    store: &TaskStore,
    from: Position,
    to: Position,
) -> Result<(), Error> {
    let id = &task.front.id;
    if from.status == Status::Draft && to.status == Status::Ready {
        let open = scan::count_markers(&task.body).0;
        if open > 0 {
            return Err(usage(format!(
                "`{id}` has {open} open ^^^ remark(s); resolve them and compress the dialogue into the plan first"
            )));
        }
    }
    if to.status == Status::InProgress
        && !from.status.is_active()
        && let Some(other) = store
            .tasks
            .iter()
            .find(|t| t.front.id != *id && t.front.status.is_active())
    {
        return Err(usage(format!(
            "`{}` is already active ({}); one task at a time",
            other.front.id,
            other.position()
        )));
    }
    let ticked = |section: &str| -> Result<(), Error> {
        let (done, total) = md::outline(&task.body).checkbox_counts_in(section);
        if done < total {
            return Err(usage(format!(
                "`{id}`: {} of {total} `## {section}` items unticked; tick them before moving on",
                total - done
            )));
        }
        Ok(())
    };
    match (from.stage, to.stage) {
        (Some(Stage::Implement), Some(Stage::Verify)) => ticked("Plan"),
        (Some(Stage::Verify), Some(Stage::Review)) => ticked("Acceptance"),
        _ => Ok(()),
    }
}

pub fn block(
    root: &Path,
    query: &str,
    reason: &str,
    format: Format,
) -> Result<(), Error> {
    let reason = one_line("--reason", reason)?.to_string();
    apply(root, query, format, |task, _, _| {
        if !transition::can_block(task.front.status) {
            return Err(usage(format!(
                "a `{}` task can't be blocked",
                task.position()
            )));
        }
        task.front.status = Status::Blocked;
        task.front.stage = None;
        task.front.blocked_reason = Some(reason.clone());
        Ok(Change::log(vec![LogEvent::Blocked { reason }]))
    })
}

/// Fields owned by other commands, with where to go instead.
const RESERVED: [(&str, &str); 7] = [
    ("id", "ids are fixed at creation"),
    ("created", "fixed at creation"),
    ("status", "use `task advance` or `task block`"),
    ("stage", "use `task advance`"),
    ("scope", "use `task scope <id> add|rm`"),
    ("attempts", "bumped by `task advance --to fix`"),
    ("blocked_reason", "use `task block --reason`"),
];

pub fn set(
    root: &Path,
    query: &str,
    field: &str,
    value: &str,
    format: Format,
) -> Result<(), Error> {
    if let Some((_, hint)) = RESERVED.iter().find(|(f, _)| *f == field) {
        return Err(usage(format!("`{field}` can't be set directly: {hint}")));
    }
    let value = value.trim();
    if value.contains(['\n', '\r']) {
        return Err(usage("value must be a single line"));
    }
    apply(root, query, format, |task, store, config| {
        let logged = match field {
            "source" => {
                task.front.source = (!value.is_empty()).then(|| value.to_string());
                value.to_string()
            }
            "depends" => {
                let mut ids: Vec<TaskId> = Vec::new();
                for q in value.split(',').map(str::trim).filter(|q| !q.is_empty()) {
                    let id = store
                        .find(q)
                        .ok_or_else(|| usage(format!("no task matching `{q}`")))?
                        .front
                        .id
                        .clone();
                    if !ids.contains(&id) {
                        ids.push(id);
                    }
                }
                task.front.depends = ids;
                task.front
                    .depends
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            }
            f if config.task.extra_fields.iter().any(|e| e == f) => {
                let extra = &mut task.front.extra;
                extra.retain(|(k, _)| k != f);
                if !value.is_empty() {
                    extra.push((f.to_string(), Value::Scalar(value.to_string())));
                }
                value.to_string()
            }
            f => {
                return Err(usage(format!(
                    "unknown field `{f}`; settable: source, depends, and `[task] extra_fields` from specdev.toml"
                )));
            }
        };
        Ok(Change::log(vec![LogEvent::Set {
            field: field.to_string(),
            value: logged,
        }]))
    })
}

/// A scope entry; `, ` and ` — ` separate fields in log lines.
fn scope_path(path: &str) -> Result<String, Error> {
    let path = one_line("path", path)?;
    if path.contains(", ") || path.contains(" — ") {
        return Err(usage("scope paths can't contain `, ` or ` — `"));
    }
    Ok(path.to_string())
}

pub fn scope_add(
    root: &Path,
    query: &str,
    path: &str,
    reason: &str,
    format: Format,
) -> Result<(), Error> {
    let path = scope_path(path)?;
    let reason = one_line("--reason", reason)?.to_string();
    apply(root, query, format, |task, _, _| {
        if task.front.scope.contains(&path) {
            return Err(usage(format!("`{path}` is already in scope")));
        }
        task.front.scope.push(path.clone());
        Ok(Change::log(vec![LogEvent::ScopeAdd { path, reason }]))
    })
}

pub fn scope_rm(
    root: &Path,
    query: &str,
    path: &str,
    format: Format,
) -> Result<(), Error> {
    let path = scope_path(path)?;
    apply(root, query, format, |task, _, _| {
        if !task.front.scope.contains(&path) {
            return Err(usage(format!("`{path}` is not in scope")));
        }
        task.front.scope.retain(|p| *p != path);
        Ok(Change::log(vec![LogEvent::ScopeRemove { path }]))
    })
}

#[derive(Serialize)]
pub(super) struct StateReport {
    pub id: TaskId,
    pub from: String,
    pub to: String,
    pub log: Vec<String>,
    pub note: Option<String>,
    pub index: String,
}

impl Report for StateReport {
    fn text(&self) -> String {
        let mut out = vec![if self.from == self.to {
            format!("{} — {}", self.id, self.to)
        } else {
            format!("{} — {} → {}", self.id, self.from, self.to)
        }];
        out.extend(self.log.iter().map(|line| format!("  log: {line}")));
        if let Some(note) = &self.note {
            out.push(format!("Note: {note}"));
        }
        out.push(format!("Updated {}", self.index));
        out.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn targets_parse() {
        for (input, want) in [
            ("ready", "ready"),
            ("draft", "draft"),
            ("approval", "approval"),
            ("fix", "in-progress/fix"),
            ("implement", "in-progress/implement"),
            ("in-progress/review", "in-progress/review"),
        ] {
            assert_eq!(parse_target(input).unwrap().to_string(), want);
        }
        for bad in ["blocked", "done", "in-progress", "ready/fix", "doing"] {
            assert!(parse_target(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn single_line_inputs() {
        assert_eq!(one_line("x", "  a reason ").unwrap(), "a reason");
        assert!(one_line("x", "  ").is_err());
        assert!(one_line("x", "a\nb").is_err());
        assert!(scope_path("src/a.rs, src/b.rs").is_err());
        assert!(scope_path("src/a — b.rs").is_err());
        assert_eq!(scope_path("src/**/*.rs").unwrap(), "src/**/*.rs");
    }
}
