# Current Task Context: New specdev version and updated workflow
State: in progress (started 2026-10-02)

## Plan

Design and Phases 1–5 are done (CHANGELOG.md, 2026-10-03 and 2026-10-04).

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

Task, frontmatter, log, config, and diagnostic types are implemented (`src/task.rs`, `src/task/*`, `src/config.rs`, `src/diag.rs`); the per-task `## Log` format is documented in `src/task/log.rs` and `factory.md`. Still ahead, for Phase 7:

```rust
struct Metrics { lines, words, sections, max_depth, plan_steps, scope_len, code_blocks, tables, open_remarks: usize }
```

## Findings
- Dependency tree (runtime, unique crates; 2026-10-03): 23 → 53 after adding the Phase 1–2 set. Per-crate subtrees, overlapping: `comrak` 0.55 (defaults off) 12 — caseless, finl_unicode, jetscii, phf ×3, rustc-hash, smallvec, tinyvec, typed-arena, unicode-normalization; `toon-format` 0.5 (defaults off) 17, but all shared (serde, serde_json, indexmap, thiserror); `serde_json` 8; `toml` 1.x 7; `serde` derive 7 (syn/quote/proc-macro2 already came with thiserror); `chrono` 0.4 (clock, std) 4 — iana-time-zone + core-foundation-sys on macOS (Windows gets windows-* instead). `glob` 0.3 added in Phase 5 (2026-10-04): +1 crate, no dependencies of its own; `globset` rejected (regex + aho-corasick).
- Transition table (`transition::is_legal`): draft ⇄ ready; ready → implement; implement → verify; verify → review | fix; review → fix | approval; fix → verify; approval → fix | done; blocked → draft | ready | any stage. Blocking is the `Blocked` log event, legal from any status but done/blocked.
- Replay rules: `created` first (title and date must match frontmatter); `scope approved` only while ready, required before ready → in-progress; `ready → draft` drops the approval; scope add/rm count only after approval (draft scope is free); `set` events are informational — `source`/`depends` aren't compared yet.
- `advance` without `--to` takes the happy-path successor (draft → ready → implement → verify → review → approval; fix → verify); from approval and blocked `--to` is required. `--to` takes a status or a bare stage name. `--to done` points at `task done` (Phase 6), `--to blocked` at `task block`.
- Gates (user decision 2026-10-04): no open `^^^` for draft → ready; all Plan boxes ticked for implement → verify; all Acceptance boxes ticked for verify → review; one active task. Entering fix past `max_attempts` blocks the task instead (`blocked: attempts cap reached (N)`), exit 0 with a note.
- Safety net: every state command runs `check::task_diagnostics` before and after on a copy and refuses if the change adds errors (exact diagnostic match), so commands never write a file `check` rejects. Any `log-*` error beforehand refuses outright: the history must be repaired first.
- `set` with an empty value clears the field and logs `set <field> ""`. `depends` values resolve through `TaskStore::find`, so `set 3 depends 1,slug` works. Scope paths can't contain `, ` or ` — ` (log separators).
- `check` codes: `frontmatter`/`title`/`log` (parse), `id-filename`, `stage`, `blocked-reason`, `done-status`, `section-missing`, `depends`, `scope-empty`, `plan-empty`, `acceptance-empty`, `scope-glob`, `scope-path` (warning), `log-missing`, `log-created`, `log-title`, `log-transition`, `log-attempts`, `log-scope`, `log-mismatch`, `active-tasks`, `index-stale`, `ctx-all-done`, `ctx-forbidden`, `root-file-missing`, `ctx-task-inactive` (warnings).
- Completeness (`scope-empty`, `plan-empty`, `acceptance-empty`) is a warning in draft and blocked, an error otherwise (user decision 2026-10-04). A missing `_index.md` is fine while there are no tasks (`init` doesn't write one).
- Log diagnostics have no line numbers: `LogEntry` doesn't keep its source line. Follow-up if agents need them.
- Scope check rules (user decision 2026-10-04): runs only with exactly one active task; never flags anything under `specs/` or matching `[scope] always_allowed`; a scope entry without glob chars also covers everything under it. Working tree → `out-of-scope` errors, `--staged` → warnings, no git / not a repo → one `scope-skipped` warning.
- Changed files = `git diff HEAD` + untracked, so earlier commits of a multi-commit task aren't re-checked (fits the one-commit task target). Before the first commit, every file counts as changed. Paths come from `--relative` / `ls-files`, so a project in a repo subdirectory works. `specdev.toml` is not exempt; allowlist it if config changes are routine.
- The git environment passes through unchanged so `--staged` inside a hook reads the hook's `GIT_INDEX_FILE` (verified: the suite passes with `GIT_DIR`/`GIT_INDEX_FILE`/`GIT_WORK_TREE` set).
- comrak counts `1. [x]` as a task item (GFM); `md` counts only bullet-list task items to keep the existing semantics.
- `init` and `skill` are still text-only; `--format json|toon` on them exits 1 with "does not support --format … yet".
- E2e tests: `skill` tests point `HOME` at a tempdir; every spawned command strips git hook variables and sets `GIT_CEILING_DIRECTORIES` (`isolated()` in `tests/cli.rs`).
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
  - Scope matching: `glob` 0.3 (zero deps), `require_literal_separator` so `*` stays in a directory and `**` crosses.
  - Dates: `chrono` with `default-features = false, features = ["clock", "std"]` (local date); 4 crates (see Findings).
  - Git: call the `git` CLI, read-only; no `git2`/`gix` bindings (C dependency, large tree, can diverge from real `git`). Reconsider only if specdev ever needs history.
- `factory.md` has no open design questions (2026-10-03); quality thresholds start as placeholders and get tuned in Phase 8 dogfooding.

## Next
User review of Phase 5 (in a scratch git repo: walk a task to `implement`, change a file outside scope, `check`). Then Phase 6 — `task done`: approval → done, move to `done/`, CHANGELOG entry, index.
