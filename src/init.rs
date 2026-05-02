use std::fs;
use std::path::Path;

use crate::Error;

const OVERVIEW_TEMPLATE: &str = "\
# Project Overview

## Architecture

## Data Flow
";

const CLEANUP_TEMPLATE: &str = "# Cleanup\n";

pub fn run() -> Result<(), Error> {
    init_at(Path::new("specs"))
}

pub fn init_at(specs_dir: &Path) -> Result<(), Error> {
    if specs_dir.exists() {
        let overview = specs_dir.join("overview.md");
        let ctx = specs_dir.join("ctx.md");
        let ideas = specs_dir.join("ideas.md");
        let cleanup = specs_dir.join("cleanup.md");

        if overview.exists() && ctx.exists() && ideas.exists() && cleanup.exists() {
            println!("specs/ already initialized with all core files.");
            return Ok(());
        }
    } else {
        fs::create_dir(specs_dir)?;
        println!("Created specs/");
    }

    write_if_missing(&specs_dir.join("overview.md"), OVERVIEW_TEMPLATE)?;
    write_if_missing(&specs_dir.join("ctx.md"), "")?;
    write_if_missing(&specs_dir.join("ideas.md"), "# Ideas\n")?;
    write_if_missing(&specs_dir.join("cleanup.md"), CLEANUP_TEMPLATE)?;

    println!("Done.");
    Ok(())
}

fn write_if_missing(path: &Path, content: &str) -> Result<(), Error> {
    if path.exists() {
        println!("  {} already exists, skipping", path.display());
        return Ok(());
    }
    fs::write(path, content)?;
    println!("  Created {}", path.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn creates_all_core_files() {
        let tmp = TempDir::new().unwrap();
        let specs = tmp.path().join("specs");

        init_at(&specs).unwrap();

        assert!(specs.join("overview.md").exists());
        assert!(specs.join("ctx.md").exists());
        assert!(specs.join("ideas.md").exists());
        assert!(specs.join("cleanup.md").exists());
    }

    #[test]
    fn overview_has_template_content() {
        let tmp = TempDir::new().unwrap();
        let specs = tmp.path().join("specs");

        init_at(&specs).unwrap();

        let content = fs::read_to_string(specs.join("overview.md")).unwrap();
        assert!(content.contains("# Project Overview"));
        assert!(content.contains("## Architecture"));
    }

    #[test]
    fn ctx_starts_empty() {
        let tmp = TempDir::new().unwrap();
        let specs = tmp.path().join("specs");

        init_at(&specs).unwrap();

        let content = fs::read_to_string(specs.join("ctx.md")).unwrap();
        assert!(content.is_empty());
    }

    #[test]
    fn ideas_has_header() {
        let tmp = TempDir::new().unwrap();
        let specs = tmp.path().join("specs");

        init_at(&specs).unwrap();

        let content = fs::read_to_string(specs.join("ideas.md")).unwrap();
        assert!(content.contains("# Ideas"));
    }

    #[test]
    fn does_not_overwrite_existing() {
        let tmp = TempDir::new().unwrap();
        let specs = tmp.path().join("specs");
        fs::create_dir_all(&specs).unwrap();
        fs::write(specs.join("overview.md"), "my project").unwrap();
        fs::write(specs.join("ctx.md"), "my context").unwrap();
        fs::write(specs.join("ideas.md"), "my ideas").unwrap();
        fs::write(specs.join("cleanup.md"), "my cleanup").unwrap();

        init_at(&specs).unwrap();

        assert_eq!(
            fs::read_to_string(specs.join("overview.md")).unwrap(),
            "my project"
        );
        assert_eq!(
            fs::read_to_string(specs.join("ctx.md")).unwrap(),
            "my context"
        );
        assert_eq!(
            fs::read_to_string(specs.join("ideas.md")).unwrap(),
            "my ideas"
        );
        assert_eq!(
            fs::read_to_string(specs.join("cleanup.md")).unwrap(),
            "my cleanup"
        );
    }

    #[test]
    fn skips_returning_ok_when_all_exist() {
        let tmp = TempDir::new().unwrap();
        let specs = tmp.path().join("specs");
        fs::create_dir_all(&specs).unwrap();
        fs::write(specs.join("overview.md"), "").unwrap();
        fs::write(specs.join("ctx.md"), "").unwrap();
        fs::write(specs.join("ideas.md"), "").unwrap();
        fs::write(specs.join("cleanup.md"), "").unwrap();

        let result = init_at(&specs);
        assert!(result.is_ok());
    }

    #[test]
    fn fills_missing_files_in_existing_dir() {
        let tmp = TempDir::new().unwrap();
        let specs = tmp.path().join("specs");
        fs::create_dir_all(&specs).unwrap();
        fs::write(specs.join("overview.md"), "existing").unwrap();

        init_at(&specs).unwrap();

        assert_eq!(
            fs::read_to_string(specs.join("overview.md")).unwrap(),
            "existing"
        );
        assert!(specs.join("ctx.md").exists());
        assert!(specs.join("ideas.md").exists());
        assert!(specs.join("cleanup.md").exists());
    }

    #[test]
    fn cleanup_has_header() {
        let tmp = TempDir::new().unwrap();
        let specs = tmp.path().join("specs");

        init_at(&specs).unwrap();

        let content = fs::read_to_string(specs.join("cleanup.md")).unwrap();
        assert!(content.contains("# Cleanup"));
    }
}
