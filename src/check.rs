//! `specdev check`: validates every task file against the task contract and
//! its own `## Log`, the generated `_index.md`, and the spec-level rules
//! `status` reports. Any error exits 1; warnings alone exit 0.

use std::fs;
use std::path::Path;

use serde::Serialize;

use crate::Error;
use crate::config::Config;
use crate::diag::{Diagnostic, Severity};
use crate::md;
use crate::output::{self, Format, Report};
use crate::status;
use crate::task::transition;
use crate::task::{INDEX_FILE, Status, TASKS_DIR, Task, TaskId, TaskStore};
use crate::vcs;

pub fn run(root: &Path, staged: bool, format: Format) -> Result<(), Error> {
    if !root.join("specs").is_dir() {
        return Err(Error::Usage(
            "No specs/ directory found. Run `specdev init` first.".to_string(),
        ));
    }
    let config = Config::load(root)?;
    let store = if root.join(TASKS_DIR).is_dir() {
        Some(TaskStore::open(root)?)
    } else {
        None
    };

    let mut diagnostics = Vec::new();
    if let Some(store) = &store {
        diagnostics.extend(check_store(store, &config, root)?);
        diagnostics.extend(scope_check(store, &config, root, staged));
    }
    diagnostics.extend(status::spec_diagnostics(root)?);
    diagnostics.extend(ctx_task_refs(root, store.as_ref())?);

    let errors = diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .count();
    let report = CheckReport {
        warnings: diagnostics.len() - errors,
        errors,
        diagnostics,
    };
    output::emit(&report, format)?;
    match errors {
        0 => Ok(()),
        n => Err(Error::CheckFailed(n)),
    }
}

/// Every task in `specs/tasks/` and `done/`, plus the repo-wide rules.
fn check_store(
    store: &TaskStore,
    config: &Config,
    root: &Path,
) -> Result<Vec<Diagnostic>, Error> {
    let mut diags: Vec<Diagnostic> = store
        .problems
        .iter()
        .map(|d| d.clone().with_severity(Severity::Error))
        .collect();
    for task in &store.tasks {
        diags.extend(check_task(task, store, config, root, false));
    }
    for task in &store.done {
        diags.extend(check_task(task, store, config, root, true));
    }

    let active: Vec<&Task> = store
        .tasks
        .iter()
        .filter(|t| t.front.status.is_active())
        .collect();
    if active.len() > 1 {
        let ids: Vec<String> =
            active.iter().map(|t| t.front.id.to_string()).collect();
        for task in &active {
            diags.push(Diagnostic::error(
                &task.path,
                None,
                "active-tasks",
                format!(
                    "{} tasks are active ({}); only one may be in-progress or approval",
                    active.len(),
                    ids.join(", ")
                ),
            ));
        }
    }

    // `init` creates `specs/tasks/` without an index; that's fine until the
    // first task exists.
    let index = store.dir.join(INDEX_FILE);
    let current = fs::read_to_string(&index).ok();
    let no_tasks = store.tasks.is_empty() && store.done.is_empty();
    let fresh = current.is_none() && no_tasks;
    if !fresh && current.as_deref() != Some(store.render_index().as_str()) {
        let what = if current.is_some() {
            "out of date"
        } else {
            "missing"
        };
        diags.push(Diagnostic::error(
            index,
            None,
            "index-stale",
            format!("{what}; run `specdev task index`"),
        ));
    }
    Ok(diags)
}

/// The per-task rules for an open task; state commands run it before and
/// after a change and refuse changes that add errors.
pub fn task_diagnostics(
    task: &Task,
    store: &TaskStore,
    config: &Config,
    root: &Path,
) -> Vec<Diagnostic> {
    check_task(task, store, config, root, false)
}

