# Current Task Context: 0.4.0 — trust + papercuts
State: in progress
## Plan
- [x] Tasks 0001–0007: the 0.4.0 batch — closed; release ready for the user's commit/merge
- [ ] Task 0010: skill-stale warning in `check`/`status` — at approval, awaiting acceptance
## Context
The user commits and merges the release PR himself. Task 0010 shipped the `skill-stale` warning: content-first drift detection on `.agents/skills/specdev/`, surfaced through the `spec_diagnostics` shared by `check` and `status`, remedy `specdev skill install --local`. Dogfooded live in this repo (SKILL.md edit → warning → reinstall with the working-tree build → clean).
## Next
On acceptance of 0010: `task done 10`, hand the tree to the user for his commit. The 0.4.0 ctx task then has every box ticked — ask about archiving it to CHANGELOG and resetting ctx to header-only.
