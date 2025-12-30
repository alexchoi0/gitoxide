use bstr::BString;
use gix_hash::ObjectId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefInfo {
    pub name: String,
    pub target: ObjectId,
    pub is_symbolic: bool,
    pub symbolic_target: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectInfo {
    pub id: ObjectId,
    pub kind: ObjectKind,
    pub size: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectData {
    pub id: ObjectId,
    pub kind: ObjectKind,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObjectKind {
    Blob,
    Tree,
    Commit,
    Tag,
}

impl std::fmt::Display for ObjectKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ObjectKind::Blob => write!(f, "blob"),
            ObjectKind::Tree => write!(f, "tree"),
            ObjectKind::Commit => write!(f, "commit"),
            ObjectKind::Tag => write!(f, "tag"),
        }
    }
}

impl From<gix_object::Kind> for ObjectKind {
    fn from(kind: gix_object::Kind) -> Self {
        match kind {
            gix_object::Kind::Blob => ObjectKind::Blob,
            gix_object::Kind::Tree => ObjectKind::Tree,
            gix_object::Kind::Commit => ObjectKind::Commit,
            gix_object::Kind::Tag => ObjectKind::Tag,
        }
    }
}

impl From<ObjectKind> for gix_object::Kind {
    fn from(kind: ObjectKind) -> Self {
        match kind {
            ObjectKind::Blob => gix_object::Kind::Blob,
            ObjectKind::Tree => gix_object::Kind::Tree,
            ObjectKind::Commit => gix_object::Kind::Commit,
            ObjectKind::Tag => gix_object::Kind::Tag,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeEntry {
    pub name: BString,
    pub id: ObjectId,
    pub mode: EntryMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntryMode {
    Blob,
    BlobExecutable,
    Tree,
    Link,
    Commit,
}

impl std::fmt::Display for EntryMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntryMode::Blob => write!(f, "blob"),
            EntryMode::BlobExecutable => write!(f, "blob (executable)"),
            EntryMode::Tree => write!(f, "tree"),
            EntryMode::Link => write!(f, "link"),
            EntryMode::Commit => write!(f, "commit"),
        }
    }
}

impl From<gix_object::tree::EntryMode> for EntryMode {
    fn from(mode: gix_object::tree::EntryMode) -> Self {
        use gix_object::tree::EntryKind;
        match mode.kind() {
            EntryKind::Blob => EntryMode::Blob,
            EntryKind::BlobExecutable => EntryMode::BlobExecutable,
            EntryKind::Tree => EntryMode::Tree,
            EntryKind::Link => EntryMode::Link,
            EntryKind::Commit => EntryMode::Commit,
        }
    }
}

impl From<EntryMode> for gix_object::tree::EntryMode {
    fn from(mode: EntryMode) -> Self {
        use gix_object::tree::EntryKind;
        match mode {
            EntryMode::Blob => EntryKind::Blob.into(),
            EntryMode::BlobExecutable => EntryKind::BlobExecutable.into(),
            EntryMode::Tree => EntryKind::Tree.into(),
            EntryMode::Link => EntryKind::Link.into(),
            EntryMode::Commit => EntryKind::Commit.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitInfo {
    pub id: ObjectId,
    pub tree_id: ObjectId,
    pub parent_ids: Vec<ObjectId>,
    pub author: Signature,
    pub committer: Signature,
    pub message: BString,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signature {
    pub name: BString,
    pub email: BString,
    pub time: i64,
}

impl From<gix_actor::SignatureRef<'_>> for Signature {
    fn from(sig: gix_actor::SignatureRef<'_>) -> Self {
        let time_seconds = sig.time().map(|t| t.seconds).unwrap_or(0);
        Signature {
            name: sig.name.into(),
            email: sig.email.into(),
            time: time_seconds,
        }
    }
}

impl From<gix_actor::Signature> for Signature {
    fn from(sig: gix_actor::Signature) -> Self {
        Signature {
            name: sig.name,
            email: sig.email,
            time: sig.time.seconds,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PoolStats {
    pub cached_count: usize,
    pub open_count: usize,
    pub hit_count: usize,
    pub hit_rate: f64,
}

// === Diff Types ===

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChangeKind {
    Added,
    Deleted,
    Modified,
}

impl std::fmt::Display for ChangeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChangeKind::Added => write!(f, "added"),
            ChangeKind::Deleted => write!(f, "deleted"),
            ChangeKind::Modified => write!(f, "modified"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffEntry {
    pub path: BString,
    pub change: ChangeKind,
    pub old_mode: Option<EntryMode>,
    pub new_mode: Option<EntryMode>,
    pub old_id: Option<ObjectId>,
    pub new_id: Option<ObjectId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlobDiff {
    pub old_id: ObjectId,
    pub new_id: ObjectId,
    pub hunks: Vec<DiffHunk>,
    pub additions: u32,
    pub deletions: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffHunk {
    pub old_start: u32,
    pub old_lines: u32,
    pub new_start: u32,
    pub new_lines: u32,
    pub lines: Vec<DiffLine>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiffLineKind {
    Context,
    Addition,
    Deletion,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffLine {
    pub kind: DiffLineKind,
    pub content: BString,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DiffStats {
    pub files_changed: usize,
    pub additions: u32,
    pub deletions: u32,
    pub entries: Vec<FileStats>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileStats {
    pub path: BString,
    pub additions: u32,
    pub deletions: u32,
}

// === Blame Types ===

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlameResult {
    pub entries: Vec<BlameEntry>,
    pub lines: Vec<BString>,
    pub statistics: BlameStatistics,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlameEntry {
    pub commit_id: ObjectId,
    pub start_line: u32,
    pub line_count: u32,
    pub original_start_line: u32,
    pub original_path: Option<BString>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BlameStatistics {
    pub commits_traversed: usize,
    pub blobs_diffed: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlameOptions {
    pub range: Option<(u32, u32)>,
    pub follow_renames: bool,
}

impl Default for BlameOptions {
    fn default() -> Self {
        BlameOptions {
            range: None,
            follow_renames: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_kind_display() {
        assert_eq!(ObjectKind::Blob.to_string(), "blob");
        assert_eq!(ObjectKind::Tree.to_string(), "tree");
        assert_eq!(ObjectKind::Commit.to_string(), "commit");
        assert_eq!(ObjectKind::Tag.to_string(), "tag");
    }

    #[test]
    fn test_entry_mode_display() {
        assert_eq!(EntryMode::Blob.to_string(), "blob");
        assert_eq!(EntryMode::BlobExecutable.to_string(), "blob (executable)");
        assert_eq!(EntryMode::Tree.to_string(), "tree");
        assert_eq!(EntryMode::Link.to_string(), "link");
        assert_eq!(EntryMode::Commit.to_string(), "commit");
    }

    #[test]
    fn test_change_kind_display() {
        assert_eq!(ChangeKind::Added.to_string(), "added");
        assert_eq!(ChangeKind::Deleted.to_string(), "deleted");
        assert_eq!(ChangeKind::Modified.to_string(), "modified");
    }

    #[test]
    fn test_object_kind_from_gix_object_kind() {
        assert_eq!(ObjectKind::from(gix_object::Kind::Blob), ObjectKind::Blob);
        assert_eq!(ObjectKind::from(gix_object::Kind::Tree), ObjectKind::Tree);
        assert_eq!(ObjectKind::from(gix_object::Kind::Commit), ObjectKind::Commit);
        assert_eq!(ObjectKind::from(gix_object::Kind::Tag), ObjectKind::Tag);
    }

    #[test]
    fn test_gix_object_kind_from_object_kind() {
        assert_eq!(gix_object::Kind::from(ObjectKind::Blob), gix_object::Kind::Blob);
        assert_eq!(gix_object::Kind::from(ObjectKind::Tree), gix_object::Kind::Tree);
        assert_eq!(gix_object::Kind::from(ObjectKind::Commit), gix_object::Kind::Commit);
        assert_eq!(gix_object::Kind::from(ObjectKind::Tag), gix_object::Kind::Tag);
    }

    #[test]
    fn test_entry_mode_from_gix_entry_mode() {
        use gix_object::tree::EntryKind;

        let blob_mode: gix_object::tree::EntryMode = EntryKind::Blob.into();
        let blob_exec_mode: gix_object::tree::EntryMode = EntryKind::BlobExecutable.into();
        let tree_mode: gix_object::tree::EntryMode = EntryKind::Tree.into();
        let link_mode: gix_object::tree::EntryMode = EntryKind::Link.into();
        let commit_mode: gix_object::tree::EntryMode = EntryKind::Commit.into();

        assert_eq!(EntryMode::from(blob_mode), EntryMode::Blob);
        assert_eq!(EntryMode::from(blob_exec_mode), EntryMode::BlobExecutable);
        assert_eq!(EntryMode::from(tree_mode), EntryMode::Tree);
        assert_eq!(EntryMode::from(link_mode), EntryMode::Link);
        assert_eq!(EntryMode::from(commit_mode), EntryMode::Commit);
    }

    #[test]
    fn test_gix_entry_mode_from_entry_mode() {
        use gix_object::tree::EntryKind;

        let blob: gix_object::tree::EntryMode = EntryMode::Blob.into();
        let blob_exec: gix_object::tree::EntryMode = EntryMode::BlobExecutable.into();
        let tree: gix_object::tree::EntryMode = EntryMode::Tree.into();
        let link: gix_object::tree::EntryMode = EntryMode::Link.into();
        let commit: gix_object::tree::EntryMode = EntryMode::Commit.into();

        assert_eq!(blob.kind(), EntryKind::Blob);
        assert_eq!(blob_exec.kind(), EntryKind::BlobExecutable);
        assert_eq!(tree.kind(), EntryKind::Tree);
        assert_eq!(link.kind(), EntryKind::Link);
        assert_eq!(commit.kind(), EntryKind::Commit);
    }

    #[test]
    fn test_signature_from_gix_actor_signature_ref() {
        use gix_actor::SignatureRef;

        let sig_ref = SignatureRef {
            name: b"Test User".as_slice().into(),
            email: b"test@example.com".as_slice().into(),
            time: "1234567890 +0000",
        };

        let signature = Signature::from(sig_ref);
        assert_eq!(signature.name, BString::from("Test User"));
        assert_eq!(signature.email, BString::from("test@example.com"));
        assert_eq!(signature.time, 1234567890);
    }

    #[test]
    fn test_signature_from_gix_actor_signature_ref_with_invalid_time() {
        use gix_actor::SignatureRef;

        let sig_ref = SignatureRef {
            name: b"Test User".as_slice().into(),
            email: b"test@example.com".as_slice().into(),
            time: "invalid",
        };

        let signature = Signature::from(sig_ref);
        assert_eq!(signature.name, BString::from("Test User"));
        assert_eq!(signature.email, BString::from("test@example.com"));
        assert_eq!(signature.time, 0);
    }

    #[test]
    fn test_signature_from_gix_actor_signature() {
        use gix_actor::Signature as GixSignature;
        use gix_actor::date::Time;

        let gix_sig = GixSignature {
            name: BString::from("Test User"),
            email: BString::from("test@example.com"),
            time: Time::new(1234567890, 0),
        };

        let signature = Signature::from(gix_sig);
        assert_eq!(signature.name, BString::from("Test User"));
        assert_eq!(signature.email, BString::from("test@example.com"));
        assert_eq!(signature.time, 1234567890);
    }

    #[test]
    fn test_blame_options_default() {
        let opts = BlameOptions::default();
        assert!(opts.range.is_none());
        assert!(opts.follow_renames);
    }
}
