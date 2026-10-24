use std::cmp::Ordering;
use std::fs;
use std::path::{Path, PathBuf};

use crate::Error;
use crate::diag::Diagnostic;

const SKILL_MD: &str = include_str!("../skills/SKILL.md");
const EXAMPLES_MD: &str = include_str!("../references/examples.md");
const CONFIG_MD: &str = include_str!("../references/config.md");
const META_FILENAME: &str = "specdev.meta";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Every file the skill ships besides `SKILL.md`, as `(file name under
/// `references/`, bundled content)`. A new reference drops in here; both
/// `install` and `check` pick it up.
const REFERENCES: &[(&str, &str)] =
    &[("examples.md", EXAMPLES_MD), ("config.md", CONFIG_MD)];

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
    for (name, content) in REFERENCES {
        fs::write(refs_dir.join(name), content)?;
    }
    write_manifest(target)?;

    println!("Installed specdev skill to {}", target.display());
    Ok(())
}

pub fn check() -> Result<(), Error> {
    let global = home_dir().join(".agents/skills/specdev");
    let local = PathBuf::from(".agents/skills/specdev");

    for (label, target) in
        [("Global", global.as_path()), ("Local", local.as_path())]
    {
        let display = format_target(label, target);
        let report = check_at(target, CURRENT_VERSION)?;
        if !report.installed {
            println!("{display}:\n  not installed");
            continue;
        }
        println!("{display}:");
        for f in &report.files {
            println!("  {:<24} {}", f.path, render_state(f, CURRENT_VERSION));
        }
    }
    Ok(())
}

/// Per-file classification of one install target.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum FileState {
    UpToDate,
    Stale,
    LocallyModified,
    NewerThanCurrent,
    UnknownOrigin,
    Missing,
}

#[derive(Debug)]
pub struct FileReport {
    pub path: String,
    pub state: FileState,
    pub installed_by: Option<String>,
}

#[derive(Debug)]
pub struct TargetReport {
    pub installed: bool,
    pub files: Vec<FileReport>,
}

fn check_at(target: &Path, current_version: &str) -> Result<TargetReport, Error> {
    let manifest_version = read_manifest_version(target);
    let installed = target.join("SKILL.md").exists();

    // Install paths carry the `references/` prefix.
    let entries: Vec<(String, &str)> =
        std::iter::once(("SKILL.md".to_string(), SKILL_MD))
            .chain(
                REFERENCES.iter().map(|(name, content)| {
                    (format!("references/{name}"), *content)
                }),
            )
            .collect();
    let mut files = Vec::new();
    for (rel, bundled) in &entries {
        let path = target.join(rel);
        let state = if !path.exists() {
            FileState::Missing
        } else {
            let installed_content = fs::read_to_string(&path)?;
            if installed_content == *bundled {
                FileState::UpToDate
            } else {
                match &manifest_version {
                    None => FileState::UnknownOrigin,
                    Some(v) => match cmp_version(v, current_version) {
                        Ordering::Less => FileState::Stale,
                        Ordering::Equal => FileState::LocallyModified,
                        Ordering::Greater => FileState::NewerThanCurrent,
                    },
                }
            }
        };
        files.push(FileReport {
            path: rel.to_string(),
            state,
            installed_by: manifest_version.clone(),
        });
    }

    Ok(TargetReport { installed, files })
}

fn write_manifest(target: &Path) -> Result<(), Error> {
    let content = format!("specdev_version={CURRENT_VERSION}\n");
    fs::write(target.join(META_FILENAME), content)?;
    Ok(())
}

fn read_manifest_version(target: &Path) -> Option<String> {
    let content = fs::read_to_string(target.join(META_FILENAME)).ok()?;
    content
        .lines()
        .map(str::trim)
        .filter_map(|line| line.strip_prefix("specdev_version="))
        .map(str::trim)
        .find(|v| !v.is_empty())
        .map(str::to_string)
}

/// Compare two `X.Y.Z` (numeric) version strings component-wise, padding missing
/// components with 0. Non-numeric components are dropped.
fn cmp_version(a: &str, b: &str) -> Ordering {
    let pa = parse_parts(a);
    let pb = parse_parts(b);
    let len = pa.len().max(pb.len());
    for i in 0..len {
        let x = pa.get(i).copied().unwrap_or(0);
        let y = pb.get(i).copied().unwrap_or(0);
        match x.cmp(&y) {
            Ordering::Equal => continue,
            ord => return ord,
        }
    }
    Ordering::Equal
}

fn parse_parts(s: &str) -> Vec<u64> {
    s.split('.').filter_map(|p| p.parse::<u64>().ok()).collect()
}

fn format_target(label: &str, target: &Path) -> String {
    match label {
        "Global" => {
            let home = home_dir();
            let rest = target.strip_prefix(&home).unwrap_or(target);
            format!("Global (~/{})", rest.display())
        }
        _ => format!("Local ({})", target.display()),
    }
}

