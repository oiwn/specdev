use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use tempfile::TempDir;

const BIN: &str = env!("CARGO_BIN_EXE_specdev");

/// Run the specdev binary in `cwd` with `args`, returning (stdout, stderr, exit_code).
fn run(cwd: &Path, args: &[&str]) -> (String, String, i32) {
    run_env(cwd, args, &[])
}

/// Variables git exports to hooks; inherited, they'd point a test's git (and
/// specdev's scope check) at this repository instead of the tempdir.
const GIT_HOOK_ENV: [&str; 3] = ["GIT_DIR", "GIT_INDEX_FILE", "GIT_WORK_TREE"];

/// A command isolated from any enclosing repository: hook variables removed,
/// and repository discovery stops at `cwd` (a tempdir is never inside one).
fn isolated(program: &str, cwd: &Path) -> Command {
    let mut cmd = Command::new(program);
    for var in GIT_HOOK_ENV {
        cmd.env_remove(var);
    }
    cmd.env("GIT_CEILING_DIRECTORIES", cwd.parent().unwrap_or(cwd))
        .current_dir(cwd);
    cmd
}

/// Like `run`, with extra environment variables (e.g. `HOME` for skill tests).
fn run_env(
    cwd: &Path,
    args: &[&str],
    env: &[(&str, &Path)],
) -> (String, String, i32) {
    let output = isolated(BIN, cwd)
        .args(args)
        .envs(env.iter().copied())
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
    assert_eq!(out, "0001-x — draft → ready\n", "one line per move");
    let content = fs::read_to_string(&file).unwrap();
    assert!(
        content.contains(" scope approved: src/a.rs\n"),
        "got {content}"
    );
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
    step(dir, &["task", "advance", "x", "--to", "fix"]);
    step(dir, &["task", "advance", "x"]);
    edit(&file, "- [ ] cargo test", "- [x] cargo test");
    // One walk from verify to approval; each step's gate still applies.
    refused(
        dir,
        &file,
        &["task", "advance", "x", "--to", "approval"],
        "stopped at `in-progress/review → approval`: `0001-x` needs a `## Review`",
    );
    edit(&file, "## Log", "## Review\n\nMatches the plan.\n\n## Log");
    let out = step(dir, &["task", "advance", "x", "--to", "approval"]);
    assert_eq!(
        out,
        "0001-x — in-progress/verify → approval (via in-progress/review)\n"
    );

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
        "`0001-x` is already in progress",
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
    // Review notes and user feedback are revisions: they never hit the cap.
    edit(&file, "- [ ] cargo test", "- [x] cargo test");
    for _ in 0..3 {
        step(dir, &["task", "advance", "x"]);
        step(dir, &["task", "advance", "x", "--to", "fix"]);
        step(dir, &["task", "advance", "x"]);
    }
    let content = fs::read_to_string(&file).unwrap();
    assert_eq!(
        content.matches("in-progress/fix (revision)").count(),
        3,
        "got {content}"
    );
    // A second failed verify is a second repair: blocked.
    let out = step(dir, &["task", "advance", "x", "--to", "fix"]);
    assert!(out.contains("in-progress/verify → blocked"), "got {out}");
    assert!(
        out.contains("Note: not moved to fix: repair attempts cap reached (1)"),
        "got {out}"
    );
    let content = fs::read_to_string(&file).unwrap();
    assert!(
        content.contains("blocked_reason: repair attempts cap reached (1)\n"),
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

/// Run git in `dir` (isolated from this repository); must succeed.
fn git(dir: &Path, args: &[&str]) {
    let out = isolated("git", dir)
        .args(["-c", "user.name=t", "-c", "user.email=t@example.com"])
        .args([
            "-c",
            "commit.gpgsign=false",
            "-c",
            "init.defaultBranch=main",
        ])
        .args(args)
        .output()
        .expect("git must be installed for these tests");
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// `drafted()` advanced to in-progress/implement.
fn implementing(dir: &Path) {
    step(dir, &["task", "advance", "x"]);
    step(dir, &["task", "advance", "x"]);
}

#[test]
fn scope_check_against_git_working_tree() {
    let (tmp, _) = drafted();
    let dir = tmp.path();
    fs::write(dir.join("src/a.rs"), "// a\n").unwrap();
    fs::write(
        dir.join("specdev.toml"),
        "[scope]\nalways_allowed = [\"Cargo.lock\"]\n",
    )
    .unwrap();
    git(dir, &["init", "-q"]);
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", "baseline"]);
    implementing(dir);

    // In scope, under specs/, or allowlisted: fine.
    fs::write(dir.join("src/a.rs"), "// a, edited\n").unwrap();
    fs::write(dir.join("specs/ctx.md"), "# Current Task Context: x\n").unwrap();
    fs::write(dir.join("Cargo.lock"), "# lock\n").unwrap();
    let (out, _, code) = run(dir, &["check"]);
    assert_eq!((code, out.as_str()), (0, "OK\n"));

    // An untracked file outside scope fails until the scope is expanded.
    fs::write(dir.join("src/b.rs"), "// b\n").unwrap();
    let (out, _, code) = run(dir, &["check"]);
    assert_eq!(code, 1);
    assert!(out.contains("src/b.rs: error[out-of-scope]"), "got {out}");
    assert!(out.contains("task scope 0001-x add src/b.rs"), "got {out}");
    step(
        dir,
        &[
            "task", "scope", "x", "add", "src/b.rs", "--reason", "helper",
        ],
    );

    // --staged looks only at the index and only warns.
    fs::write(dir.join("README.md"), "staged\n").unwrap();
    fs::write(dir.join("NOTES.md"), "not staged\n").unwrap();
    git(dir, &["add", "README.md"]);
    let (out, _, code) = run(dir, &["check", "--staged"]);
    assert_eq!(code, 0, "got {out}");
    assert!(
        out.contains("README.md: warning[out-of-scope]"),
        "got {out}"
    );
    assert!(!out.contains("NOTES.md"), "got {out}");
    let (out, _, code) = run(dir, &["check"]);
    assert_eq!(code, 1);
    assert!(out.contains("README.md: error[out-of-scope]"), "got {out}");
    assert!(out.contains("NOTES.md: error[out-of-scope]"), "got {out}");
}

#[test]
fn scope_check_before_the_first_commit() {
    // Without a commit every file is a change, `init`'s own files included.
    // Those predate the task: its start baseline keeps them out of scope
    // until they change again.
    let (tmp, file) = drafted();
    let dir = tmp.path();
    git(dir, &["init", "-q"]);
    for _ in 0..2 {
        let (_, err, code) = run(dir, &["task", "advance", "x"]);
        assert_eq!(code, 0, "{err}");
    }
    let content = fs::read_to_string(&file).unwrap();
    assert!(content.contains(" baseline: AGENTS.md@"), "got {content}");
    fs::write(dir.join("src/a.rs"), "// a\n").unwrap();
    fs::write(dir.join("src/z.rs"), "// z\n").unwrap();
    git(dir, &["add", "src/a.rs"]);
    let (out, _, code) = run(dir, &["check"]);
    assert_eq!(code, 1, "got {out}");
    assert!(out.contains("src/z.rs: error[out-of-scope]"), "got {out}");
    assert!(!out.contains("AGENTS.md"), "baseline file: {out}");
    assert!(!out.contains("src/a.rs:"), "got {out}");
    assert!(!out.contains("specs/"), "got {out}");
    assert!(!out.contains("scope-skipped"), "got {out}");

    fs::write(dir.join("AGENTS.md"), "# AGENTS.md\n\nedited by the task\n")
        .unwrap();
    // `task done` writes CHANGELOG.md: specdev's file, never out of scope.
    fs::write(dir.join("CHANGELOG.md"), "# Changelog\n\n## new entry\n").unwrap();
    let (out, _, _) = run(dir, &["check"]);
    assert!(out.contains("AGENTS.md: error[out-of-scope]"), "got {out}");
    assert!(!out.contains("CHANGELOG.md: error"), "got {out}");
}

#[test]
fn scope_check_skipped_without_git() {
    let (tmp, _) = drafted();
    let dir = tmp.path();
    implementing(dir);
    let (out, _, code) = run(dir, &["check"]);
    assert_eq!(code, 0, "got {out}");
    assert!(
        out.contains("warning[scope-skipped]: scope check skipped"),
        "got {out}"
    );
}

#[test]
fn task_new_prefills_acceptance_defaults() {
    let tmp = init_tmp();
    let dir = tmp.path();
    fs::write(
        dir.join("specdev.toml"),
        "[acceptance]\ndefault = [\"cargo test\", \"cargo clippy\"]\n",
    )
    .unwrap();
    step(dir, &["task", "new", "x"]);
    let content = fs::read_to_string(dir.join("specs/tasks/0001-x.md")).unwrap();
    assert!(
        content.contains(
            "## Acceptance\n\n- [ ] `cargo test`\n- [ ] `cargo clippy`\n\n## Log\n"
        ),
        "got {content}"
    );
}

/// A new task `slug` with scope, ticked Plan and Acceptance, and a Summary:
/// everything `advance` and `done` need, walked to `ready`.
fn ready_to_go(dir: &Path, slug: &str) -> std::path::PathBuf {
    step(dir, &["task", "new", slug]);
    let file = fs::read_dir(dir.join("specs/tasks"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .find(|p| p.to_string_lossy().ends_with(&format!("-{slug}.md")))
        .unwrap();
    step(
        dir,
        &["task", "scope", slug, "add", "src/a.rs", "--reason", "r"],
    );
    edit(&file, "## Plan\n\n", "## Plan\n\n- [x] build it\n\n");
    edit(
        &file,
        "## Acceptance\n\n",
        "## Acceptance\n\n- [x] cargo test\n\n## Review\n\nMatches the plan.\n\n",
    );
    step(dir, &["task", "advance", slug]);
    file
}

/// ready → in-progress → … → approval.
fn to_approval(dir: &Path, slug: &str) {
    for _ in 0..4 {
        step(dir, &["task", "advance", slug]);
    }
}

#[test]
fn advance_walks_forward_checking_every_gate() {
    let tmp = init_tmp();
    let dir = tmp.path();
    fs::create_dir(dir.join("src")).unwrap();

    // Never out of draft: approving the draft is the user's step.
    step(dir, &["task", "new", "d"]);
    let d = dir.join("specs/tasks/0001-d.md");
    refused(
        dir,
        &d,
        &["task", "advance", "d", "--to", "implement"],
        "can't move from `draft` to `in-progress/implement`",
    );

    // A failing gate on the way refuses the whole walk.
    let x = ready_to_go(dir, "x");
    edit(&x, "- [x] cargo test", "- [ ] cargo test");
    refused(
        dir,
        &x,
        &["task", "advance", "x", "--to", "approval"],
        "stopped at `in-progress/verify → in-progress/review`: `0002-x`: 1 of 1 `## Acceptance` items unticked",
    );

    // All gates pass: ready to approval in one call, one log line per step.
    edit(&x, "- [ ] cargo test", "- [x] cargo test");
    let out = step(dir, &["task", "advance", "x", "--to", "approval"]);
    assert_eq!(
        out,
        "0002-x — ready → approval (via in-progress/implement, in-progress/verify, in-progress/review)\n"
    );
    let content = fs::read_to_string(&x).unwrap();
    for line in [
        " advance ready → in-progress/implement\n",
        " advance in-progress/implement → in-progress/verify\n",
        " advance in-progress/verify → in-progress/review\n",
        " advance in-progress/review → approval\n",
    ] {
        assert!(content.contains(line), "missing {line:?} in {content}");
    }
}

#[test]
fn task_done_closes_a_batch_all_or_nothing() {
    let tmp = init_tmp();
    let dir = tmp.path();
    fs::create_dir(dir.join("src")).unwrap();
    let changelog = dir.join("CHANGELOG.md");
    let x = ready_to_go(dir, "x");
    let y = ready_to_go(dir, "y");
    let z = ready_to_go(dir, "z");
    // Tasks waiting in approval don't hold the slot: all three get there.
    for slug in ["x", "y", "z"] {
        to_approval(dir, slug);
    }
    edit(&x, "## Log", "## Summary\n\nShips x.\n\n## Log");
    edit(
        &y,
        "## Log",
        "## Manual checks\n\n- [ ] look at it\n\n## Summary\n\nShips y.\n\n## Log",
    );

    // One blocked task stops the batch; every blocker is reported.
    let before = fs::read_to_string(&changelog).unwrap();
    let (_, err, code) = run(dir, &["task", "done", "x", "y", "z"]);
    assert_eq!(code, 1);
    assert!(err.contains("nothing closed"), "got {err}");
    assert!(
        err.contains("`0002-y`: 1 of 1 `## Manual checks`"),
        "got {err}"
    );
    assert!(
        err.contains("`0003-z` needs a one-line `## Summary`"),
        "got {err}"
    );
    assert!(x.exists() && y.exists() && z.exists());
    assert_eq!(fs::read_to_string(&changelog).unwrap(), before);

    edit(
        &y,
        "- [ ] look at it",
        "- [x] look at it — waived by user: \"skip\"",
    );
    edit(&z, "## Log", "## Summary\n\nShips z.\n\n## Log");
    let (out, _, code) = run(dir, &["task", "done", "x", "y", "--dry-run"]);
    assert_eq!(code, 0);
    assert!(out.starts_with("Dry run: nothing written"), "got {out}");
    assert!(x.exists() && y.exists());
    assert_eq!(fs::read_to_string(&changelog).unwrap(), before);

    let out = step(
        dir,
        &["task", "done", "x", "y", "--approval", "both look good"],
    );
    assert!(out.contains("0002-y — approval → done"), "got {out}");
    let archived =
        fs::read_to_string(dir.join("specs/tasks/done/0001-x.md")).unwrap();
    assert!(
        archived.contains(" approved: both look good\n- ")
            && archived.ends_with(" advance approval → done\n"),
        "got {archived}"
    );

    // A retry after a partial close (changelog written, file not moved)
    // doesn't duplicate the entry.
    let (with_z, _, _) = run(dir, &["task", "done", "z", "--dry-run"]);
    assert!(with_z.contains("0003-z"), "got {with_z}");
    let log = fs::read_to_string(&changelog).unwrap();
    fs::write(
        &changelog,
        log.replacen(
            "\n## ",
            "\n## 2026-10-08 — z\n\n- Ships z.\n- Task `0003-z`\n\n## ",
            1,
        ),
    )
    .unwrap();
    step(dir, &["task", "done", "z"]);
    let log = fs::read_to_string(&changelog).unwrap();
    assert_eq!(log.matches("- Task `0003-z`").count(), 1, "got {log}");
    assert_eq!(log.matches("- Task `0001-x`").count(), 1, "got {log}");
}

#[test]
fn task_done_archives_and_writes_changelog() {
    let tmp = init_tmp();
    let dir = tmp.path();
    fs::create_dir(dir.join("src")).unwrap();
    let changelog = dir.join("CHANGELOG.md");
    let original = fs::read_to_string(&changelog).unwrap();

    let x = ready_to_go(dir, "x");
    let y = ready_to_go(dir, "y");
    refused(
        dir,
        &x,
        &["task", "done", "x"],
        "only tasks in approval can be done",
    );
    to_approval(dir, "x");
    let (out, _, _) = run(dir, &["check"]);
    assert!(out.contains("warning[summary-missing]"), "got {out}");

    // Gates: a Summary is required; Manual checks must be ticked.
    refused(
        dir,
        &x,
        &["task", "done", "x"],
        "needs a one-line `## Summary`",
    );
    edit(
        &x,
        "## Log",
        "## Manual checks\n\n- [ ] try it by hand\n\n## Summary\n\nShips x.\n\n## Log",
    );
    refused(
        dir,
        &x,
        &["task", "done", "x"],
        "1 of 1 `## Manual checks` items unticked",
    );
    edit(&x, "- [ ] try it", "- [x] try it");
    // x waits for acceptance without holding the slot: y may start, and then
    // x can't go back to in-progress while y is there.
    step(dir, &["task", "advance", "y"]);
    refused(
        dir,
        &x,
        &["task", "advance", "x", "--to", "fix"],
        "`0002-y` is already in progress",
    );

    fs::write(
        dir.join("specs/ctx.md"),
        "# Current Task Context: x\n\nTask: specs/tasks/0001-x.md\n",
    )
    .unwrap();
    let out = step(dir, &["task", "done", "x"]);
    assert_eq!(
        out,
        "0001-x — approval → done (specs/tasks/done/0001-x.md)\nUpdated CHANGELOG.md\n"
    );

    let archived = dir.join("specs/tasks/done/0001-x.md");
    assert!(!x.exists(), "old file must be gone");
    let content = fs::read_to_string(&archived).unwrap();
    assert!(content.contains("status: done\n"), "got {content}");
    assert!(
        content.ends_with(" advance approval → done\n"),
        "got {content}"
    );

    let log = fs::read_to_string(&changelog).unwrap();
    assert!(
        log.starts_with(&original),
        "existing content must stay intact"
    );
    assert!(
        log.contains(" — x\n\n- Ships x.\n- Task `0001-x`\n"),
        "got {log}"
    );
    let index = fs::read_to_string(dir.join("specs/tasks/_index.md")).unwrap();
    assert!(
        index.contains("## Done\n\n- **0001-x** — done — x"),
        "got {index}"
    );
    let (out, _, _) = run(dir, &["check"]);
    assert!(out.contains("warning[ctx-task-inactive]"), "got {out}");

    let (_, err, code) = run(dir, &["task", "done", "x"]);
    assert_eq!(code, 1);
    assert!(err.contains("archived in done/"), "got {err}");

    // The queue moves on; the newer entry goes above the older one.
    for _ in 0..3 {
        step(dir, &["task", "advance", "y"]);
    }
    edit(&y, "## Log", "## Summary\n\n- Ships y.\n\n## Log");
    step(dir, &["task", "done", "y"]);
    let log = fs::read_to_string(&changelog).unwrap();
    let (at_y, at_x) = (
        log.find("- Ships y.").unwrap(),
        log.find("- Ships x.").unwrap(),
    );
    assert!(at_y < at_x, "newest entry first: {log}");
}

#[test]
fn quality_gate_in_check() {
    let tmp = init_tmp();
    let dir = tmp.path();
    fs::create_dir(dir.join("src")).unwrap();
    step(dir, &["task", "new", "x"]);
    let file = dir.join("specs/tasks/0001-x.md");
    let steps: String = (0..9).map(|i| format!("- [ ] step {i}\n")).collect();
    edit(&file, "## Plan\n\n", &format!("## Plan\n\n{steps}\n"));

    // Warnings by default: check passes.
    let (out, _, code) = run(dir, &["check"]);
    assert_eq!(code, 0, "got {out}");
    assert!(
        out.contains(
            "warning[quality-plan-steps]: 9 plan steps (max 8 for task files)"
        ),
        "got {out}"
    );

    // Errors when configured; the safety net then refuses growing scope.
    let config = "[quality]\nerrors = true\n\n[quality.task]\nmax_scope = 1\n";
    fs::write(dir.join("specdev.toml"), config).unwrap();
    let (out, _, code) = run(dir, &["check"]);
    assert_eq!(code, 1, "got {out}");
    assert!(out.contains("error[quality-plan-steps]"), "got {out}");
    let (_, err, code) = run(
        dir,
        &["task", "scope", "x", "add", "src/a.rs", "--reason", "r"],
    );
    assert_eq!(code, 0, "first entry is within max_scope: {err}");
    refused(
        dir,
        &file,
        &["task", "scope", "x", "add", "src/b.rs", "--reason", "r"],
        "quality-scope",
    );
}

#[test]
fn quality_gate_for_spec_files() {
    let tmp = init_tmp();
    let dir = tmp.path();
    let changelog = dir.join("CHANGELOG.md");
    let mut content = fs::read_to_string(&changelog).unwrap();
    content.push_str(&"- old entry line\n".repeat(400));
    fs::write(&changelog, content).unwrap();
    fs::write(
        dir.join("specs/ideas.md"),
        "# Ideas\n\n| idea | why |\n|---|---|\n| a | b |\n",
    )
    .unwrap();

    let (out, _, code) = run(dir, &["check"]);
    assert_eq!(code, 0, "got {out}");
    assert!(
        out.contains("CHANGELOG.md: warning[quality-lines]"),
        "got {out}"
    );
    assert!(out.contains("compress older entries"), "got {out}");
    assert!(
        !out.contains("md-table"),
        "tables are allowed by default: {out}"
    );

    fs::write(
        dir.join("specdev.toml"),
        "[quality]\nforbid_tables = true\n",
    )
    .unwrap();
    let (out, _, _) = run(dir, &["check"]);
    assert!(
        out.contains("specs/ideas.md:3: warning[md-table]"),
        "got {out}"
    );

    let (out, _, code) = run(dir, &["list", "--stats"]);
    assert_eq!(code, 0);
    for col in ["lines", "code", "tbl"] {
        assert!(out.contains(col), "stats header missing {col}: {out}");
    }

    // Committed, the CHANGELOG warning is old news: one summary line, unless
    // --verbose. Touch the file and it's listed again.
    git(dir, &["init", "-q"]);
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", "baseline"]);
    let (out, _, code) = run(dir, &["check"]);
    assert_eq!(code, 0, "got {out}");
    assert!(!out.contains("warning[quality-lines]"), "got {out}");
    assert!(
        out.contains("1 existing size warning in unchanged files"),
        "got {out}"
    );
    let (out, _, _) = run(dir, &["check", "--verbose"]);
    assert!(out.contains("warning[quality-lines]"), "got {out}");
    fs::write(
        &changelog,
        format!("{}- new\n", fs::read_to_string(&changelog).unwrap()),
    )
    .unwrap();
    let (out, _, _) = run(dir, &["check"]);
    assert!(out.contains("warning[quality-lines]"), "got {out}");
}

#[test]
fn scan_and_stats_include_open_task_files() {
    let tmp = init_tmp();
    let dir = tmp.path();
    step(dir, &["task", "new", "x"]);
    let task = dir.join("specs/tasks/0001-x.md");
    edit(&task, "## Plan\n\n", "## Plan\n\n^^^ which locale?\n\n");
    let (out, _, code) = run(dir, &["scan"]);
    assert_eq!(code, 0);
    assert!(out.contains("tasks/0001-x.md\n"), "got {out}");
    assert!(out.contains("[    open]  which locale?"), "got {out}");
    let (out, _, _) = run(dir, &["list", "--stats"]);
    assert!(out.contains("tasks/0001-x.md "), "got {out}");
    assert!(!out.contains("_index"), "got {out}");
}

#[test]
fn fmt_normalizes_specs_and_check_flags_unformatted() {
    let tmp = init_tmp();
    let dir = tmp.path();
    let (out, _, _) = run(dir, &["check"]);
    assert_eq!(out, "OK\n", "init's templates are already formatted");

    let ideas = dir.join("specs/ideas.md");
    let roadmap = dir.join("specs/roadmap.md");
    fs::write(&ideas, "# Ideas\n* one idea that\n  wraps\n^^^ keep me\n").unwrap();
    fs::write(&roadmap, "# Roadmap\nA paragraph that\nwraps.\n").unwrap();
    let (out, _, code) = run(dir, &["check"]);
    assert_eq!(code, 0, "unformatted is a warning: {out}");
    assert!(
        out.contains("specs/ideas.md: warning[unformatted]"),
        "got {out}"
    );
    assert!(
        out.contains("specs/roadmap.md: warning[unformatted]"),
        "got {out}"
    );

    // One file at a time.
    let (out, err, code) = run(dir, &["fmt", "specs/ideas.md"]);
    assert_eq!(code, 0, "{err}");
    assert_eq!(
        out,
        "Formatted specs/ideas.md\n1 formatted, 0 already formatted\n"
    );
    assert_eq!(
        fs::read_to_string(&ideas).unwrap(),
        "# Ideas\n\n- one idea that wraps\n^^^ keep me\n"
    );
    assert!(
        fs::read_to_string(&roadmap)
            .unwrap()
            .contains("that\nwraps")
    );

    // Everything; then nothing left to do.
    let (out, _, _) = run(dir, &["fmt"]);
    assert!(out.contains("Formatted specs/roadmap.md"), "got {out}");
    let (out, _, _) = run(dir, &["fmt"]);
    assert!(out.starts_with("All "), "got {out}");
    let (out, _, _) = run(dir, &["check"]);
    assert_eq!(out, "OK\n");

    // Task files are covered and stay valid.
    step(dir, &["task", "new", "x"]);
    let task = dir.join("specs/tasks/0001-x.md");
    edit(
        &task,
        "## Plan\n\n",
        "## Plan\n\n* [ ] a step\n  wrapped\n\n",
    );
    let (out, _, _) = run(dir, &["check"]);
    assert!(out.contains("0001-x.md: warning[unformatted]"), "got {out}");
    let (_, err, code) = run(dir, &["fmt"]);
    assert_eq!(code, 0, "{err}");
    assert!(
        fs::read_to_string(&task)
            .unwrap()
            .contains("- [ ] a step wrapped\n")
    );
    let (out, _, code) = run(dir, &["check"]);
    assert_eq!(code, 0, "got {out}");
    assert!(!out.contains("unformatted"), "got {out}");

    let (_, err, code) = run(dir, &["fmt", "specs/missing.md"]);
    assert_eq!(code, 1);
    assert!(err.contains("specs/missing.md"), "got {err}");
}
