//! The spec quality gate: per-file metrics, and the thresholds from
//! `[quality.*]` in `specdev.toml`. `check` reports violations as warnings
//! (errors with `errors = true`); `list --stats` shows the metrics.

use std::fs;
use std::path::Path;

use serde::Serialize;

use crate::Error;
use crate::config::{Config, Limits, Quality};
use crate::diag::{Diagnostic, Severity};
use crate::md;
use crate::scan;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Metrics {
    pub lines: usize,
    pub words: usize,
    pub sections: usize,
    pub max_depth: u8,
    pub plan_steps: usize,
    pub code_blocks: usize,
    pub tables: usize,
    pub diagrams: usize,
    pub open_remarks: usize,
}

/// Metrics of `content`, given its outline (callers usually need both).
pub fn measure_with(content: &str, outline: &md::Outline) -> Metrics {
    Metrics {
        lines: outline.lines,
        words: content.split_whitespace().count(),
        sections: outline.sections(),
        max_depth: outline.max_depth(),
        plan_steps: outline.checkbox_counts_in("Plan").1,
        code_blocks: outline.code_blocks.len(),
        tables: outline.tables.len(),
        diagrams: outline.diagrams.len(),
        open_remarks: scan::count_markers(content).0,
    }
}

/// Which `[quality.<kind>]` limits apply. `Spec` files (roadmap, ideas, …)
/// have no size limits; only the table/diagram rule reaches them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Task,
    Ctx,
    Overview,
    Changelog,
    Spec,
}

impl Kind {
    fn limits(self, quality: &Quality) -> Option<&Limits> {
        match self {
            Self::Task => Some(&quality.task),
            Self::Ctx => Some(&quality.ctx),
            Self::Overview => Some(&quality.overview),
            Self::Changelog => Some(&quality.changelog),
            Self::Spec => None,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Task => "task files",
            Self::Ctx => "ctx.md",
            Self::Overview => "overview.md",
            Self::Changelog => "CHANGELOG.md",
            Self::Spec => "spec files",
        }
    }

    fn advice(self) -> &'static str {
        match self {
            Self::Task => "split the task or move detail out",
            Self::Ctx => {
                "route content to its home file (overview, roadmap, task files)"
            }
            Self::Overview => "move detail into focused specs",
            Self::Changelog => "compress older entries; history stays in git",
            Self::Spec => "",
        }
    }
}

/// Threshold and table/diagram diagnostics for one file. `scope_len` is
/// the task's scope size (tasks only).
pub fn diagnostics(
    path: &Path,
    content: &str,
    kind: Kind,
    scope_len: Option<usize>,
    config: &Config,
) -> Vec<Diagnostic> {
    let quality = &config.quality;
    let severity = if quality.errors {
        Severity::Error
    } else {
        Severity::Warning
    };
    let outline = md::outline(content);
    let metrics = measure_with(content, &outline);
    let mut diags = Vec::new();

    if let Some(limits) = kind.limits(quality) {
        let mut over =
            |code: &'static str, what: &str, value: usize, max: Option<usize>| {
                if let Some(max) = max
                    && max > 0
                    && value > max
                {
                    diags.push(
                        Diagnostic::error(
                            path,
                            None,
                            code,
                            format!(
                                "{value} {what} (max {max} for {}); {}",
                                kind.name(),
                                kind.advice()
                            ),
                        )
                        .with_severity(severity),
                    );
                }
            };
        over("quality-lines", "lines", metrics.lines, limits.max_lines);
        over("quality-words", "words", metrics.words, limits.max_words);
        over(
            "quality-plan-steps",
            "plan steps",
            metrics.plan_steps,
            limits.max_plan_steps,
        );
        if let Some(scope_len) = scope_len {
            over(
                "quality-scope",
                "scope entries",
                scope_len,
                limits.max_scope,
            );
        }
    }

    if quality.forbid_tables {
        for &line in &outline.tables {
            diags.push(
                Diagnostic::error(
                    path,
                    Some(line),
                    "md-table",
                    "Markdown table; use bullets or short sections (tables read badly in terminal editors)",
                )
                .with_severity(severity),
            );
        }
        for &line in &outline.diagrams {
            diags.push(
                Diagnostic::error(
                    path,
                    Some(line),
                    "ascii-diagram",
                    "ASCII diagram; describe it with bullets or numbered steps",
                )
                .with_severity(severity),
            );
        }
    }
    diags
}

