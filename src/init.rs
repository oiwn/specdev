use std::fs;
use std::path::Path;

use crate::Error;

const OVERVIEW_TEMPLATE: &str = "\
# Project Overview

## Architecture

## Data Flow

## Gotchas / Knowledge
";

/// ctx.md always carries this header; the no-task state is header-only, not blank.
const CTX_TEMPLATE: &str = "# Current Task Context\n";

const ROADMAP_TEMPLATE: &str = "\
# Roadmap

Committed future direction, in rough priority order. Promote items here from
`ideas.md` when decided; move items into `ctx.md` when they become the active
task.

## Next

## Later
";

const IDEAS_TEMPLATE: &str = "\
# Ideas

Uncommitted possibilities — noted but not decided to do right now. Promote to
`roadmap.md` only when we decide to pursue something.
";

const CLEANUP_TEMPLATE: &str = "\
# Cleanup

Code smells, duplication, and refactor notes. Read on demand only — not session
context. Each entry is a `##` heading with the file or component, followed by
1-3 sentences.
";

const CHANGELOG_TEMPLATE: &str = "\
# Changelog

Completed tasks, moved here from `specs/ctx.md` once every checkbox in their
plan is done. Newest entry at the top, dated. Freeform — one block per finished
task. Coexists with any conventional release notes already here.

<!-- Entry template:
## YYYY-MM-DD — <task title>

- what shipped
- files touched / decisions locked
-->
";

/// The specdev section injected into a project's `AGENTS.md`. Deliberately a
/// minimal pointer — the full workflow lives in the agent skill
/// (`skills/SKILL.md`). Wrapped in markers so `init` can append idempotently.
const AGENTS_MARKER: &str = "<!-- BEGIN specdev -->";
const AGENTS_SECTION: &str = "\
<!-- BEGIN specdev -->
## specdev

This project uses **specdev** (specification-driven development). Load the
specdev skill when starting a session, continuing from specs, or picking up a
task. Always read `specs/overview.md` and `specs/ctx.md` before coding.
<!-- END specdev -->
";

const SPEC_CORE_FILES: [&str; 5] = [
    "overview.md",
    "ctx.md",
    "ideas.md",
    "roadmap.md",
    "cleanup.md",
];

pub fn run() -> Result<(), Error> {
    init_at(Path::new("specs"))
}

pub fn init_at(specs_dir: &Path) -> Result<(), Error> {
    let root = specs_dir.parent().unwrap_or_else(|| Path::new("."));

    if specs_dir.exists() {
        let all_specs_present = SPEC_CORE_FILES
            .iter()
            .all(|name| specs_dir.join(name).exists());
        if all_specs_present
            && root.join("CHANGELOG.md").exists()
            && agents_has_section(root)
        {
            println!("specs/ already initialized with all core files.");
            return Ok(());
        }
    } else {
        fs::create_dir(specs_dir)?;
        println!("Created specs/");
    }

    write_if_missing(&specs_dir.join("overview.md"), OVERVIEW_TEMPLATE)?;
    write_if_missing(&specs_dir.join("ctx.md"), CTX_TEMPLATE)?;
    write_if_missing(&specs_dir.join("ideas.md"), IDEAS_TEMPLATE)?;
    write_if_missing(&specs_dir.join("roadmap.md"), ROADMAP_TEMPLATE)?;
    write_if_missing(&specs_dir.join("cleanup.md"), CLEANUP_TEMPLATE)?;

    write_if_missing(&root.join("CHANGELOG.md"), CHANGELOG_TEMPLATE)?;
    ensure_agents_section(root)?;

    println!("Done.");
    Ok(())
}

fn write_if_missing(path: &Path, content: &str) -> Result<(), Error> {
    if path.exists() {
        println!("  {} already exists, skipping", path.display());
        return Ok(());
    }
    fs::write(path, content)?;
    println!("  Created {}", path.display());
    Ok(())
}

/// True if `root/AGENTS.md` already contains the specdev section.
fn agents_has_section(root: &Path) -> bool {
    fs::read_to_string(root.join("AGENTS.md"))
        .map(|content| content.contains(AGENTS_MARKER))
        .unwrap_or(false)
}

