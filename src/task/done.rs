//! `task done`: the last state change. Records `approval → done` in the log,
//! moves the task file to `specs/tasks/done/`, adds a dated `CHANGELOG.md`
//! entry built from the task's `## Summary`, and regenerates `_index.md`.

use std::fs;
use std::path::Path;

use chrono::NaiveDate;

use crate::Error;
use crate::md;
use crate::output::{self, Format};

use super::state::{self, StateReport, usage};
use super::{LogEvent, Position, Status, Task, TaskId};

const CHANGELOG: &str = "CHANGELOG.md";
const SUMMARY: &str = "Summary";
const MANUAL_CHECKS: &str = "Manual checks";

pub fn done(root: &Path, query: &str, format: Format) -> Result<(), Error> {
    let mut loaded = state::load(root, query)?;
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
                "`{id}`: {} of {total} `## {MANUAL_CHECKS}` items unticked; the human runs them before the task is done",
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
    let log = state::append(
        &mut task,
        vec![LogEvent::Advance {
            from,
            to,
            attempts: None,
        }],
    );
    let old_path = task.path.clone();
    let done_dir = loaded.store.done_dir();
    task.path = done_dir.join(old_path.file_name().unwrap_or_default());
    if task.path.exists() {
        return Err(usage(format!("{} already exists", task.path.display())));
    }
    state::refuse_new_errors(&loaded, &task, root)?;
    let changelog_path = root.join(CHANGELOG);
    let existing = if changelog_path.exists() {
        fs::read_to_string(&changelog_path)?
    } else {
        "# Changelog\n".to_string()
    };
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
    let changelog = insert_entry(&existing, &entry);

    fs::create_dir_all(&done_dir)?;
    loaded.store.write_task(&task)?;
    fs::remove_file(&old_path)?;
    fs::write(&changelog_path, changelog)?;
    loaded.store.tasks.retain(|t| t.front.id != id);
    let moved = task.path.display().to_string();
    loaded.store.done.push(task);
    let index = loaded.store.write_index()?;

    output::emit(
        &StateReport {
            id,
            from: from.to_string(),
            to: to.to_string(),
            log,
            note: Some(format!("Moved to {moved}; added a {CHANGELOG} entry")),
            index: index.display().to_string(),
        },
        format,
    )
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
