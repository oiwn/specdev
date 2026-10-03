# Current Task Context: New specdev version and updated workflow
State: in progress (started 2026-10-02)

## Plan

Design and Phases 1–2 are done (CHANGELOG.md, 2026-10-03).

Phase 3 — `specdev check`:
- [ ] Per-task: required fields/sections, enum values, `stage` iff `in-progress`, `blocked_reason` iff `blocked`, id matches filename, `depends` ids exist
- [ ] Log consistency: frontmatter equals state replayed from `## Log`; every logged transition legal; `scope` = approved + logged expansions
- [ ] Repo-wide: at most one active task; `_index.md` matches generated output
- [ ] Fold existing `status` warnings (ctx forbidden content, archive nudge) into `check`; `status` keeps its summary view

Phase 4 — state commands: `task advance` / `task block` / `task set` / `task scope`:
- [ ] Transition table in one place (`task::transition`), used by `advance` and by `check`'s log replay
- [ ] `advance`: legal moves only; `draft → ready` refuses open `^^^` and records approved scope; `ready → in-progress` refuses if another task is active; `→ fix` bumps `attempts`; cap → `blocked`
- [ ] `block --reason`, `set <field> <value>` (plain fields only: `source`, `depends`, extra fields), `scope add|rm <path> --reason`
- [ ] Every command: rewrite frontmatter + append `## Log` line only, regenerate `_index.md`, never touch prose

Phase 5 — scope check + Acceptance gate (specdev runs no project commands):
- [ ] `vcs` module: `changed_files()` via the `git` CLI, read-only (`git diff --name-only -z HEAD`, `git diff --cached --name-only -z`, `git ls-files --others --exclude-standard -z`); faked in tests
- [ ] `check`: working-tree files outside the active task's `scope` → error; `check --staged`: staged files outside `scope` → warning; wire `--staged` into `prek.toml`
- [ ] Without git: scope check skipped with a warning, rest still runs
- [ ] `advance` refuses `verify → review` while any `## Acceptance` checkbox is unticked
- [ ] `task new` pre-fills `## Acceptance` with `acceptance.default` as unchecked items; add `Manual checks` to default `optional_sections`

Phase 6 — `task done`:
- [ ] Requires status `approval`; moves file to `specs/tasks/done/`, appends a dated entry to `CHANGELOG.md` (title + source + one-line summary), regenerates `_index.md`

Phase 7 — quality gate + `fmt`:
- [ ] `Metrics` per file (extends `list --stats`): lines, words, sections, max heading depth, plan steps, scope size, code blocks, tables, open `^^^`
- [ ] Thresholds from `[quality.*]`; reported by `check` as warnings (errors if configured); CHANGELOG size warning
- [ ] `fmt [<file>...]`: unwrap hard-wrapped paragraphs/bullets to soft wraps, normalize list markers and blank lines around headings; leave code blocks, frontmatter, `^^^`/`&&&` lines alone; `check` flags unformatted files

Phase 8 — skill, docs, dogfood:
- [ ] `skills/SKILL.md`: task workflow per stage, which commands own which state, scope contract, `ctx.md` as working context
- [ ] `README.md`, `specs/overview.md`, `references/examples.md` (a full task lifecycle example)
- [ ] Dogfood: run 2–3 real specdev tasks through the pipeline; tune quality thresholds

Each phase ends green: `cargo build && cargo test && cargo clippy --all-targets -- -D warnings`.

## Data types

Sketch, to be refined in Phase 1. Names are Rust.

```rust
enum Status { Draft, Ready, InProgress, Approval, Done, Blocked }
enum Stage { Implement, Verify, Review, Fix }  // hardcoded
struct TaskId { seq: u32, slug: String }    // Display: "0007-status-freshness"

struct Frontmatter {
    id: TaskId,
    status: Status,
    stage: Option<Stage>,                   // Some iff InProgress
    scope: Vec<String>,                     // globs, non-empty
    created: NaiveDate,                     // chrono::NaiveDate, YYYY-MM-DD
    source: Option<String>,                 // free-form
    depends: Vec<TaskId>,
    attempts: u32,                          // default 0
    blocked_reason: Option<String>,         // Some iff Blocked
    extra: Vec<(String, String)>,           // unknown/extra fields, original order
}

struct Task {
    path: PathBuf,
    front: Frontmatter,
    title: String,                          // from "# Task: ..."
    body: String,                           // prose, preserved verbatim
    log: Vec<LogEntry>,                     // parsed "## Log", owned by specdev
}

struct LogEntry { date: NaiveDate, event: LogEvent }
enum LogEvent {
    Created { title: String },              // title recorded so renames are caught
    Advance { from: (Status, Option<Stage>), to: (Status, Option<Stage>) },
    ScopeApproved(Vec<String>),
    ScopeAdd { path: String, reason: String },
    ScopeRemove { path: String },
    Blocked { reason: String },
    Set { field: String, value: String },
}

struct Config { task: TaskDef, pipeline: Pipeline, acceptance: Acceptance, quality: Quality }
struct TaskDef { required_sections: Vec<String>, optional_sections: Vec<String>, extra_fields: Vec<String> }
struct Pipeline { max_attempts: u32 }
struct Acceptance { default: Vec<String> }
struct Quality { task: Limits, changelog: Limits, /* ctx, overview */ }

struct Diagnostic { file: PathBuf, line: Option<usize>, severity: Severity, code: &'static str, message: String }
enum Severity { Warning, Error }

struct Metrics { lines, words, sections, max_depth, plan_steps, scope_len, code_blocks, tables, open_remarks: usize }

enum Format { Text, Json, Toon }            // global --format flag
trait Report: Serialize { fn text(&self) -> String; }   // every command returns one; output::emit renders by Format
```

