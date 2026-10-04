//! Changed files, read from the `git` CLI. Read-only: specdev never writes to
//! git, and uses no git bindings. Paths are relative to the project root, so a
//! project living in a repository subdirectory works.
//!
//! The git environment is passed through untouched: inside a pre-commit hook,
//! `GIT_INDEX_FILE` is exactly the index `--staged` should read.

use std::path::Path;
use std::process::Command;

#[derive(Debug, PartialEq, Eq)]
pub enum Changes {
    Files(Vec<String>),
    /// No git, or not inside a work tree; the reason, for a warning.
    Unavailable(String),
}

/// Files changed in the working tree (vs `HEAD`, plus untracked files), or
/// only the staged ones.
pub fn changed_files(root: &Path, staged: bool) -> Changes {
    match git(root, &["rev-parse", "--is-inside-work-tree"]) {
        Ok(out) if out.trim() == "true" => {}
        Ok(_) => return Changes::Unavailable("not inside a git work tree".into()),
        Err(reason) => return Changes::Unavailable(reason),
    }
    let cached = ["diff", "--cached", "--name-only", "-z", "--relative"];
    let lists = if staged {
        vec![git(root, &cached)]
    } else {
        // Before the first commit there is no HEAD; the index is the diff.
        let has_head = git(root, &["rev-parse", "--verify", "-q", "HEAD"]).is_ok();
        let tracked = if has_head {
            git(root, &["diff", "--name-only", "-z", "--relative", "HEAD"])
        } else {
            git(root, &cached)
        };
        vec![
            tracked,
            git(root, &["ls-files", "--others", "--exclude-standard", "-z"]),
        ]
    };
    let mut files = Vec::new();
    for list in lists {
        match list {
            Ok(out) => files.extend(split_nul(&out)),
            Err(reason) => return Changes::Unavailable(reason),
        }
    }
    files.sort();
    files.dedup();
    Changes::Files(files)
}

/// Run `git -C <root> <args>`; stdout on success, a reason otherwise.
fn git(root: &Path, args: &[&str]) -> Result<String, String> {
    let dir = if root.as_os_str().is_empty() {
        Path::new(".")
    } else {
        root
    };
    let output = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .map_err(|e| format!("can't run git: {e}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let first = stderr.lines().next().unwrap_or("").trim();
        return Err(format!("`git {}` failed: {first}", args.join(" ")));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn split_nul(out: &str) -> impl Iterator<Item = String> + '_ {
    out.split('\0')
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_nul_separated_paths() {
        let paths: Vec<String> = split_nul("a.rs\0dir/with space.rs\0\0").collect();
        assert_eq!(paths, ["a.rs", "dir/with space.rs"]);
        assert_eq!(split_nul("").count(), 0);
    }
}
