use std::fs;
use std::path::{Path, PathBuf};

use crate::Error;

pub struct Remark {
    pub line: usize,
    pub text: String,
    pub resolved: bool,
}

pub struct FileRemarks {
    pub path: PathBuf,
    pub remarks: Vec<Remark>,
}

pub fn run() -> Result<(), Error> {
    let specs_dir = Path::new("specs");
    if !specs_dir.exists() {
        eprintln!("No specs/ directory found. Run `specdev init` first.");
        return Ok(());
    }

    let files = collect_spec_files(specs_dir)?;
    if files.is_empty() {
        println!("No spec files found in specs/");
        return Ok(());
    }

    let mut all_results: Vec<FileRemarks> = Vec::new();

    for path in &files {
        let content = fs::read_to_string(path)?;
        let remarks = parse_remarks(&content);
        all_results.push(FileRemarks {
            path: path.strip_prefix("specs/").unwrap_or(path).to_path_buf(),
            remarks,
        });
    }

    let mut total_open = 0;
    let mut total_resolved = 0;

    for fr in &all_results {
        println!("{}", fr.path.display());
        if fr.remarks.is_empty() {
            println!("  (no markers)");
            continue;
        }
        for r in &fr.remarks {
            let status = if r.resolved { "resolved" } else { "open" };
            println!("  L{:>3}  [{:>8}]  {}", r.line, status, r.text);
            if r.resolved {
                total_resolved += 1;
            } else {
                total_open += 1;
            }
        }
    }

    println!();
    println!(
        "Total: {} open, {} resolved across {} files",
        total_open,
        total_resolved,
        all_results.len()
    );

    Ok(())
}

pub fn count_markers(content: &str) -> (usize, usize) {
    let remarks = parse_remarks(content);
    let open = remarks.iter().filter(|r| !r.resolved).count();
    let resolved = remarks.iter().filter(|r| r.resolved).count();
    (open, resolved)
}

pub fn parse_remarks(content: &str) -> Vec<Remark> {
    let mut remarks = Vec::new();
    let mut pending_idx: Option<usize> = None;

    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("^^^") {
            let text = trimmed.strip_prefix("^^^").unwrap().trim().to_string();
            remarks.push(Remark {
                line: i + 1,
                text,
                resolved: false,
            });
            pending_idx = Some(remarks.len() - 1);
        } else if trimmed.starts_with("&&&") {
            if let Some(idx) = pending_idx.take() {
                remarks[idx].resolved = true;
            }
        }
    }

    remarks
}

fn collect_spec_files(dir: &Path) -> Result<Vec<PathBuf>, Error> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|e| e == "md") {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_markers() {
        let content = "# Title\n\nSome text\n";
        let remarks = parse_remarks(content);
        assert!(remarks.is_empty());
        assert_eq!(count_markers(content), (0, 0));
    }

    #[test]
    fn single_resolved() {
        let content = "^^^ should we use REST?\n&&& Addressed: yes.\n";
        let remarks = parse_remarks(content);
        assert_eq!(remarks.len(), 1);
        assert_eq!(remarks[0].line, 1);
        assert_eq!(remarks[0].text, "should we use REST?");
        assert!(remarks[0].resolved);
        assert_eq!(count_markers(content), (0, 1));
    }

    #[test]
    fn single_open() {
        let content = "^^^ what about rate limiting?\n";
        let remarks = parse_remarks(content);
        assert_eq!(remarks.len(), 1);
        assert_eq!(remarks[0].text, "what about rate limiting?");
        assert!(!remarks[0].resolved);
        assert_eq!(count_markers(content), (1, 0));
    }

    #[test]
    fn mixed_resolved_and_open() {
        let content = "\
^^^ REST or GraphQL?
&&& Addressed: REST.
^^^ Should we version the API?
&&& Addressed: yes, URL-based.
^^^ What about rate limiting?
";
        let remarks = parse_remarks(content);
        assert_eq!(remarks.len(), 3);
        assert!(remarks[0].resolved);
        assert!(remarks[1].resolved);
        assert!(!remarks[2].resolved);
        assert_eq!(count_markers(content), (1, 2));
    }

    #[test]
    fn consecutive_updown_then_resolved() {
        let content = "\
^^^ first question
^^^ second question
&&& Addressed: second question answer
";
        let remarks = parse_remarks(content);
        assert_eq!(remarks.len(), 2);
        assert!(!remarks[0].resolved);
        assert!(remarks[1].resolved);
        assert_eq!(count_markers(content), (1, 1));
    }

    #[test]
    fn markers_with_surrounding_text() {
        let content = "\
## Section

Some paragraph text.

^^^ should this be async?
&&& Addressed: yes.

More text.

^^^ TODO: add tests
";
        let remarks = parse_remarks(content);
        assert_eq!(remarks.len(), 2);
        assert_eq!(remarks[0].line, 5);
        assert!(remarks[0].resolved);
        assert_eq!(remarks[1].line, 10);
        assert!(!remarks[1].resolved);
        assert_eq!(count_markers(content), (1, 1));
    }

    #[test]
    fn markers_with_indented_lines() {
        let content = "  ^^^ indented remark\n  &&& Addressed: ok\n";
        let remarks = parse_remarks(content);
        assert_eq!(remarks.len(), 1);
        assert!(remarks[0].resolved);
    }

    #[test]
    fn orphaned_agent_answer() {
        let content = "&&& Addressed: something without a question\n";
        let remarks = parse_remarks(content);
        assert!(remarks.is_empty());
        assert_eq!(count_markers(content), (0, 0));
    }

    #[test]
    fn text_starts_with_but_is_not_marker() {
        let content = "The ^^^ syntax is used in spec files.\n";
        let remarks = parse_remarks(content);
        assert!(remarks.is_empty());
    }
}
