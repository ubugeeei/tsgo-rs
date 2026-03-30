use std::{path::Path, process::Command};
use tsgo_rs_core::{
    Result, TsgoError,
    fast::{CompactString, compact_format},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommitMetadata {
    pub commit: CompactString,
    pub tree: CompactString,
    pub committer_date: CompactString,
    pub author: CompactString,
    pub subject: CompactString,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RepositorySnapshot {
    pub remote_url: CompactString,
    pub commit: CompactString,
    pub tree: CompactString,
    pub committer_date: CompactString,
    pub author: CompactString,
    pub subject: CompactString,
    pub branch: Option<CompactString>,
    pub dirty: bool,
}

pub fn canonical_repository_id(url: &str) -> CompactString {
    let trimmed = url.trim().trim_end_matches(".git");
    let ssh = trimmed
        .strip_prefix("git@")
        .map(|value| value.replacen(':', "/", 1));
    let https = trimmed
        .strip_prefix("https://")
        .or_else(|| trimmed.strip_prefix("http://"))
        .or_else(|| trimmed.strip_prefix("ssh://"));
    ssh.or(https.map(str::to_owned))
        .map(CompactString::from)
        .unwrap_or_else(|| CompactString::from(trimmed))
}

pub fn canonical_repository_url(url: &str) -> CompactString {
    let repository = canonical_repository_id(url);
    let mut canonical = CompactString::from("https://");
    canonical.push_str(repository.as_str());
    canonical.push_str(".git");
    canonical
}

pub fn snapshot(path: &Path) -> Result<RepositorySnapshot> {
    let commit = run_git(path, &["rev-parse", "HEAD"])?;
    let metadata = metadata(path, "HEAD")?;
    Ok(RepositorySnapshot {
        remote_url: run_git(path, &["remote", "get-url", "origin"])?,
        commit,
        tree: metadata.tree,
        committer_date: metadata.committer_date,
        author: metadata.author,
        subject: metadata.subject,
        branch: run_git_allow_failure(path, &["symbolic-ref", "-q", "--short", "HEAD"]),
        dirty: !run_git(path, &["status", "--short", "--untracked-files=all"])?.is_empty(),
    })
}

pub fn metadata(path: &Path, revision: &str) -> Result<CommitMetadata> {
    let body = run_git(
        path,
        &[
            "show",
            "--no-patch",
            "--format=%H%n%T%n%cI%n%an%n%s",
            revision,
        ],
    )?;
    let mut lines = body.lines();
    Ok(CommitMetadata {
        commit: next_line(&mut lines, "commit")?,
        tree: next_line(&mut lines, "tree")?,
        committer_date: next_line(&mut lines, "committer date")?,
        author: next_line(&mut lines, "author")?,
        subject: next_line(&mut lines, "subject")?,
    })
}

pub fn clone_no_checkout(repository: &str, path: &Path) -> Result<()> {
    run_git_inherit(
        None,
        &[
            "clone",
            "--origin",
            "origin",
            "--no-checkout",
            repository,
            path.to_str().unwrap(),
        ],
    )?;
    Ok(())
}

pub fn fetch_commit(path: &Path, commit: &str) -> Result<()> {
    run_git_inherit(Some(path), &["fetch", "--depth", "1", "origin", commit])?;
    Ok(())
}

pub fn switch_detached(path: &Path, commit: &str) -> Result<()> {
    run_git_inherit(Some(path), &["switch", "--detach", commit])?;
    Ok(())
}

fn next_line<'a>(lines: &mut impl Iterator<Item = &'a str>, name: &str) -> Result<CompactString> {
    lines.next().map(CompactString::from).ok_or_else(|| {
        TsgoError::Protocol(compact_format(format_args!("git output missing {name}")))
    })
}

fn run_git(path: &Path, args: &[&str]) -> Result<CompactString> {
    run_git_at(Some(path), args)
}

fn run_git_allow_failure(path: &Path, args: &[&str]) -> Option<CompactString> {
    run_git_at(Some(path), args)
        .ok()
        .filter(|value| !value.is_empty())
}

fn run_git_inherit(path: Option<&Path>, args: &[&str]) -> Result<()> {
    let status = command(path, args).status()?;
    if status.success() {
        return Ok(());
    }
    Err(git_command_error(args))
}

fn run_git_at(path: Option<&Path>, args: &[&str]) -> Result<CompactString> {
    let output = command(path, args).output()?;
    if !output.status.success() {
        return Err(git_command_error(args));
    }
    Ok(CompactString::from_utf8_lossy(&output.stdout).trim().into())
}

fn git_command_error(args: &[&str]) -> TsgoError {
    let mut command = CompactString::from("git");
    for arg in args {
        command.push(' ');
        command.push_str(arg);
    }
    TsgoError::Protocol(compact_format(format_args!(
        "git command failed: {command}"
    )))
}

fn command(path: Option<&Path>, args: &[&str]) -> Command {
    let mut command = Command::new("git");
    command.args(args);
    if let Some(path) = path {
        command.current_dir(path);
    }
    command
}

#[cfg(test)]
mod tests {
    use super::{canonical_repository_id, canonical_repository_url};

    #[test]
    fn canonicalizes_https_and_ssh_urls() {
        assert_eq!(
            canonical_repository_id("https://github.com/microsoft/typescript-go.git"),
            "github.com/microsoft/typescript-go"
        );
        assert_eq!(
            canonical_repository_id("git@github.com:microsoft/typescript-go.git"),
            "github.com/microsoft/typescript-go"
        );
        assert_eq!(
            canonical_repository_url("git@github.com:microsoft/typescript-go.git"),
            "https://github.com/microsoft/typescript-go.git"
        );
    }
}
