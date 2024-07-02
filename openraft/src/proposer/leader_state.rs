use crate::alias::NodeIdOf;
use crate::proposer::Leader;
use crate::quorum::Joint;

/// The quorum set type used by `Leader`.
pub(crate) type LeaderQuorumSet<NID> = Joint<NID, Vec<NID>, Vec<Vec<NID>>>;

pub(crate) type LeaderState<C> = Option<Box<Leader<C, LeaderQuorumSet<NodeIdOf<C>>>>>;
