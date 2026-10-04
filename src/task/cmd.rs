//! `specdev task ...` handlers. Each builds a report and emits it in the
//! requested format; files that fail to parse are reported on stderr.

use std::path::Path;

use chrono::Local;
use serde::Serialize;

use crate::Error;
use crate::config::Config;
use crate::md;
use crate::output::{self, Format, Report};
use crate::scan;

use super::frontmatter::DATE_FORMAT;
use super::store::TaskStore;
use super::{
    Frontmatter, LogEntry, LogEvent, Position, Stage, Status, Task, TaskId,
    validate_slug,
};

pub fn new(root: &Path, slug: &str, format: Format) -> Result<(), Error> {
    validate_slug(slug).map_err(Error::Usage)?;
    let config = Config::load(root)?;
    let mut store = TaskStore::open(root)?;
    report_problems(&store);

    let id = TaskId::new(store.next_seq(), slug).map_err(Error::Usage)?;
    let path = store.dir.join(format!("{id}.md"));
    if path.exists() {
        return Err(Error::Usage(format!("{} already exists", path.display())));
    }

    let today = Local::now().date_naive();
    let title = slug.replace('-', " ");
    let mut body = format!("# Task: {title}\n\n");
    for section in &config.task.required_sections {
        body.push_str(&format!("## {section}\n\n"));
        if section == "Acceptance" && !config.acceptance.default.is_empty() {
            for item in &config.acceptance.default {
                body.push_str(&format!("- [ ] `{item}`\n"));
            }
            body.push('\n');
        }
    }
    let task = Task {
        path: path.clone(),
        front: Frontmatter {
            id: id.clone(),
            status: Status::Draft,
            stage: None,
            scope: Vec::new(),
            created: today,
            source: None,
            depends: Vec::new(),
            attempts: 0,
            blocked_reason: None,
            extra: Vec::new(),
        },
        title: title.clone(),
        body,
        log: vec![LogEntry {
            date: today,
            event: LogEvent::Created { title },
        }],
    };
    store.write_task(&task)?;
    store.tasks.push(task);
    let index = store.write_index()?;

    output::emit(
        &NewReport {
            id,
            path: path.display().to_string(),
            index: index.display().to_string(),
        },
        format,
    )
}

pub fn list(root: &Path, format: Format) -> Result<(), Error> {
    let store = TaskStore::open(root)?;
    report_problems(&store);
    let tasks = store.queue().into_iter().map(TaskSummary::from).collect();
    output::emit(&ListReport { tasks }, format)
}

pub fn show(root: &Path, query: &str, format: Format) -> Result<(), Error> {
    let config = Config::load(root)?;
    let store = TaskStore::open(root)?;
    report_problems(&store);
    let task = store.find(query).ok_or_else(|| {
        Error::Usage(format!("no task matching `{query}` in specs/tasks/"))
    })?;

    let (done, total) = md::outline(&task.body).checkbox_counts_in("Plan");
    let front = &task.front;
    let report = ShowReport {
        id: front.id.clone(),
        path: task.path.display().to_string(),
        title: task.title.clone(),
        status: front.status,
        stage: front.stage,
        scope: front.scope.clone(),
        created: front.created.format(DATE_FORMAT).to_string(),
        source: front.source.clone(),
        depends: front.depends.clone(),
        attempts: front.attempts,
        max_attempts: config.pipeline.max_attempts,
        blocked_reason: front.blocked_reason.clone(),
        plan: Progress { done, total },
        open_remarks: scan::count_markers(&task.body).0,
        log: task.log.iter().map(ToString::to_string).collect(),
        next: next_step(task.position()).to_string(),
    };
    output::emit(&report, format)
}

pub fn index(root: &Path, format: Format) -> Result<(), Error> {
    let store = TaskStore::open(root)?;
    report_problems(&store);
    let path = store.write_index()?;
    output::emit(
        &IndexReport {
            path: path.display().to_string(),
            tasks: store.tasks.len() + store.done.len(),
        },
        format,
    )
}

fn report_problems(store: &TaskStore) {
    for problem in &store.problems {
        eprintln!("{problem}");
    }
}

