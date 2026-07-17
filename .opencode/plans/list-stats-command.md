# Plan: ctx.md header + `list`/`list --stats` commands

Status: agreed, ready to execute (blocked by plan mode). Exit plan mode to run.

## Locked decisions

- **ctx.md H1** = `# Current Task Context: <one-line task>`; no-task state = header-only
  (`# Current Task Context`), not blank. init writes header-only; archive resets to
  header-only.
- **One command**: `specdev list` (file / first-header description / lines) and
  `specdev list --stats` (structural breakdown).
- **Base stats**: `#`/`##`/`###`/`H4+` heading counts + checkboxes `[x]`/`[ ]`.
- **Extras**: open `^^^` count column + word count column.
- **Scope**: `specs/*.md` only (consistent with `scan`/`status`).

## Output mockups

`specdev list`:
```
File            Description                                Lines
overview.md     specdev — overview                           48
ctx.md          Current Task Context: list + stats           18
roadmap.md      Roadmap                                      30
ideas.md        Ideas                                        12
cleanup.md      Cleanup                                       5
```

`specdev list --stats`:
```
File            H1  H2  H3  H4+   [x]  [ ]   ^^^   words
overview.md      1   3   0   0      0    0     0      312
ctx.md           1   4   0   0      0    6     0       88
roadmap.md       1   2   0   0      5    1     0      120
ideas.md         1   0   0   0      0    0     0       40
cleanup.md       1   0   0   0      0    0     0       24
```
Description column truncated (~40 chars) with ellipsis if needed. Files ordered:
canonical five (`overview, ctx, roadmap, ideas, cleanup`) then extras alphabetical.
No `specs/` dir -> same "Run `specdev init` first" message as `scan`/`status`.

## Execution steps

### 1. New module `src/list.rs`
- `pub fn run(stats: bool) -> Result<(), Error>` — lists `specs/*.md`; if `stats`,
  print the stats table, else the list table.
- Helpers (pure, unit-tested):
  - `first_header(content) -> Option<String>` — first `# ` line, text without the
    leading `#`s.
  - `count_headings(content) -> (h1, h2, h3, h4plus)` — bucket by leading-`#` count.
  - reuse `status::count_checkboxes` and `scan::count_markers` (open count).
  - word count: `content.split_whitespace().count()`.
- Canonical file order constant shared with status/scan if helpful.

### 2. `src/main.rs`
- Add subcommand:
  ```rust
  /// List spec files; --stats for a structural breakdown
  List {
      #[arg(long)]
      stats: bool,
  },
  ```
- Dispatch: `Commands::List { stats } => list::run(stats)`.

### 3. `src/init.rs` — ctx template
- Change ctx template from `""` to `"# Current Task Context\n"`.
- Rename/replace `ctx_starts_empty` test -> `ctx_has_header_only` (asserts the
  header line, and that no task body is present).

### 4. ctx.md header ripple (docs/templates)
- `skills/SKILL.md`, root `AGENTS.md`, `AGENTS_TEMPLATE` in `init.rs`, `README.md`:
  update the ctx.md soft-template example to `# Current Task Context: <one line>`,
  and state the no-task/archive-reset state is header-only (not blank).
- Keep the forbidden-content rules unchanged (header "Current Task Context" triggers
  none of them).

### 5. Dogfood
- Write `specs/ctx.md` as the new active task:
  `# Current Task Context: list + stats command`, State in progress, checkbox plan
  (the steps above), narrow Context, Next.
- After verify passes, tick boxes and archive into `CHANGELOG.md` (new dated entry),
  reset ctx.md to header-only.

### 6. Docs
- `README.md`: document `list` and `list --stats` with the mockups.

### 7. Verify
- `cargo build && cargo test && cargo clippy -- -D warnings`
- `cargo run -- list` and `cargo run -- list --stats` on the dogfooded repo.
