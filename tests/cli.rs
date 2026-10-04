use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use tempfile::TempDir;

const BIN: &str = env!("CARGO_BIN_EXE_specdev");

/// Run the specdev binary in `cwd` with `args`, returning (stdout, stderr, exit_code).
fn run(cwd: &Path, args: &[&str]) -> (String, String, i32) {
    run_env(cwd, args, &[])
}

/// Like `run`, with extra environment variables (e.g. `HOME` for skill tests).
fn run_env(
    cwd: &Path,
    args: &[&str],
    env: &[(&str, &Path)],
) -> (String, String, i32) {
    let output = Command::new(BIN)
        .args(args)
        .envs(env.iter().copied())
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
    let (_, err, code) = run(tmp.path(), &["init", "--format", "json"]);
    assert_eq!(code, 1);
    assert!(err.contains("does not support --format"), "got {err}");
}

#[test]
fn scan_and_status_support_structured_formats() {
    let tmp = init_tmp();
    fs::write(tmp.path().join("specs/ideas.md"), "^^^ open\n").unwrap();

    let (out, err, code) = run(tmp.path(), &["--format", "json", "scan"]);
    assert_eq!(code, 0, "stderr: {err}");
    let json: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(json["open"], 1);
    assert!(
        json["files"]
            .as_array()
            .unwrap()
            .iter()
            .any(|f| f["path"] == "ideas.md")
    );

    let (out, err, code) = run(tmp.path(), &["--format", "json", "status"]);
    assert_eq!(code, 0, "stderr: {err}");
    let json: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(json["open"], 1);
    assert_eq!(json["core"][0]["name"], "overview.md");
    assert!(json["warnings"].is_array());

    let (out, _, code) = run(tmp.path(), &["--format", "toon", "scan"]);
    assert_eq!(code, 0);
    assert!(out.contains("open: 1"), "got {out}");
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

#[test]
fn status_without_specs_dir_is_ok() {
    let tmp = TempDir::new().unwrap();
    let (_out, err, code) = run(tmp.path(), &["status"]);
    assert_eq!(code, 0);
    assert!(err.contains("No specs/ directory found"), "got {err}");
}

#[test]
fn task_index_regenerates_index() {
    let tmp = init_tmp();
    let tasks = tmp.path().join("specs").join("tasks");
    let index = tasks.join("_index.md");

    let (_, err, code) = run(tmp.path(), &["task", "index"]);
    assert_eq!(code, 0, "stderr: {err}");
    assert!(
        fs::read_to_string(&index)
            .unwrap()
            .contains("No tasks yet.")
    );

    run(tmp.path(), &["task", "new", "a-one"]);
    run(tmp.path(), &["task", "new", "b-two"]);
    fs::remove_file(&index).unwrap();
    let (out, err, code) = run(tmp.path(), &["task", "index"]);
    assert_eq!(code, 0, "stderr: {err}");
    assert!(out.contains("specs/tasks/_index.md (2 tasks)"), "got {out}");
    let text = fs::read_to_string(&index).unwrap();
    assert!(text.contains("**0001-a-one**"), "got {text}");
    assert!(text.contains("**0002-b-two**"), "got {text}");

    fs::remove_file(tasks.join("0002-b-two.md")).unwrap();
    fs::write(&index, "junk\n").unwrap();
    let (_, _, code) = run(tmp.path(), &["task", "index"]);
    assert_eq!(code, 0);
    let text = fs::read_to_string(&index).unwrap();
    assert!(text.starts_with("<!-- generated by specdev; do not edit -->\n"));
    assert!(!text.contains("junk"));
    assert!(text.contains("**0001-a-one**"));
    assert!(!text.contains("b-two"), "got {text}");

    let (out, _, code) = run(tmp.path(), &["--format", "json", "task", "index"]);
    assert_eq!(code, 0);
    assert!(out.contains("\"path\""), "got {out}");
    assert!(out.contains("\"tasks\": 1"), "got {out}");
}

#[test]
fn skill_install_local_and_global() {
    let home = TempDir::new().unwrap();
    let project = TempDir::new().unwrap();
    let env = [("HOME", home.path())];
    let version = env!("CARGO_PKG_VERSION");
    let files = ["SKILL.md", "references/examples.md", "specdev.meta"];

    let (_, err, code) =
        run_env(project.path(), &["skill", "install", "--local"], &env);
    assert_eq!(code, 0, "stderr: {err}");
    let local = project.path().join(".agents/skills/specdev");
    for f in files {
        assert!(local.join(f).exists(), "local {f} missing");
    }
    let meta = fs::read_to_string(local.join("specdev.meta")).unwrap();
    assert!(meta.contains(&format!("specdev_version={version}")));
    assert!(
        !home.path().join(".agents").exists(),
        "--local touched HOME"
    );

    let (_, err, code) = run_env(project.path(), &["skill", "install"], &env);
    assert_eq!(code, 0, "stderr: {err}");
    let global = home.path().join(".agents/skills/specdev");
    for f in files {
        assert!(global.join(f).exists(), "global {f} missing");
    }
}

#[test]
fn skill_check_reports_states() {
    let home = TempDir::new().unwrap();
    let project = TempDir::new().unwrap();
    let env = [("HOME", home.path())];
    let check = |what: &str| {
        let (out, err, code) = run_env(project.path(), &["skill", "check"], &env);
        assert_eq!(code, 0, "stderr: {err}");
        assert!(out.contains(what), "expected {what:?}, got {out}");
        out
    };

    let out = check("Global (~/.agents/skills/specdev):\n  not installed");
    assert!(out.contains("Local (.agents/skills/specdev):\n  not installed"));

    run_env(project.path(), &["skill", "install", "--local"], &env);
    let version = env!("CARGO_PKG_VERSION");
    check(&format!("up to date (installed by specdev {version})"));

    let local = project.path().join(".agents/skills/specdev");
    let skill = local.join("SKILL.md");
    let mut content = fs::read_to_string(&skill).unwrap();
    content.push_str("\nlocal edit\n");
    fs::write(&skill, content).unwrap();
    check("LOCALLY MODIFIED");

    fs::write(local.join("specdev.meta"), "specdev_version=0.0.1\n").unwrap();
    check("STALE");

    fs::remove_file(local.join("specdev.meta")).unwrap();
    check("UNKNOWN ORIGIN");
}

#[test]
fn scan_reports_markers() {
    let tmp = init_tmp();
    let specs = tmp.path().join("specs");
    fs::write(
        specs.join("ideas.md"),
        "# Ideas\n\n^^^ open question\n\n^^^ answered\n&&& yes\n\n```\n^^^ in code\n```\n",
    )
    .unwrap();
    let files = fs::read_dir(&specs)
        .unwrap()
        .filter(|e| {
            let p = e.as_ref().unwrap().path();
            p.is_file() && p.extension().is_some_and(|x| x == "md")
        })
        .count();

    let (out, err, code) = run(tmp.path(), &["scan"]);
    assert_eq!(code, 0, "stderr: {err}");
    assert!(out.contains("[    open]  open question"), "got {out}");
    assert!(out.contains("[resolved]  answered"), "got {out}");
    assert!(!out.contains("in code"), "got {out}");
    assert!(
        out.contains(&format!("Total: 1 open, 1 resolved across {files} files")),
        "got {out}"
    );
}

#[test]
fn status_reports_warnings() {
    let tmp = init_tmp();
    let specs = tmp.path().join("specs");
    fs::write(
        specs.join("ctx.md"),
        "# Current Task Context: demo\n\n- [x] a\n- [x] b\n\n## Gotchas\n\n^^^ pending\n\n```\n## Deferred\n```\n",
    )
    .unwrap();
    fs::write(specs.join("extra.md"), "# Extra\n").unwrap();
    fs::remove_file(tmp.path().join("CHANGELOG.md")).unwrap();

    let (out, err, code) = run(tmp.path(), &["status"]);
    assert_eq!(code, 0, "stderr: {err}");
    for want in [
        "task progress: 2/2 steps done",
        "Additional specs:",
        "extra.md",
        "Markers: 1 open, 0 resolved",
        "all 2 plan steps are checked",
        "forbidden content (Gotchas/Quirks block)",
        "CHANGELOG.md: missing at project root",
    ] {
        assert!(out.contains(want), "expected {want:?}, got {out}");
    }
    assert!(!out.contains("Deferred section"), "got {out}");
}

/// A valid task, as the Phase 4 commands would leave it, at `ready`.
fn ready_task(id: &str) -> String {
    let title = id.split_once('-').unwrap().1;
    format!(
        "---\nid: {id}\nstatus: ready\nscope: [src/a.rs]\ncreated: 2026-10-03\n---\n# Task: {title}\n\n## Plan\n\n- [ ] do it\n\n## Acceptance\n\n- [ ] cargo test\n\n## Log\n\n- 2026-10-03 created: {title}\n- 2026-10-03 advance draft → ready\n- 2026-10-03 scope approved: src/a.rs\n"
    )
}

/// `ready_task` advanced to in-progress/implement.
fn active_task(id: &str) -> String {
    format!(
        "{}- 2026-10-04 advance ready → in-progress/implement\n",
        ready_task(id)
    )
    .replace("status: ready", "status: in-progress\nstage: implement")
}

#[test]
fn check_fresh_project_and_new_task() {
    let tmp = init_tmp();
    let (out, err, code) = run(tmp.path(), &["check"]);
    assert_eq!(code, 0, "stderr: {err}");
    assert_eq!(out, "OK\n");

    run(tmp.path(), &["task", "new", "x"]);
    let (out, err, code) = run(tmp.path(), &["check"]);
    assert_eq!(code, 0, "stderr: {err}");
    assert!(out.contains("warning[scope-empty]"), "got {out}");
    assert!(out.contains("0 errors, 3 warnings"), "got {out}");
}

#[test]
fn check_catches_hand_edited_state() {
    let tmp = init_tmp();
    run(tmp.path(), &["task", "new", "x"]);
    let path = tmp.path().join("specs/tasks/0001-x.md");
    let content = fs::read_to_string(&path).unwrap();
    fs::write(&path, content.replace("status: draft", "status: ready")).unwrap();

    let (out, err, code) = run(tmp.path(), &["check"]);
    assert_eq!(code, 1);
    assert!(
        out.contains("0001-x.md: error[log-mismatch]: frontmatter status is `ready` but the log replays to `draft`"),
        "got {out}"
    );
    assert!(err.contains("check failed"), "got {err}");
}

#[test]
fn check_valid_task_passes_and_repo_rules_fail() {
    let tmp = init_tmp();
    fs::create_dir(tmp.path().join("src")).unwrap();
    let tasks = tmp.path().join("specs/tasks");
    fs::write(tasks.join("0001-a.md"), ready_task("0001-a")).unwrap();
    run(tmp.path(), &["task", "index"]);
    let (out, err, code) = run(tmp.path(), &["check"]);
    assert_eq!(code, 0, "stderr: {err}");
    assert_eq!(out, "OK\n");

    fs::write(tasks.join("0001-a.md"), active_task("0001-a")).unwrap();
    fs::write(tasks.join("0002-b.md"), active_task("0002-b")).unwrap();
    fs::remove_file(tasks.join("_index.md")).unwrap();
    let (out, _, code) = run(tmp.path(), &["--format", "json", "check"]);
    assert_eq!(code, 1);
    let json: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(json["errors"], 3, "got {out}");
    assert_eq!(json["warnings"], 0, "got {out}");
    let codes: Vec<&str> = json["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| d["code"].as_str().unwrap())
        .collect();
    assert_eq!(codes, ["active-tasks", "active-tasks", "index-stale"]);
}

#[test]
fn check_without_specs_dir_fails() {
    let tmp = TempDir::new().unwrap();
    let (_, err, code) = run(tmp.path(), &["check"]);
    assert_eq!(code, 1);
    assert!(err.contains("specdev init"), "got {err}");
}

/// Run a command that must succeed, then `check`, which must pass too.
fn step(dir: &Path, args: &[&str]) -> String {
    let (out, err, code) = run(dir, args);
    assert_eq!(code, 0, "{args:?} failed: {err}");
    let (check, _, code) = run(dir, &["check"]);
    assert_eq!(code, 0, "check failed after {args:?}:\n{check}");
    out
}

/// Run a command that must be refused, leaving `file` unchanged.
fn refused(dir: &Path, file: &Path, args: &[&str], why: &str) {
    let before = fs::read_to_string(file).unwrap();
    let (_, err, code) = run(dir, args);
    assert_eq!(code, 1, "{args:?} should be refused");
    assert!(err.contains(why), "{args:?}: expected {why:?}, got {err}");
    assert_eq!(fs::read_to_string(file).unwrap(), before, "{args:?} wrote");
}

/// Replace text in a task file, as an agent editing prose would.
fn edit(file: &Path, from: &str, to: &str) {
    let content = fs::read_to_string(file).unwrap();
    assert!(content.contains(from), "{from:?} not in {content}");
    fs::write(file, content.replacen(from, to, 1)).unwrap();
}

/// `init` + `task new x` with a scope and one Plan and Acceptance item.
fn drafted() -> (TempDir, std::path::PathBuf) {
    let tmp = init_tmp();
    fs::create_dir(tmp.path().join("src")).unwrap();
    step(tmp.path(), &["task", "new", "x"]);
    let file = tmp.path().join("specs/tasks/0001-x.md");
    step(
        tmp.path(),
        &["task", "scope", "x", "add", "src/a.rs", "--reason", "core"],
    );
    edit(&file, "## Plan\n\n", "## Plan\n\n- [ ] build it\n\n");
    edit(
        &file,
        "## Acceptance\n\n",
        "## Acceptance\n\n- [ ] cargo test\n\n",
    );
    (tmp, file)
}

#[test]
fn task_lifecycle_through_commands() {
    let (tmp, file) = drafted();
    let dir = tmp.path();

    let out = step(dir, &["task", "advance", "x"]);
    assert!(out.contains("0001-x — draft → ready"), "got {out}");
    assert!(out.contains("scope approved: src/a.rs"), "got {out}");
    step(dir, &["task", "advance", "1"]);
    refused(
        dir,
        &file,
        &["task", "advance", "x"],
        "1 of 1 `## Plan` items unticked",
    );
    edit(&file, "- [ ] build it", "- [x] build it");
    step(dir, &["task", "advance", "x"]);
    refused(
        dir,
        &file,
        &["task", "advance", "x"],
        "`## Acceptance` items unticked",
    );
    let out = step(dir, &["task", "advance", "x", "--to", "fix"]);
    assert!(out.contains("(attempts 1)"), "got {out}");
    step(dir, &["task", "advance", "x"]);
    edit(&file, "- [ ] cargo test", "- [x] cargo test");
    step(dir, &["task", "advance", "x"]);
    step(dir, &["task", "advance", "x"]);

    let content = fs::read_to_string(&file).unwrap();
    assert!(content.contains("status: approval\n"), "got {content}");
    assert!(content.contains("attempts: 1\n"), "got {content}");
    let log: Vec<&str> = content
        .split("## Log\n\n")
        .nth(1)
        .unwrap()
        .lines()
        .map(|l| l.split_once(' ').unwrap().1.split_once(' ').unwrap().1)
        .collect();
    assert_eq!(
        log,
        [
            "created: x",
            "scope add src/a.rs — core",
            "advance draft → ready",
            "scope approved: src/a.rs",
            "advance ready → in-progress/implement",
            "advance in-progress/implement → in-progress/verify",
            "advance in-progress/verify → in-progress/fix (attempts 1)",
            "advance in-progress/fix → in-progress/verify",
            "advance in-progress/verify → in-progress/review",
            "advance in-progress/review → approval",
        ]
    );
    let index = fs::read_to_string(dir.join("specs/tasks/_index.md")).unwrap();
    assert!(index.contains("## Active"), "got {index}");

    refused(
        dir,
        &file,
        &["task", "advance", "x"],
        "choose where it goes with --to",
    );
    refused(
        dir,
        &file,
        &["task", "advance", "x", "--to", "done"],
        "task done",
    );
}

#[test]
fn state_commands_refuse_bad_changes() {
    let tmp = init_tmp();
    let dir = tmp.path();
    step(dir, &["task", "new", "x"]);
    let file = dir.join("specs/tasks/0001-x.md");

    // Safety net: draft → ready with empty scope/Plan/Acceptance adds errors.
    refused(
        dir,
        &file,
        &["task", "advance", "x"],
        "would make `specdev check` fail",
    );
    refused(
        dir,
        &file,
        &["task", "advance", "x", "--to", "fix"],
        "can't move from `draft`",
    );
    refused(
        dir,
        &file,
        &["task", "set", "x", "status", "ready"],
        "use `task advance`",
    );
    refused(
        dir,
        &file,
        &["task", "set", "x", "owner", "me"],
        "unknown field `owner`",
    );
    refused(
        dir,
        &file,
        &["task", "scope", "x", "add", "**", "--reason", "all"],
        "scope-glob",
    );
    refused(
        dir,
        &file,
        &["task", "scope", "x", "rm", "src/a.rs"],
        "not in scope",
    );
    refused(
        dir,
        &file,
        &["task", "set", "x", "depends", "9"],
        "no task matching `9`",
    );

    let (tmp, file) = drafted();
    let dir = tmp.path();
    edit(&file, "- [ ] build it", "- [ ] build it\n\n^^^ really?");
    refused(dir, &file, &["task", "advance", "x"], "open ^^^ remark");
    edit(&file, "\n\n^^^ really?", "");

    step(dir, &["task", "advance", "x"]);
    step(dir, &["task", "advance", "x"]);
    step(dir, &["task", "new", "y"]);
    let y = dir.join("specs/tasks/0002-y.md");
    step(
        dir,
        &["task", "scope", "y", "add", "src/b.rs", "--reason", "r"],
    );
    edit(&y, "## Plan\n\n", "## Plan\n\n- [ ] b\n\n");
    edit(&y, "## Acceptance\n\n", "## Acceptance\n\n- [ ] t\n\n");
    step(dir, &["task", "advance", "y"]);
    refused(
        dir,
        &y,
        &["task", "advance", "y"],
        "`0001-x` is already active",
    );

    // A hand edit breaks the log; commands refuse until it's repaired.
    edit(&y, "status: ready", "status: draft");
    refused(
        dir,
        &y,
        &["task", "set", "y", "source", "gh-1"],
        "disagrees with its own log",
    );
}

#[test]
fn attempts_cap_blocks_the_task() {
    let (tmp, file) = drafted();
    let dir = tmp.path();
    fs::write(dir.join("specdev.toml"), "[pipeline]\nmax_attempts = 1\n").unwrap();
    edit(&file, "- [ ] build it", "- [x] build it");
    for _ in 0..3 {
        step(dir, &["task", "advance", "x"]);
    }
    step(dir, &["task", "advance", "x", "--to", "fix"]);
    step(dir, &["task", "advance", "x"]);
    let out = step(dir, &["task", "advance", "x", "--to", "fix"]);
    assert!(out.contains("in-progress/verify → blocked"), "got {out}");
    assert!(
        out.contains("Note: not moved to fix: attempts cap reached (1)"),
        "got {out}"
    );
    let content = fs::read_to_string(&file).unwrap();
    assert!(
        content.contains("blocked_reason: attempts cap reached (1)\n"),
        "got {content}"
    );
    assert!(content.contains("attempts: 1\n"), "got {content}");
}

#[test]
fn block_set_and_scope_commands() {
    let (tmp, file) = drafted();
    let dir = tmp.path();
    step(dir, &["task", "advance", "x"]);

    let out = step(
        dir,
        &["task", "block", "x", "--reason", "waiting on design"],
    );
    assert!(out.contains("ready → blocked"), "got {out}");
    let content = fs::read_to_string(&file).unwrap();
    assert!(content.contains("blocked_reason: waiting on design\n"));
    refused(
        dir,
        &file,
        &["task", "advance", "x"],
        "choose where it goes with --to",
    );
    step(dir, &["task", "advance", "x", "--to", "ready"]);
    let content = fs::read_to_string(&file).unwrap();
    assert!(!content.contains("blocked_reason"), "got {content}");

    step(dir, &["task", "new", "y"]);
    step(dir, &["task", "set", "y", "source", "gh issue 12"]);
    step(dir, &["task", "set", "y", "depends", "x"]);
    let y = fs::read_to_string(dir.join("specs/tasks/0002-y.md")).unwrap();
    assert!(y.contains("source: gh issue 12\n"), "got {y}");
    assert!(y.contains("depends: [0001-x]\n"), "got {y}");
    assert!(y.contains("set depends 0001-x\n"), "got {y}");
    step(dir, &["task", "set", "y", "depends", ""]);
    let y = fs::read_to_string(dir.join("specs/tasks/0002-y.md")).unwrap();
    assert!(!y.contains("depends: ["), "got {y}");
    assert!(y.contains("set depends \"\"\n"), "got {y}");

    // After approval, scope changes are measured against the approved scope.
    step(
        dir,
        &[
            "task",
            "scope",
            "x",
            "add",
            "tests/cli.rs",
            "--reason",
            "e2e",
        ],
    );
    step(dir, &["task", "scope", "x", "rm", "src/a.rs"]);
    let content = fs::read_to_string(&file).unwrap();
    assert!(content.contains("scope: [tests/cli.rs]\n"), "got {content}");

    let (out, _, code) = run(
        dir,
        &["--format", "json", "task", "set", "y", "source", "x"],
    );
    assert_eq!(code, 0);
    let json: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(json["id"], "0002-y");
    assert_eq!(json["to"], "draft");
    assert!(json["log"][0].as_str().unwrap().ends_with("set source x"));
}
