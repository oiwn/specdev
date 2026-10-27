---
id: 0012-homebrew-tap
status: draft
scope: [.github/workflows/publish.yml, packaging/homebrew/specdev.rb, README.md]
created: 2026-10-27
source: session 2026-10-27: install via homebrew
---
# Task: homebrew tap

## Plan

- [ ] Extend `.github/workflows/publish.yml`: a build job before the release — matrix over `aarch64-apple-darwin`, `x86_64-apple-darwin`, `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, `cargo build --release --locked --target`, tar the binary, generate a `sha256` manifest; the GitHub release attaches everything (idempotent like the rest of the workflow)
- [ ] Draft `packaging/homebrew/specdev.rb`: formula installing from the release tarballs with per-arch sha256 (macOS assets; linux for Linuxbrew)
- [ ] README: `brew install oiwn/tap/specdev` in Install
- [ ] Keep the first tap bump manual (one-line version+sha edit per release); automation is a later nicety

## Acceptance

- [ ] cargo fmt --check
- [ ] cargo build
- [ ] cargo test
- [ ] cargo clippy --all-targets -- -D warnings
- [ ] specdev check

## Manual checks

- [ ] `oiwn/homebrew-tap` repo created with `Formula/specdev.rb` — user: creates the repo, copies the formula
- [ ] `brew install oiwn/tap/specdev` works after the next release — user

## Context

Source: user request 2026-10-27 ("install via homebrew or nix"); consumes the "Release binaries" idea from `specs/ideas.md`. The tap must be a separate repo — the user creates it; this repo ships the build pipeline and the formula source. Tarballs make install possible without a Rust toolchain, which is the point of both channels.

## Log

- 2026-10-27 created: homebrew tap
- 2026-10-27 set source session 2026-10-27: install via homebrew
- 2026-10-27 scope add .github/workflows/publish.yml — build matrix + release artifacts; formula source; install docs
- 2026-10-27 scope add packaging/homebrew/specdev.rb — build matrix + release artifacts; formula source; install docs
- 2026-10-27 scope add README.md — build matrix + release artifacts; formula source; install docs