/// Ensure `root/AGENTS.md` has the specdev section:
/// - missing -> create `# AGENTS.md` + the section
/// - present without the marker -> append the section (preserving existing text)
/// - present with the marker -> no-op (idempotent)
fn ensure_agents_section(root: &Path) -> Result<(), Error> {
    let path = root.join("AGENTS.md");
    if !path.exists() {
        let content = format!("# AGENTS.md\n\n{AGENTS_SECTION}\n");
        fs::write(&path, content)?;
        println!("  Created {}", path.display());
        return Ok(());
    }

    let existing = fs::read_to_string(&path)?;
    if existing.contains(AGENTS_MARKER) {
        println!("  AGENTS.md already has a specdev section, skipping");
        return Ok(());
    }

    let prefix = if existing.ends_with('\n') { "" } else { "\n" };
    let updated = format!("{existing}{prefix}\n{AGENTS_SECTION}\n");
    fs::write(&path, updated)?;
    println!("  Appended specdev section to AGENTS.md");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn creates_all_core_files() {
        let tmp = TempDir::new().unwrap();
        let specs = tmp.path().join("specs");

        init_at(&specs).unwrap();

        for name in SPEC_CORE_FILES {
            assert!(specs.join(name).exists(), "missing specs/{name}");
        }
        assert!(tmp.path().join("CHANGELOG.md").exists());
        assert!(tmp.path().join("AGENTS.md").exists());
    }

    #[test]
    fn overview_has_template_content() {
        let tmp = TempDir::new().unwrap();
        let specs = tmp.path().join("specs");

        init_at(&specs).unwrap();

        let content = fs::read_to_string(specs.join("overview.md")).unwrap();
        assert!(content.contains("# Project Overview"));
        assert!(content.contains("## Gotchas / Knowledge"));
    }

    #[test]
    fn ctx_has_header_only() {
        let tmp = TempDir::new().unwrap();
        let specs = tmp.path().join("specs");

        init_at(&specs).unwrap();

        let content = fs::read_to_string(specs.join("ctx.md")).unwrap();
        assert_eq!(content, "# Current Task Context\n");
    }

    #[test]
    fn roadmap_has_template_content() {
        let tmp = TempDir::new().unwrap();
        let specs = tmp.path().join("specs");

        init_at(&specs).unwrap();

        let content = fs::read_to_string(specs.join("roadmap.md")).unwrap();
        assert!(content.contains("# Roadmap"));
        assert!(content.contains("## Next"));
    }

    #[test]
    fn ideas_has_header() {
        let tmp = TempDir::new().unwrap();
        let specs = tmp.path().join("specs");

        init_at(&specs).unwrap();

        let content = fs::read_to_string(specs.join("ideas.md")).unwrap();
        assert!(content.contains("# Ideas"));
    }

    #[test]
    fn cleanup_has_header() {
        let tmp = TempDir::new().unwrap();
        let specs = tmp.path().join("specs");

        init_at(&specs).unwrap();

        let content = fs::read_to_string(specs.join("cleanup.md")).unwrap();
        assert!(content.contains("# Cleanup"));
    }

    #[test]
    fn creates_root_changelog_and_agents() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        let specs = root.join("specs");

        init_at(&specs).unwrap();

        let changelog = fs::read_to_string(root.join("CHANGELOG.md")).unwrap();
        assert!(changelog.contains("# Changelog"));
        assert!(changelog.contains("Entry template"));

        let agents = fs::read_to_string(root.join("AGENTS.md")).unwrap();
        assert!(agents.contains("# AGENTS.md"));
        assert!(agents.contains("<!-- BEGIN specdev -->"));
        assert!(agents.contains("specdev"));
        assert!(agents.contains("specdev skill"));
        assert!(agents.contains("specs/overview.md"));
        assert!(agents.contains("specs/ctx.md"));
    }

    #[test]
    fn does_not_overwrite_existing() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        let specs = root.join("specs");
        fs::create_dir_all(&specs).unwrap();
        fs::write(specs.join("overview.md"), "my project").unwrap();
        fs::write(specs.join("ctx.md"), "my context").unwrap();
        fs::write(specs.join("ideas.md"), "my ideas").unwrap();
        fs::write(specs.join("roadmap.md"), "my roadmap").unwrap();
        fs::write(specs.join("cleanup.md"), "my cleanup").unwrap();
        fs::write(root.join("CHANGELOG.md"), "my changelog").unwrap();
        fs::write(root.join("AGENTS.md"), "my agents").unwrap();

        init_at(&specs).unwrap();

        assert_eq!(
            fs::read_to_string(specs.join("overview.md")).unwrap(),
            "my project"
        );
        assert_eq!(
            fs::read_to_string(specs.join("ctx.md")).unwrap(),
            "my context"
        );
        assert_eq!(
            fs::read_to_string(specs.join("ideas.md")).unwrap(),
            "my ideas"
        );
        assert_eq!(
            fs::read_to_string(specs.join("roadmap.md")).unwrap(),
            "my roadmap"
        );
        assert_eq!(
            fs::read_to_string(specs.join("cleanup.md")).unwrap(),
            "my cleanup"
        );
        assert_eq!(
            fs::read_to_string(root.join("CHANGELOG.md")).unwrap(),
            "my changelog"
        );
        // AGENTS.md is append-aware: user content is preserved AND the section added.
        let agents = fs::read_to_string(root.join("AGENTS.md")).unwrap();
        assert!(agents.contains("my agents"));
        assert!(agents.contains("<!-- BEGIN specdev -->"));
    }

    #[test]
    fn fills_missing_files_in_existing_dir() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        let specs = root.join("specs");
        fs::create_dir_all(&specs).unwrap();
        fs::write(specs.join("overview.md"), "existing").unwrap();
        fs::write(root.join("AGENTS.md"), "existing agents").unwrap();

        init_at(&specs).unwrap();

        assert_eq!(
            fs::read_to_string(specs.join("overview.md")).unwrap(),
            "existing"
        );
        for name in ["ctx.md", "ideas.md", "roadmap.md", "cleanup.md"] {
            assert!(specs.join(name).exists(), "missing {name}");
        }
        let agents = fs::read_to_string(root.join("AGENTS.md")).unwrap();
        assert!(agents.contains("existing agents"));
        assert!(agents.contains("<!-- BEGIN specdev -->"));
        assert!(root.join("CHANGELOG.md").exists());
    }

    #[test]
    fn skips_returning_ok_when_all_exist() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        let specs = root.join("specs");
        fs::create_dir_all(&specs).unwrap();
        for name in SPEC_CORE_FILES {
            fs::write(specs.join(name), "").unwrap();
        }
        fs::write(root.join("CHANGELOG.md"), "").unwrap();
        // AGENTS.md must already carry the specdev section to count as "done".
        fs::write(root.join("AGENTS.md"), AGENTS_SECTION).unwrap();

        let result = init_at(&specs);
        assert!(result.is_ok());
    }

    #[test]
    fn appends_section_to_existing_agents() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        let specs = root.join("specs");
        fs::create_dir_all(&specs).unwrap();
        let original = "# My Project\n\nCustom rules go here.\n";
        fs::write(root.join("AGENTS.md"), original).unwrap();

        init_at(&specs).unwrap();

        let agents = fs::read_to_string(root.join("AGENTS.md")).unwrap();
        // user content fully preserved
        assert!(agents.starts_with("# My Project"));
        assert!(agents.contains("Custom rules go here."));
        // section appended
        assert!(agents.contains("<!-- BEGIN specdev -->"));
        assert!(agents.contains("<!-- END specdev -->"));
        assert!(agents.contains("Load the"));
        assert!(agents.contains("specdev skill"));
        // exactly one section, not duplicated
        assert_eq!(agents.matches("<!-- BEGIN specdev -->").count(), 1);
    }

    #[test]
    fn agents_append_is_idempotent() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        let specs = root.join("specs");
        fs::create_dir_all(&specs).unwrap();
        fs::write(root.join("AGENTS.md"), AGENTS_SECTION).unwrap();
        let before = fs::read_to_string(root.join("AGENTS.md")).unwrap();

        init_at(&specs).unwrap();

        let after = fs::read_to_string(root.join("AGENTS.md")).unwrap();
        assert_eq!(before, after, "AGENTS.md must be unchanged on re-run");
    }
}
