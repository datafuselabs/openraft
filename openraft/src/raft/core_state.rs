use crate::error::Fatal;
use crate::type_config::alias::JoinHandleOf;
use crate::RaftTypeConfig;

/// The running state of RaftCore
pub(in crate::raft) enum CoreState<C>
where C: RaftTypeConfig
{
    /// The RaftCore task is still running.
    Running(JoinHandleOf<C, Result<(), Fatal<C>>>),

    /// The RaftCore task has finished. The return value of the task is stored.
    Done(Result<(), Fatal<C>>),
}
