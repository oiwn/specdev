---
id: 0011-nix-flake
status: draft
scope: [flake.nix, flake.lock, .gitignore, README.md]
created: 2026-10-27
source: session 2026-10-27: install via nix
---
# Task: nix flake

## Plan

- [ ] `flake.nix` at the repo root: `rustPlatform.buildRustPackage` from the crate source with `cargoLock.lockFile = ./Cargo.lock`, exposing `packages.aarch64-darwin`/`x86_64-darwin`/`x86_64-linux` (std flake schema), plus a `devShell` with the stable Rust toolchain
- [ ] Generate `flake.lock`; ignore Nix build symlinks (`result`) in `.gitignore`
- [ ] README: `nix run github:oiwn/specdev` and the dev-shell line in Install
- [ ] Verify what's verifiable locally; the rest is a Manual check

## Acceptance

- [ ] cargo fmt --check
- [ ] cargo build
- [ ] cargo test
- [ ] cargo clippy --all-targets -- -D warnings
- [ ] specdev check

## Manual checks

- [ ] `nix build .` produces a working `specdev` binary — user: runs it on a machine with Nix

## Context

Source: user request 2026-10-27 ("install via homebrew or nix"). No Cargo-tree or CI changes — the flake builds from the repo as-is. `cargoLock` keeps the build hermetic against the committed lockfile.

## Log

- 2026-10-27 created: nix flake
- 2026-10-27 set source session 2026-10-27: install via nix
- 2026-10-27 scope add flake.nix — the flake and its lock; ignore result symlinks; install docs
- 2026-10-27 scope add flake.lock — the flake and its lock; ignore result symlinks; install docs
- 2026-10-27 scope add .gitignore — the flake and its lock; ignore result symlinks; install docs
- 2026-10-27 scope add README.md — the flake and its lock; ignore result symlinks; install docs
