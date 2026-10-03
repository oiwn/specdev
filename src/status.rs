use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::time::SystemTime;

use serde::Serialize;

use crate::Error;
use crate::diag::Diagnostic;
use crate::md;
use crate::output::{self, Format, Report};
use crate::scan;

const CORE_FILES: [&str; 4] = ["overview.md", "ctx.md", "roadmap.md", "ideas.md"];

pub fn run(root: &Path, format: Format) -> Result<(), Error> {
    let specs_dir = root.join("specs");
    if !specs_dir.exists() {
        eprintln!("No specs/ directory found. Run `specdev init` first.");
        return Ok(());
    }

    let mut report = StatusReport::default();
    for name in CORE_FILES {
        let path = specs_dir.join(name);
        if !path.exists() {
            report.core.push(CoreFile {
                name,
                present: false,
                lines: 0,
                age: None,
                markers: 0,
                open: 0,
                steps: None,
            });
            continue;
        }
        let content = fs::read_to_string(&path)?;
        let (open, resolved) = scan::count_markers(&content);
        report.open += open;
        report.resolved += resolved;
        let steps = (name == "ctx.md")
            .then(|| count_checkboxes(&content))
            .filter(|&(_, total)| total > 0)
            .map(|(done, total)| Steps { done, total });
        report.core.push(CoreFile {
            name,
            present: true,
            lines: content.lines().count(),
            age: Some(format_age(fs::metadata(&path)?.modified()?)),
            markers: open + resolved,
            open,
            steps,
        });
    }

    let mut extra: Vec<String> = Vec::new();
    for entry in fs::read_dir(&specs_dir)? {
        let fname = entry?.file_name().to_string_lossy().to_string();
        if fname.ends_with(".md") && !CORE_FILES.contains(&fname.as_str()) {
            extra.push(fname);
        }
    }
    extra.sort();
    for name in extra {
        let content = fs::read_to_string(specs_dir.join(&name))?;
        let (open, resolved) = scan::count_markers(&content);
        report.open += open;
        report.resolved += resolved;
        report.extra.push(ExtraFile {
            lines: content.lines().count(),
            markers: open + resolved,
            open,
            name,
        });
    }

    report.warnings = spec_diagnostics(root)?;
    output::emit(&report, format)
}

/// Spec-level warnings shared by `status` and `check`: `ctx.md` content that
/// belongs elsewhere, a finished-looking `ctx.md`, missing root files.
pub fn spec_diagnostics(root: &Path) -> Result<Vec<Diagnostic>, Error> {
    let ctx_path = root.join("specs").join("ctx.md");
    let mut diags = if ctx_path.exists() {
        ctx_diagnostics(&ctx_path, &fs::read_to_string(&ctx_path)?)
    } else {
        Vec::new()
    };
    diags.extend(root_file_diagnostics(root));
    Ok(diags)
}

fn ctx_diagnostics(path: &Path, content: &str) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    let (done, total) = count_checkboxes(content);
    if total > 0 && done == total {
        diags.push(Diagnostic::warning(
            path,
            None,
            "ctx-all-done",
            format!(
                "all {total} plan steps are checked. Confirm with the user that the whole task is done before archiving."
            ),
        ));
    }
    for f in find_forbidden(content) {
        diags.push(Diagnostic::warning(
            path,
            None,
            "ctx-forbidden",
            format!(
                "forbidden content ({f}) — route it to its home file (see AGENTS.md)."
            ),
        ));
    }
    diags
}

fn root_file_diagnostics(root: &Path) -> Vec<Diagnostic> {
    ["CHANGELOG.md", "AGENTS.md"]
        .into_iter()
        .map(|name| root.join(name))
        .filter(|path| !path.exists())
        .map(|path| {
            Diagnostic::warning(
                path,
                None,
                "root-file-missing",
                "missing at project root — run `specdev init` to add it.",
            )
        })
        .collect()
}

