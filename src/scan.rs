use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::Error;
use crate::fmt;
use crate::md;
use crate::output::{self, Format, Report};

#[derive(Serialize)]
pub struct Remark {
    pub line: usize,
    pub text: String,
    pub resolved: bool,
}

#[derive(Serialize)]
pub struct FileRemarks {
    pub path: PathBuf,
    pub remarks: Vec<Remark>,
}

pub fn run(root: &Path, format: Format) -> Result<(), Error> {
    let specs_dir = root.join("specs");
    if !specs_dir.exists() {
        eprintln!("No specs/ directory found. Run `specdev init` first.");
        return Ok(());
    }

    let mut report = ScanReport {
        files: Vec::new(),
        open: 0,
        resolved: 0,
    };
    // Spec files plus open task files (remarks gate `advance` there too).
    for path in fmt::default_files(root)? {
        let remarks = parse_remarks(&fs::read_to_string(&path)?);
        for r in &remarks {
            if r.resolved {
                report.resolved += 1;
            } else {
                report.open += 1;
            }
        }
        report.files.push(FileRemarks {
            path: path.strip_prefix(&specs_dir).unwrap_or(&path).to_path_buf(),
            remarks,
        });
    }
    output::emit(&report, format)
}

#[derive(Serialize)]
struct ScanReport {
    files: Vec<FileRemarks>,
    open: usize,
    resolved: usize,
}

impl Report for ScanReport {
    fn text(&self) -> String {
        if self.files.is_empty() {
            return "No spec files found in specs/".to_string();
        }
        let mut out = Vec::new();
        for fr in &self.files {
            out.push(fr.path.display().to_string());
            if fr.remarks.is_empty() {
                out.push("  (no markers)".to_string());
            }
            for r in &fr.remarks {
                let status = if r.resolved { "resolved" } else { "open" };
                out.push(format!("  L{:>3}  [{:>8}]  {}", r.line, status, r.text));
            }
        }
        out.push(String::new());
        out.push(format!(
            "Total: {} open, {} resolved across {} files",
            self.open,
            self.resolved,
            self.files.len()
        ));
        out.join("\n")
    }
}

pub fn count_markers(content: &str) -> (usize, usize) {
    let remarks = parse_remarks(content);
    let open = remarks.iter().filter(|r| !r.resolved).count();
    let resolved = remarks.iter().filter(|r| r.resolved).count();
    (open, resolved)
}

pub fn parse_remarks(content: &str) -> Vec<Remark> {
    let outline = md::outline(content);
    let mut remarks = Vec::new();
    let mut pending_idx: Option<usize> = None;

    for (i, line) in content.lines().enumerate() {
        if outline.is_in_code(i + 1) {
            continue;
        }
        let trimmed = line.trim();
        if trimmed.starts_with("^^^") {
            let text = trimmed.strip_prefix("^^^").unwrap().trim().to_string();
            remarks.push(Remark {
                line: i + 1,
                text,
                resolved: false,
            });
            pending_idx = Some(remarks.len() - 1);
        } else if trimmed.starts_with("&&&")
            && let Some(idx) = pending_idx.take()
        {
            remarks[idx].resolved = true;
        }
    }

    remarks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markers_inside_code_blocks_are_ignored() {
        let content =
            "```\n^^^ example remark\n&&& example answer\n```\n^^^ real\n";
        let remarks = parse_remarks(content);
        assert_eq!(remarks.len(), 1);
        assert_eq!(remarks[0].text, "real");
        assert_eq!(remarks[0].line, 5);
    }

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