Log location: per task — a `## Log` section at the end of each `specs/tasks/<id>.md`, written only by specdev commands. Line format (one line per event, human-readable, parseable):

```
- 2026-10-03 created: warn when ctx.md is stale while in progress
- 2026-10-03 advance draft → ready
- 2026-10-03 scope approved: src/status.rs, tests/cli.rs
- 2026-10-04 advance ready → in-progress/implement
- 2026-10-04 scope add src/scan.rs — needs the shared remark parser
- 2026-10-04 advance in-progress/verify → in-progress/fix (attempts 1)
```

^^^ where this log will be? common, separated file or per task?
&&& Per task, as the last section of the task file (now stated above the format block; also in `factory.md` → Task contract). Why not the alternatives: a common log file would be one more shared hot file every command writes to (conflicts once tasks run in parallel) and would split a task's history from the task; a separate `<id>.log` file doubles the file count and can drift from its task on rename/archive. In-file, the log moves to `done/` with the task, and `check` validates each task file on its own.

## Findings
- Dependency tree (runtime, unique crates; 2026-10-03): 23 → 53 after adding the Phase 1–2 set. Per-crate subtrees, overlapping: `comrak` 0.55 (defaults off) 12 — caseless, finl_unicode, jetscii, phf ×3, rustc-hash, smallvec, tinyvec, typed-arena, unicode-normalization; `toon-format` 0.5 (defaults off) 17, but all shared (serde, serde_json, indexmap, thiserror); `serde_json` 8; `toml` 1.x 7; `serde` derive 7 (syn/quote/proc-macro2 already came with thiserror); `chrono` 0.4 (clock, std) 4 — iana-time-zone + core-foundation-sys on macOS (Windows gets windows-* instead). `glob` deferred to Phase 3.
- Phases 1–2 landed 2026-10-03; gate green (109 unit + 15 e2e tests, clippy `-D warnings`). Layout: `src/{md,diag,output,config}.rs`, `src/task.rs` + `src/task/{id,frontmatter,log,store,cmd}.rs`.
- comrak counts `1. [x]` as a task item (GFM); `md` counts only bullet-list task items to keep the existing semantics.
- Fixed in passing: `list --stats` `[ ]` column printed the total checkbox count; it now prints unchecked.
- `scan`, `status`, `init`, `skill` are still text-only; `--format json|toon` on them exits 1 with "does not support --format … yet" until they migrate (Phase 3 for scan/status).
- Task lookup (`task show`) accepts the full id, the sequence number (`1`, `0001`), or the slug. Broken task files are warnings on stderr, and their number stays reserved.
- `TaskId` and `Status` serialize as strings; TOON quotes ids that start with digits (`"0001-x"`).

## Context
- Source of truth for the design: `specs/factory.md` (compressed 2026-10-03). This file only holds the build order and types.
- Key decisions: specdev is an agent-agnostic tool, not an orchestrator; one active task, others queued; `scope` approved at `draft → ready`; state changes only via commands, validated against the per-task `## Log` (no git dependency); task contract defined in `specdev.toml`; human owns branches, PR, merge; git is only ever read (`check`, `check --staged`); specdev never runs project commands (tests, builds) — the agent does; macOS first, then Linux, no Windows.
- Dependencies (decided 2026-10-03; keep the tree small, justify any addition):
  - Frontmatter: hand-written parser for the flat-YAML subset (scalars + inline lists); no YAML crate.
  - Markdown: `comrak` with `default-features = false` (no syntect, no CLI) — an investment, since most of specdev is Markdown work. comrak's round-trip is not byte-preserving, so: read via AST + source positions; state commands splice frontmatter/`## Log` edits by position; only `fmt` re-renders. `^^^`/`&&&` lines sit inside paragraphs as soft breaks — `fmt` must keep them on their own lines.
  - Config: `toml` + `serde` derive.
  - Output: `serde_json` for JSON; `toon-format` for TOON (official implementation from the toon-format org, github.com/toon-format/toon-rust) with `default-features = false` — its default `cli` feature pulls clap, ratatui, syntect, chrono, and more. Required deps with defaults off not yet confirmed.
  - Scope matching: `glob`.
  - Dates: `chrono` with `default-features = false, features = ["clock", "std"]` (local date) — tentative; check what it pulls in.
  - Git: call the `git` CLI, read-only; no `git2`/`gix` bindings (C dependency, large tree, can diverge from real `git`). Reconsider only if specdev ever needs history.
- `factory.md` has no open design questions (2026-10-03); quality thresholds start as placeholders and get tuned in Phase 8 dogfooding.

## Next
User review of Phases 1–2 (try `specdev task new|list|show|index` in a scratch dir); then Phase 3 — `specdev check` (adds `glob`).
