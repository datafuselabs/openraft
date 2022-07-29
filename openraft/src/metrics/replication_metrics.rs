use std::collections::BTreeMap;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use crate::versioned::Update;
use crate::versioned::UpdateError;
use crate::LeaderId;
use crate::LogId;
use crate::MessageSummary;
use crate::NodeType;

/// The metrics about the leader. It is Some() only when this node is leader.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize), serde(bound = ""))]
pub struct ReplicationMetrics<NT: NodeType> {
    /// Replication metrics of all known replication target: voters and learners
    pub replication: BTreeMap<NT::NodeId, ReplicationTargetMetrics<NT>>,
}

impl<NT: NodeType> MessageSummary<ReplicationMetrics<NT>> for ReplicationMetrics<NT> {
    fn summary(&self) -> String {
        let mut res = vec!["LeaderMetrics{".to_string()];
        for (i, (k, v)) in self.replication.iter().enumerate() {
            if i > 0 {
                res.push(", ".to_string());
            }
            res.push(format!("{}:{}", k, v.summary()));
        }

        res.push("}".to_string());
        res.join("")
    }
}

/// Update one replication metrics in `LeaderMetrics.replication`.
pub(crate) struct UpdateMatchedLogId<NT: NodeType> {
    pub target: NT::NodeId,
    pub matched: LogId<NT::NodeId>,
}

impl<NT: NodeType> Update<ReplicationMetrics<NT>> for UpdateMatchedLogId<NT> {
    /// If there is already a record for the target node. Just modify the atomic u64.
    fn apply_in_place(&self, to: &Arc<ReplicationMetrics<NT>>) -> Result<(), UpdateError> {
        let target_metrics = to.replication.get(&self.target).ok_or(UpdateError::CanNotUpdateInPlace)?;

        if target_metrics.matched_leader_id == self.matched.leader_id {
            target_metrics.matched_index.store(self.matched.index, Ordering::Relaxed);
            return Ok(());
        }

        Err(UpdateError::CanNotUpdateInPlace)
    }

    /// To insert a new record always work.
    fn apply_mut(&self, to: &mut ReplicationMetrics<NT>) {
        to.replication.insert(self.target, ReplicationTargetMetrics {
            matched_leader_id: self.matched.leader_id,
            matched_index: AtomicU64::new(self.matched.index),
        });
    }
}

/// Remove one replication metrics in `LeaderMetrics.replication`.
pub(crate) struct RemoveTarget<NT: NodeType> {
    pub target: NT::NodeId,
}

impl<NT: NodeType> Update<ReplicationMetrics<NT>> for RemoveTarget<NT> {
    /// Removing can not be done in place
    fn apply_in_place(&self, _to: &Arc<ReplicationMetrics<NT>>) -> Result<(), UpdateError> {
        Err(UpdateError::CanNotUpdateInPlace)
    }

    fn apply_mut(&self, to: &mut ReplicationMetrics<NT>) {
        to.replication.remove(&self.target);
    }
}

#[derive(Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize), serde(bound = ""))]
pub struct ReplicationTargetMetrics<NT: NodeType> {
    pub(crate) matched_leader_id: LeaderId<NT::NodeId>,
    pub(crate) matched_index: AtomicU64,
}

impl<NT: NodeType> Clone for ReplicationTargetMetrics<NT> {
    fn clone(&self) -> Self {
        Self {
            matched_leader_id: self.matched_leader_id,
            matched_index: AtomicU64::new(self.matched_index.load(Ordering::Relaxed)),
        }
    }
}

impl<NT: NodeType> PartialEq for ReplicationTargetMetrics<NT> {
    fn eq(&self, other: &Self) -> bool {
        self.matched_leader_id == other.matched_leader_id
            && self.matched_index.load(Ordering::Relaxed) == other.matched_index.load(Ordering::Relaxed)
    }
}

impl<NT: NodeType> Eq for ReplicationTargetMetrics<NT> {}

impl<NT: NodeType> ReplicationTargetMetrics<NT> {
    pub fn new(log_id: LogId<NT::NodeId>) -> Self {
        Self {
            matched_leader_id: log_id.leader_id,
            matched_index: AtomicU64::new(log_id.index),
        }
    }

    pub fn matched(&self) -> LogId<NT::NodeId> {
        let index = self.matched_index.load(Ordering::Relaxed);
        LogId {
            leader_id: self.matched_leader_id,
            index,
        }
    }
}

impl<NT: NodeType> MessageSummary<ReplicationTargetMetrics<NT>> for ReplicationTargetMetrics<NT> {
    fn summary(&self) -> String {
        format!("{}", self.matched())
    }
}
