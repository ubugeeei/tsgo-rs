//! Small helper functions for the in-process Raft model.
//!
//! Keeping these helpers in a separate module makes the state transitions easier
//! to test and to read beside the higher-level cluster orchestration code.

use super::{LogEntry, RaftNode, RaftRole};
use crate::Result;

/// Returns the current log length and final term for a node.
pub(super) fn log_signature(node: &RaftNode) -> (usize, u64) {
    (
        node.log.len(),
        node.log.last().map(|entry| entry.term).unwrap_or(0),
    )
}

/// Returns the majority threshold for `nodes` participants.
pub(super) fn quorum_size(nodes: usize) -> usize {
    (nodes / 2) + 1
}

/// Attempts to grant a vote to a candidate.
///
/// The candidate must be in at least the follower's current term and must have
/// a log that is at least as up to date as the follower's log.
pub(super) fn grant_vote(
    node: &mut RaftNode,
    candidate_term: u64,
    candidate_id: &str,
    candidate_len: usize,
    candidate_last_term: u64,
) -> bool {
    if candidate_term < node.current_term {
        return false;
    }
    let (node_len, node_last_term) = log_signature(node);
    let up_to_date = candidate_last_term > node_last_term
        || (candidate_last_term == node_last_term && candidate_len >= node_len);
    if !up_to_date {
        return false;
    }
    if candidate_term > node.current_term {
        node.current_term = candidate_term;
        node.voted_for = None;
        node.role = RaftRole::Follower;
    }
    if node
        .voted_for
        .as_deref()
        .is_some_and(|vote| vote != candidate_id)
    {
        return false;
    }
    node.voted_for = Some(candidate_id.into());
    true
}

/// Attempts to append a new entry from the leader to a follower.
///
/// The append is accepted only when the follower agrees about the previous log
/// position and term.
pub(super) fn append_to_follower(
    node: &mut RaftNode,
    leader_term: u64,
    leader_id: &str,
    prev_len: usize,
    prev_term: u64,
    entry: &LogEntry,
) -> bool {
    if leader_term < node.current_term {
        return false;
    }
    if prev_len > 0 && node.log.get(prev_len - 1).map(|entry| entry.term) != Some(prev_term) {
        return false;
    }
    node.current_term = leader_term;
    node.role = RaftRole::Follower;
    node.voted_for = Some(leader_id.into());
    node.log.truncate(prev_len);
    node.log.push(entry.clone());
    true
}

/// Applies newly committed log entries to the follower's replicated state.
pub(super) fn apply_commits(node: &mut RaftNode) -> Result<()> {
    while node.applied_len < node.commit_len {
        let entry = node.log[node.applied_len].clone();
        // The replicated state is deterministic, so replaying commands in log
        // order is enough to rebuild it.
        node.state.apply(&entry.command)?;
        node.applied_len += 1;
    }
    Ok(())
}
