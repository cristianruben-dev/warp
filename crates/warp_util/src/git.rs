use std::path::Path;

use anyhow::{Result, anyhow};

/// Runs a git command and returns the output as a string.
/// Thin wrapper over [`run_git_command_with_env`] with no `PATH` override.
#[cfg(not(target_family = "wasm"))]
pub async fn run_git_command(repo_path: &Path, args: &[&str]) -> Result<String> {
    run_git_command_with_env(repo_path, args, None).await
}

/// Like [`run_git_command`] but sets `PATH` on the child when `path_env` is
/// `Some`. Used by callers whose hooks need user-installed binaries (e.g.
/// the LFS `pre-push` hook → `git-lfs`). See `specs/APP-4188/TECH.md`.
#[cfg(not(target_family = "wasm"))]
pub async fn run_git_command_with_env(
    repo_path: &Path,
    args: &[&str],
    path_env: Option<&str>,
) -> Result<String> {
    use command::Stdio;

    log::debug!(
        "[GIT OPERATION] git.rs run_git_command git {}",
        args.join(" ")
    );
    let mut git_args = vec!["-c", "diff.autoRefreshIndex=false"];
    git_args.extend_from_slice(args);
    let mut env = vec![("GIT_OPTIONAL_LOCKS", "0")];
    if let Some(path_env) = path_env {
        env.push(("PATH", path_env));
    }

    let mut cmd = git_command(repo_path, &git_args, &env);
    cmd.stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    let output = cmd
        .output()
        .await
        .map_err(|e| anyhow!("Failed to execute git command: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Handle git diff specific behavior:
    // - Exit code 0: no differences
    // - Exit code 1: differences found (this is normal for diff commands)
    // - Exit code > 1: actual error
    if output.status.success() || (output.status.code() == Some(1) && !stdout.is_empty()) {
        Ok(stdout)
    } else {
        Err(anyhow!("Git command failed: {}, {}", stderr, stdout))
    }
}

/// Builds the command that runs `git` with `args` in `repo_path`, with `env` set on the child.
#[cfg(not(target_family = "wasm"))]
fn git_command(repo_path: &Path, args: &[&str], env: &[(&str, &str)]) -> command::r#async::Command {
    use command::r#async::Command;

    let mut cmd = Command::new("git");
    cmd.args(args).current_dir(repo_path);
    for (key, value) in env {
        cmd.env(key, value);
    }
    cmd
}

#[cfg(target_family = "wasm")]
pub async fn run_git_command(_repo_path: &Path, _args: &[&str]) -> Result<String> {
    Err(anyhow!("Not supported on wasm"))
}

#[cfg(target_family = "wasm")]
pub async fn run_git_command_with_env(
    _repo_path: &Path,
    _args: &[&str],
    _path_env: Option<&str>,
) -> Result<String> {
    Err(anyhow!("Not supported on wasm"))
}
