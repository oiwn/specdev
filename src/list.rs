use std::fs;
use std::path::Path;

use serde::Serialize;

use crate::Error;
use crate::md;
use crate::output::{self, Format, Report};
use crate::quality::{self, Metrics};

/// Canonical display order for the core spec files; any others follow alphabetically.
const CANONICAL_ORDER: [&str; 5] = [
    "overview.md",
    "ctx.md",
    "roadmap.md",
    "ideas.md",
    "cleanup.md",
];

pub fn run(stats: bool, format: Format) -> Result<(), Error> {
    let specs_dir = Path::new("specs");
    if !specs_dir.exists() {
        eprintln!("No specs/ directory found. Run `specdev init` first.");
        return Ok(());
    }

    let files = collect_ordered(specs_dir)?;
    if stats {
        output::emit(&stats_report(specs_dir, &files)?, format)
    } else {
        output::emit(&list_report(specs_dir, &files)?, format)
    }
}

#[derive(Serialize)]
struct ListReport {
    files: Vec<ListEntry>,
}

#[derive(Serialize)]
struct ListEntry {
    file: String,
    description: Option<String>,
    lines: usize,
}

impl Report for ListReport {
    fn text(&self) -> String {
        if self.files.is_empty() {
            return "No spec files found in specs/".to_string();
        }
        let mut out =
            format!("{:<16} {:<42} {:>5}", "File", "Description", "Lines");
        for f in &self.files {
            let desc = f.description.as_deref().unwrap_or("(no header)");
            out.push_str(&format!(
                "\n{:<16} {:<42} {:>5}",
                f.file,
                truncate(desc, 42),
                f.lines
            ));
        }
        out
    }
}

fn list_report(specs_dir: &Path, files: &[String]) -> Result<ListReport, Error> {
    let mut entries = Vec::new();
    for name in files {
        let content = fs::read_to_string(specs_dir.join(name))?;
        entries.push(ListEntry {
            file: name.clone(),
            description: first_header(&content),
            lines: content.lines().count(),
        });
    }
    Ok(ListReport { files: entries })
}

#[derive(Serialize)]
struct StatsReport {
    files: Vec<FileStats>,
}

#[derive(Serialize)]
struct FileStats {
    file: String,
    h1: usize,
    h2: usize,
    h3: usize,
    h4_plus: usize,
    checked: usize,
    unchecked: usize,
    #[serde(flatten)]
    metrics: Metrics,
}

impl Report for StatsReport {
    fn text(&self) -> String {
        if self.files.is_empty() {
            return "No spec files found in specs/".to_string();
        }
        let mut out = format!(
            "{:<16} {:>2} {:>2} {:>2} {:>3} {:>4} {:>4} {:>5} {:>6} {:>5} {:>4} {:>3}",
            "File",
            "H1",
            "H2",
            "H3",
            "H4+",
            "[x]",
            "[ ]",
            "^^^",
            "words",
            "lines",
            "code",
            "tbl"
        );
        for s in &self.files {
            let m = &s.metrics;
            out.push_str(&format!(
                "\n{:<16} {:>2} {:>2} {:>2} {:>3} {:>4} {:>4} {:>5} {:>6} {:>5} {:>4} {:>3}",
                s.file,
                s.h1,
                s.h2,
                s.h3,
                s.h4_plus,
                s.checked,
                s.unchecked,
                m.open_remarks,
                m.words,
                m.lines,
                m.code_blocks,
                m.tables + m.diagrams
            ));
        }
        out
    }
}

fn stats_report(specs_dir: &Path, files: &[String]) -> Result<StatsReport, Error> {
    let mut stats = Vec::new();
    for name in files {
        let content = fs::read_to_string(specs_dir.join(name))?;
        let outline = md::outline(&content);
        let (h1, h2, h3, h4_plus) = outline.heading_counts();
        let (checked, total) = outline.checkbox_counts();
        stats.push(FileStats {
            file: name.clone(),
            h1,
            h2,
            h3,
            h4_plus,
            checked,
            unchecked: total - checked,
            metrics: quality::measure_with(&content, &outline),
        });
    }
    Ok(StatsReport { files: stats })
}

