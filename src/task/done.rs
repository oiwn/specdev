//! `task done`: the last state change, for one task or a batch. Each task
//! records the user's approval (optional) and `approval → done` in its log,
//! moves to `specs/tasks/done/`, and gets a dated `CHANGELOG.md` entry built
//! from its `## Summary`; `_index.md` is regenerated once at the end.
//!
//! Every task is checked before anything is written, so one blocked task
//! stops the whole batch. Writes go changelog first, then the task file: a
//! failure in between leaves the task in approval, and the retry skips the
//! changelog entry it already wrote.

use std::fs;
use std::path::{Path, PathBuf};

use chrono::NaiveDate;
use serde::Serialize;

use crate::Error;
use crate::md;
use crate::output::{self, Format, Report};

use super::state::{self, StateReport, usage};
use super::store::TaskStore;
use super::{LogEvent, Position, Status, Task, TaskId};

pub const CHANGELOG: &str = "CHANGELOG.md";
const SUMMARY: &str = "Summary";
const MANUAL_CHECKS: &str = "Manual checks";

/// A task that passed every gate, with what closing it will write.
struct Closing {
    task: Task,
    old_path: PathBuf,
    entry: String,
    report: StateReport,
}

pub fn done(
    root: &Path,
    queries: &[String],
    approval: Option<&str>,
    dry_run: bool,
    format: Format,
) -> Result<(), Error> {
    let approval = approval
        .map(|a| state::one_line("--approval", a))
        .transpose()?;
    let mut closing: Vec<Closing> = Vec::new();
    let mut blockers = Vec::new();
    for query in queries {
        match prepare(root, query, approval) {
            Ok(c) if closing.iter().any(|o| o.task.front.id == c.task.front.id) => {
                blockers.push(format!("`{}` is listed twice", c.task.front.id));
            }
            Ok(c) => closing.push(c),
            Err(e) => blockers.push(e.to_string()),
        }
    }
    if !blockers.is_empty() {
        let what = if queries.len() > 1 {
            "nothing closed; fix these first:\n"
        } else {
            ""
        };
        return Err(usage(format!("{what}{}", blockers.join("\n"))));
    }

    let changelog_path = root.join(CHANGELOG);
    if !dry_run {
        let mut changelog = if changelog_path.exists() {
            fs::read_to_string(&changelog_path)?
        } else {
            "# Changelog\n".to_string()
        };
        for c in &closing {
            if !has_entry(&changelog, &c.task.front.id) {
                changelog = insert_entry(&changelog, &c.entry);
            }
        }
        fs::write(&changelog_path, changelog)?;
        let store = TaskStore::open(root)?;
        fs::create_dir_all(store.done_dir())?;
        for c in &closing {
            store.write_task(&c.task)?;
            fs::remove_file(&c.old_path)?;
        }
    }
    let index = if dry_run {
        None
    } else {
        Some(TaskStore::open(root)?.write_index()?.display().to_string())
    };

    output::emit(
        &DoneReport {
            dry_run,
            closed: closing.into_iter().map(|c| c.report).collect(),
            changelog: changelog_path.display().to_string(),
            index,
        },
        format,
    )
}

