//! Command output: every command builds a serializable report, rendered as
//! text for humans, JSON for tools, or TOON for LLM agents.

use serde::Serialize;

use crate::Error;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, clap::ValueEnum)]
pub enum Format {
    #[default]
    Text,
    Json,
    Toon,
}

pub trait Report: Serialize {
    /// Human-readable rendering, without a trailing newline.
    fn text(&self) -> String;
}

pub fn render(report: &impl Report, format: Format) -> Result<String, Error> {
    match format {
        Format::Text => Ok(report.text()),
        Format::Json => serde_json::to_string_pretty(report)
            .map_err(|e| Error::Output(e.to_string())),
        Format::Toon => toon_format::encode_default(report)
            .map_err(|e| Error::Output(e.to_string())),
    }
}

pub fn emit(report: &impl Report, format: Format) -> Result<(), Error> {
    println!("{}", render(report, format)?);
    Ok(())
}

/// Commands not yet migrated to reports accept only text output.
pub fn require_text(command: &str, format: Format) -> Result<(), Error> {
    match format {
        Format::Text => Ok(()),
        _ => Err(Error::Usage(format!(
            "`{command}` does not support --format json|toon yet"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Serialize)]
    struct Sample {
        name: &'static str,
        count: u32,
    }

    impl Report for Sample {
        fn text(&self) -> String {
            format!("{} = {}", self.name, self.count)
        }
    }

    #[test]
    fn renders_each_format() {
        let s = Sample {
            name: "tasks",
            count: 2,
        };
        assert_eq!(render(&s, Format::Text).unwrap(), "tasks = 2");
        let json: serde_json::Value =
            serde_json::from_str(&render(&s, Format::Json).unwrap()).unwrap();
        assert_eq!(json["count"], 2);
        let toon = render(&s, Format::Toon).unwrap();
        assert!(toon.contains("name: tasks"), "got {toon:?}");
    }

    #[test]
    fn require_text_rejects_other_formats() {
        assert!(require_text("scan", Format::Text).is_ok());
        assert!(require_text("scan", Format::Json).is_err());
    }
}
