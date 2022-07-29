mod store_builder;
mod suite;

pub use store_builder::DefensiveStoreBuilder;
pub use store_builder::StoreBuilder;
pub use suite::Suite;

use crate::NodeType;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub(crate) struct DummyNodeType {}
impl NodeType for DummyNodeType {
    type NodeId = u64;
    type NodeData = ();
}
crate::declare_raft_types!(
    /// Dummy Raft types for the purpose of testing internal structures requiring
    /// `RaftTypeConfig`, like `MembershipConfig`.
    pub(crate) DummyConfig: D = u64, R = u64, NodeType = DummyNodeType
);