/// Load one task and run every gate; nothing is written.
fn prepare(
    root: &Path,
    query: &str,
    approval: Option<&str>,
) -> Result<Closing, Error> {
    let loaded = state::load(root, query)?;
    let id = loaded.task.front.id.clone();
    let from = loaded.task.position();
    if from.status != Status::Approval {
        return Err(usage(format!(
            "`{id}` is `{from}`; only tasks in approval can be done"
        )));
    }
    let outline = md::outline(&loaded.task.body);
    if outline.section(MANUAL_CHECKS).is_some() {
        let (ticked, total) = outline.checkbox_counts_in(MANUAL_CHECKS);
        if ticked < total {
            return Err(usage(format!(
                "`{id}`: {} of {total} `## {MANUAL_CHECKS}` items unticked; resolve each (agent or user evidence, or a user waiver) before the task is done",
                total - ticked
            )));
        }
    }
    let summary = summary(&loaded.task).ok_or_else(|| {
        usage(format!(
            "`{id}` needs a one-line `## {SUMMARY}` section; it becomes the CHANGELOG entry"
        ))
    })?;

    let to = Position {
        status: Status::Done,
        stage: None,
    };
    let mut task = loaded.task.clone();
    task.front.status = to.status;
    task.front.stage = to.stage;
    let mut events = Vec::new();
    if let Some(quote) = approval {
        events.push(LogEvent::Approved {
            quote: quote.to_string(),
        });
    }
    events.push(LogEvent::Advance {
        from,
        to,
        fix: None,
    });
    let log = state::append(&mut task, events);
    let old_path = task.path.clone();
    task.path = loaded
        .store
        .done_dir()
        .join(old_path.file_name().unwrap_or_default());
    if task.path.exists() {
        return Err(usage(format!("{} already exists", task.path.display())));
    }
    state::refuse_new_errors(&loaded, &task, root)?;
    let date = task
        .log
        .last()
        .map(|e| e.date)
        .unwrap_or(task.front.created);
    let entry = changelog_entry(
        date,
        &task.title,
        &summary,
        &id,
        task.front.source.as_deref(),
    );
    let report = StateReport {
        id,
        from: from.to_string(),
        to: to.to_string(),
        via: Vec::new(),
        log,
        note: Some(task.path.display().to_string()),
        index: String::new(),
    };
    Ok(Closing {
        task,
        old_path,
        entry,
        report,
    })
}

/// Whether the changelog already has this task's entry (a retried close).
fn has_entry(changelog: &str, id: &TaskId) -> bool {
    changelog.contains(&format!("- Task `{id}`"))
}

#[derive(Serialize)]
struct DoneReport {
    dry_run: bool,
    closed: Vec<StateReport>,
    changelog: String,
    index: Option<String>,
}

impl Report for DoneReport {
    fn text(&self) -> String {
        let mut out = Vec::new();
        if self.dry_run {
            out.push("Dry run: nothing written. Would close:".to_string());
        }
        // One line per task (the note is where it moves to); the log and
        // index details are in the JSON output.
        for r in &self.closed {
            let moved = r.note.as_deref().unwrap_or_default();
            out.push(format!("{} — {} → {} ({moved})", r.id, r.from, r.to));
        }
        let verb = if self.dry_run {
            "Would update"
        } else {
            "Updated"
        };
        out.push(format!("{verb} {}", self.changelog));
        out.join("\n")
    }
}

/// The first prose line of `## Summary`: blank lines, code blocks, sub-
/// headings, and `^^^`/`&&&` remarks are skipped; a list marker is dropped.
pub fn summary(task: &Task) -> Option<String> {
    let outline = md::outline(&task.body);
    let range = outline.section(SUMMARY)?;
    task.body
        .lines()
        .enumerate()
        .map(|(i, line)| (i + 1, line.trim()))
        .filter(|(n, _)| range.contains(n) && !outline.is_in_code(*n))
        .map(|(_, line)| line)
        .find(|line| {
            !line.is_empty()
                && !line.starts_with('#')
                && !line.starts_with("```")
                && !line.starts_with("^^^")
                && !line.starts_with("&&&")
        })
        .map(|line| {
            line.strip_prefix("- ")
                .or_else(|| line.strip_prefix("* "))
                .unwrap_or(line)
                .trim()
                .to_string()
        })
        .filter(|line| !line.is_empty())
}

fn changelog_entry(
    date: NaiveDate,
    title: &str,
    summary: &str,
    id: &TaskId,
    source: Option<&str>,
) -> String {
    let source = source.map(|s| format!("; source: {s}")).unwrap_or_default();
    format!("## {date} — {title}\n\n- {summary}\n- Task `{id}`{source}\n")
}

