use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::time::SystemTime;

use crate::scan;
use crate::Error;

pub fn run() -> Result<(), Error> {
    let specs_dir = Path::new("specs");
    let root = Path::new(".");
    if !specs_dir.exists() {
        eprintln!("No specs/ directory found. Run `specdev init` first.");
        return Ok(());
    }

    // Load ctx.md once for step-progress + forbidden-content checks.
    let ctx_path = specs_dir.join("ctx.md");
    let ctx_content = if ctx_path.exists() {
        fs::read_to_string(&ctx_path).ok()
    } else {
        None
    };
    let ctx_steps = ctx_content.as_ref().map(|c| count_checkboxes(c));

    let core_files = ["overview.md", "ctx.md", "roadmap.md", "ideas.md"];
    let mut total_open = 0;
    let mut total_resolved = 0;

    println!("Core spec files:");
    for name in &core_files {
        let path = specs_dir.join(name);
        if path.exists() {
            let meta = fs::metadata(&path)?;
            let content = fs::read_to_string(&path)?;
            let lines = content.lines().count();
            let (open, resolved) = scan::count_markers(&content);
            total_open += open;
            total_resolved += resolved;
            let age = format_age(meta.modified()?);
            let markers = open + resolved;
            if markers > 0 {
                println!(
                    "  [OK] {:<14} {:>4} lines  {:<10} {} markers ({} open)",
                    name, lines, age, markers, open
                );
            } else {
                println!("  [OK] {:<14} {:>4} lines  {:<10}", name, lines, age);
            }
            if *name == "ctx.md" {
                if let Some((done, total)) = ctx_steps {
                    if total > 0 {
                        println!(
                            "       task progress: {}/{} steps done",
                            done, total
                        );
                    }
                }
            }
        } else {
            println!("  [--] {:<14} missing", name);
        }
    }

    let mut extra: Vec<String> = Vec::new();
    for entry in fs::read_dir(specs_dir)? {
        let entry = entry?;
        let fname = entry.file_name().to_string_lossy().to_string();
        if fname.ends_with(".md") && !core_files.contains(&fname.as_str()) {
            extra.push(fname);
        }
    }

    if !extra.is_empty() {
        extra.sort();
        println!("\nAdditional specs:");
        for name in &extra {
            let path = specs_dir.join(name);
            let content = fs::read_to_string(&path)?;
            let lines = content.lines().count();
            let (open, resolved) = scan::count_markers(&content);
            total_open += open;
            total_resolved += resolved;
            let markers = open + resolved;
            if markers > 0 {
                println!(
                    "  {:<15} {:>4} lines  {} markers ({} open)",
                    name, lines, markers, open
                );
            } else {
                println!("  {:<15} {:>4} lines", name, lines);
            }
        }
    }

    println!(
        "\nMarkers: {} open, {} resolved",
        total_open, total_resolved
    );

    let warnings = collect_warnings(&ctx_content, ctx_steps, root);
    if !warnings.is_empty() {
        println!("\nWarnings:");
        for w in &warnings {
            println!("  - {w}");
        }
    }

    Ok(())
}

fn collect_warnings(
    ctx_content: &Option<String>,
    ctx_steps: Option<(usize, usize)>,
    root: &Path,
) -> Vec<String> {
    let mut warnings = Vec::new();

    if let Some(content) = ctx_content {
        if let Some((done, total)) = ctx_steps {
            if total > 0 && done == total && !content.trim().is_empty() {
                warnings.push(format!(
                    "ctx.md: all {total} plan steps are checked. Archive the task to CHANGELOG.md and reset ctx.md."
                ));
            }
        }
        let forbidden = unique_preserve(find_forbidden(content));
        for f in forbidden {
            warnings.push(format!(
                "ctx.md: forbidden content ({f}) — route it to its home file (see AGENTS.md)."
            ));
        }
    }

    for root_file in ["CHANGELOG.md", "AGENTS.md"] {
        if !root.join(root_file).exists() {
            warnings.push(format!(
                "{root_file} missing at project root — run `specdev init` to add it."
            ));
        }
    }

    warnings
}

/// Count Markdown task checkboxes. Returns (checked, total).
pub fn count_checkboxes(content: &str) -> (usize, usize) {
    let mut done = 0;
    let mut total = 0;
    for line in content.lines() {
        let t = line.trim_start();
        if let Some(rest) = t.strip_prefix("- [") {
            let mut chars = rest.chars();
            if let (Some(marker), Some(after)) = (chars.next(), chars.next()) {
                if after == ']' {
                    total += 1;
                    if marker == 'x' || marker == 'X' {
                        done += 1;
                    }
                }
            }
        }
    }
    (done, total)
}

/// Detect content forbidden in ctx.md (it belongs in another spec file).
/// Returns human-readable labels of what was found.
pub fn find_forbidden(content: &str) -> Vec<String> {
    let mut found = Vec::new();
    for line in content.lines() {
        let lower = line.trim().to_lowercase();
        if line.trim_start().starts_with('#') {
            let heading = lower.trim_start_matches('#').trim();
            if heading.contains("deferred") {
                found.push("Deferred section".to_string());
            }
            if heading.contains("gotchas") || heading.contains("quirks") {
                found.push("Gotchas/Quirks block".to_string());
            }
            if heading.contains("roadmap") {
                found.push("Roadmap content".to_string());
            }
        }
        if line_starts_with(&lower, "roadmap pointer") {
            found.push("Roadmap pointer".to_string());
        }
    }
    unique_preserve(found)
}