fn check_task(
    task: &Task,
    store: &TaskStore,
    config: &Config,
    root: &Path,
    archived: bool,
) -> Vec<Diagnostic> {
    let front = &task.front;
    let mut diags = Vec::new();
    let mut error = |code: &'static str, msg: String| {
        diags.push(Diagnostic::error(&task.path, None, code, msg));
    };

    let stem = task.path.file_stem().unwrap_or_default().to_string_lossy();
    if stem != front.id.to_string() {
        error(
            "id-filename",
            format!("id `{}` doesn't match the file name `{stem}.md`", front.id),
        );
    }
    let in_progress = front.status == Status::InProgress;
    if in_progress != front.stage.is_some() {
        error(
            "stage",
            "`stage` is required iff status is `in-progress`".to_string(),
        );
    }
    let blocked = front.status == Status::Blocked;
    if blocked != front.blocked_reason.is_some() {
        error(
            "blocked-reason",
            "`blocked_reason` is required iff status is `blocked`".to_string(),
        );
    }
    if archived && front.status != Status::Done {
        error(
            "done-status",
            format!("in done/ but status is `{}`", front.status),
        );
    }

    let outline = md::outline(&task.body);
    for section in &config.task.required_sections {
        if outline.section(section).is_none() {
            error(
                "section-missing",
                format!("missing required section `## {section}`"),
            );
        }
    }
    for dep in &front.depends {
        if *dep == front.id {
            error("depends", "a task can't depend on itself".to_string());
        } else if store.find(&dep.to_string()).is_none() {
            error("depends", format!("depends on unknown task `{dep}`"));
        }
    }

    if !archived && front.status != Status::Done {
        diags.extend(completeness(task, &outline));
        diags.extend(scope_lint(task, root));
    }
    diags.extend(log_consistency(task));
    diags
}

/// Empty scope / Plan / Acceptance: warnings while a task is still being
/// written (draft) or parked (blocked), errors once it is approved.
fn completeness(task: &Task, outline: &md::Outline) -> Vec<Diagnostic> {
    let severity = match task.front.status {
        Status::Draft | Status::Blocked => Severity::Warning,
        _ => Severity::Error,
    };
    let mut diags = Vec::new();
    let mut push = |code: &'static str, msg: &str| {
        diags.push(
            Diagnostic::error(&task.path, None, code, msg).with_severity(severity),
        );
    };
    if task.front.scope.is_empty() {
        push(
            "scope-empty",
            "`scope` is empty; list the files this task may change",
        );
    }
    for (section, code) in
        [("Plan", "plan-empty"), ("Acceptance", "acceptance-empty")]
    {
        if outline.section(section).is_some()
            && outline.checkbox_counts_in(section).1 == 0
        {
            push(code, &format!("`## {section}` has no checkbox items"));
        }
    }
    diags
}

/// Scope entries: never match everything; point at paths that exist or at
/// new files in an existing directory.
fn scope_lint(task: &Task, root: &Path) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    for entry in &task.front.scope {
        if matches!(entry.as_str(), "*" | "**" | "**/*") {
            diags.push(Diagnostic::error(
                &task.path,
                None,
                "scope-glob",
                format!("scope entry `{entry}` matches everything; list files"),
            ));
            continue;
        }
        if let Err(e) = glob::Pattern::new(entry) {
            diags.push(Diagnostic::error(
                &task.path,
                None,
                "scope-glob",
                format!("scope entry `{entry}` is not a valid glob: {e}"),
            ));
            continue;
        }
        let literal = entry
            .split(['*', '?', '['])
            .next()
            .unwrap_or_default()
            .trim_end_matches('/');
        if literal.is_empty() {
            continue;
        }
        let path = Path::new(literal);
        let parent_exists = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .is_none_or(|p| root.join(p).exists());
        if !root.join(path).exists() && !parent_exists {
            diags.push(Diagnostic::warning(
                &task.path,
                None,
                "scope-path",
                format!("scope entry `{entry}`: neither `{literal}` nor its directory exists"),
            ));
        }
    }
    diags
}

/// Changed files (from git) against the active task's scope. Working-tree
/// violations are errors; `--staged` (the pre-commit hook) only warns.
fn scope_check(
    store: &TaskStore,
    config: &Config,
    root: &Path,
    staged: bool,
) -> Vec<Diagnostic> {
    let active: Vec<&Task> = store
        .tasks
        .iter()
        .filter(|t| t.front.status.is_active())
        .collect();
    // No active task: nothing to compare. Several: `active-tasks` reports it.
    let [task] = active.as_slice() else {
        return Vec::new();
    };
    match vcs::changed_files(root, staged) {
        vcs::Changes::Files(changed) => {
            let severity = if staged {
                Severity::Warning
            } else {
                Severity::Error
            };
            scope_violations(
                task,
                root,
                &changed,
                &config.scope.always_allowed,
                severity,
            )
        }
        vcs::Changes::Unavailable(reason) => vec![Diagnostic::warning(
            &task.path,
            None,
            "scope-skipped",
            format!("scope check skipped: {reason}"),
        )],
    }
}

