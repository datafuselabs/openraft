//! Test the `declare_raft_types` macro with default values

use std::io::Cursor;

use crate::declare_raft_types;
use crate::TokioRuntime;

declare_raft_types!(
    All:
        D = (),
        R = (),
        NodeId = u64,
        Node = (),
        Entry = crate::Entry<Self>,
        SnapshotData = Cursor<Vec<u8>>,
        AsyncRuntime = TokioRuntime,
);

declare_raft_types!(
    WithoutD:
        R = (),
        NodeId = u64,
        Node = (),
        Entry = crate::Entry<Self>,
        SnapshotData = Cursor<Vec<u8>>,
        AsyncRuntime = TokioRuntime,
);

declare_raft_types!(
    WithoutR:
        D = (),
        NodeId = u64,
        Node = (),
        Entry = crate::Entry<Self>,
        SnapshotData = Cursor<Vec<u8>>,
        AsyncRuntime = TokioRuntime,
);

declare_raft_types!(
    NoneWithColon:
);

declare_raft_types!(None);
