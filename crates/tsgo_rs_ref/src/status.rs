use crate::{LockedRepository, RepositorySnapshot, canonical_repository_id};
use tsgo_rs_core::fast::{CompactString, SmallVec, compact_format};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RepositoryProblem {
    RepositoryMismatch {
        expected: CompactString,
        actual: CompactString,
    },
    CommitMismatch {
        expected: CompactString,
        actual: CompactString,
    },
    AttachedHead {
        branch: CompactString,
    },
    DirtyWorktree,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RepositoryStatus {
    pub exact: bool,
    pub problems: SmallVec<[RepositoryProblem; 4]>,
    pub snapshot: RepositorySnapshot,
}

impl RepositoryStatus {
    pub fn from_snapshot(lock: &LockedRepository, snapshot: RepositorySnapshot) -> Self {
        let mut problems = SmallVec::<[RepositoryProblem; 4]>::new();
        let expected_repository = canonical_repository_id(&lock.repository);
        let actual_repository = canonical_repository_id(&snapshot.remote_url);
        if expected_repository != actual_repository {
            problems.push(RepositoryProblem::RepositoryMismatch {
                expected: expected_repository,
                actual: actual_repository,
            });
        }
        if lock.commit != snapshot.commit {
            problems.push(RepositoryProblem::CommitMismatch {
                expected: lock.commit.as_str().into(),
                actual: snapshot.commit.as_str().into(),
            });
        }
        if let Some(branch) = &snapshot.branch {
            problems.push(RepositoryProblem::AttachedHead {
                branch: branch.as_str().into(),
            });
        }
        if snapshot.dirty {
            problems.push(RepositoryProblem::DirtyWorktree);
        }
        Self {
            exact: problems.is_empty(),
            problems,
            snapshot,
        }
    }

    pub fn describe(&self) -> CompactString {
        if self.exact {
            return compact_format(format_args!(
                "typescript_go is pinned exactly at {} ({})",
                self.snapshot.commit, self.snapshot.subject
            ));
        }
        let mut description = CompactString::default();
        for (index, problem) in self.problems.iter().enumerate() {
            if index != 0 {
                description.push('\n');
            }
            description.push_str(problem_text(problem).as_str());
        }
        description
    }
}

fn problem_text(problem: &RepositoryProblem) -> CompactString {
    match problem {
        RepositoryProblem::RepositoryMismatch { expected, actual } => compact_format(format_args!(
            "repository mismatch: expected {expected}, got {actual}"
        )),
        RepositoryProblem::CommitMismatch { expected, actual } => compact_format(format_args!(
            "commit mismatch: expected {expected}, got {actual}"
        )),
        RepositoryProblem::AttachedHead { branch } => {
            compact_format(format_args!("HEAD must be detached, found {branch}"))
        }
        RepositoryProblem::DirtyWorktree => CompactString::from("worktree must be clean"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lock() -> LockedRepository {
        LockedRepository {
            path: "ref/typescript-go".into(),
            repository: "https://github.com/microsoft/typescript-go.git".into(),
            commit: "abc".into(),
            tree: "tree".into(),
            committer_date: "2026-03-30T00:00:00Z".into(),
            author: "Example".into(),
            subject: "Pinned".into(),
        }
    }

    fn snapshot() -> RepositorySnapshot {
        RepositorySnapshot {
            remote_url: "git@github.com:microsoft/typescript-go.git".into(),
            commit: "abc".into(),
            tree: "tree".into(),
            committer_date: "2026-03-30T00:00:00Z".into(),
            author: "Example".into(),
            subject: "Pinned".into(),
            branch: None,
            dirty: false,
        }
    }

    #[test]
    fn exact_status_requires_detached_clean_exact_commit() {
        let status = RepositoryStatus::from_snapshot(&lock(), snapshot());
        assert!(status.exact);
        assert!(status.problems.is_empty());
    }

    #[test]
    fn status_collects_drift_problems() {
        let mut drifted = snapshot();
        drifted.commit = "def".into();
        drifted.branch = Some("main".into());
        drifted.dirty = true;
        let status = RepositoryStatus::from_snapshot(&lock(), drifted);
        assert!(!status.exact);
        assert_eq!(status.problems.len(), 3);
    }

    #[test]
    fn exact_status_has_human_readable_description() {
        let status = RepositoryStatus::from_snapshot(&lock(), snapshot());
        assert_eq!(
            status.describe(),
            "typescript_go is pinned exactly at abc (Pinned)"
        );
    }

    #[test]
    fn drift_description_lists_each_problem_on_its_own_line() {
        let mut drifted = snapshot();
        drifted.remote_url = "https://example.com/other/repo.git".into();
        drifted.commit = "def".into();
        drifted.branch = Some("main".into());
        drifted.dirty = true;
        let status = RepositoryStatus::from_snapshot(&lock(), drifted);
        let description = status.describe();
        assert!(description.contains("repository mismatch"));
        assert!(description.contains("commit mismatch"));
        assert!(description.contains("HEAD must be detached"));
        assert!(description.contains("worktree must be clean"));
        assert_eq!(description.matches('\n').count(), 3);
    }
}