/// Changed paths (relative to `root`) that neither the task's scope, `specs/`,
/// nor the project's `always_allowed` globs cover.
fn scope_violations(
    task: &Task,
    root: &Path,
    changed: &[String],
    always_allowed: &[String],
    severity: Severity,
) -> Vec<Diagnostic> {
    let id = &task.front.id;
    changed
        .iter()
        .filter(|path| {
            !path.starts_with("specs/")
                && !always_allowed.iter().any(|e| covers(e, path))
                && !task.front.scope.iter().any(|e| covers(e, path))
        })
        .map(|path| {
            Diagnostic::error(
                root.join(path),
                None,
                "out-of-scope",
                format!(
                    "changed but not in the scope of `{id}`; if the task needs it: `specdev task scope {id} add {path} --reason \"...\"`"
                ),
            )
            .with_severity(severity)
        })
        .collect()
}

/// Whether a scope entry covers `path`: a glob match (`*` stays inside a
/// directory, `**` crosses), or for a plain entry the path itself or
/// anything under it.
fn covers(entry: &str, path: &str) -> bool {
    let entry = entry.strip_prefix("./").unwrap_or(entry);
    if !entry.contains(['*', '?', '[']) {
        let entry = entry.trim_end_matches('/');
        return path == entry
            || path
                .strip_prefix(entry)
                .is_some_and(|rest| rest.starts_with('/'));
    }
    let options = glob::MatchOptions {
        case_sensitive: true,
        require_literal_separator: true,
        require_literal_leading_dot: false,
    };
    glob::Pattern::new(entry).is_ok_and(|p| p.matches_with(path, options))
}

/// Frontmatter must equal the state its `## Log` replays to.
fn log_consistency(task: &Task) -> Vec<Diagnostic> {
    let (replayed, mut diags) = transition::replay(task);
    if diags.iter().any(|d| d.code == "log-missing") {
        return diags;
    }
    let front = &task.front;
    let mut mismatch = |what: &str, front_value: String, log_value: String| {
        diags.push(Diagnostic::error(
            &task.path,
            None,
            "log-mismatch",
            format!(
                "frontmatter {what} is `{front_value}` but the log replays to `{log_value}`; change state with specdev commands, not by hand"
            ),
        ));
    };
    if task.position() != replayed.position {
        mismatch(
            "status",
            task.position().to_string(),
            replayed.position.to_string(),
        );
    }
    if front.attempts != replayed.attempts {
        mismatch(
            "attempts",
            front.attempts.to_string(),
            replayed.attempts.to_string(),
        );
    }
    if front.status == Status::Blocked
        && front.blocked_reason.is_some()
        && front.blocked_reason != replayed.blocked_reason
    {
        mismatch(
            "blocked_reason",
            front.blocked_reason.clone().unwrap_or_default(),
            replayed.blocked_reason.clone().unwrap_or_default(),
        );
    }
    if let Some(approved) = &replayed.scope {
        let mut want = approved.clone();
        let mut have = front.scope.clone();
        want.sort();
        have.sort();
        if want != have {
            mismatch("scope", have.join(", "), want.join(", "));
        }
    }
    diags
}

/// `specs/tasks/<id>.md` references in `ctx.md` (outside code blocks) that
/// point at a task which isn't active.
fn ctx_task_refs(
    root: &Path,
    store: Option<&TaskStore>,
) -> Result<Vec<Diagnostic>, Error> {
    let path = root.join("specs").join("ctx.md");
    let (Some(store), true) = (store, path.exists()) else {
        return Ok(Vec::new());
    };
    let content = fs::read_to_string(&path)?;
    let outline = md::outline(&content);
    let prefix = format!("{TASKS_DIR}/");
    let mut diags = Vec::new();
    for (i, line) in content.lines().enumerate() {
        if outline.is_in_code(i + 1) {
            continue;
        }
        for (start, _) in line.match_indices(&prefix) {
            let rest = &line[start + prefix.len()..];
            let Some(id) = rest
                .split_once(".md")
                .and_then(|(id, _)| id.parse::<TaskId>().ok())
            else {
                continue;
            };
            if let Some(task) = store.find(&id.to_string())
                && !task.front.status.is_active()
            {
                diags.push(Diagnostic::warning(
                    &path,
                    Some(i + 1),
                    "ctx-task-inactive",
                    format!(
                        "points at task `{id}`, which is `{}`, not active",
                        task.position()
                    ),
                ));
            }
        }
    }
    Ok(diags)
}

#[derive(Serialize)]
struct CheckReport {
    diagnostics: Vec<Diagnostic>,
    errors: usize,
    warnings: usize,
}

