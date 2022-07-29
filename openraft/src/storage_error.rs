use std::fmt::Formatter;
use std::ops::Bound;

use anyerror::AnyError;

use crate::LogId;
use crate::NodeType;
use crate::SnapshotMeta;
use crate::Vote;

/// Convert error to StorageError::IO();
pub trait ToStorageResult<NT, T>
where NT: NodeType
{
    /// Convert Result<T, E> to Result<T, StorageError::IO(StorageIOError)>
    ///
    /// `f` provides error context for building the StorageIOError.
    fn sto_res<F>(self, f: F) -> Result<T, StorageError<NT>>
    where F: FnOnce() -> (ErrorSubject<NT>, ErrorVerb);
}

impl<NT, T> ToStorageResult<NT, T> for Result<T, std::io::Error>
where NT: NodeType
{
    fn sto_res<F>(self, f: F) -> Result<T, StorageError<NT>>
    where F: FnOnce() -> (ErrorSubject<NT>, ErrorVerb) {
        match self {
            Ok(x) => Ok(x),
            Err(e) => {
                let (subject, verb) = f();
                let io_err = StorageIOError::new(subject, verb, AnyError::new(&e));
                Err(io_err.into())
            }
        }
    }
}

/// An error that occurs when the RaftStore impl runs defensive check of input or output.
/// E.g. re-applying an log entry is a violation that may be a potential bug.
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize), serde(bound = ""))]
pub struct DefensiveError<NT>
where NT: NodeType
{
    /// The subject that violates store defensive check, e.g. hard-state, log or state machine.
    pub subject: ErrorSubject<NT>,

    /// The description of the violation.
    pub violation: Violation<NT>,

    pub backtrace: Option<String>,
}

impl<NT> DefensiveError<NT>
where NT: NodeType
{
    pub fn new(subject: ErrorSubject<NT>, violation: Violation<NT>) -> Self {
        Self {
            subject,
            violation,
            backtrace: anyerror::backtrace_str(),
        }
    }
}

impl<NT> std::fmt::Display for DefensiveError<NT>
where NT: NodeType
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "'{:?}' violates: '{}'", self.subject, self.violation)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize), serde(bound = ""))]
pub enum ErrorSubject<NT>
where NT: NodeType
{
    /// A general storage error
    Store,

    /// HardState related error.
    Vote,

    /// Error that is happened when operating a series of log entries
    Logs,

    /// Error about a single log entry
    Log(LogId<NT::NodeId>),

    /// Error about a single log entry without knowing the log term.
    LogIndex(u64),

    /// Error happened when applying a log entry
    Apply(LogId<NT::NodeId>),

    /// Error happened when operating state machine.
    StateMachine,

    /// Error happened when operating snapshot.
    Snapshot(SnapshotMeta<NT>),

    None,
}

/// What it is doing when an error occurs.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum ErrorVerb {
    Read,
    Write,
    Seek,
    Delete,
}

/// Violations a store would return when running defensive check.
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize), serde(bound = ""))]
pub enum Violation<NT: NodeType> {
    #[error("term can only be change to a greater value, current: {curr}, change to {to}")]
    TermNotAscending { curr: u64, to: u64 },

    #[error("voted_for can not change from Some() to other Some(), current: {curr:?}, change to {to:?}")]
    NonIncrementalVote {
        curr: Vote<NT::NodeId>,
        to: Vote<NT::NodeId>,
    },

    #[error("log at higher index is obsolete: {higher_index_log_id:?} should GT {lower_index_log_id:?}")]
    DirtyLog {
        higher_index_log_id: LogId<NT::NodeId>,
        lower_index_log_id: LogId<NT::NodeId>,
    },

    #[error("try to get log at index {want} but got {got:?}")]
    LogIndexNotFound { want: u64, got: Option<u64> },

    #[error("range is empty: start: {start:?}, end: {end:?}")]
    RangeEmpty { start: Option<u64>, end: Option<u64> },

    #[error("range is not half-open: start: {start:?}, end: {end:?}")]
    RangeNotHalfOpen { start: Bound<u64>, end: Bound<u64> },

    // TODO(xp): rename this to some input related error name.
    #[error("empty log vector")]
    LogsEmpty,

    #[error("all logs are removed. It requires at least one log to track continuity")]
    StoreLogsEmpty,

    #[error("logs are not consecutive, prev: {prev:?}, next: {next}")]
    LogsNonConsecutive {
        prev: Option<LogId<NT::NodeId>>,
        next: LogId<NT::NodeId>,
    },

    #[error("invalid next log to apply: prev: {prev:?}, next: {next}")]
    ApplyNonConsecutive {
        prev: Option<LogId<NT::NodeId>>,
        next: LogId<NT::NodeId>,
    },

    #[error("applied log can not conflict, last_applied: {last_applied:?}, delete since: {first_conflict_log_id}")]
    AppliedWontConflict {
        last_applied: Option<LogId<NT::NodeId>>,
        first_conflict_log_id: LogId<NT::NodeId>,
    },

    #[error("not allowed to purge non-applied logs, last_applied: {last_applied:?}, purge upto: {purge_upto}")]
    PurgeNonApplied {
        last_applied: Option<LogId<NT::NodeId>>,
        purge_upto: LogId<NT::NodeId>,
    },
}

/// A storage error could be either a defensive check error or an error occurred when doing the actual io operation.
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize), serde(bound = ""))]
pub enum StorageError<NT>
where NT: NodeType
{
    /// An error raised by defensive check.
    #[error(transparent)]
    Defensive {
        #[from]
        #[cfg_attr(feature = "bt", backtrace)]
        source: DefensiveError<NT>,
    },

    /// An error raised by io operation.
    #[error(transparent)]
    IO {
        #[from]
        #[cfg_attr(feature = "bt", backtrace)]
        source: StorageIOError<NT>,
    },
}

impl<NT> StorageError<NT>
where NT: NodeType
{
    pub fn into_defensive(self) -> Option<DefensiveError<NT>> {
        match self {
            StorageError::Defensive { source } => Some(source),
            _ => None,
        }
    }

    pub fn into_io(self) -> Option<StorageIOError<NT>> {
        match self {
            StorageError::IO { source } => Some(source),
            _ => None,
        }
    }

    pub fn from_io_error(subject: ErrorSubject<NT>, verb: ErrorVerb, io_error: std::io::Error) -> Self {
        let sto_io_err = StorageIOError::new(subject, verb, AnyError::new(&io_error));
        StorageError::IO { source: sto_io_err }
    }
}

/// Error that occurs when operating the store.
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize), serde(bound = ""))]
pub struct StorageIOError<NT>
where NT: NodeType
{
    subject: ErrorSubject<NT>,
    verb: ErrorVerb,
    source: AnyError,
    backtrace: Option<String>,
}

impl<NT> std::fmt::Display for StorageIOError<NT>
where NT: NodeType
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "when {:?} {:?}: {}", self.verb, self.subject, self.source)
    }
}

impl<NT> StorageIOError<NT>
where NT: NodeType
{
    pub fn new(subject: ErrorSubject<NT>, verb: ErrorVerb, source: AnyError) -> Self {
        Self {
            subject,
            verb,
            source,
            backtrace: anyerror::backtrace_str(),
        }
    }
}
