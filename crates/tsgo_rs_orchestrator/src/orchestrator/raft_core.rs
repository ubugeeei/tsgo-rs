use super::{LogEntry, RaftNode, RaftRole};
use crate::Result;

pub(super) fn log_signature(node: &RaftNode) -> (usize, u64) {
    (
        node.log.len(),
        node.log.last().map(|entry| entry.term).unwrap_or(0),
    )
}

pub(super) fn quorum_size(nodes: usize) -> usize {
    (nodes / 2) + 1
}

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

pub(super) fn apply_commits(node: &mut RaftNode) -> Result<()> {
    while node.applied_len < node.commit_len {
        let entry = node.log[node.applied_len].clone();
        node.state.apply(&entry.command)?;
        node.applied_len += 1;
    }
    Ok(())
}
