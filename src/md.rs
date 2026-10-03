//! Markdown outline built on comrak: headings, task checkboxes, code blocks,
//! and tables, with 1-based source lines. Parse once, then query owned data —
//! callers never touch the comrak arena.

use std::ops::RangeInclusive;

use comrak::nodes::{AstNode, ListType, NodeValue};
use comrak::{Arena, Options, parse_document};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Heading {
    pub level: u8,
    pub text: String,
    pub line: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Checkbox {
    pub checked: bool,
    pub line: usize,
}

#[derive(Debug, Default)]
pub struct Outline {
    pub headings: Vec<Heading>,
    pub checkboxes: Vec<Checkbox>,
    /// Line ranges (1-based, inclusive) covered by code blocks, fences included.
    pub code_blocks: Vec<RangeInclusive<usize>>,
    pub tables: usize,
    pub lines: usize,
}

pub fn outline(content: &str) -> Outline {
    let arena = Arena::new();
    let mut options = Options::default();
    options.extension.tasklist = true;
    options.extension.table = true;
    options.extension.front_matter_delimiter = Some("---".to_string());
    let root = parse_document(&arena, content, &options);

    let mut out = Outline {
        lines: content.lines().count(),
        ..Default::default()
    };
    for node in root.descendants() {
        let ast = node.data();
        let start = ast.sourcepos.start.line;
        match &ast.value {
            NodeValue::Heading(h) => out.headings.push(Heading {
                level: h.level,
                text: inline_text(node),
                line: start,
            }),
            // Only bullet-list task items count; `1. [x]` is not a task step.
            NodeValue::TaskItem(item) if in_bullet_list(node) => {
                out.checkboxes.push(Checkbox {
                    checked: item.symbol.is_some(),
                    line: start,
                })
            }
            NodeValue::CodeBlock(_) => {
                out.code_blocks.push(start..=ast.sourcepos.end.line)
            }
            NodeValue::Table(_) => out.tables += 1,
            _ => {}
        }
    }
    out
}

impl Outline {
    /// Text of the first non-empty heading.
    pub fn first_heading(&self) -> Option<&str> {
        self.headings
            .iter()
            .map(|h| h.text.as_str())
            .find(|t| !t.is_empty())
    }

    /// Heading counts by level: (h1, h2, h3, h4+).
    pub fn heading_counts(&self) -> (usize, usize, usize, usize) {
        let mut counts = (0, 0, 0, 0);
        for h in &self.headings {
            match h.level {
                1 => counts.0 += 1,
                2 => counts.1 += 1,
                3 => counts.2 += 1,
                _ => counts.3 += 1,
            }
        }
        counts
    }

    /// Task checkboxes: (checked, total).
    pub fn checkbox_counts(&self) -> (usize, usize) {
        count(self.checkboxes.iter())
    }

    /// Task checkboxes inside the `## <name>` section: (checked, total).
    pub fn checkbox_counts_in(&self, name: &str) -> (usize, usize) {
        match self.section(name) {
            Some(range) => {
                count(self.checkboxes.iter().filter(|c| range.contains(&c.line)))
            }
            None => (0, 0),
        }
    }

    /// Lines of the `## <name>` section body: from the line after the heading
    /// to the line before the next heading of level 1 or 2 (or end of file).
    pub fn section(&self, name: &str) -> Option<RangeInclusive<usize>> {
        let idx = self
            .headings
            .iter()
            .position(|h| h.level == 2 && h.text == name)?;
        let start = self.headings[idx].line + 1;
        let end = self.headings[idx + 1..]
            .iter()
            .find(|h| h.level <= 2)
            .map_or(self.lines, |h| h.line - 1);
        Some(start..=end)
    }

    pub fn is_in_code(&self, line: usize) -> bool {
        self.code_blocks.iter().any(|r| r.contains(&line))
    }
}

fn count<'a>(boxes: impl Iterator<Item = &'a Checkbox>) -> (usize, usize) {
    boxes.fold((0, 0), |(done, total), c| {
        (done + usize::from(c.checked), total + 1)
    })
}

fn in_bullet_list<'a>(node: &'a AstNode<'a>) -> bool {
    node.parent().is_some_and(|p| {
        matches!(&p.data().value, NodeValue::List(l) if l.list_type == ListType::Bullet)
    })
}

/// Concatenated text and inline code under `node`.
fn inline_text<'a>(node: &'a AstNode<'a>) -> String {
    let mut text = String::new();
    for d in node.descendants() {
        match &d.data().value {
            NodeValue::Text(t) => text.push_str(t),
            NodeValue::Code(c) => text.push_str(&c.literal),
            _ => {}
        }
    }
    text.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn headings_with_levels_and_lines() {
        let o = outline("# A\n\n## B `code`\n\ntext\n### C\n");
        let got: Vec<_> = o
            .headings
            .iter()
            .map(|h| (h.level, h.text.as_str(), h.line))
            .collect();
        assert_eq!(got, vec![(1, "A", 1), (2, "B code", 3), (3, "C", 6)]);
    }

    #[test]
    fn headings_inside_fences_are_ignored() {
        let o = outline("# Real\n\n```md\n# Fake\n## Also fake\n```\n");
        assert_eq!(o.heading_counts(), (1, 0, 0, 0));
        assert_eq!(o.first_heading(), Some("Real"));
    }

    #[test]
    fn checkboxes_inside_fences_are_ignored() {
        let o = outline("- [x] real\n\n```\n- [ ] fake\n- [x] fake\n```\n");
        assert_eq!(o.checkbox_counts(), (1, 1));
    }

    #[test]
    fn ordered_task_items_do_not_count() {
        let o = outline("1. [x] numbered\n");
        assert_eq!(o.checkbox_counts(), (0, 0));
    }

    #[test]
    fn code_block_line_ranges() {
        let o = outline("text\n\n```\na\nb\n```\nafter\n");
        assert_eq!(o.code_blocks, vec![3..=6]);
        assert!(o.is_in_code(4));
        assert!(!o.is_in_code(7));
    }

    #[test]
    fn section_bounds_and_scoped_checkboxes() {
        let content = "\
# Task: t

## Plan
- [x] one
- [ ] two

### Sub
- [x] three

## Acceptance
- [ ] not a plan step
";
        let o = outline(content);
        assert_eq!(o.section("Plan"), Some(4..=9));
        assert_eq!(o.checkbox_counts_in("Plan"), (2, 3));
        assert_eq!(o.checkbox_counts_in("Missing"), (0, 0));
    }

    #[test]
    fn front_matter_is_not_a_heading() {
        let o = outline("---\nid: 0001-x\n---\n# Task: x\n");
        assert_eq!(o.headings.len(), 1);
        assert_eq!(o.headings[0].line, 4);
    }

    #[test]
    fn counts_tables() {
        let o = outline("| a | b |\n|---|---|\n| 1 | 2 |\n");
        assert_eq!(o.tables, 1);
    }
}
