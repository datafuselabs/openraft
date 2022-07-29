use std::collections::BTreeSet;

use crate::NodeType;

#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize), serde(bound = ""))]
pub enum ChangeMembers<NT: NodeType> {
    Add(BTreeSet<NT::NodeId>),
    Remove(BTreeSet<NT::NodeId>),
    Replace(BTreeSet<NT::NodeId>),
}

/// Convert a series of ids to a `Replace` operation.
impl<NT, I> From<I> for ChangeMembers<NT>
where
    NT: NodeType,
    I: IntoIterator<Item = NT::NodeId>,
{
    fn from(r: I) -> Self {
        let ids = r.into_iter().collect::<BTreeSet<NT::NodeId>>();
        ChangeMembers::Replace(ids)
    }
}

impl<NT: NodeType> ChangeMembers<NT> {
    /// Apply the `ChangeMembers` to `old` node set, return new node set
    pub fn apply_to(self, old: &BTreeSet<NT::NodeId>) -> BTreeSet<NT::NodeId> {
        match self {
            ChangeMembers::Replace(c) => c,
            ChangeMembers::Add(add_members) => old.union(&add_members).cloned().collect::<BTreeSet<_>>(),
            ChangeMembers::Remove(remove_members) => old.difference(&remove_members).cloned().collect::<BTreeSet<_>>(),
        }
    }
}
