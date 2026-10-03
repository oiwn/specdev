# Ideas

<!-- Uncommitted possibilities; promote to roadmap.md when decided. -->

- **More supply-chain checks in CI.** `cargo audit` is in place (`deps-audit.yml`: weekly + on manifest changes). Still open: `cargo deny` for licenses, banned and duplicate crates; a dependency-count budget that fails CI when the runtime tree grows past an agreed number, so growth is always a deliberate decision.
- **Release binaries.** The release workflow publishes to crates.io and creates a GitHub release; attaching prebuilt macOS/Linux binaries to the release would allow install without a Rust toolchain.
