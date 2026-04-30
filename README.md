# specdev

Specification-driven development toolkit for humans and AI agents.

`specdev` provides a CLI and an agent skill that establish a structured workflow
around project specs. The CLI scaffolds and inspects specs. The skill teaches any
coding agent how to read specs, address user remarks, and keep specs current.

## Install

```bash
cargo install specdev
```

## Quick start

```bash
# Scaffold a specs/ directory in your project
specdev init

# Install the agent skill globally
specdev skill install
```

This creates:

```
specs/
  overview.md    # durable project overview
  ctx.md         # current task context (starts empty)
  ideas.md       # loose backlog
```

And installs the skill to `~/.agents/skills/specdev/SKILL.md`, where any
[Agent Skills](https://agentskills.io)-compatible agent will find it.

## CLI commands

### `specdev init`

Scaffold `specs/` with `overview.md`, `ctx.md`, and `ideas.md`. Does not
overwrite existing files.

### `specdev scan`

Scan `specs/` for `^^^` user remarks and report which are resolved (have an
adjacent `&&&` answer) and which are still open.

```
overview.md
  L 18  [resolved]  And general workflow with agent!
  L 23  [open]      need to highlight that this used only when editing specs

Total: 1 open, 10 resolved across 2 files
```

### `specdev status`

Show spec file health: which core files exist, line counts, last modified
times, and marker counts.

```
Core spec files:
  [OK] overview.md     294 lines  3h ago     11 markers (1 open)
  [OK] ctx.md           55 lines  2h ago
  [--] ideas.md       missing

Markers: 1 open, 10 resolved
```

### `specdev skill install`

Install the specdev agent skill. Defaults to `~/.agents/skills/specdev/`
(global). Use `--local` to install to `.agents/skills/specdev/` within the
current project.

## The spec workflow

### Core files

- **`specs/overview.md`** — durable project overview: architecture, data flow,
  public surfaces. Should be reasonably current across sessions.
- **`specs/ctx.md`** — current task context: active task, state, decisions,
  blockers, next steps. Updated during development. Empty when no task is
  active.
- **`specs/ideas.md`** — loose backlog for possible future work. Not a
  committed plan.

### Marker protocol

When editing spec files, `^^^` and `&&&` distinguish user remarks from agent
answers:

- `^^^` — user remark, question, correction, or TODO
- `&&&` — agent answer or resolution addressing a nearby `^^^`

The agent preserves both lines until the user asks to compress.

### Compression

When discussions in a spec are settled, the user can ask the agent to compress
it. The agent rewrites the spec, folding `^^^`/`&&&` dialogue into clean prose
that reflects the final decisions. Unresolved `^^^` remarks are preserved.

Compression is agent-driven, on user request only. There is no CLI command for
this.

## Agent skill

The bundled skill (`skills/SKILL.md`) follows the
[Agent Skills specification](https://agentskills.io). It teaches agents to:

1. Start each session by reading `specs/overview.md` and `specs/ctx.md`
2. Address `^^^` remarks before writing code
3. Keep specs current as decisions change
4. Compress specs on user request

The skill is intentionally concise (under 150 lines) with a separate
`references/examples.md` for before/after examples, loaded on demand via
progressive disclosure.

## License

MIT