/// The gate for non-task files: `specs/*.md` (ctx and overview have limits,
/// the rest only the table rule) and the root `CHANGELOG.md`.
pub fn spec_diagnostics(
    root: &Path,
    config: &Config,
) -> Result<Vec<Diagnostic>, Error> {
    let specs = root.join("specs");
    let mut files: Vec<(std::path::PathBuf, Kind)> = Vec::new();
    for entry in fs::read_dir(&specs)? {
        let path = entry?.path();
        if !path.is_file() || path.extension().is_none_or(|e| e != "md") {
            continue;
        }
        let kind = match path.file_name().and_then(|n| n.to_str()) {
            Some("ctx.md") => Kind::Ctx,
            Some("overview.md") => Kind::Overview,
            _ => Kind::Spec,
        };
        files.push((path, kind));
    }
    files.sort_by(|a, b| a.0.cmp(&b.0));
    let changelog = root.join("CHANGELOG.md");
    if changelog.is_file() {
        files.push((changelog, Kind::Changelog));
    }

    let mut diags = Vec::new();
    for (path, kind) in files {
        let content = fs::read_to_string(&path)?;
        diags.extend(diagnostics(&path, &content, kind, None, config));
    }
    Ok(diags)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn codes(diags: &[Diagnostic]) -> Vec<(&'static str, Severity)> {
        diags.iter().map(|d| (d.code, d.severity)).collect()
    }

    #[test]
    fn measures_a_task_like_file() {
        let content = "\
# Task: x

## Plan

- [x] one
- [ ] two

## Notes

### Deep

```
code
```

^^^ open question
";
        let m = measure_with(content, &md::outline(content));
        assert_eq!(
            m,
            Metrics {
                lines: 16,
                words: 22,
                sections: 2,
                max_depth: 3,
                plan_steps: 2,
                code_blocks: 1,
                tables: 0,
                diagrams: 0,
                open_remarks: 1,
            }
        );
    }

    fn task_with_steps(n: usize) -> String {
        let mut s = "# Task: x\n\n## Plan\n\n".to_string();
        for i in 0..n {
            s.push_str(&format!("- [ ] step {i}\n"));
        }
        s
    }

    #[test]
    fn limits_fire_above_max_only() {
        let config = Config::default();
        let path = Path::new("t.md");
        let at =
            diagnostics(path, &task_with_steps(8), Kind::Task, Some(6), &config);
        assert!(at.is_empty(), "{at:?}");
        let over =
            diagnostics(path, &task_with_steps(9), Kind::Task, Some(7), &config);
        assert_eq!(
            codes(&over),
            [
                ("quality-plan-steps", Severity::Warning),
                ("quality-scope", Severity::Warning)
            ]
        );
        assert!(
            over[0]
                .message
                .contains("9 plan steps (max 8 for task files)")
        );

        let long = "line\n".repeat(401);
        let log = diagnostics(path, &long, Kind::Changelog, None, &config);
        assert_eq!(codes(&log), [("quality-lines", Severity::Warning)]);
        assert!(log[0].message.contains("compress older entries"));
        assert!(diagnostics(path, &long, Kind::Spec, None, &config).is_empty());

        let mut off = Config::default();
        off.quality.changelog.max_lines = Some(0);
        assert!(diagnostics(path, &long, Kind::Changelog, None, &off).is_empty());
    }

    #[test]
    fn word_limit_catches_soft_wrapped_files() {
        let config = Config::default();
        let path = Path::new("t.md");
        let wordy = format!("# Task: x\n\n{}\n", "word ".repeat(1000));
        let diags = diagnostics(path, &wordy, Kind::Task, None, &config);
        assert_eq!(codes(&diags), [("quality-words", Severity::Warning)]);
        assert!(
            diags[0]
                .message
                .contains("1003 words (max 1000 for task files)")
        );
    }

    #[test]
    fn errors_mode_and_table_rule() {
        let mut config = Config::default();
        let content = "# Ideas\n\n| a | b |\n|---|---|\n| 1 | 2 |\n\n```\n+--+\n|  |\n+--+\n```\n";
        let path = Path::new("specs/ideas.md");
        assert!(diagnostics(path, content, Kind::Spec, None, &config).is_empty());

        config.quality.forbid_tables = true;
        let diags = diagnostics(path, content, Kind::Spec, None, &config);
        assert_eq!(
            codes(&diags),
            [
                ("md-table", Severity::Warning),
                ("ascii-diagram", Severity::Warning)
            ]
        );
        assert_eq!((diags[0].line, diags[1].line), (Some(3), Some(7)));

        config.quality.errors = true;
        let diags = diagnostics(path, content, Kind::Spec, None, &config);
        assert!(diags.iter().all(|d| d.severity == Severity::Error));
    }
}
