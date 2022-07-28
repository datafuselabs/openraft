use std::sync::Arc;

use crate::core::ServerState;
use crate::error::Fatal;
use crate::membership::EffectiveMembership;
use crate::metrics::ReplicationMetrics;
use crate::node::NodeData;
use crate::summary::MessageSummary;
use crate::versioned::Versioned;
use crate::LogId;
use crate::NodeId;

/// A set of metrics describing the current state of a Raft node.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize), serde(bound = ""))]
pub struct RaftMetrics<NID, ND>
where
    NID: NodeId,
    ND: NodeData,
{
    pub running_state: Result<(), Fatal<NID, ND>>,

    /// The ID of the Raft node.
    pub id: NID,

    // ---
    // --- data ---
    // ---
    /// The current term of the Raft node.
    pub current_term: u64,

    /// The last log index has been appended to this Raft node's log.
    pub last_log_index: Option<u64>,

    /// The last log index has been applied to this Raft node's state machine.
    pub last_applied: Option<LogId<NID>>,

    /// The id of the last log included in snapshot.
    /// If there is no snapshot, it is (0,0).
    pub snapshot: Option<LogId<NID>>,

    // ---
    // --- cluster ---
    // ---
    /// The state of the Raft node.
    pub state: ServerState,

    /// The current cluster leader.
    pub current_leader: Option<NID>,

    /// The current membership config of the cluster.
    pub membership_config: Arc<EffectiveMembership<NID, ND>>,

    // ---
    // --- replication ---
    // ---
    /// The metrics about the leader. It is Some() only when this node is leader.
    pub replication: Option<Versioned<ReplicationMetrics<NID>>>,
}

impl<NID, ND> MessageSummary<RaftMetrics<NID, ND>> for RaftMetrics<NID, ND>
where
    NID: NodeId,
    ND: NodeData,
{
    fn summary(&self) -> String {
        format!("Metrics{{id:{},{:?}, term:{}, last_log:{:?}, last_applied:{:?}, leader:{:?}, membership:{}, snapshot:{:?}, replication:{}",
                self.id,
                self.state,
                self.current_term,
                self.last_log_index,
                self.last_applied,
                self.current_leader,
                self.membership_config.summary(),
                self.snapshot,
                self.replication.as_ref().map(|x| x.summary()).unwrap_or_default(),
        )
    }
}

impl<NID, ND> RaftMetrics<NID, ND>
where
    NID: NodeId,
    ND: NodeData,
{
    pub fn new_initial(id: NID) -> Self {
        Self {
            running_state: Ok(()),
            id,
            state: ServerState::Follower,
            current_term: 0,
            last_log_index: None,
            last_applied: None,
            current_leader: None,
            membership_config: Arc::new(EffectiveMembership::default()),
            snapshot: None,
            replication: None,
        }
    }
}