/// Insert `entry` above the newest entry (the first level-2 heading outside
/// code and HTML blocks), keeping every other byte; append when the
/// changelog has no entries yet.
fn insert_entry(existing: &str, entry: &str) -> String {
    let outline = md::outline(existing);
    match outline.headings.iter().find(|h| h.level == 2) {
        Some(h) => {
            let at: usize = existing
                .split_inclusive('\n')
                .take(h.line - 1)
                .map(str::len)
                .sum();
            format!("{}{entry}\n{}", &existing[..at], &existing[at..])
        }
        None => {
            let mut out = existing.to_string();
            if !out.is_empty() && !out.ends_with('\n') {
                out.push('\n');
            }
            if !out.is_empty() && !out.ends_with("\n\n") {
                out.push('\n');
            }
            out.push_str(entry);
            out
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn task_with(body: &str) -> Task {
        let content = format!(
            "---\nid: 0001-x\nstatus: approval\nscope: [a]\ncreated: 2026-10-04\n---\n# Task: x\n\n{body}\n## Log\n\n- 2026-10-04 created: x\n"
        );
        Task::parse(Path::new("specs/tasks/0001-x.md"), &content).unwrap()
    }

    #[test]
    fn summary_takes_the_first_prose_line() {
        let s = |body: &str| summary(&task_with(body));
        assert_eq!(
            s("## Summary\n\nWarns when ctx.md is stale.\nMore detail.\n")
                .as_deref(),
            Some("Warns when ctx.md is stale.")
        );
        assert_eq!(s("## Summary\n\n- a bullet\n").as_deref(), Some("a bullet"));
        assert_eq!(
            s("## Summary\n\n```\ncode\n```\n\n^^^ fix?\n\nreal one\n").as_deref(),
            Some("real one")
        );
        assert_eq!(s("## Summary\n\n## Plan\n\n- [x] a\n"), None);
        assert_eq!(s("## Plan\n\n- [x] a\n"), None);
    }

    const TEMPLATE: &str = "\
# Changelog

Intro.

<!-- Entry template:
## YYYY-MM-DD — <task title>

- what shipped
-->
";

    #[test]
    fn inserts_above_the_newest_entry() {
        let old = format!("{TEMPLATE}\n## 2026-10-03 — older\n\n- shipped\n");
        let out = insert_entry(&old, "## NEW\n\n- x\n");
        assert_eq!(
            out,
            format!(
                "{TEMPLATE}\n## NEW\n\n- x\n\n## 2026-10-03 — older\n\n- shipped\n"
            )
        );
    }

    #[test]
    fn appends_when_there_are_no_entries() {
        let out = insert_entry(TEMPLATE, "## NEW\n\n- x\n");
        assert_eq!(out, format!("{TEMPLATE}\n## NEW\n\n- x\n"));
        assert_eq!(
            insert_entry("# Changelog\n", "## NEW\n"),
            "# Changelog\n\n## NEW\n"
        );
        assert_eq!(insert_entry("", "## NEW\n"), "## NEW\n");
    }

    #[test]
    fn headings_in_code_blocks_are_not_entries() {
        let old = "# Changelog\n\n```\n## not an entry\n```\n\n## real\n";
        let out = insert_entry(old, "## NEW\n");
        assert_eq!(
            out,
            "# Changelog\n\n```\n## not an entry\n```\n\n## NEW\n\n## real\n"
        );
    }

    #[test]
    fn entry_format() {
        let date = NaiveDate::from_ymd_opt(2026, 10, 5).unwrap();
        let id: TaskId = "0007-status-freshness".parse().unwrap();
        assert_eq!(
            changelog_entry(
                date,
                "warn on stale ctx",
                "Warns.",
                &id,
                Some("gh-12")
            ),
            "## 2026-10-05 — warn on stale ctx\n\n- Warns.\n- Task `0007-status-freshness`; source: gh-12\n"
        );
        assert!(
            changelog_entry(date, "t", "s", &id, None)
                .ends_with("`0007-status-freshness`\n")
        );
    }
}
