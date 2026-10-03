//! Task ids: a zero-padded sequence plus a slug, e.g. `0007-status-freshness`.

use std::fmt;
use std::str::FromStr;

use serde::{Serialize, Serializer};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TaskId {
    pub seq: u32,
    pub slug: String,
}

impl TaskId {
    pub fn new(seq: u32, slug: &str) -> Result<Self, String> {
        validate_slug(slug)?;
        Ok(Self {
            seq,
            slug: slug.to_string(),
        })
    }
}

/// Slugs are lowercase ASCII words joined by single dashes.
pub fn validate_slug(slug: &str) -> Result<(), String> {
    let valid = !slug.is_empty()
        && slug
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !slug.starts_with('-')
        && !slug.ends_with('-')
        && !slug.contains("--");
    if valid {
        Ok(())
    } else {
        Err(format!(
            "invalid slug `{slug}`: use lowercase letters, digits, and single dashes"
        ))
    }
}

impl FromStr for TaskId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, String> {
        let invalid = || format!("invalid task id `{s}`: expected NNNN-slug");
        let (num, slug) = s.split_once('-').ok_or_else(invalid)?;
        if num.len() < 4 || !num.chars().all(|c| c.is_ascii_digit()) {
            return Err(invalid());
        }
        let seq = num.parse().map_err(|_| invalid())?;
        Self::new(seq, slug).map_err(|e| format!("invalid task id `{s}`: {e}"))
    }
}

impl fmt::Display for TaskId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{}", self.seq, self.slug)
    }
}

impl Serialize for TaskId {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.collect_str(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_and_display_round_trip() {
        let id: TaskId = "0007-status-freshness".parse().unwrap();
        assert_eq!(id.seq, 7);
        assert_eq!(id.slug, "status-freshness");
        assert_eq!(id.to_string(), "0007-status-freshness");
    }

    #[test]
    fn display_pads_to_four_digits_and_grows() {
        assert_eq!(TaskId::new(3, "x").unwrap().to_string(), "0003-x");
        assert_eq!(TaskId::new(12345, "x").unwrap().to_string(), "12345-x");
    }

    #[test]
    fn rejects_bad_ids() {
        for bad in [
            "7-x",
            "0007",
            "000a-x",
            "0007-",
            "0007-Bad",
            "0007-a--b",
            "0007--a",
        ] {
            assert!(bad.parse::<TaskId>().is_err(), "{bad} should be rejected");
        }
    }

    #[test]
    fn slug_rules() {
        assert!(validate_slug("scan-groups-2").is_ok());
        for bad in ["", "-a", "a-", "a--b", "A", "a_b", "a b"] {
            assert!(validate_slug(bad).is_err(), "{bad:?} should be rejected");
        }
    }
}
