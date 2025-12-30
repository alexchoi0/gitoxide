#[coverage(off)]
mod refs;
mod objects;
#[coverage(off)]
mod trees;
#[coverage(off)]
mod commits;
#[coverage(off)]
mod blame;
#[coverage(off)]
mod diff;
mod submodule;
#[coverage(off)]
mod attributes;
#[coverage(off)]
mod archive;
#[coverage(off)]
mod grep;

pub use refs::*;
pub use objects::*;
pub use trees::*;
pub use commits::*;
pub use blame::*;
pub use diff::*;
pub use submodule::*;
pub use attributes::*;
pub use archive::*;
pub use grep::*;

pub use crate::types::{
    ChangeKind, DiffEntry, BlobDiff, DiffHunk, DiffLine, DiffLineKind, DiffStats, FileStats,
    EntryMode, TreeEntry, RefInfo, ObjectInfo, ObjectData, ObjectKind, CommitInfo, Signature,
    BlameResult, BlameEntry, BlameStatistics, BlameOptions,
};