impl Report for CheckReport {
    fn text(&self) -> String {
        if self.diagnostics.is_empty() {
            return "OK".to_string();
        }
        let mut out: Vec<String> =
            self.diagnostics.iter().map(ToString::to_string).collect();
        out.push(format!(
            "{} {}, {} {}",
            self.errors,
            plural(self.errors, "error"),
            self.warnings,
            plural(self.warnings, "warning")
        ));
        out.join("\n")
    }
}

fn plural(n: usize, word: &str) -> String {
    if n == 1 {
        word.to_string()
    } else {
        format!("{word}s")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    const VALID_READY: &str = "\
---
id: 0001-a
status: ready
scope: [src/a.rs]
created: 2026-10-03
---
# Task: a

## Plan

- [ ] do it

## Acceptance

- [ ] cargo test

## Log

- 2026-10-03 created: a
- 2026-10-03 advance draft → ready
- 2026-10-03 scope approved: src/a.rs
";

    /// A project dir with `src/`, the given task files, and a fresh index.
    fn project(files: &[(&str, &str)]) -> TempDir {
        let tmp = TempDir::new().unwrap();
        let tasks = tmp.path().join(TASKS_DIR);
        fs::create_dir_all(tasks.join("done")).unwrap();
        fs::create_dir_all(tmp.path().join("src")).unwrap();
        for (name, content) in files {
            fs::write(tasks.join(name), content).unwrap();
        }
        let store = TaskStore::open(tmp.path()).unwrap();
        store.write_index().unwrap();
        tmp
    }

    fn codes(tmp: &TempDir) -> Vec<(&'static str, Severity)> {
        let store = TaskStore::open(tmp.path()).unwrap();
        check_store(&store, &Config::default(), tmp.path())
            .unwrap()
            .iter()
            .map(|d| (d.code, d.severity))
            .collect()
    }

    use Severity::{Error as E, Warning as W};

    #[test]
    fn valid_ready_task_passes() {
        let tmp = project(&[("0001-a.md", VALID_READY)]);
        assert_eq!(codes(&tmp), []);
    }

    #[test]
    fn hand_edited_status_is_a_log_mismatch() {
        let edited = VALID_READY.replace("status: ready", "status: draft");
        let tmp = project(&[("0001-a.md", &edited)]);
        assert_eq!(codes(&tmp), [("log-mismatch", E)]);
    }

    #[test]
    fn hand_edited_scope_after_approval_is_caught() {
        let edited =
            VALID_READY.replace("scope: [src/a.rs]", "scope: [src/a.rs, src/b.rs]");
        let tmp = project(&[("0001-a.md", &edited)]);
        assert_eq!(codes(&tmp), [("log-mismatch", E)]);
    }

    #[test]
    fn fresh_draft_only_warns() {
        let draft = "---\nid: 0001-a\nstatus: draft\nscope: []\ncreated: 2026-10-03\n---\n# Task: a\n\n## Plan\n\n## Acceptance\n\n## Log\n\n- 2026-10-03 created: a\n";
        let tmp = project(&[("0001-a.md", draft)]);
        assert_eq!(
            codes(&tmp),
            [
                ("scope-empty", W),
                ("plan-empty", W),
                ("acceptance-empty", W)
            ]
        );
    }

    #[test]
    fn completeness_is_an_error_once_ready() {
        let empty = VALID_READY.replace("- [ ] cargo test\n", "");
        let tmp = project(&[("0001-a.md", &empty)]);
        assert_eq!(codes(&tmp), [("acceptance-empty", E)]);
    }

    #[test]
    fn structural_rules() {
        let bad = VALID_READY
            .replace("id: 0001-a", "id: 0001-b")
            .replace(
                "status: ready\n",
                "status: ready\nstage: verify\ndepends: [0009-nope]\n",
            )
            .replace("## Plan\n\n- [ ] do it\n\n", "")
            .replace(
                "scope: [src/a.rs]",
                "scope: [src/a.rs, \"**\", nowhere/x.rs]",
            )
            .replace("approved: src/a.rs", "approved: src/a.rs, **, nowhere/x.rs");
        let tmp = project(&[("0001-a.md", &bad)]);
        assert_eq!(
            codes(&tmp),
            [
                ("id-filename", E),
                ("stage", E),
                ("section-missing", E),
                ("depends", E),
                ("scope-glob", E),
                ("scope-path", W),
                ("log-mismatch", E),
            ]
        );
    }

    #[test]
    fn repo_wide_rules() {
        let active = |id: &str| {
            format!(
                "{}- 2026-10-03 advance ready → in-progress/implement\n",
                VALID_READY.replace("0001-a", id)
            )
            .replace("status: ready", "status: in-progress\nstage: implement")
        };
        let tmp = project(&[
            ("0001-a.md", &active("0001-a")),
            ("0002-b.md", &active("0002-b")),
        ]);
        fs::write(tmp.path().join(TASKS_DIR).join("0003-broken.md"), "nope\n")
            .unwrap();
        fs::write(
            tmp.path().join(TASKS_DIR).join("done/0000-old.md"),
            VALID_READY.replace("0001-a", "0000-old"),
        )
        .unwrap();
        assert_eq!(
            codes(&tmp),
            [
                ("frontmatter", E),
                ("done-status", E),
                ("active-tasks", E),
                ("active-tasks", E),
                ("index-stale", E),
            ]
        );
    }

    #[test]
    fn ctx_pointing_at_inactive_task_warns() {
        let tmp = project(&[("0001-a.md", VALID_READY)]);
        fs::write(
            tmp.path().join("specs/ctx.md"),
            "# Ctx\n\nActive: specs/tasks/0001-a.md\n\n```\nspecs/tasks/0001-a.md\n```\n",
        )
        .unwrap();
        let store = TaskStore::open(tmp.path()).unwrap();
        let diags = ctx_task_refs(tmp.path(), Some(&store)).unwrap();
        assert_eq!(diags.len(), 1, "{diags:?}");
        assert_eq!(
            (diags[0].code, diags[0].line),
            ("ctx-task-inactive", Some(3))
        );
    }

    #[test]
    fn scope_entries_cover_paths() {
        for (entry, path) in [
            ("src/a.rs", "src/a.rs"),
            ("./src/a.rs", "src/a.rs"),
            ("src/*.rs", "src/a.rs"),
            ("src/**/*.rs", "src/task/state.rs"),
            ("src/**/*.rs", "src/a.rs"),
            ("src/task", "src/task/state.rs"),
            ("src/task/", "src/task/state.rs"),
            ("Cargo.lock", "Cargo.lock"),
        ] {
            assert!(covers(entry, path), "{entry} should cover {path}");
        }
        for (entry, path) in [
            ("src/a.rs", "src/a.rs.bak"),
            ("src/*.rs", "src/task/state.rs"),
            ("src/task", "src/tasks.rs"),
            ("src/task", "src/taskx/a.rs"),
            ("src/[", "src/["),
        ] {
            assert!(!covers(entry, path), "{entry} should not cover {path}");
        }
    }

    #[test]
    fn scope_violations_skip_specs_and_allowlist() {
        let task = Task::parse(
            Path::new("specs/tasks/0001-a.md"),
            &VALID_READY
                .replace("scope: [src/a.rs]", "scope: [src/a.rs, src/task]"),
        )
        .unwrap();
        let changed: Vec<String> = [
            "src/a.rs",
            "src/task/state.rs",
            "specs/ctx.md",
            "specs/tasks/_index.md",
            "Cargo.lock",
            "src/b.rs",
            "README.md",
        ]
        .map(String::from)
        .to_vec();
        let allowed = ["Cargo.lock".to_string()];
        let diags = scope_violations(
            &task,
            Path::new(""),
            &changed,
            &allowed,
            Severity::Error,
        );
        let flagged: Vec<String> =
            diags.iter().map(|d| d.file.display().to_string()).collect();
        assert_eq!(flagged, ["src/b.rs", "README.md"]);
        assert!(diags.iter().all(|d| d.code == "out-of-scope"));
        assert!(diags[0].message.contains("task scope 0001-a add src/b.rs"));

        let warned = scope_violations(
            &task,
            Path::new(""),
            &changed,
            &[],
            Severity::Warning,
        );
        assert_eq!(warned.len(), 3, "Cargo.lock flagged without the allowlist");
        assert!(warned.iter().all(|d| d.severity == Severity::Warning));
    }

    #[test]
    fn invalid_glob_in_scope_is_an_error() {
        let bad = VALID_READY
            .replace("scope: [src/a.rs]", "scope: [src/a.rs, \"src/[\"]")
            .replace("approved: src/a.rs", "approved: src/a.rs, src/[");
        let tmp = project(&[("0001-a.md", &bad)]);
        assert_eq!(codes(&tmp), [("scope-glob", E)]);
    }
}
