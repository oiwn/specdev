//! Diagnostics shared by every check: a file, an optional line, a severity, a
//! stable code agents can match on, and a human message.

use std::fmt;
use std::path::PathBuf;

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Diagnostic {
    pub file: PathBuf,
    pub line: Option<usize>,
    pub severity: Severity,
    pub code: &'static str,
    pub message: String,
}

impl Diagnostic {
    pub fn error(
        file: impl Into<PathBuf>,
        line: Option<usize>,
        code: &'static str,
        message: impl Into<String>,
    ) -> Self {
        Self {
            file: file.into(),
            line,
            severity: Severity::Error,
            code,
            message: message.into(),
        }
    }

    /// The same diagnostic, downgraded to a warning.
    pub fn into_warning(mut self) -> Self {
        self.severity = Severity::Warning;
        self
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let severity = match self.severity {
            Severity::Warning => "warning",
            Severity::Error => "error",
        };
        write!(f, "{}", self.file.display())?;
        if let Some(line) = self.line {
            write!(f, ":{line}")?;
        }
        write!(f, ": {severity}[{}]: {}", self.code, self.message)
    }
}

impl std::error::Error for Diagnostic {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_with_and_without_line() {
        let d = Diagnostic::error(
            "specs/tasks/0001-x.md",
            Some(3),
            "frontmatter",
            "bad",
        );
        assert_eq!(
            d.to_string(),
            "specs/tasks/0001-x.md:3: error[frontmatter]: bad"
        );
        let w = Diagnostic::error("a.md", None, "log", "odd").into_warning();
        assert_eq!(w.to_string(), "a.md: warning[log]: odd");
    }
}