/// What the skill says to do next, per status and stage.
fn next_step(position: Position) -> &'static str {
    match (position.status, position.stage) {
        (Status::Draft, _) => {
            "Fill Plan, scope, and Acceptance; the human reviews with ^^^. Once approved, compress the remarks and advance to ready."
        }
        (Status::Ready, _) => {
            "Queued. Starts when no other task is active: advance to in-progress."
        }
        (Status::InProgress, Some(Stage::Implement)) => {
            "Implement inside scope and tick Plan boxes; then advance to verify."
        }
        (Status::InProgress, Some(Stage::Verify)) => {
            "Run the Acceptance commands and the scope check; pass → review, fail → fix."
        }
        (Status::InProgress, Some(Stage::Review)) => {
            "Review the diff against Plan and scope, write ## Review; approve → approval, changes → fix."
        }
        (Status::InProgress, Some(Stage::Fix)) => {
            "Address ## Review notes or verify failures; then advance to verify."
        }
        (Status::InProgress, None) => "Invalid state: in-progress without a stage.",
        (Status::Approval, _) => {
            "Human: tick ## Manual checks, open the PR, and merge; with a one-line ## Summary in place, then `specdev task done <id>`."
        }
        (Status::Done, _) => "Done.",
        (Status::Blocked, _) => {
            "Blocked: the human decides — back to a stage, or back to draft."
        }
    }
}

#[derive(Serialize)]
struct NewReport {
    id: TaskId,
    path: String,
    index: String,
}

impl Report for NewReport {
    fn text(&self) -> String {
        format!("Created {}\nUpdated {}", self.path, self.index)
    }
}

#[derive(Serialize)]
struct TaskSummary {
    id: TaskId,
    status: Status,
    stage: Option<Stage>,
    title: String,
    source: Option<String>,
}

impl From<&Task> for TaskSummary {
    fn from(task: &Task) -> Self {
        Self {
            id: task.front.id.clone(),
            status: task.front.status,
            stage: task.front.stage,
            title: task.title.clone(),
            source: task.front.source.clone(),
        }
    }
}

#[derive(Serialize)]
struct ListReport {
    tasks: Vec<TaskSummary>,
}

impl Report for ListReport {
    fn text(&self) -> String {
        if self.tasks.is_empty() {
            return "No tasks.".to_string();
        }
        self.tasks
            .iter()
            .map(|t| {
                let position = Position {
                    status: t.status,
                    stage: t.stage,
                };
                format!(
                    "{:<28} {:<22} {}",
                    t.id.to_string(),
                    position.to_string(),
                    t.title
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[derive(Serialize)]
struct Progress {
    done: usize,
    total: usize,
}

#[derive(Serialize)]
struct ShowReport {
    id: TaskId,
    path: String,
    title: String,
    status: Status,
    stage: Option<Stage>,
    scope: Vec<String>,
    created: String,
    source: Option<String>,
    depends: Vec<TaskId>,
    attempts: u32,
    max_attempts: u32,
    blocked_reason: Option<String>,
    plan: Progress,
    open_remarks: usize,
    log: Vec<String>,
    next: String,
}

impl Report for ShowReport {
    fn text(&self) -> String {
        let position = Position {
            status: self.status,
            stage: self.stage,
        };
        let mut lines = vec![
            format!("{} — {}", self.id, position),
            format!("Title:    {}", self.title),
            format!("File:     {}", self.path),
        ];
        let scope = if self.scope.is_empty() {
            "(none yet)".to_string()
        } else {
            self.scope.join(", ")
        };
        lines.push(format!("Scope:    {scope}"));
        lines.push(format!("Created:  {}", self.created));
        if let Some(source) = &self.source {
            lines.push(format!("Source:   {source}"));
        }
        if !self.depends.is_empty() {
            let ids: Vec<String> =
                self.depends.iter().map(ToString::to_string).collect();
            lines.push(format!("Depends:  {}", ids.join(", ")));
        }
        if self.attempts > 0 {
            lines
                .push(format!("Attempts: {}/{}", self.attempts, self.max_attempts));
        }
        if let Some(reason) = &self.blocked_reason {
            lines.push(format!("Blocked:  {reason}"));
        }
        lines.push(format!(
            "Plan:     {}/{} steps done",
            self.plan.done, self.plan.total
        ));
        if self.open_remarks > 0 {
            lines.push(format!("Remarks:  {} open ^^^", self.open_remarks));
        }
        lines.push("Log:".to_string());
        lines.extend(self.log.iter().map(|entry| format!("  - {entry}")));
        lines.push(format!("Next:     {}", self.next));
        lines.join("\n")
    }
}

#[derive(Serialize)]
struct IndexReport {
    path: String,
    tasks: usize,
}

impl Report for IndexReport {
    fn text(&self) -> String {
        format!("Wrote {} ({} tasks)", self.path, self.tasks)
    }
}