/// List specs/*.md in canonical order first, then the rest alphabetically.
fn collect_ordered(specs_dir: &Path) -> Result<Vec<String>, Error> {
    let mut found: Vec<String> = Vec::new();
    for entry in fs::read_dir(specs_dir)? {
        let entry = entry?;
        let fname = entry.file_name().to_string_lossy().to_string();
        if fname.ends_with(".md") {
            found.push(fname);
        }
    }

    let mut ordered: Vec<String> = Vec::new();
    for c in CANONICAL_ORDER {
        if found.iter().any(|f| f == c) {
            ordered.push(c.to_string());
        }
    }
    let mut rest: Vec<String> = found
        .into_iter()
        .filter(|f| !CANONICAL_ORDER.contains(&f.as_str()))
        .collect();
    rest.sort();
    ordered.extend(rest);
    Ok(ordered)
}

/// Text of the first Markdown heading in the file (without the leading `#`s).
/// Returns `None` if there is no well-formed heading (a `#` run must be
/// followed by a space or end-of-line).
pub fn first_header(content: &str) -> Option<String> {
    md::outline(content).first_heading().map(str::to_string)
}

/// Truncate to `max` chars, appending an ellipsis if shortened.
fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let kept: String = s.chars().take(max.saturating_sub(1)).collect();
    format!("{kept}…")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn first_header_basic() {
        let content = "# specdev — overview\n\nbody\n";
        assert_eq!(
            first_header(content),
            Some("specdev — overview".to_string())
        );
    }

    #[test]
    fn first_header_returns_the_first() {
        let content = "# A\n## B\n# C\n";
        assert_eq!(first_header(content), Some("A".to_string()));
    }

    #[test]
    fn first_header_skips_hash_without_space() {
        let content = "##not a heading\n# real\n";
        assert_eq!(first_header(content), Some("real".to_string()));
    }

    #[test]
    fn first_header_none_when_absent() {
        let content = "no headings here\njust prose\n";
        assert_eq!(first_header(content), None);
    }

    fn count_headings(content: &str) -> (usize, usize, usize, usize) {
        md::outline(content).heading_counts()
    }

    #[test]
    fn count_headings_by_level() {
        let content = "# a\n## b\n### c\n#### d\n##### e\n## e2\n";
        assert_eq!(count_headings(content), (1, 2, 1, 2));
    }

    #[test]
    fn count_headings_ignores_non_headings() {
        let content = "##nope\n- not a heading\n#real\n";
        assert_eq!(count_headings(content), (0, 0, 0, 0));
    }

    #[test]
    fn count_headings_empty() {
        assert_eq!(count_headings(""), (0, 0, 0, 0));
    }

    #[test]
    fn truncate_short_unchanged() {
        assert_eq!(truncate("hello", 10), "hello");
    }

    #[test]
    fn truncate_long_gets_ellipsis() {
        let out = truncate("abcdefghijklmnopqrstuvwxyz", 10);
        assert_eq!(out.chars().count(), 10);
        assert!(out.ends_with('…'));
        assert!(out.starts_with("abcdefghi"));
    }

    #[test]
    fn collect_ordered_canonical_first_then_alpha() {
        let tmp = TempDir::new().unwrap();
        let specs = tmp.path().join("specs");
        fs::create_dir_all(&specs).unwrap();
        fs::write(specs.join("zebra.md"), "# z").unwrap();
        fs::write(specs.join("ctx.md"), "# c").unwrap();
        fs::write(specs.join("overview.md"), "# o").unwrap();
        fs::write(specs.join("alpha.md"), "# a").unwrap();

        let order = collect_ordered(&specs).unwrap();
        assert_eq!(
            order,
            vec![
                "overview.md".to_string(),
                "ctx.md".to_string(),
                "alpha.md".to_string(),
                "zebra.md".to_string(),
            ]
        );
    }
}
