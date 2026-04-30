# Current Task: Design specdev CLI + Skill

## State: v0.1 CLI implemented, all commands working

Implemented the `specdev` CLI — a Rust binary with 4 commands, 20 tests, clippy
clean. The companion agent skill (`SKILL.md`) is bundled and installable.

## Decisions

- **Name**: `specdev` (not `sdd`)
- **Skill install path**: `~/.agents/skills/specdev/` (global) or `.agents/skills/specdev/` (local)
- **Compression is agent-driven**: No `compress` CLI command. When the user
  requests it, the agent rewrites the spec folding resolved `^^^`/`&&&`
  dialogue into clean prose. The agent understands semantics — the CLI does not
  rewrite specs.
- **Skill format**: Follows Agent Skills specification (agentskills.io). YAML
  frontmatter with `name` and `description` (max 1024 chars), markdown body
  under 500 lines / 5000 tokens. Use `references/` for longer examples.
- **Marker protocol is spec-editing only**: `^^^`/`&&&` markers are used only
  when editing spec files, not in general conversation or code.

## CLI Commands (planned)

```
specdev init                          # scaffold specs/ with overview.md, ctx.md, ideas.md
specdev scan                          # list unresolved ^^^ markers across specs/
specdev status                        # show spec health (files, freshness, open remarks)
specdev skill install [--global|--local]  # install SKILL.md to agents skills dir
```

No `compress` command — compression is done by the agent on user request.

## Skill Structure (planned)

```
~/.agents/skills/specdev/
  SKILL.md              # core instructions (< 500 lines)
  references/
    examples.md         # before/after examples of spec edits, compression
```

## Next Steps

- [x] Fix naming in `specs/overview.md` (`sdd` → `specdev`)
- [x] Address unresolved `^^^` on line 23 of `specs/overview.md` (marker
      protocol is spec-editing only)
- [x] Add compression section to `specs/overview.md`
- [x] Write the `SKILL.md` skill file → `skills/SKILL.md` (130 lines)
- [x] Write `references/examples.md` → `references/examples.md`
- [x] Design Rust CLI project structure
- [x] Implement `specdev init` (with tests: creates files, no overwrite, fills missing)
- [x] Implement `specdev scan` (with tests: parse_remarks, count_markers, edge cases)
- [x] Implement `specdev status` (core files, markers, additional specs, freshness)
- [x] Implement `specdev skill install` (with tests: creates files, overwrites on reinstall)
