//! `specdev fmt`: normalizes spec Markdown with targeted splices — unwraps
//! hard-wrapped paragraphs, turns `*`/`+` bullets into `-`, and puts blank
//! lines around `#` headings — keeping every other byte. Where those things
//! are comes from `md::fmt_facts`; nothing here parses Markdown.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::Error;
use crate::diag::Diagnostic;
use crate::md;
use crate::output::{self, Format, Report};
use crate::task::TASKS_DIR;

pub fn run(root: &Path, files: &[PathBuf], format: Format) -> Result<(), Error> {
    let targets = if files.is_empty() {
        default_files(root)?
    } else {
        files.iter().map(|f| root.join(f)).collect()
    };
    let mut report = FmtReport {
        changed: Vec::new(),
        unchanged: 0,
    };
    for path in targets {
        let content = read(&path)?;
        let normalized = normalize(&content);
        if normalized == content {
            report.unchanged += 1;
        } else {
            fs::write(&path, normalized)?;
            report.changed.push(path.display().to_string());
        }
    }
    output::emit(&report, format)
}

/// `specs/*.md` and the open task files `specs/tasks/*.md` (no generated
/// `_` files, nothing in `done/`).
pub fn default_files(root: &Path) -> Result<Vec<PathBuf>, Error> {
    let specs = root.join("specs");
    if !specs.is_dir() {
        return Err(Error::Usage(
            "No specs/ directory found. Run `specdev init` first.".to_string(),
        ));
    }
    let mut files = markdown_in(&specs)?;
    let tasks = root.join(TASKS_DIR);
    if tasks.is_dir() {
        files.extend(markdown_in(&tasks)?);
    }
    Ok(files)
}

fn markdown_in(dir: &Path) -> Result<Vec<PathBuf>, Error> {
    let mut files: Vec<PathBuf> = fs::read_dir(dir)?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            p.is_file()
                && p.extension().is_some_and(|e| e == "md")
                && !p
                    .file_name()
                    .is_some_and(|n| n.to_string_lossy().starts_with('_'))
        })
        .collect();
    files.sort();
    Ok(files)
}

fn read(path: &Path) -> Result<String, Error> {
    fs::read_to_string(path)
        .map_err(|e| Error::Usage(format!("{}: {e}", path.display())))
}

/// `unformatted` warnings for `check`, over the same files `fmt` covers.
pub fn unformatted(root: &Path) -> Result<Vec<Diagnostic>, Error> {
    let mut diags = Vec::new();
    for path in default_files(root)? {
        let content = read(&path)?;
        if normalize(&content) != content {
            diags.push(Diagnostic::warning(
                &path,
                None,
                "unformatted",
                "hard-wrapped paragraphs, `*`/`+` bullets, or headings without blank lines around them; run `specdev fmt`",
            ));
        }
    }
    Ok(diags)
}

/// The three normalizations. Idempotent: `normalize(normalize(x)) == normalize(x)`.
pub fn normalize(content: &str) -> String {
    let facts = md::fmt_facts(content);
    let mut lines: Vec<String> = content.lines().map(str::to_string).collect();

    for &(line, column) in &facts.bullets {
        if let Some(text) = lines.get_mut(line - 1) {
            let at = column - 1;
            if matches!(text.as_bytes().get(at), Some(b'*' | b'+')) {
                text.replace_range(at..=at, "-");
            }
        }
    }

    // `join_next[i]`: line i+1 (1-based) continues on the next line.
    let mut join_next = vec![false; lines.len()];
    for paragraph in facts.paragraphs.iter().filter(|p| !p.quoted) {
        for line in *paragraph.lines.start()..*paragraph.lines.end() {
            let keep_break = paragraph.hard_breaks.contains(&line)
                || is_remark(&lines[line - 1])
                || is_remark(&lines[line]);
            join_next[line - 1] = !keep_break;
        }
    }

    let headings: HashSet<usize> = facts.atx_headings.iter().copied().collect();
    let mut out: Vec<String> = Vec::with_capacity(lines.len());
    let mut i = 0;
    while i < lines.len() {
        let number = i + 1;
        if headings.contains(&number) {
            let after_frontmatter = facts.frontmatter_end == Some(number - 1);
            if !after_frontmatter
                && out.last().is_some_and(|l| !l.trim().is_empty())
            {
                out.push(String::new());
            }
            out.push(lines[i].clone());
            if lines.get(i + 1).is_some_and(|l| !l.trim().is_empty()) {
                out.push(String::new());
            }
            i += 1;
            continue;
        }
        let mut line = lines[i].clone();
        while join_next[i] {
            line = format!("{} {}", line.trim_end(), lines[i + 1].trim_start());
            i += 1;
        }
        out.push(line);
        i += 1;
    }

    let mut result = out.join("\n");
    if content.ends_with('\n') {
        result.push('\n');
    }
    result
}

