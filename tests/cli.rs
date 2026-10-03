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

    assert!(
        tmp.path().join("specdev.toml").exists(),
        "missing specdev.toml"
    );
    assert!(
        specs.join("tasks").join("done").is_dir(),
        "missing specs/tasks/done/"
    );
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
    assert!(out.contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn unknown_command_exits_nonzero() {
    let tmp = TempDir::new().unwrap();
    let (_out, err, code) = run(tmp.path(), &["frobnicate"]);
    assert_ne!(code, 0);
    assert!(!err.is_empty());
}

#[test]
fn task_new_creates_numbered_files_and_index() {
    let tmp = init_tmp();
    let tasks = tmp.path().join("specs").join("tasks");

    let (out, err, code) = run(tmp.path(), &["task", "new", "status-freshness"]);
    assert_eq!(code, 0, "stderr: {err}");
    assert!(
        out.contains("specs/tasks/0001-status-freshness.md"),
        "got {out}"
    );
    let (_, _, code) = run(tmp.path(), &["task", "new", "scan-groups"]);
    assert_eq!(code, 0);

    let first = fs::read_to_string(tasks.join("0001-status-freshness.md")).unwrap();
    assert!(
        first.starts_with(
            "---\nid: 0001-status-freshness\nstatus: draft\nscope: []\n"
        )
    );
    assert!(first.contains(
        "# Task: status freshness\n\n## Plan\n\n## Acceptance\n\n## Log\n\n- "
    ));
    assert!(first.ends_with(" created: status freshness\n"));
    assert!(tasks.join("0002-scan-groups.md").exists());

    let index = fs::read_to_string(tasks.join("_index.md")).unwrap();
    assert!(index.contains("## Draft"));
    assert!(
        index.contains("- **0001-status-freshness** — draft — status freshness")
    );
    assert!(index.contains("- **0002-scan-groups** — draft — scan groups"));
}

#[test]
fn task_list_and_show() {
    let tmp = init_tmp();
    run(tmp.path(), &["task", "new", "status-freshness"]);
    run(tmp.path(), &["task", "new", "scan-groups"]);

    let (out, _, code) = run(tmp.path(), &["task", "list"]);
    assert_eq!(code, 0);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), 2, "got {out}");
    assert!(
        lines[0].starts_with("0001-status-freshness") && lines[0].contains("draft")
    );
    assert!(lines[1].starts_with("0002-scan-groups"));

    let (out, _, code) = run(tmp.path(), &["task", "show", "1"]);
    assert_eq!(code, 0);
    assert!(
        out.starts_with("0001-status-freshness — draft"),
        "got {out}"
    );
    assert!(out.contains("Plan:     0/0 steps done"));
    assert!(out.contains("Next:     Fill Plan, scope, and Acceptance"));
}

#[test]
fn task_output_formats() {
    let tmp = init_tmp();
    run(tmp.path(), &["task", "new", "status-freshness"]);

    let (out, _, code) = run(
        tmp.path(),
        &["task", "show", "status-freshness", "--format", "json"],
    );
    assert_eq!(code, 0);
    let json: serde_json::Value = serde_json::from_str(&out).expect("valid JSON");
    assert_eq!(json["id"], "0001-status-freshness");
    assert_eq!(json["status"], "draft");
    assert_eq!(json["plan"]["total"], 0);

    let (out, _, code) = run(tmp.path(), &["--format", "json", "task", "list"]);
    assert_eq!(code, 0);
    let json: serde_json::Value = serde_json::from_str(&out).expect("valid JSON");
    assert_eq!(json["tasks"][0]["title"], "status freshness");

    let (out, _, code) = run(tmp.path(), &["--format", "toon", "task", "list"]);
    assert_eq!(code, 0);
    assert!(out.contains("status freshness"), "got {out}");

    let (out, _, code) = run(tmp.path(), &["list", "--stats", "--format", "json"]);
    assert_eq!(code, 0);
    let json: serde_json::Value = serde_json::from_str(&out).expect("valid JSON");
    assert_eq!(json["files"][0]["file"], "overview.md");
}

#[test]
fn unmigrated_commands_reject_structured_formats() {
    let tmp = init_tmp();
    let (_, err, code) = run(tmp.path(), &["scan", "--format", "json"]);
    assert_eq!(code, 1);
    assert!(err.contains("does not support --format"), "got {err}");
}

#[test]
fn task_errors_exit_nonzero() {
    let tmp = init_tmp();
    let (_, err, code) = run(tmp.path(), &["task", "new", "Bad_Slug"]);
    assert_eq!(code, 1);
    assert!(err.contains("invalid slug"), "got {err}");

    let (_, err, code) = run(tmp.path(), &["task", "show", "42"]);
    assert_eq!(code, 1);
    assert!(err.contains("no task matching"), "got {err}");

    let bare = TempDir::new().unwrap();
    let (_, err, code) = run(bare.path(), &["task", "list"]);
    assert_eq!(code, 1);
    assert!(err.contains("specdev init"), "got {err}");
}

#[test]
fn broken_task_file_is_a_warning_not_a_failure() {
    let tmp = init_tmp();
    let tasks = tmp.path().join("specs").join("tasks");
    fs::write(tasks.join("0005-broken.md"), "no frontmatter\n").unwrap();

    let (_, err, code) = run(tmp.path(), &["task", "new", "next-one"]);
    assert_eq!(code, 0);
    assert!(
        err.contains("0005-broken.md:1: warning[frontmatter]"),
        "got {err}"
    );
    assert!(
        tasks.join("0006-next-one.md").exists(),
        "number 5 stays reserved"
    );
}

#[test]
fn scan_without_specs_dir_is_ok() {
    let tmp = TempDir::new().unwrap();
    let (_out, err, code) = run(tmp.path(), &["scan"]);
    assert_eq!(code, 0);
    assert!(err.contains("No specs/ directory"));
}
