//! Error types exposed by this crate.

use std::collections::BTreeSet;
use std::error::Error;
use std::fmt::Debug;
use std::time::Duration;

use anyerror::AnyError;

use crate::raft::AppendEntriesResponse;
use crate::raft_types::SnapshotSegmentId;
use crate::LogId;
use crate::Membership;
use crate::Node;
use crate::NodeId;
use crate::NodeType;
use crate::RPCTypes;
use crate::StorageError;
use crate::Vote;

/// Fatal is unrecoverable and shuts down raft at once.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize), serde(bound = ""))]
pub enum Fatal<NT>
where NT: NodeType
{
    #[error(transparent)]
    StorageError(#[from] StorageError<NT>),

    #[error("panicked")]
    Panicked,

    #[error("raft stopped")]
    Stopped,
}

/// Extract Fatal from a Result.
///
/// Fatal will shutdown the raft and needs to be dealt separately,
/// such as StorageError.
pub trait ExtractFatal<NT>
where
    Self: Sized,
    NT: NodeType,
{
    fn extract_fatal(self) -> Result<Self, Fatal<NT>>;
}

impl<NT, T, E> ExtractFatal<NT> for Result<T, E>
where
    NT: NodeType,
    E: TryInto<Fatal<NT>> + Clone,
{
    fn extract_fatal(self) -> Result<Self, Fatal<NT>> {
        if let Err(e) = &self {
            let fatal = e.clone().try_into();
            if let Ok(f) = fatal {
                return Err(f);
            }
        }
        Ok(self)
    }
}

// TODO: not used, remove
#[derive(Debug, Clone, thiserror::Error, derive_more::TryInto)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize), serde(bound = ""))]
pub enum AppendEntriesError<NT>
where NT: NodeType
{
    #[error(transparent)]
    Fatal(#[from] Fatal<NT>),
}

// TODO: not used, remove
#[derive(Debug, Clone, thiserror::Error, derive_more::TryInto)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize), serde(bound = ""))]
pub enum VoteError<NT>
where NT: NodeType
{
    #[error(transparent)]
    Fatal(#[from] Fatal<NT>),
}

// TODO: remove
#[derive(Debug, Clone, thiserror::Error, derive_more::TryInto)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize), serde(bound = ""))]
pub enum InstallSnapshotError<NT>
where NT: NodeType
{
    #[error(transparent)]
    SnapshotMismatch(#[from] SnapshotMismatch),

    #[error(transparent)]
    Fatal(#[from] Fatal<NT>),
}

/// An error related to a is_leader request.
#[derive(Debug, Clone, thiserror::Error, derive_more::TryInto)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize), serde(bound = ""))]
pub enum CheckIsLeaderError<NT>
where NT: NodeType
{
    #[error(transparent)]
    ForwardToLeader(#[from] ForwardToLeader<NT>),

    #[error(transparent)]
    QuorumNotEnough(#[from] QuorumNotEnough<NT>),

    #[error(transparent)]
    Fatal(#[from] Fatal<NT>),
}

/// An error related to a client write request.
#[derive(Debug, Clone, thiserror::Error, derive_more::TryInto)]
#[derive(PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize), serde(bound = ""))]
pub enum ClientWriteError<NT>
where NT: NodeType
{
    #[error(transparent)]
    ForwardToLeader(#[from] ForwardToLeader<NT>),

    /// When writing a change-membership entry.
    #[error(transparent)]
    ChangeMembershipError(#[from] ChangeMembershipError<NT>),

    #[error(transparent)]
    Fatal(#[from] Fatal<NT>),
}

/// The set of errors which may take place when requesting to propose a config change.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize), serde(bound = ""))]
pub enum ChangeMembershipError<NT: NodeType> {
    #[error(transparent)]
    InProgress(#[from] InProgress<NT>),

    #[error(transparent)]
    EmptyMembership(#[from] EmptyMembership),

    #[error(transparent)]
    LearnerNotFound(#[from] LearnerNotFound<NT>),

    #[error(transparent)]
    LearnerIsLagging(#[from] LearnerIsLagging<NT>),

    #[error(transparent)]
    MissingNodeInfo(#[from] MissingNodeInfo<NT>),
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize), serde(bound = ""))]
pub enum AddLearnerError<NT>
where NT: NodeType
{
    #[error(transparent)]
    ForwardToLeader(#[from] ForwardToLeader<NT>),

    #[error(transparent)]
    MissingNodeInfo(#[from] MissingNodeInfo<NT>),

    #[error(transparent)]
    Fatal(#[from] Fatal<NT>),
}

impl<NT> TryFrom<AddLearnerError<NT>> for ForwardToLeader<NT>
where NT: NodeType
{
    type Error = AddLearnerError<NT>;

    fn try_from(value: AddLearnerError<NT>) -> Result<Self, Self::Error> {
        if let AddLearnerError::ForwardToLeader(e) = value {
            return Ok(e);
        }
        Err(value)
    }
}

/// The set of errors which may take place when initializing a pristine Raft node.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, derive_more::TryInto)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize), serde(bound = ""))]
pub enum InitializeError<NT>
where NT: NodeType
{
    #[error(transparent)]
    NotAllowed(#[from] NotAllowed<NT>),

    #[error(transparent)]
    NotInMembers(#[from] NotInMembers<NT>),

    #[error(transparent)]
    NotAMembershipEntry(#[from] NotAMembershipEntry),

    #[error(transparent)]
    MissingNodeInfo(#[from] MissingNodeInfo<NT>),

    #[error(transparent)]
    Fatal(#[from] Fatal<NT>),
}

impl<NT> From<StorageError<NT>> for AppendEntriesError<NT>
where NT: NodeType
{
    fn from(s: StorageError<NT>) -> Self {
        let f: Fatal<NT> = s.into();
        f.into()
    }
}
impl<NT> From<StorageError<NT>> for VoteError<NT>
where NT: NodeType
{
    fn from(s: StorageError<NT>) -> Self {
        let f: Fatal<NT> = s.into();
        f.into()
    }
}
impl<NT> From<StorageError<NT>> for InstallSnapshotError<NT>
where NT: NodeType
{
    fn from(s: StorageError<NT>) -> Self {
        let f: Fatal<NT> = s.into();
        f.into()
    }
}
impl<NT> From<StorageError<NT>> for CheckIsLeaderError<NT>
where NT: NodeType
{
    fn from(s: StorageError<NT>) -> Self {
        let f: Fatal<NT> = s.into();
        f.into()
    }
}
impl<NT> From<StorageError<NT>> for InitializeError<NT>
where NT: NodeType
{
    fn from(s: StorageError<NT>) -> Self {
        let f: Fatal<NT> = s.into();
        f.into()
    }
}
impl<NT> From<StorageError<NT>> for AddLearnerError<NT>
where NT: NodeType
{
    fn from(s: StorageError<NT>) -> Self {
        let f: Fatal<NT> = s.into();
        f.into()
    }
}

/// Error variants related to the Replication.
#[derive(Debug, thiserror::Error)]
#[allow(clippy::large_enum_variant)]
pub enum ReplicationError<NT>
where NT: NodeType
{
    #[error(transparent)]
    HigherVote(#[from] HigherVote<NT::NodeId>),

    #[error("Replication is closed")]
    Closed,

    #[error(transparent)]
    LackEntry(#[from] LackEntry<NT>),

    #[error(transparent)]
    CommittedAdvanceTooMany(#[from] CommittedAdvanceTooMany),

    // TODO(xp): two sub type: StorageError / TransportError
    // TODO(xp): a sub error for just send_append_entries()
    #[error(transparent)]
    StorageError(#[from] StorageError<NT>),

    #[error(transparent)]
    NodeNotFound(#[from] NodeNotFound<NT>),

    #[error(transparent)]
    Timeout(#[from] Timeout<NT>),

    #[error(transparent)]
    Network(#[from] NetworkError),

    #[error(transparent)]
    RemoteError(#[from] RemoteError<NT, AppendEntriesError<NT>>),
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Deserialize, serde::Serialize),
    serde(bound = "T:serde::Serialize + for <'d> serde::Deserialize<'d>")
)]
pub enum RPCError<NT: NodeType, T: Error> {
    #[error(transparent)]
    NodeNotFound(#[from] NodeNotFound<NT>),

    #[error(transparent)]
    Timeout(#[from] Timeout<NT>),

    #[error(transparent)]
    Network(#[from] NetworkError),

    #[error(transparent)]
    RemoteError(#[from] RemoteError<NT, T>),
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[error("error occur on remote peer {target}: {source}")]
pub struct RemoteError<NT: NodeType, T: std::error::Error> {
    // #[serde(bound = "")]
    #[cfg_attr(feature = "serde", serde(bound = ""))]
    pub target: NT::NodeId,
    #[cfg_attr(feature = "serde", serde(bound = ""))]
    pub target_node: Option<Node<NT>>,
    pub source: T,
}

impl<NT: NodeType, T: std::error::Error> RemoteError<NT, T> {
    pub fn new(target: NT::NodeId, e: T) -> Self {
        Self {
            target,
            target_node: None,
            source: e,
        }
    }
    pub fn new_with_node(target: NT::NodeId, node: Node<NT>, e: T) -> Self {
        Self {
            target,
            target_node: Some(node),
            source: e,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize), serde(bound = ""))]
#[error("seen a higher vote: {higher} GT mine: {mine}")]
pub struct HigherVote<NID: NodeId> {
    pub higher: Vote<NID>,
    pub mine: Vote<NID>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize), serde(bound = ""))]
#[error("leader committed index {committed_index} advances target log index {target_index} too many")]
pub struct CommittedAdvanceTooMany {
    pub committed_index: u64,
    pub target_index: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize), serde(bound = ""))]
#[error("NetworkError: {source}")]
pub struct NetworkError {
    #[from]
    source: AnyError,
}

impl NetworkError {
    pub fn new<E: Error + 'static>(e: &E) -> Self {
        Self {
            source: AnyError::new(e),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize), serde(bound = ""))]
#[error("timeout after {timeout:?} when {action} {id}->{target}")]
pub struct Timeout<NT: NodeType> {
    pub action: RPCTypes,
    pub id: NT::NodeId,
    pub target: NT::NodeId,
    pub timeout: Duration,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize), serde(bound = ""))]
#[error("store has no log at: {index:?}, last purged: {last_purged_log_id:?}")]
pub struct LackEntry<NT: NodeType> {
    pub index: Option<u64>,
    pub last_purged_log_id: Option<LogId<NT::NodeId>>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize), serde(bound = ""))]
#[error("has to forward request to: {leader_id:?}, {leader_node:?}")]
pub struct ForwardToLeader<NT>
where NT: NodeType
{
    pub leader_id: Option<NT::NodeId>,
    pub leader_node: Option<Node<NT>>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize), serde(bound = ""))]
#[error("snapshot segment id mismatch, expect: {expect}, got: {got}")]
pub struct SnapshotMismatch {
    pub expect: SnapshotSegmentId,
    pub got: SnapshotSegmentId,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize), serde(bound = ""))]
#[error("not enough for a quorum, cluster: {cluster}, got: {got:?}")]
pub struct QuorumNotEnough<NT: NodeType> {
    pub cluster: String,
    pub got: BTreeSet<NT::NodeId>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize), serde(bound = ""))]
#[error("the cluster is already undergoing a configuration change at log {membership_log_id:?}, committed log id: {committed:?}")]
pub struct InProgress<NT: NodeType> {
    pub committed: Option<LogId<NT::NodeId>>,
    pub membership_log_id: Option<LogId<NT::NodeId>>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize), serde(bound = ""))]
#[error("Learner {node_id} not found: add it as learner before adding it as a voter")]
pub struct LearnerNotFound<NT: NodeType> {
    pub node_id: NT::NodeId,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize), serde(bound = ""))]
#[error("replication to learner {node_id} is lagging {distance}, matched: {matched:?}, can not add as member")]
pub struct LearnerIsLagging<NT: NodeType> {
    pub node_id: NT::NodeId,
    pub matched: Option<LogId<NT::NodeId>>,
    pub distance: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize), serde(bound = ""))]
#[error("not allowed to initialize due to current raft state: last_log_id: {last_log_id:?} vote: {vote}")]
pub struct NotAllowed<NT: NodeType> {
    pub last_log_id: Option<LogId<NT::NodeId>>,
    pub vote: Vote<NT::NodeId>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize), serde(bound = ""))]
#[error("node {node_id} {reason}")]
pub struct MissingNodeInfo<NT: NodeType> {
    pub node_id: NT::NodeId,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize), serde(bound = ""))]
#[error("node {node_id} has to be a member. membership:{membership:?}")]
pub struct NotInMembers<NT>
where NT: NodeType
{
    pub node_id: NT::NodeId,
    pub membership: Membership<NT>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize), serde(bound = ""))]
#[error("initializing log entry has to be a membership config entry")]
pub struct NotAMembershipEntry {}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[error("new membership can not be empty")]
pub struct EmptyMembership {}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize), serde(bound = ""))]
#[error("node not found: {node_id}, source: {source}")]
pub struct NodeNotFound<NT: NodeType> {
    pub node_id: NT::NodeId,
    pub source: AnyError,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[error("infallible")]
pub enum Infallible {}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize), serde(bound = ""))]
pub(crate) enum RejectVoteRequest<NT: NodeType> {
    #[error("reject vote request by a greater vote: {0}")]
    ByVote(Vote<NT::NodeId>),

    #[error("reject vote request by a greater last-log-id: {0:?}")]
    ByLastLogId(Option<LogId<NT::NodeId>>),
}

impl<NT: NodeType> From<RejectVoteRequest<NT>> for AppendEntriesResponse<NT> {
    fn from(r: RejectVoteRequest<NT>) -> Self {
        match r {
            RejectVoteRequest::ByVote(v) => AppendEntriesResponse::HigherVote(v),
            RejectVoteRequest::ByLastLogId(_) => {
                unreachable!("the leader should always has a greater last log id")
            }
        }
    }
}
