//! Task frontmatter in a flat-YAML subset: one `key: value` per line, where a
//! value is a bare scalar, a `"quoted string"`, or an inline list `[a, b]`.
//! Comments (` # ...`) and blank lines are allowed on input; rendering is
//! normalized (fixed field order, no comments) because specdev owns this block.

use std::str::FromStr;

use chrono::NaiveDate;

use super::{Stage, Status, TaskId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Scalar(String),
    List(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frontmatter {
    pub id: TaskId,
    pub status: Status,
    pub stage: Option<Stage>,
    pub scope: Vec<String>,
    pub created: NaiveDate,
    pub source: Option<String>,
    pub depends: Vec<TaskId>,
    pub attempts: u32,
    pub blocked_reason: Option<String>,
    /// Unknown or project-specific fields, in their original order.
    pub extra: Vec<(String, Value)>,
}

/// A parse error: line within the frontmatter block (1-based; 0 means the
/// block as a whole) and a message.
pub type ParseError = (usize, String);

type Fields = Vec<(String, Value, usize)>;

pub const DATE_FORMAT: &str = "%Y-%m-%d";

impl Frontmatter {
    pub fn parse(text: &str) -> Result<Self, ParseError> {
        let mut fields: Fields = Vec::new();
        for (i, raw) in text.lines().enumerate() {
            let line_no = i + 1;
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let (key, value) = line.split_once(':').ok_or_else(|| {
                (line_no, format!("expected `key: value`, got `{line}`"))
            })?;
            let key = key.trim();
            let key_ok = !key.is_empty()
                && key.chars().all(|c| {
                    c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'
                });
            if !key_ok {
                return Err((line_no, format!("invalid key `{key}`")));
            }
            if fields.iter().any(|(k, ..)| k == key) {
                return Err((line_no, format!("duplicate key `{key}`")));
            }
            let value = parse_value(strip_comment(value).trim())
                .map_err(|m| (line_no, format!("`{key}`: {m}")))?;
            fields.push((key.to_string(), value, line_no));
        }

        let id = required(&mut fields, "id")
            .and_then(|(v, l)| scalar("id", v, l))
            .and_then(|(s, l)| s.parse::<TaskId>().map_err(|e| (l, e)))?;
        let status = required(&mut fields, "status")
            .and_then(|(v, l)| scalar("status", v, l))
            .and_then(|(s, l)| Status::from_str(&s).map_err(|e| (l, e)))?;
        let stage = optional_scalar(&mut fields, "stage")?
            .map(|(s, l)| Stage::from_str(&s).map_err(|e| (l, e)))
            .transpose()?;
        let scope = required(&mut fields, "scope")
            .and_then(|(v, l)| list("scope", v, l))?;
        let created = required(&mut fields, "created")
            .and_then(|(v, l)| scalar("created", v, l))
            .and_then(|(s, l)| {
                NaiveDate::parse_from_str(&s, DATE_FORMAT).map_err(|_| {
                    (l, format!("`created`: expected YYYY-MM-DD, got `{s}`"))
                })
            })?;
        let source = optional_scalar(&mut fields, "source")?.map(|(s, _)| s);
        let depends = match take(&mut fields, "depends") {
            None => Vec::new(),
            Some((v, l)) => list("depends", v, l)?
                .iter()
                .map(|s| s.parse::<TaskId>().map_err(|e| (l, e)))
                .collect::<Result<_, _>>()?,
        };
        let attempts = match optional_scalar(&mut fields, "attempts")? {
            None => 0,
            Some((s, l)) => s.parse().map_err(|_| {
                (l, format!("`attempts`: expected a number, got `{s}`"))
            })?,
        };
        let blocked_reason =
            optional_scalar(&mut fields, "blocked_reason")?.map(|(s, _)| s);
        let extra = fields.into_iter().map(|(k, v, _)| (k, v)).collect();

        Ok(Self {
            id,
            status,
            stage,
            scope,
            created,
            source,
            depends,
            attempts,
            blocked_reason,
            extra,
        })
    }

    /// Canonical rendering, one field per line, each ending in `\n`.
    pub fn render(&self) -> String {
        let mut out = String::new();
        let mut push = |key: &str, value: String| {
            out.push_str(key);
            out.push_str(": ");
            out.push_str(&value);
            out.push('\n');
        };
        push("id", self.id.to_string());
        push("status", self.status.to_string());
        if let Some(stage) = self.stage {
            push("stage", stage.to_string());
        }
        push("scope", render_list(&self.scope));
        push("created", self.created.format(DATE_FORMAT).to_string());
        if let Some(source) = &self.source {
            push("source", render_scalar(source));
        }
        if !self.depends.is_empty() {
            let ids: Vec<String> =
                self.depends.iter().map(ToString::to_string).collect();
            push("depends", render_list(&ids));
        }
        if self.attempts > 0 {
            push("attempts", self.attempts.to_string());
        }
        if let Some(reason) = &self.blocked_reason {
            push("blocked_reason", render_scalar(reason));
        }
        for (key, value) in &self.extra {
            let rendered = match value {
                Value::Scalar(s) => render_scalar(s),
                Value::List(items) => render_list(items),
            };
            push(key, rendered);
        }
        out
    }
}

fn take(fields: &mut Fields, key: &str) -> Option<(Value, usize)> {
    let i = fields.iter().position(|(k, ..)| k == key)?;
    let (_, value, line) = fields.remove(i);
    Some((value, line))
}

fn required(fields: &mut Fields, key: &str) -> Result<(Value, usize), ParseError> {
    take(fields, key).ok_or_else(|| (0, format!("missing required field `{key}`")))
}

fn scalar(
    key: &str,
    value: Value,
    line: usize,
) -> Result<(String, usize), ParseError> {
    match value {
        Value::Scalar(s) if !s.is_empty() => Ok((s, line)),
        Value::Scalar(_) => Err((line, format!("`{key}` must not be empty"))),
        Value::List(_) => Err((line, format!("`{key}` must be a single value"))),
    }
}

/// An optional scalar; an empty value counts as absent.
fn optional_scalar(
    fields: &mut Fields,
    key: &str,
) -> Result<Option<(String, usize)>, ParseError> {
    match take(fields, key) {
        None => Ok(None),
        Some((Value::Scalar(s), _)) if s.is_empty() => Ok(None),
        Some((value, line)) => scalar(key, value, line).map(Some),
    }
}

fn list(key: &str, value: Value, line: usize) -> Result<Vec<String>, ParseError> {
    match value {
        Value::List(items) => Ok(items),
        Value::Scalar(_) => {
            Err((line, format!("`{key}` must be a list like [a, b]")))
        }
    }
}

/// Drop a trailing ` # comment` that is outside quotes.
fn strip_comment(value: &str) -> &str {
    let mut in_quote = false;
    let mut prev = ' ';
    for (i, c) in value.char_indices() {
        match c {
            '"' if prev != '\\' => in_quote = !in_quote,
            '#' if !in_quote && prev.is_whitespace() => return &value[..i],
            _ => {}
        }
        prev = c;
    }
    value
}

fn parse_value(s: &str) -> Result<Value, String> {
    if let Some(inner) = s.strip_prefix('[') {
        let inner = inner
            .strip_suffix(']')
            .ok_or("unterminated list: missing `]`")?
            .trim();
        if inner.is_empty() {
            return Ok(Value::List(Vec::new()));
        }
        let items = split_list(inner)?
            .into_iter()
            .map(|item| {
                if item.is_empty() {
                    Err("empty list item".to_string())
                } else {
                    unquote(item)
                }
            })
            .collect::<Result<_, _>>()?;
        Ok(Value::List(items))
    } else {
        unquote(s).map(Value::Scalar)
    }
}

/// Split list contents on commas that are outside quotes.
fn split_list(inner: &str) -> Result<Vec<&str>, String> {
    let mut items = Vec::new();
    let mut in_quote = false;
    let mut prev = ' ';
    let mut start = 0;
    for (i, c) in inner.char_indices() {
        match c {
            '"' if prev != '\\' => in_quote = !in_quote,
            ',' if !in_quote => {
                items.push(inner[start..i].trim());
                start = i + 1;
            }
            _ => {}
        }
        prev = c;
    }
    if in_quote {
        return Err("unterminated quoted string".to_string());
    }
    items.push(inner[start..].trim());
    Ok(items)
}

fn unquote(s: &str) -> Result<String, String> {
    let Some(rest) = s.strip_prefix('"') else {
        return Ok(s.to_string());
    };
    let inner = rest.strip_suffix('"').ok_or("unterminated quoted string")?;
    let mut out = String::with_capacity(inner.len());
    let mut chars = inner.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some(e @ ('"' | '\\')) => out.push(e),
                _ => {
                    return Err("invalid escape: only \\\" and \\\\ are allowed"
                        .to_string());
                }
            }
        } else {
            out.push(c);
        }
    }
    Ok(out)
}

