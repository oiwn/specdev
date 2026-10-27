# Current Task Context: 0.4.0 — trust + papercuts

State: in progress

## Plan

- [x] Tasks 0001–0010: the 0.4.0 batch + skill-stale — all closed and committed by the user
- [ ] Task 0011: nix flake — drafted, awaiting user approve
- [ ] Task 0012: homebrew tap (build matrix + release tarballs + formula) — drafted, awaiting user approve
- [ ] Parallel-tasks design (#21.5 + entity registry) — next, in ctx, after the distribution tasks

## Context

Release commits ac0b179 and 5efb3a2 are in; the user runs the release PR/merge himself. Agreed 2026-10-27: distribution first (nix, homebrew — consuming the "Release binaries" idea), then the A+B design (non-overlap, dep graph, entities; entity shape decided during design). The 0.4.0 ctx task has every box ticked — archiving it to CHANGELOG and resetting this file waits for the user's word.

## Next

User approves 0011 and 0012 (`task approve 11|12 --note "…"`), then implement both; the tap task needs the user to create the `oiwn/homebrew-tap` repo before its Manual checks can close.