fn render_state(f: &FileReport, current: &str) -> String {
    match f.state {
        FileState::UpToDate => match &f.installed_by {
            Some(v) => format!("up to date (installed by specdev {v})"),
            None => "up to date".to_string(),
        },
        FileState::Stale => format!(
            "STALE — installed by specdev {}, current is {}. Reinstall: `specdev skill install`.",
            f.installed_by.as_deref().unwrap_or("?"),
            current
        ),
        FileState::LocallyModified => {
            "LOCALLY MODIFIED — differs from bundled. Reinstall will overwrite."
                .to_string()
        }
        FileState::NewerThanCurrent => format!(
            "differs — installed by newer specdev {}, current is {}.",
            f.installed_by.as_deref().unwrap_or("?"),
            current
        ),
        FileState::UnknownOrigin => {
            "UNKNOWN ORIGIN (no version stamp). Reinstall: `specdev skill install`."
                .to_string()
        }
        FileState::Missing => "missing".to_string(),
    }
}

fn home_dir() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| Path::new(".").to_path_buf())
}

/// `skill-stale` warnings for a drifted local install (`.agents/skills/
/// specdev/`), surfaced by `check` and `status` through
/// `status::spec_diagnostics`. No local install, or an unreadable one,
/// stays silent: the drift only matters where agents load the skill.
pub fn local_drift(root: &Path) -> Vec<Diagnostic> {
    let target = root.join(".agents/skills/specdev");
    if !target.join("SKILL.md").exists() {
        return Vec::new();
    }
    let Ok(report) = check_at(&target, CURRENT_VERSION) else {
        return Vec::new();
    };
    let reinstall = "run `specdev skill install --local`";
    report
        .files
        .iter()
        .filter(|f| f.state != FileState::UpToDate)
        .map(|f| {
            let by = f.installed_by.as_deref().unwrap_or("?");
            let what = match f.state {
                FileState::Stale => format!(
                    "is stale (installed by specdev {by}, current is {CURRENT_VERSION}); {reinstall}"
                ),
                FileState::LocallyModified => {
                    format!("differs from the bundled skill; {reinstall} to overwrite")
                }
                FileState::NewerThanCurrent => format!(
                    "was installed by a newer specdev ({by}); reinstall the matching version after upgrading"
                ),
                FileState::UnknownOrigin => {
                    format!("has no version stamp; {reinstall}")
                }
                FileState::Missing => format!("is missing from the local skill; {reinstall}"),
                FileState::UpToDate => unreachable!("filtered above"),
            };
            Diagnostic::warning(
                target.join(&f.path),
                None,
                "skill-stale",
                format!("local skill file {what}"),
            )
        })
        .collect()
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
    fn installs_every_bundled_reference() {
        let tmp = TempDir::new().unwrap();
        let target = tmp.path().join("skills/specdev");

        install_to(&target).unwrap();

        // Whatever joins `REFERENCES` ships; the list and the bundle agree.
        for (name, bundled) in REFERENCES {
            let path = target.join("references").join(name);
            let content = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("{name}: {e}"));
            assert_eq!(&content, bundled, "{name} installed verbatim");
        }
        let config =
            std::fs::read_to_string(target.join("references").join("config.md"))
                .unwrap();
        assert!(config.contains("[quality.grade."), "got {config:?}");
    }

    #[test]
    fn check_reports_a_missing_reference() {
        let tmp = TempDir::new().unwrap();
        let target = tmp.path().join("skills/specdev");

        install_to(&target).unwrap();
        std::fs::remove_file(target.join("references/config.md")).unwrap();

        let report = check_at(&target, CURRENT_VERSION).unwrap();
        let config = report
            .files
            .iter()
            .find(|f| f.path == "references/config.md")
            .unwrap();
        assert_eq!(config.state, FileState::Missing);
        let examples = report
            .files
            .iter()
            .find(|f| f.path == "references/examples.md")
            .unwrap();
        assert_eq!(examples.state, FileState::UpToDate);
    }

    #[test]
    fn local_drift_reports_only_real_drift() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        // No local install: silent.
        assert!(local_drift(root).is_empty());

        // Fresh install: silent — content is what matters, so an old stamp
        // alone (identical files) is not drift either.
        let target = root.join(".agents/skills/specdev");
        install_to(&target).unwrap();
        assert!(local_drift(root).is_empty());
        std::fs::write(target.join(META_FILENAME), "specdev_version=0.0.1\n")
            .unwrap();
        assert!(
            local_drift(root).is_empty(),
            "identical content with an old stamp is fine"
        );

        // Content differs and the stamp is older: stale, with the remedy.
        std::fs::write(target.join("SKILL.md"), "tampered\n").unwrap();
        let diags = local_drift(root);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, "skill-stale");
        assert!(
            diags[0].file.ends_with("SKILL.md"),
            "got {}",
            diags[0].file.display()
        );
        assert!(
            diags[0].message.contains("is stale"),
            "got {}",
            diags[0].message
        );
        assert!(
            diags[0].message.contains("specdev skill install --local"),
            "got {}",
            diags[0].message
        );

        // Tampered content with the same version: locally modified.
        std::fs::write(
            target.join(META_FILENAME),
            format!("specdev_version={CURRENT_VERSION}\n"),
        )
        .unwrap();
        std::fs::write(target.join("SKILL.md"), "tampered\n").unwrap();
        let diags = local_drift(root);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("differs from the bundled skill"));

        // Restored content: silent again; a missing file says so.
        std::fs::write(target.join("SKILL.md"), SKILL_MD).unwrap();
        assert!(local_drift(root).is_empty());
        std::fs::remove_file(target.join("references/config.md")).unwrap();
        let diags = local_drift(root);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("is missing"));
    }

    #[test]
    fn install_writes_manifest_with_version() {
        let tmp = TempDir::new().unwrap();
        let target = tmp.path().join("skills/specdev");

        install_to(&target).unwrap();

        let meta = std::fs::read_to_string(target.join(META_FILENAME)).unwrap();
        assert!(meta.contains("specdev_version="));
        assert_eq!(
            read_manifest_version(&target).as_deref(),
            Some(CURRENT_VERSION)
        );
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

    #[test]
    fn check_up_to_date_after_install() {
        let tmp = TempDir::new().unwrap();
        let target = tmp.path().join("skills/specdev");
        install_to(&target).unwrap();

        let report = check_at(&target, CURRENT_VERSION).unwrap();
        assert!(report.installed);
        for f in &report.files {
            assert_eq!(f.state, FileState::UpToDate, "{}: {:?}", f.path, f.state);
        }
    }

    #[test]
    fn check_stale() {
        let tmp = TempDir::new().unwrap();
        let target = tmp.path().join("skills/specdev");
        install_to(&target).unwrap();
        // simulate an older install: older content + older version stamp
        std::fs::write(target.join("SKILL.md"), "# old skill\n").unwrap();
        std::fs::write(target.join(META_FILENAME), "specdev_version=0.1.0\n")
            .unwrap();

        let report = check_at(&target, CURRENT_VERSION).unwrap();
        let skill = report.files.iter().find(|f| f.path == "SKILL.md").unwrap();
        assert_eq!(skill.state, FileState::Stale);
    }

    #[test]
    fn check_locally_modified() {
        let tmp = TempDir::new().unwrap();
        let target = tmp.path().join("skills/specdev");
        install_to(&target).unwrap();
        std::fs::write(target.join("SKILL.md"), "# tampered\n").unwrap();
        // manifest still carries CURRENT_VERSION

        let report = check_at(&target, CURRENT_VERSION).unwrap();
        let skill = report.files.iter().find(|f| f.path == "SKILL.md").unwrap();
        assert_eq!(skill.state, FileState::LocallyModified);
        // the untouched file is still up to date
        let ex = report
            .files
            .iter()
            .find(|f| f.path == "references/examples.md")
            .unwrap();
        assert_eq!(ex.state, FileState::UpToDate);
    }

    #[test]
    fn check_unknown_origin_without_manifest() {
        let tmp = TempDir::new().unwrap();
        let target = tmp.path().join("skills/specdev");
        install_to(&target).unwrap();
        std::fs::remove_file(target.join(META_FILENAME)).unwrap();
        std::fs::write(target.join("SKILL.md"), "# tampered\n").unwrap();

        let report = check_at(&target, CURRENT_VERSION).unwrap();
        let skill = report.files.iter().find(|f| f.path == "SKILL.md").unwrap();
        assert_eq!(skill.state, FileState::UnknownOrigin);
    }

    #[test]
    fn check_not_installed() {
        let tmp = TempDir::new().unwrap();
        let target = tmp.path().join("skills/specdev");
        // nothing installed

        let report = check_at(&target, CURRENT_VERSION).unwrap();
        assert!(!report.installed);
        for f in &report.files {
            assert_eq!(f.state, FileState::Missing, "{}: {:?}", f.path, f.state);
        }
    }

    #[test]
    fn cmp_version_orders_correctly() {
        assert_eq!(cmp_version("0.1.0", "0.2.0"), Ordering::Less);
        assert_eq!(cmp_version("0.2.0", "0.2.0"), Ordering::Equal);
        assert_eq!(cmp_version("0.3.0", "0.2.0"), Ordering::Greater);
        assert_eq!(cmp_version("1.0.0", "0.99.99"), Ordering::Greater);
        assert_eq!(cmp_version("0.2", "0.2.0"), Ordering::Equal); // missing parts padded
        assert_eq!(cmp_version("0.10.0", "0.9.0"), Ordering::Greater); // numeric, not lexical
    }
}
