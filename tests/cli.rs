use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use tempfile::TempDir;

const BIN: &str = env!("CARGO_BIN_EXE_specdev");

/// Run the specdev binary in `cwd` with `args`, returning (stdout, stderr, exit_code).
fn run(cwd: &Path, args: &[&str]) -> (String, String, i32) {
    let output = Command::new(BIN)
        .args(args)
        .current_dir(cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("failed to spawn specdev");
    (
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
        output.status.code().unwrap_or(-1),
    )
}

/// Create a tempdir and run `specdev init` in it.
fn init_tmp() -> TempDir {
    let tmp = TempDir::new().unwrap();
    let (_, _, code) = run(tmp.path(), &["init"]);
    assert_eq!(code, 0, "`init` should exit 0");
    tmp
}

#[test]
fn init_creates_all_files() {
    let tmp = init_tmp();
    let specs = tmp.path().join("specs");
    for name in [
        "overview.md",
        "ctx.md",
        "roadmap.md",
        "ideas.md",
        "cleanup.md",
    ] {
        assert!(specs.join(name).exists(), "missing specs/{name}");
    }
    assert!(
        tmp.path().join("CHANGELOG.md").exists(),
        "missing CHANGELOG.md"
    );
    assert!(tmp.path().join("AGENTS.md").exists(), "missing AGENTS.md");

    let ctx = fs::read_to_string(specs.join("ctx.md")).unwrap();
    assert_eq!(ctx, "# Current Task Context\n");
}

#[test]
fn init_is_idempotent_no_overwrite() {
    let tmp = TempDir::new().unwrap();
    let specs = tmp.path().join("specs");
    fs::create_dir_all(&specs).unwrap();
    fs::write(specs.join("overview.md"), "KEEP ME").unwrap();

    let (_, _, code) = run(tmp.path(), &["init"]);
    assert_eq!(code, 0);

    assert_eq!(
        fs::read_to_string(specs.join("overview.md")).unwrap(),
        "KEEP ME"
    );
    // second run also clean
    let (_, _, code) = run(tmp.path(), &["init"]);
    assert_eq!(code, 0);
}

#[test]
fn scan_runs_in_initialized_dir() {
    let tmp = init_tmp();
    let (out, _err, code) = run(tmp.path(), &["scan"]);
    assert_eq!(code, 0);
    assert!(out.contains("Total:"), "scan should print a total line");
}

#[test]
fn status_runs_in_initialized_dir() {
    let tmp = init_tmp();
    let (out, _err, code) = run(tmp.path(), &["status"]);
    assert_eq!(code, 0);
    assert!(out.contains("Core spec files"));
}

#[test]
fn list_shows_descriptions_and_lines() {
    let tmp = init_tmp();
    let (out, _err, code) = run(tmp.path(), &["list"]);
    assert_eq!(code, 0);
    assert!(out.contains("File"));
    assert!(out.contains("Description"));
    assert!(out.contains("Lines"));
    assert!(out.contains("overview.md"));
    assert!(out.contains("Project Overview"));
    assert!(out.contains("ctx.md"));
    assert!(out.contains("Current Task Context"));
}

#[test]
fn list_stats_shows_breakdown_columns() {
    let tmp = init_tmp();
    let (out, _err, code) = run(tmp.path(), &["list", "--stats"]);
    assert_eq!(code, 0);
    for col in ["H1", "H2", "H3", "H4+", "[x]", "[ ]", "^^^", "words"] {
        assert!(out.contains(col), "stats header missing {col}");
    }
    assert!(out.contains("overview.md"));
}

#[test]
fn help_and_version_exit_zero() {
    let tmp = TempDir::new().unwrap();
    let (out, _err, code) = run(tmp.path(), &["--help"]);
    assert_eq!(code, 0);
    assert!(out.contains("specdev"));

    let (out, _err, code) = run(tmp.path(), &["--version"]);
    assert_eq!(code, 0);
    assert!(out.contains("0.1.0"));
}

#[test]
fn unknown_command_exits_nonzero() {
    let tmp = TempDir::new().unwrap();
    let (_out, err, code) = run(tmp.path(), &["frobnicate"]);
    assert_ne!(code, 0);
    assert!(!err.is_empty());
}

#[test]
fn scan_without_specs_dir_is_ok() {
    let tmp = TempDir::new().unwrap();
    let (_out, err, code) = run(tmp.path(), &["scan"]);
    assert_eq!(code, 0);
    assert!(err.contains("No specs/ directory"));
}