fn needs_quotes(s: &str, in_list: bool) -> bool {
    s.is_empty()
        || s.trim() != s
        || s.starts_with(['[', '"', '#'])
        || s.contains(" #")
        || s.contains(['"', '\\'])
        || (in_list && s.contains([',', '[', ']']))
}

fn quote(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

fn render_scalar(s: &str) -> String {
    if needs_quotes(s, false) {
        quote(s)
    } else {
        s.to_string()
    }
}

fn render_list(items: &[String]) -> String {
    let rendered: Vec<String> = items
        .iter()
        .map(|s| {
            if needs_quotes(s, true) {
                quote(s)
            } else {
                s.clone()
            }
        })
        .collect();
    format!("[{}]", rendered.join(", "))
}

#[cfg(test)]
mod tests {
    use super::*;

    const CANONICAL: &str = "\
id: 0007-status-freshness
status: in-progress
stage: verify
scope: [src/status.rs, tests/cli.rs]
created: 2026-10-02
source: https://github.com/oiwn/specdev/issues/12
depends: [0005-scan-groups]
attempts: 1
blocked_reason: \"waiting on #12, see notes\"
owner: alex
labels: [cli, \"a, b\"]
";

    #[test]
    fn canonical_round_trip_is_exact() {
        let fm = Frontmatter::parse(CANONICAL).unwrap();
        assert_eq!(fm.render(), CANONICAL);
    }

    #[test]
    fn parses_typed_fields_and_extra() {
        let fm = Frontmatter::parse(CANONICAL).unwrap();
        assert_eq!(fm.id.to_string(), "0007-status-freshness");
        assert_eq!(fm.status, Status::InProgress);
        assert_eq!(fm.stage, Some(Stage::Verify));
        assert_eq!(fm.scope, vec!["src/status.rs", "tests/cli.rs"]);
        assert_eq!(fm.attempts, 1);
        assert_eq!(
            fm.blocked_reason.as_deref(),
            Some("waiting on #12, see notes")
        );
        assert_eq!(
            fm.extra,
            vec![
                ("owner".to_string(), Value::Scalar("alex".to_string())),
                (
                    "labels".to_string(),
                    Value::List(vec!["cli".to_string(), "a, b".to_string()])
                ),
            ]
        );
    }

    #[test]
    fn comments_and_blank_lines_are_dropped_on_render() {
        let input = "\
# a comment
id: 0001-x    # trailing comment

status: draft
scope: []
created: 2026-10-03
source:
";
        let fm = Frontmatter::parse(input).unwrap();
        assert_eq!(fm.source, None);
        assert_eq!(
            fm.render(),
            "id: 0001-x\nstatus: draft\nscope: []\ncreated: 2026-10-03\n"
        );
    }

    #[test]
    fn errors_carry_line_numbers() {
        let missing =
            Frontmatter::parse("status: draft\nscope: []\ncreated: 2026-10-03\n");
        assert_eq!(
            missing.unwrap_err(),
            (0, "missing required field `id`".to_string())
        );

        let bad_status = Frontmatter::parse(
            "id: 0001-x\nstatus: doing\nscope: []\ncreated: 2026-10-03\n",
        );
        assert_eq!(bad_status.unwrap_err().0, 2);

        let not_list = Frontmatter::parse(
            "id: 0001-x\nstatus: draft\nscope: a.rs\ncreated: 2026-10-03\n",
        );
        assert_eq!(not_list.unwrap_err().0, 3);

        let bad_date = Frontmatter::parse(
            "id: 0001-x\nstatus: draft\nscope: []\ncreated: 03.10.2026\n",
        );
        assert_eq!(bad_date.unwrap_err().0, 4);

        let dup = Frontmatter::parse("id: 0001-x\nid: 0002-y\n");
        assert_eq!(dup.unwrap_err(), (2, "duplicate key `id`".to_string()));

        let no_colon = Frontmatter::parse("id 0001-x\n");
        assert_eq!(no_colon.unwrap_err().0, 1);
    }

    #[test]
    fn value_syntax() {
        assert_eq!(parse_value("[]"), Ok(Value::List(vec![])));
        assert_eq!(
            parse_value("[a, \"b, c\"]"),
            Ok(Value::List(vec!["a".to_string(), "b, c".to_string()]))
        );
        assert!(parse_value("[a, b").is_err());
        assert!(parse_value("[a, , b]").is_err());
        assert_eq!(
            parse_value("\"say \\\"hi\\\"\""),
            Ok(Value::Scalar("say \"hi\"".to_string()))
        );
        assert!(parse_value("\"open").is_err());
        assert_eq!(strip_comment("url#frag # note"), "url#frag ");
        assert_eq!(strip_comment("\"a # b\" # c"), "\"a # b\" ");
    }

    #[test]
    fn quoting_round_trips() {
        for s in [
            "",
            " padded",
            "#hash",
            "a # b",
            "say \"hi\"",
            "back\\slash",
            "plain-text",
        ] {
            let rendered = render_scalar(s);
            assert_eq!(
                parse_value(&rendered),
                Ok(Value::Scalar(s.to_string())),
                "{rendered}"
            );
        }
        let items = vec!["a,b".to_string(), "[x]".to_string(), "c".to_string()];
        assert_eq!(parse_value(&render_list(&items)), Ok(Value::List(items)));
    }
}
