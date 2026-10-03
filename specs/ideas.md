# Ideas

<!-- Uncommitted possibilities; promote to roadmap.md when decided. -->

- **Proper CI with supply-chain checks.** Extend `.github/workflows/ci.yml`: `cargo audit` (RustSec advisories) on every PR plus a weekly scheduled run, so new advisories surface without a code change; maybe `cargo deny` for licenses, banned and duplicate crates; a dependency-count budget that fails CI when the runtime tree grows past an agreed number, so growth is always a deliberate decision.
