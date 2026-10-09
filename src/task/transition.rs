//! The legal status/stage moves, in one place, and a replay of a task's
//! `## Log` that rebuilds the state its frontmatter must match. `check` uses
//! both; the Phase 4 state commands reuse `is_legal`.

use crate::diag::Diagnostic;

use super::{Fix, LogEvent, Position, Stage, Status, Task};

/// Whether `advance` may move a task from `from` to `to`. Blocking is not an
/// advance: it is the `Blocked` event, legal from any open status.
pub fn is_legal(from: Position, to: Position) -> bool {
    use Stage::*;
    use Status::*;
    matches!(
        (from.status, from.stage, to.status, to.stage),
        (Draft, None, Ready, None)
            | (Ready, None, Draft, None)
            | (Ready, None, InProgress, Some(Implement))
            | (InProgress, Some(Implement), InProgress, Some(Verify))
            | (InProgress, Some(Verify), InProgress, Some(Review | Fix))
            | (InProgress, Some(Review), InProgress, Some(Fix))
            | (InProgress, Some(Fix), InProgress, Some(Verify))
            | (InProgress, Some(Review), Approval, None)
            | (Approval, None, InProgress, Some(Fix))
            | (Approval, None, Done, None)
            | (Blocked, None, Draft | Ready, None)
            | (Blocked, None, InProgress, Some(_))
    )
}

/// Every position, in workflow order.
fn all_positions() -> Vec<Position> {
    use Stage::*;
    use Status::*;
    let at = |status, stage| Position { status, stage };
    vec![
        at(Draft, None),
        at(Ready, None),
        at(InProgress, Some(Implement)),
        at(InProgress, Some(Verify)),
        at(InProgress, Some(Review)),
        at(InProgress, Some(Fix)),
        at(Approval, None),
        at(Done, None),
        at(Blocked, None),
    ]
}

/// The positions `advance` may move to from `from`.
pub fn targets(from: Position) -> Vec<Position> {
    all_positions()
        .into_iter()
        .filter(|&to| is_legal(from, to))
        .collect()
}

/// The happy-path successor `advance` takes without `--to`. `None` where the
/// move is a decision (approval, blocked) or there is nowhere to go (done).
pub fn next(from: Position) -> Option<Position> {
    use Stage::*;
    use Status::*;
    let to = match (from.status, from.stage) {
        (Draft, _) => (Ready, None),
        (Ready, _) => (InProgress, Some(Implement)),
        (InProgress, Some(Implement | Fix)) => (InProgress, Some(Verify)),
        (InProgress, Some(Verify)) => (InProgress, Some(Review)),
        (InProgress, Some(Review)) => (Approval, None),
        _ => return None,
    };
    Some(Position {
        status: to.0,
        stage: to.1,
    })
}

/// The steps from `from` to a later `to` along the happy path (`next`
/// repeated), `to` included; `None` when `to` isn't ahead. Never starts from
/// draft: approving the draft and its scope is the user's call, one step.
pub fn path(from: Position, to: Position) -> Option<Vec<Position>> {
    if from.status == Status::Draft {
        return None;
    }
    let mut steps = Vec::new();
    let mut at = from;
    while let Some(step) = next(at) {
        steps.push(step);
        if step == to {
            return Some(steps);
        }
        at = step;
    }
    None
}

/// Whether a task in `status` may be blocked.
pub fn can_block(status: Status) -> bool {
    !matches!(status, Status::Done | Status::Blocked)
}

/// The state a task's `## Log` replays to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Replayed {
    pub position: Position,
    pub attempts: u32,
    /// `None` until `scope approved`; in draft, scope is free.
    pub scope: Option<Vec<String>>,
    pub blocked_reason: Option<String>,
}

impl Default for Replayed {
    fn default() -> Self {
        Self {
            position: Position {
                status: Status::Draft,
                stage: None,
            },
            attempts: 0,
            scope: None,
            blocked_reason: None,
        }
    }
}

