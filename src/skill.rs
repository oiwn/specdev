use std::fs;
use std::path::{Path, PathBuf};

use crate::Error;

const SKILL_MD: &str = include_str!("../skills/SKILL.md");
const EXAMPLES_MD: &str = include_str!("../references/examples.md");

pub fn install(local: bool) -> Result<(), Error> {
    let base = if local {
        PathBuf::from(".agents/skills/specdev")
    } else {
        home_dir().join(".agents/skills/specdev")
    };

    install_to(&base)
}

pub fn install_to(target: &Path) -> Result<(), Error> {
    let refs_dir = target.join("references");
    fs::create_dir_all(&refs_dir)?;

    fs::write(target.join("SKILL.md"), SKILL_MD)?;
    fs::write(refs_dir.join("examples.md"), EXAMPLES_MD)?;

    println!("Installed specdev skill to {}", target.display());
    Ok(())
}

fn home_dir() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| Path::new(".").to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn creates_skill_file() {
        let tmp = TempDir::new().unwrap();
        let target = tmp.path().join("skills/specdev");

        install_to(&target).unwrap();

        let skill = target.join("SKILL.md");
        assert!(skill.exists());
        let content = std::fs::read_to_string(skill).unwrap();
        assert!(content.contains("name: specdev"));
        assert!(content.contains("^^^"));
        assert!(content.contains("&&&"));
    }

    #[test]
    fn creates_references_dir_with_examples() {
        let tmp = TempDir::new().unwrap();
        let target = tmp.path().join("skills/specdev");

        install_to(&target).unwrap();

        let examples = target.join("references/examples.md");
        assert!(examples.exists());
        let content = std::fs::read_to_string(examples).unwrap();
        assert!(content.contains("Example 1"));
        assert!(content.contains("Compression"));
    }

    #[test]
    fn overwrites_on_reinstall() {
        let tmp = TempDir::new().unwrap();
        let target = tmp.path().join("skills/specdev");

        install_to(&target).unwrap();

        let skill = target.join("SKILL.md");
        let original = std::fs::read_to_string(&skill).unwrap();
        std::fs::write(&skill, "tampered").unwrap();

        install_to(&target).unwrap();

        let reinstalled = std::fs::read_to_string(&skill).unwrap();
        assert_eq!(original, reinstalled);
        assert_ne!(reinstalled, "tampered");
    }

    #[test]
    fn skill_content_is_not_empty() {
        assert!(!SKILL_MD.is_empty());
        assert!(!EXAMPLES_MD.is_empty());
    }
}