/// `^^^` (human) and `&&&` (agent) remark lines always stay on their own line.
fn is_remark(line: &str) -> bool {
    let line = line.trim_start();
    line.starts_with("^^^") || line.starts_with("&&&")
}

#[derive(Serialize)]
struct FmtReport {
    changed: Vec<String>,
    unchanged: usize,
}

impl Report for FmtReport {
    fn text(&self) -> String {
        if self.changed.is_empty() {
            return format!("All {} files already formatted", self.unchanged);
        }
        let mut out: Vec<String> = self
            .changed
            .iter()
            .map(|p| format!("Formatted {p}"))
            .collect();
        out.push(format!(
            "{} formatted, {} already formatted",
            self.changed.len(),
            self.unchanged
        ));
        out.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::Task;

    /// `normalize`, checked to be idempotent on every input.
    fn fmt(input: &str) -> String {
        let once = normalize(input);
        assert_eq!(normalize(&once), once, "not idempotent for {input:?}");
        once
    }

    #[test]
    fn unwraps_paragraphs_and_list_items() {
        assert_eq!(
            fmt("# T\n\nOne sentence\nwrapped here.\n"),
            "# T\n\nOne sentence wrapped here.\n"
        );
        assert_eq!(
            fmt("- a bullet that\n  wraps\n- next\n"),
            "- a bullet that wraps\n- next\n"
        );
    }

    #[test]
    fn keeps_hard_breaks_remarks_and_quotes() {
        let hard = "line one  \nline two\n";
        assert_eq!(fmt(hard), hard);
        let backslash = "line one\\\nline two\n";
        assert_eq!(fmt(backslash), backslash);
        let remarks = "Plan text\nmore text\n^^^ why?\n&&& because\n";
        assert_eq!(fmt(remarks), "Plan text more text\n^^^ why?\n&&& because\n");
        let quote = "> quoted\n> wrapped\n";
        assert_eq!(fmt(quote), quote);
    }

    #[test]
    fn bullets_become_dashes() {
        assert_eq!(fmt("* a\n* b\n  + nested\n"), "- a\n- b\n  - nested\n");
        let rule = "text\n\n***\n";
        assert_eq!(fmt(rule), rule, "a thematic break is not a list");
    }

    #[test]
    fn blank_lines_around_headings() {
        assert_eq!(
            fmt("# A\ntext\n## B\n- x\n"),
            "# A\n\ntext\n\n## B\n\n- x\n"
        );
        let frontmatter = "---\nid: x\n---\n# Task: x\n\n## Plan\n";
        assert_eq!(fmt(frontmatter), frontmatter);
        let setext = "Title\n=====\ntext\n";
        assert_eq!(fmt(setext), setext);
    }

    #[test]
    fn leaves_code_tables_html_and_links_alone() {
        let content = "\
# T

```
wrapped
* code
# not a heading
```

| a | b |
|---|---|
| 1 | 2 |

<!-- comment
## inside -->

A [link](https://example.com) and *emphasis* stay as written.
";
        assert_eq!(fmt(content), content);
    }

    #[test]
    fn task_files_still_parse_after_fmt() {
        let content = "\
---
id: 0001-x
status: draft
scope: [src/a.rs]
created: 2026-10-04
---
# Task: x
## Plan
* [ ] a step that is
  wrapped

## Log

- 2026-10-04 created: x
";
        let out = fmt(content);
        let task = Task::parse(Path::new("specs/tasks/0001-x.md"), &out).unwrap();
        assert_eq!(task.log.len(), 1);
        assert!(
            out.contains("# Task: x\n\n## Plan\n\n- [ ] a step that is wrapped\n"),
            "got {out:?}"
        );
        assert!(out.starts_with("---\nid: 0001-x\nstatus: draft\n"));
    }

    #[test]
    fn no_trailing_newline_is_preserved() {
        assert_eq!(fmt("a\nb"), "a b");
    }
}
