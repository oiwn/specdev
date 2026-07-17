use std::fs;
use std::path::Path;

use crate::Error;
use crate::scan;
use crate::status;

/// Canonical display order for the core spec files; any others follow alphabetically.
const CANONICAL_ORDER: [&str; 5] = [
    "overview.md",
    "ctx.md",
    "roadmap.md",
    "ideas.md",
    "cleanup.md",
];

pub fn run(stats: bool) -> Result<(), Error> {
    let specs_dir = Path::new("specs");
    if !specs_dir.exists() {
        eprintln!("No specs/ directory found. Run `specdev init` first.");
        return Ok(());
    }

    let files = collect_ordered(specs_dir)?;
    if files.is_empty() {
        println!("No spec files found in specs/");
        return Ok(());
    }

    if stats {
        print_stats(specs_dir, &files)?;
    } else {
        print_list(specs_dir, &files)?;
    }
    Ok(())
}

fn print_list(specs_dir: &Path, files: &[String]) -> Result<(), Error> {
    println!("{:<16} {:<42} {:>5}", "File", "Description", "Lines");
    for name in files {
        let path = specs_dir.join(name);
        let content = fs::read_to_string(&path)?;
        let lines = content.lines().count();
        let desc =
            first_header(&content).unwrap_or_else(|| "(no header)".to_string());
        println!("{:<16} {:<42} {:>5}", name, truncate(&desc, 42), lines);
    }
    Ok(())
}

fn print_stats(specs_dir: &Path, files: &[String]) -> Result<(), Error> {
    println!(
        "{:<16} {:>2} {:>2} {:>2} {:>3} {:>4} {:>4} {:>5} {:>6}",
        "File", "H1", "H2", "H3", "H4+", "[x]", "[ ]", "^^^", "words"
    );
    for name in files {
        let path = specs_dir.join(name);
        let content = fs::read_to_string(&path)?;
        let (h1, h2, h3, h4p) = count_headings(&content);
        let (done, open_box) = status::count_checkboxes(&content);
        let (open_mark, _resolved) = scan::count_markers(&content);
        let words = content.split_whitespace().count();
        println!(
            "{:<16} {:>2} {:>2} {:>2} {:>3} {:>4} {:>4} {:>5} {:>6}",
            name, h1, h2, h3, h4p, done, open_box, open_mark, words
        );
    }
    Ok(())
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
    for line in content.lines() {
        let t = line.trim_start();
        if !t.starts_with('#') {
            continue;
        }
        let level = t.chars().take_while(|&c| c == '#').count();
        let after = &t[level..];
        if after.is_empty() || after.starts_with(' ') {
            let text = after.trim();
            if !text.is_empty() {
                return Some(text.to_string());
            }
        }
    }
    None
}

/// Count headings by level. Returns (h1, h2, h3, h4+).
pub fn count_headings(content: &str) -> (usize, usize, usize, usize) {
    let mut h1 = 0;
    let mut h2 = 0;
    let mut h3 = 0;
    let mut h4p = 0;
    for line in content.lines() {
        let t = line.trim_start();
        if !t.starts_with('#') {
            continue;
        }
        let level = t.chars().take_while(|&c| c == '#').count();
        let after = &t[level..];
        if !(after.is_empty() || after.starts_with(' ')) {
            continue;
        }
        match level {
            1 => h1 += 1,
            2 => h2 += 1,
            3 => h3 += 1,
            _ => h4p += 1,
        }
    }
    (h1, h2, h3, h4p)
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