#[derive(Serialize, Default)]
struct StatusReport {
    core: Vec<CoreFile>,
    extra: Vec<ExtraFile>,
    open: usize,
    resolved: usize,
    warnings: Vec<Diagnostic>,
}

#[derive(Serialize)]
struct CoreFile {
    name: &'static str,
    present: bool,
    lines: usize,
    age: Option<String>,
    markers: usize,
    open: usize,
    /// `ctx.md` only: checked plan steps.
    steps: Option<Steps>,
}

#[derive(Serialize)]
struct ExtraFile {
    name: String,
    lines: usize,
    markers: usize,
    open: usize,
}

#[derive(Serialize)]
struct Steps {
    done: usize,
    total: usize,
}

impl Report for StatusReport {
    fn text(&self) -> String {
        let mut out = vec!["Core spec files:".to_string()];
        for f in &self.core {
            if !f.present {
                out.push(format!("  [--] {:<14} missing", f.name));
                continue;
            }
            let age = f.age.as_deref().unwrap_or("");
            if f.markers > 0 {
                out.push(format!(
                    "  [OK] {:<14} {:>4} lines  {:<10} {} markers ({} open)",
                    f.name, f.lines, age, f.markers, f.open
                ));
            } else {
                out.push(format!(
                    "  [OK] {:<14} {:>4} lines  {:<10}",
                    f.name, f.lines, age
                ));
            }
            if let Some(steps) = &f.steps {
                out.push(format!(
                    "       task progress: {}/{} steps done",
                    steps.done, steps.total
                ));
            }
        }
        if !self.extra.is_empty() {
            out.push("\nAdditional specs:".to_string());
            for f in &self.extra {
                if f.markers > 0 {
                    out.push(format!(
                        "  {:<15} {:>4} lines  {} markers ({} open)",
                        f.name, f.lines, f.markers, f.open
                    ));
                } else {
                    out.push(format!("  {:<15} {:>4} lines", f.name, f.lines));
                }
            }
        }
        out.push(format!(
            "\nMarkers: {} open, {} resolved",
            self.open, self.resolved
        ));
        if !self.warnings.is_empty() {
            out.push("\nWarnings:".to_string());
            for w in &self.warnings {
                let file = w.file.file_name().unwrap_or_default().to_string_lossy();
                out.push(format!("  - {file}: {}", w.message));
            }
        }
        out.join("\n")
    }
}

/// Count Markdown task checkboxes, ignoring code blocks. Returns (checked, total).
pub fn count_checkboxes(content: &str) -> (usize, usize) {
    md::outline(content).checkbox_counts()
}

/// Detect content forbidden in ctx.md (it belongs in another spec file).
/// Returns human-readable labels of what was found. Code blocks are ignored.
pub fn find_forbidden(content: &str) -> Vec<String> {
    let outline = md::outline(content);
    let mut found = Vec::new();
    for h in &outline.headings {
        let heading = h.text.to_lowercase();
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
    for (i, line) in content.lines().enumerate() {
        if !outline.is_in_code(i + 1)
            && line_starts_with(&line.trim().to_lowercase(), "roadmap pointer")
        {
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

    fn ctx_codes(content: &str) -> Vec<&'static str> {
        ctx_diagnostics(Path::new("specs/ctx.md"), content)
            .iter()
            .map(|d| d.code)
            .collect()
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
        assert_eq!(ctx_codes(content), ["ctx-all-done"]);
    }

    #[test]
    fn no_archive_warning_when_steps_remain() {
        let content = "\
# Task: thing
## Plan
- [x] one
- [ ] two
";
        assert!(ctx_codes(content).is_empty());
    }

    #[test]
    fn missing_root_file_warning() {
        let tmp = tempfile::TempDir::new().unwrap();
        fs::write(tmp.path().join("AGENTS.md"), "").unwrap();
        let diags = root_file_diagnostics(tmp.path());
        assert_eq!(diags.len(), 1);
        assert!(diags[0].file.ends_with("CHANGELOG.md"), "got {diags:?}");
        assert_eq!(diags[0].code, "root-file-missing");
    }
}
