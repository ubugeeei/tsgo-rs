use super::state::{ReplicatedCommand, ReplicatedState};
use crate::{Result, TsgoError};
use parking_lot::RwLock;
use std::sync::Arc;
use tsgo_rs_core::fast::{CompactString, FastMap, SmallVec, compact_format};

#[path = "raft_core.rs"]
mod raft_core;
use raft_core::{append_to_follower, apply_commits, grant_vote, log_signature, quorum_size};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RaftRole {
    Leader,
    Follower,
    Candidate,
}

#[derive(Clone)]
pub struct RaftCluster {
    nodes: Arc<RwLock<FastMap<CompactString, RaftNode>>>,
}

#[derive(Clone)]
pub(super) struct LogEntry {
    term: u64,
    command: ReplicatedCommand,
}

#[derive(Clone)]
pub(super) struct RaftNode {
    id: CompactString,
    current_term: u64,
    role: RaftRole,
    voted_for: Option<CompactString>,
    commit_len: usize,
    applied_len: usize,
    log: SmallVec<[LogEntry; 8]>,
    state: ReplicatedState,
}

impl RaftCluster {
    pub fn new<I, S>(node_ids: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<CompactString>,
    {
        let nodes = node_ids
            .into_iter()
            .map(Into::into)
            .map(|id| {
                (
                    id.clone(),
                    RaftNode {
                        id,
                        current_term: 0,
                        role: RaftRole::Follower,
                        voted_for: None,
                        commit_len: 0,
                        applied_len: 0,
                        log: SmallVec::new(),
                        state: ReplicatedState::default(),
                    },
                )
            })
            .collect();
        Self {
            nodes: Arc::new(RwLock::new(nodes)),
        }
    }

    pub fn leader_id(&self) -> Option<CompactString> {
        self.nodes
            .read()
            .values()
            .find(|node| node.role == RaftRole::Leader)
            .map(|node| node.id.clone())
    }

    pub fn campaign(&self, candidate_id: &str) -> Result<u64> {
        let mut nodes = self.nodes.write();
        let (last_len, last_term) = log_signature(nodes.get(candidate_id).ok_or_else(|| {
            TsgoError::Protocol(compact_format(format_args!(
                "unknown raft node: {candidate_id}"
            )))
        })?);
        let next_term = nodes
            .values()
            .map(|node| node.current_term)
            .max()
            .unwrap_or(0)
            + 1;
        let mut votes = 0;
        for node in nodes.values_mut() {
            if node.id == candidate_id {
                node.current_term = next_term;
                node.role = RaftRole::Candidate;
                node.voted_for = Some(candidate_id.into());
                votes += 1;
                continue;
            }
            if grant_vote(node, next_term, candidate_id, last_len, last_term) {
                votes += 1;
            }
        }
        if votes < quorum_size(nodes.len()) {
            if let Some(node) = nodes.get_mut(candidate_id) {
                node.role = RaftRole::Follower;
            }
            return Err(TsgoError::Protocol(
                "raft election did not reach quorum".into(),
            ));
        }
        for node in nodes.values_mut() {
            node.current_term = next_term.max(node.current_term);
            node.role = if node.id == candidate_id {
                RaftRole::Leader
            } else {
                RaftRole::Follower
            };
        }
        Ok(next_term)
    }

    pub fn append(&self, leader_id: &str, command: ReplicatedCommand) -> Result<u64> {
        let mut nodes = self.nodes.write();
        let (leader_term, prev_len, prev_term) = {
            let leader = nodes.get_mut(leader_id).ok_or_else(|| {
                TsgoError::Protocol(compact_format(format_args!(
                    "unknown raft node: {leader_id}"
                )))
            })?;
            if leader.role != RaftRole::Leader {
                return Err(TsgoError::Protocol(compact_format(format_args!(
                    "raft node is not leader: {leader_id}"
                ))));
            }
            let prev_term = leader.log.last().map(|entry| entry.term).unwrap_or(0);
            let prev_len = leader.log.len();
            leader.log.push(LogEntry {
                term: leader.current_term,
                command,
            });
            (leader.current_term, prev_len, prev_term)
        };
        let entry = nodes.get(leader_id).unwrap().log[prev_len].clone();
        let mut acknowledgements = 1;
        for (node_id, node) in nodes.iter_mut() {
            if node_id == leader_id {
                continue;
            }
            if append_to_follower(node, leader_term, leader_id, prev_len, prev_term, &entry) {
                acknowledgements += 1;
            }
        }
        if acknowledgements < quorum_size(nodes.len()) {
            return Err(TsgoError::Protocol(
                "raft append did not reach quorum".into(),
            ));
        }
        let commit_len = prev_len + 1;
        for node in nodes.values_mut() {
            if node.log.len() >= commit_len {
                node.commit_len = commit_len;
                apply_commits(node)?;
            }
        }
        Ok(commit_len as u64)
    }

    pub fn state(&self) -> Option<ReplicatedState> {
        let nodes = self.nodes.read();
        nodes
            .values()
            .find(|node| node.role == RaftRole::Leader)
            .or_else(|| nodes.values().next())
            .map(|node| node.state.clone())
    }

    pub fn node_state(&self, node_id: &str) -> Option<ReplicatedState> {
        self.nodes
            .read()
            .get(node_id)
            .map(|node| node.state.clone())
    }
}