/// True if `line_lower` (already lowercased) begins with `prefix`, ignoring a
/// leading list marker or blockquote. Avoids matching mid-sentence mentions.
fn line_starts_with(line_lower: &str, prefix: &str) -> bool {
    let s = line_lower.trim_start();
    let stripped = s
        .strip_prefix("- ")
        .or_else(|| s.strip_prefix("* "))
        .or_else(|| s.strip_prefix("> "))
        .unwrap_or(s);
    stripped.starts_with(prefix)
}

fn unique_preserve(v: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    v.into_iter().filter(|s| seen.insert(s.clone())).collect()
}

fn format_age(modified: SystemTime) -> String {
    let now = SystemTime::now();
    match now.duration_since(modified) {
        Ok(dur) => {
            let secs = dur.as_secs();
            if secs < 60 {
                format!("{}s ago", secs)
            } else if secs < 3600 {
                format!("{}m ago", secs / 60)
            } else if secs < 86400 {
                format!("{}h ago", secs / 3600)
            } else {
                format!("{}d ago", secs / 86400)
            }
        }
        Err(_) => "future".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_checkboxes() {
        assert_eq!(count_checkboxes("# Task\n\nSome prose.\n"), (0, 0));
    }

    #[test]
    fn mixed_checkboxes() {
        let content = "\
## Plan
- [x] done thing
- [ ] todo
- [X] also done
- [ ] another todo
";
        assert_eq!(count_checkboxes(content), (2, 4));
    }

    #[test]
    fn all_done_checkboxes() {
        let content = "\
## Plan
- [x] one
- [x] two
";
        assert_eq!(count_checkboxes(content), (2, 2));
    }

    #[test]
    fn ignores_non_task_list_items() {
        let content = "\
- plain bullet
1. [x] not a task bullet
[x] missing dash
";
        assert_eq!(count_checkboxes(content), (0, 0));
    }

    #[test]
    fn finds_deferred_section() {
        let content = "\
## Plan
- [ ] a step

## Deferred
- some future thing
";
        let f = find_forbidden(content);
        assert!(f.iter().any(|s| s.contains("Deferred")), "got {f:?}");
    }

    #[test]
    fn finds_gotchas_and_quirks() {
        let content = "## SurrealDB Quirks\n\n- note\n";
        let f = find_forbidden(content);
        assert!(f.iter().any(|s| s.contains("Quirks")), "got {f:?}");

        let content2 = "## Gotchas\n\n- note\n";
        let f2 = find_forbidden(content2);
        assert!(f2.iter().any(|s| s.contains("Gotchas")), "got {f2:?}");
    }

    #[test]
    fn finds_roadmap_pointer() {
        let content = "\
Active task: foo.
Roadmap pointer: this supersedes v1.8.
";
        let f = find_forbidden(content);
        assert!(f.iter().any(|s| s.contains("Roadmap pointer")), "got {f:?}");
    }

    #[test]
    fn does_not_flag_mid_sentence_mention() {
        let content = "\
## Context
- Forbidden in ctx.md: deferred sections, roadmap pointers, gotchas.
";
        let f = find_forbidden(content);
        assert!(
            f.is_empty(),
            "mid-sentence mentions should not be flagged, got {f:?}"
        );
    }

    #[test]
    fn clean_ctx_has_no_forbidden() {
        let content = "\
# Task: do a thing
State: in progress
## Plan
- [x] one
- [ ] two
## Context
nothing special
## Next
do two
";
        assert!(find_forbidden(content).is_empty());
    }

    #[test]
    fn archive_warning_when_all_done() {
        let content = "\
# Task: done thing
## Plan
- [x] one
- [x] two
## Next
nothing
";
        let steps = count_checkboxes(content);
        let warnings = collect_warnings(
            &Some(content.to_string()),
            Some(steps),
            Path::new("."),
        );
        assert!(
            warnings.iter().any(|w| w.contains("Archive the task")),
            "got {warnings:?}"
        );
    }

    #[test]
    fn no_archive_warning_when_steps_remain() {
        let content = "\
# Task: thing
## Plan
- [x] one
- [ ] two
";
        let steps = count_checkboxes(content);
        let warnings = collect_warnings(
            &Some(content.to_string()),
            Some(steps),
            Path::new("."),
        );
        assert!(
            !warnings.iter().any(|w| w.contains("Archive the task")),
            "got {warnings:?}"
        );
    }

    #[test]
    fn missing_root_file_warning() {
        let tmp = std::env::temp_dir();
        let warnings = collect_warnings(&None, None, &tmp);
        assert!(
            warnings.iter().any(|w| w.contains("CHANGELOG.md missing")),
            "got {warnings:?}"
        );
    }
}