/// Replay `task.log` from `created`. Every rule violation becomes a `log-*`
/// error quoting the entry; replay continues best-effort so one bad entry
/// doesn't cascade.
pub fn replay(task: &Task) -> (Replayed, Vec<Diagnostic>) {
    let mut state = Replayed::default();
    let mut diags = Vec::new();
    let mut error = |code: &'static str, msg: String| {
        diags.push(Diagnostic::error(&task.path, None, code, msg));
    };

    let Some(first) = task.log.first() else {
        error(
            "log-missing",
            "`## Log` has no entries; expected `created` first".into(),
        );
        return (state, diags);
    };
    match &first.event {
        LogEvent::Created { title } => {
            if *title != task.title {
                error(
                    "log-title",
                    format!(
                        "title is `{}` but the log created it as `{title}`; titles are fixed at creation",
                        task.title
                    ),
                );
            }
            if first.date != task.front.created {
                error(
                    "log-created",
                    format!(
                        "`created: {}` but the log says `{first}`",
                        task.front.created
                    ),
                );
            }
        }
        _ => error(
            "log-created",
            format!("first log entry must be `created`, got `{first}`"),
        ),
    }

    for entry in task.log.iter().skip(1) {
        match &entry.event {
            LogEvent::Created { .. } => error(
                "log-created",
                format!("`{entry}`: `created` may only be the first entry"),
            ),
            LogEvent::Advance { from, to, fix } => {
                if *from != state.position {
                    error(
                        "log-transition",
                        format!(
                            "`{entry}`: task was `{}` at this point",
                            state.position
                        ),
                    );
                } else if !is_legal(*from, *to) {
                    error(
                        "log-transition",
                        format!("`{entry}`: `{from} → {to}` is not a legal move"),
                    );
                }
                let entering_fix = to.stage == Some(Stage::Fix);
                let next = state.attempts + 1;
                // A failed verify is a repair attempt. Review notes and user
                // feedback are revisions; logs from before revisions existed
                // count them as attempts, which stays valid.
                match (entering_fix, fix) {
                    (true, Some(Fix::Attempt(n))) if *n == next => {}
                    (true, Some(Fix::Revision))
                        if from.stage != Some(Stage::Verify) => {}
                    (true, _) if from.stage == Some(Stage::Verify) => error(
                        "log-attempts",
                        format!(
                            "`{entry}`: a failed verify must record `(attempts {next})`"
                        ),
                    ),
                    (true, _) => error(
                        "log-attempts",
                        format!(
                            "`{entry}`: entering fix must record `(revision)` or `(attempts {next})`"
                        ),
                    ),
                    (false, Some(_)) => error(
                        "log-attempts",
                        format!(
                            "`{entry}`: only moves into fix record attempts or revisions"
                        ),
                    ),
                    (false, None) => {}
                }
                // Best effort: follow the recorded count even when it's
                // wrong, so one bad entry doesn't cascade.
                if let (true, Some(Fix::Attempt(n))) = (entering_fix, fix) {
                    state.attempts = *n;
                }
                if to.status == Status::InProgress
                    && from.status == Status::Ready
                    && state.scope.is_none()
                {
                    error(
                        "log-scope",
                        format!("`{entry}`: started without an approved scope"),
                    );
                }
                if to.status == Status::Draft {
                    state.scope = None;
                }
                if from.status == Status::Blocked {
                    state.blocked_reason = None;
                }
                state.position = *to;
            }
            LogEvent::ScopeApproved(paths) => {
                if state.position.status != Status::Ready {
                    error(
                        "log-scope",
                        format!(
                            "`{entry}`: scope is approved only while ready (task was `{}`)",
                            state.position
                        ),
                    );
                }
                state.scope = Some(paths.clone());
            }
            LogEvent::ScopeAdd { path, .. } => {
                if let Some(scope) = &mut state.scope
                    && !scope.contains(path)
                {
                    scope.push(path.clone());
                }
            }
            LogEvent::ScopeRemove { path } => {
                if let Some(scope) = &mut state.scope {
                    scope.retain(|p| p != path);
                }
            }
            LogEvent::Blocked { reason } => {
                if !can_block(state.position.status) {
                    error(
                        "log-transition",
                        format!(
                            "`{entry}`: a `{}` task can't be blocked",
                            state.position
                        ),
                    );
                }
                state.position = Position {
                    status: Status::Blocked,
                    stage: None,
                };
                state.blocked_reason = Some(reason.clone());
            }
            LogEvent::Approved { .. } => {
                if state.position.status != Status::Approval {
                    error(
                        "log-transition",
                        format!(
                            "`{entry}`: approval is recorded only in approval (task was `{}`)",
                            state.position
                        ),
                    );
                }
            }
            LogEvent::Set { .. } | LogEvent::Baseline(_) => {}
        }
    }
    (state, diags)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    fn pos(s: &str) -> Position {
        s.parse().unwrap()
    }

    fn task(status: &str, log: &[&str]) -> Task {
        let mut content = format!(
            "---\nid: 0001-x\nstatus: {status}\nscope: [src/a.rs]\ncreated: 2026-10-03\n---\n# Task: x\n\n## Log\n\n"
        );
        for line in log {
            content.push_str(&format!("- {line}\n"));
        }
        Task::parse(Path::new("specs/tasks/0001-x.md"), &content).unwrap()
    }

    fn codes(diags: &[Diagnostic]) -> Vec<&'static str> {
        diags.iter().map(|d| d.code).collect()
    }

    #[test]
    fn legal_moves() {
        for (from, to) in [
            ("draft", "ready"),
            ("ready", "draft"),
            ("ready", "in-progress/implement"),
            ("in-progress/implement", "in-progress/verify"),
            ("in-progress/verify", "in-progress/review"),
            ("in-progress/verify", "in-progress/fix"),
            ("in-progress/review", "in-progress/fix"),
            ("in-progress/fix", "in-progress/verify"),
            ("in-progress/review", "approval"),
            ("approval", "in-progress/fix"),
            ("approval", "done"),
            ("blocked", "draft"),
            ("blocked", "ready"),
            ("blocked", "in-progress/review"),
        ] {
            assert!(is_legal(pos(from), pos(to)), "{from} → {to}");
        }
    }

    #[test]
    fn illegal_moves() {
        for (from, to) in [
            ("draft", "in-progress/implement"),
            ("ready", "approval"),
            ("in-progress/implement", "in-progress/review"),
            ("in-progress/verify", "approval"),
            ("in-progress/fix", "in-progress/review"),
            ("approval", "ready"),
            ("done", "draft"),
            ("draft", "blocked"),
            ("blocked", "approval"),
            ("in-progress/implement", "in-progress/implement"),
        ] {
            assert!(!is_legal(pos(from), pos(to)), "{from} → {to}");
        }
        assert!(!can_block(Status::Done));
        assert!(can_block(Status::Approval));
    }

    #[test]
    fn targets_and_default_successor() {
        let show = |ps: Vec<Position>| -> Vec<String> {
            ps.iter().map(ToString::to_string).collect()
        };
        assert_eq!(
            show(targets(pos("in-progress/verify"))),
            ["in-progress/review", "in-progress/fix"]
        );
        assert_eq!(show(targets(pos("approval"))), ["in-progress/fix", "done"]);
        assert!(targets(pos("done")).is_empty());

        // Every default successor is legal; decisions have none.
        for p in all_positions() {
            if let Some(n) = next(p) {
                assert!(is_legal(p, n), "{p} → {n}");
            }
        }
        assert_eq!(
            next(pos("in-progress/fix")),
            Some(pos("in-progress/verify"))
        );
        assert_eq!(next(pos("approval")), None);
        assert_eq!(next(pos("blocked")), None);
        assert_eq!(next(pos("done")), None);
    }

    #[test]
    fn forward_paths() {
        let show = |ps: Option<Vec<Position>>| -> Option<Vec<String>> {
            ps.map(|ps| ps.iter().map(ToString::to_string).collect())
        };
        assert_eq!(
            show(path(pos("in-progress/implement"), pos("approval"))),
            Some(vec![
                "in-progress/verify".into(),
                "in-progress/review".into(),
                "approval".into()
            ])
        );
        assert_eq!(
            show(path(pos("ready"), pos("in-progress/verify"))),
            Some(vec![
                "in-progress/implement".into(),
                "in-progress/verify".into()
            ])
        );
        // Never backwards, never off the happy path, never out of draft.
        assert_eq!(
            path(pos("in-progress/review"), pos("in-progress/verify")),
            None
        );
        assert_eq!(
            path(pos("in-progress/implement"), pos("in-progress/fix")),
            None
        );
        assert_eq!(path(pos("draft"), pos("in-progress/implement")), None);
        assert_eq!(path(pos("approval"), pos("done")), None);
    }

    #[test]
    fn happy_path_replays_cleanly() {
        let t = task(
            "approval",
            &[
                "2026-10-03 created: x",
                "2026-10-03 advance draft → ready",
                "2026-10-03 scope approved: src/a.rs, src/b.rs",
                "2026-10-04 advance ready → in-progress/implement",
                "2026-10-04 scope add src/c.rs — needed",
                "2026-10-04 scope rm src/b.rs",
                "2026-10-04 advance in-progress/implement → in-progress/verify",
                "2026-10-04 advance in-progress/verify → in-progress/fix (attempts 1)",
                "2026-10-04 advance in-progress/fix → in-progress/verify",
                "2026-10-04 advance in-progress/verify → in-progress/review",
                "2026-10-04 advance in-progress/review → in-progress/fix (attempts 2)",
                "2026-10-04 advance in-progress/fix → in-progress/verify",
                "2026-10-04 advance in-progress/verify → in-progress/review",
                "2026-10-04 advance in-progress/review → approval",
            ],
        );
        let (state, diags) = replay(&t);
        assert!(diags.is_empty(), "{diags:?}");
        assert_eq!(state.position, pos("approval"));
        assert_eq!(state.attempts, 2);
        assert_eq!(
            state.scope,
            Some(vec!["src/a.rs".into(), "src/c.rs".into()])
        );
    }

    #[test]
    fn missing_and_misplaced_created() {
        let (_, diags) = replay(&task("draft", &[]));
        assert_eq!(codes(&diags), ["log-missing"]);

        let (_, diags) = replay(&task(
            "ready",
            &["2026-10-03 advance draft → ready", "2026-10-03 created: x"],
        ));
        assert_eq!(codes(&diags), ["log-created", "log-created"]);
    }

    #[test]
    fn rename_and_created_date_are_caught() {
        let (_, diags) = replay(&task("draft", &["2026-10-02 created: old name"]));
        assert_eq!(codes(&diags), ["log-title", "log-created"]);
    }

    #[test]
    fn wrong_from_illegal_move_and_attempts() {
        let (state, diags) = replay(&task(
            "in-progress",
            &[
                "2026-10-03 created: x",
                "2026-10-03 advance ready → in-progress/implement",
                "2026-10-03 advance draft → in-progress/implement",
                "2026-10-03 advance in-progress/implement → in-progress/fix",
            ],
        ));
        assert_eq!(
            codes(&diags),
            [
                "log-transition",
                "log-scope",
                "log-transition",
                "log-transition",
                "log-attempts"
            ]
        );
        assert_eq!(state.position, pos("in-progress/fix"));
        assert_eq!(state.attempts, 0, "an unmarked move isn't counted");
    }

    #[test]
    fn revisions_are_logged_but_not_counted() {
        let start = [
            "2026-10-03 created: x",
            "2026-10-03 advance draft → ready",
            "2026-10-03 scope approved: src/a.rs",
            "2026-10-04 advance ready → in-progress/implement",
            "2026-10-04 advance in-progress/implement → in-progress/verify",
            "2026-10-04 advance in-progress/verify → in-progress/review",
            "2026-10-04 advance in-progress/review → approval",
        ];
        let mut log = start.to_vec();
        for _ in 0..4 {
            log.extend([
                "2026-10-05 advance approval → in-progress/fix (revision)",
                "2026-10-05 advance in-progress/fix → in-progress/verify",
                "2026-10-05 advance in-progress/verify → in-progress/review",
                "2026-10-05 advance in-progress/review → approval",
            ]);
        }
        let (state, diags) = replay(&task("approval", &log));
        assert!(diags.is_empty(), "{diags:?}");
        assert_eq!(state.attempts, 0);

        // A failed verify is a repair: it must be counted.
        let mut log = start[..5].to_vec();
        log.push(
            "2026-10-05 advance in-progress/verify → in-progress/fix (revision)",
        );
        let (_, diags) = replay(&task("in-progress", &log));
        assert_eq!(codes(&diags), ["log-attempts"]);
    }

    #[test]
    fn scope_rules() {
        let (state, diags) = replay(&task(
            "draft",
            &[
                "2026-10-03 created: x",
                "2026-10-03 scope add src/a.rs — early",
                "2026-10-03 scope approved: src/a.rs",
                "2026-10-03 advance draft → ready",
                "2026-10-03 scope approved: src/a.rs",
                "2026-10-03 advance ready → draft",
            ],
        ));
        assert_eq!(codes(&diags), ["log-scope"]);
        assert_eq!(state.scope, None, "ready → draft clears the approval");
    }

    #[test]
    fn block_and_unblock() {
        let (state, diags) = replay(&task(
            "ready",
            &[
                "2026-10-03 created: x",
                "2026-10-03 blocked: waiting on 0002",
                "2026-10-04 advance blocked → ready",
            ],
        ));
        assert!(diags.is_empty(), "{diags:?}");
        assert_eq!(state.position, pos("ready"));
        assert_eq!(state.blocked_reason, None);

        let (state, diags) = replay(&task(
            "blocked",
            &[
                "2026-10-03 created: x",
                "2026-10-03 blocked: a",
                "2026-10-03 blocked: b",
            ],
        ));
        assert_eq!(codes(&diags), ["log-transition"]);
        assert_eq!(state.blocked_reason.as_deref(), Some("b"));
    }
}
