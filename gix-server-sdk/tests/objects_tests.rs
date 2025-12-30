mod fixtures;

use fixtures::TestRepo;
use gix_server_sdk::{ops, RepoPool, SdkConfig, SdkError};

fn create_pool() -> RepoPool {
    RepoPool::new(SdkConfig::default())
}

mod get_object {
    use super::*;

    #[test]
    fn blob_object() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let blob_id_str = repo.git_output(&["rev-parse", "HEAD:README.md"]);
        let blob_id =
            gix_hash::ObjectId::from_hex(blob_id_str.as_bytes()).expect("failed to parse blob id");

        let result = ops::get_object(&handle, blob_id).expect("failed to get blob");

        assert_eq!(result.id, blob_id);
        assert_eq!(result.kind, ops::ObjectKind::Blob);
        assert!(result.data.starts_with(b"# Test Repository"));
    }

    #[test]
    fn tree_object() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let result = ops::get_object(&handle, tree_id).expect("failed to get tree");

        assert_eq!(result.id, tree_id);
        assert_eq!(result.kind, ops::ObjectKind::Tree);
        assert!(!result.data.is_empty());
    }

    #[test]
    fn commit_object() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let result = ops::get_object(&handle, commit_id).expect("failed to get commit");

        assert_eq!(result.id, commit_id);
        assert_eq!(result.kind, ops::ObjectKind::Commit);
        let data_str = String::from_utf8_lossy(&result.data);
        assert!(data_str.contains("tree"));
        assert!(data_str.contains("author"));
        assert!(data_str.contains("committer"));
    }

    #[test]
    fn tag_object() {
        let repo = TestRepo::with_tags();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tag_id_str = repo.git_output(&["rev-parse", "refs/tags/v1.1.0"]);
        let tag_id =
            gix_hash::ObjectId::from_hex(tag_id_str.as_bytes()).expect("failed to parse tag id");

        let result = ops::get_object(&handle, tag_id).expect("failed to get tag");

        assert_eq!(result.id, tag_id);
        assert_eq!(result.kind, ops::ObjectKind::Tag);
        let data_str = String::from_utf8_lossy(&result.data);
        assert!(data_str.contains("object"));
        assert!(data_str.contains("tag v1.1.0"));
        assert!(data_str.contains("tagger"));
    }

    #[test]
    fn nonexistent_object_returns_error() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let fake_id = gix_hash::ObjectId::from_hex(b"0000000000000000000000000000000000000000")
            .expect("valid hex");
        let result = ops::get_object(&handle, fake_id);

        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_debug = format!("{:?}", err);
        let err_display = format!("{}", err);
        let is_not_found = matches!(&err, SdkError::ObjectNotFound(_))
            || err_debug.contains("NotFound")
            || err_display.contains("NotFound")
            || err_display.contains("not found");
        assert!(is_not_found, "expected not found error, got: {:?}", err);
    }
}

mod get_object_header {
    use super::*;

    #[test]
    fn blob_header() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let blob_id_str = repo.git_output(&["rev-parse", "HEAD:README.md"]);
        let blob_id =
            gix_hash::ObjectId::from_hex(blob_id_str.as_bytes()).expect("failed to parse blob id");

        let result = ops::get_object_header(&handle, blob_id).expect("failed to get header");

        assert_eq!(result.id, blob_id);
        assert_eq!(result.kind, ops::ObjectKind::Blob);
        assert!(result.size > 0);
    }

    #[test]
    fn tree_header() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let result = ops::get_object_header(&handle, tree_id).expect("failed to get header");

        assert_eq!(result.id, tree_id);
        assert_eq!(result.kind, ops::ObjectKind::Tree);
        assert!(result.size > 0);
    }

    #[test]
    fn commit_header() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let result = ops::get_object_header(&handle, commit_id).expect("failed to get header");

        assert_eq!(result.id, commit_id);
        assert_eq!(result.kind, ops::ObjectKind::Commit);
        assert!(result.size > 0);
    }

    #[test]
    fn tag_header() {
        let repo = TestRepo::with_tags();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tag_id_str = repo.git_output(&["rev-parse", "refs/tags/v1.1.0"]);
        let tag_id =
            gix_hash::ObjectId::from_hex(tag_id_str.as_bytes()).expect("failed to parse tag id");

        let result = ops::get_object_header(&handle, tag_id).expect("failed to get header");

        assert_eq!(result.id, tag_id);
        assert_eq!(result.kind, ops::ObjectKind::Tag);
        assert!(result.size > 0);
    }

    #[test]
    fn nonexistent_object_returns_error() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let fake_id = gix_hash::ObjectId::from_hex(b"0000000000000000000000000000000000000000")
            .expect("valid hex");
        let result = ops::get_object_header(&handle, fake_id);

        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_debug = format!("{:?}", err);
        let err_display = format!("{}", err);
        let is_not_found = matches!(&err, SdkError::ObjectNotFound(_))
            || err_debug.contains("NotFound")
            || err_display.contains("NotFound")
            || err_display.contains("not found");
        assert!(is_not_found, "expected not found error, got: {:?}", err);
    }

    #[test]
    fn header_size_matches_object_data_size() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let blob_id_str = repo.git_output(&["rev-parse", "HEAD:README.md"]);
        let blob_id =
            gix_hash::ObjectId::from_hex(blob_id_str.as_bytes()).expect("failed to parse blob id");

        let header = ops::get_object_header(&handle, blob_id).expect("failed to get header");
        let object = ops::get_object(&handle, blob_id).expect("failed to get object");

        assert_eq!(header.size, object.data.len());
    }
}

mod object_exists {
    use super::*;

    #[test]
    fn existing_blob() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let blob_id_str = repo.git_output(&["rev-parse", "HEAD:README.md"]);
        let blob_id =
            gix_hash::ObjectId::from_hex(blob_id_str.as_bytes()).expect("failed to parse blob id");

        assert!(ops::object_exists(&handle, &blob_id));
    }

    #[test]
    fn existing_tree() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        assert!(ops::object_exists(&handle, &tree_id));
    }

    #[test]
    fn existing_commit() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        assert!(ops::object_exists(&handle, &commit_id));
    }

    #[test]
    fn existing_tag() {
        let repo = TestRepo::with_tags();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tag_id_str = repo.git_output(&["rev-parse", "refs/tags/v1.1.0"]);
        let tag_id =
            gix_hash::ObjectId::from_hex(tag_id_str.as_bytes()).expect("failed to parse tag id");

        assert!(ops::object_exists(&handle, &tag_id));
    }

    #[test]
    fn nonexistent_object() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let fake_id = gix_hash::ObjectId::from_hex(b"0000000000000000000000000000000000000000")
            .expect("valid hex");
        assert!(!ops::object_exists(&handle, &fake_id));
    }

    #[test]
    fn random_hash_does_not_exist() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let fake_id = gix_hash::ObjectId::from_hex(b"1234567890abcdef1234567890abcdef12345678")
            .expect("valid hex");
        assert!(!ops::object_exists(&handle, &fake_id));
    }

    #[test]
    fn subtree_exists() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD:src"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        assert!(ops::object_exists(&handle, &tree_id));
    }
}

mod resolve_revision {
    use super::*;

    #[test]
    fn head() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let expected_str = repo.git_output(&["rev-parse", "HEAD"]);
        let expected = gix_hash::ObjectId::from_hex(expected_str.as_bytes())
            .expect("failed to parse expected id");

        let result = ops::resolve_revision(&handle, "HEAD").expect("failed to resolve HEAD");
        assert_eq!(result, expected);
    }

    #[test]
    fn head_parent() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let expected_str = repo.git_output(&["rev-parse", "HEAD~1"]);
        let expected = gix_hash::ObjectId::from_hex(expected_str.as_bytes())
            .expect("failed to parse expected id");

        let result = ops::resolve_revision(&handle, "HEAD~1").expect("failed to resolve HEAD~1");
        assert_eq!(result, expected);
    }

    #[test]
    fn head_grandparent() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let expected_str = repo.git_output(&["rev-parse", "HEAD~2"]);
        let expected = gix_hash::ObjectId::from_hex(expected_str.as_bytes())
            .expect("failed to parse expected id");

        let result = ops::resolve_revision(&handle, "HEAD~2").expect("failed to resolve HEAD~2");
        assert_eq!(result, expected);
    }

    #[test]
    fn head_caret_parent() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let expected_str = repo.git_output(&["rev-parse", "HEAD^"]);
        let expected = gix_hash::ObjectId::from_hex(expected_str.as_bytes())
            .expect("failed to parse expected id");

        let result = ops::resolve_revision(&handle, "HEAD^").expect("failed to resolve HEAD^");
        assert_eq!(result, expected);
    }

    #[test]
    fn branch_name() {
        let repo = TestRepo::with_branches();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let expected_str = repo.git_output(&["rev-parse", "feature-a"]);
        let expected = gix_hash::ObjectId::from_hex(expected_str.as_bytes())
            .expect("failed to parse expected id");

        let result =
            ops::resolve_revision(&handle, "feature-a").expect("failed to resolve branch");
        assert_eq!(result, expected);
    }

    #[test]
    fn main_branch() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let expected_str = repo.git_output(&["rev-parse", "main"]);
        let expected = gix_hash::ObjectId::from_hex(expected_str.as_bytes())
            .expect("failed to parse expected id");

        let result = ops::resolve_revision(&handle, "main").expect("failed to resolve main");
        assert_eq!(result, expected);
    }

    #[test]
    fn annotated_tag() {
        let repo = TestRepo::with_tags();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let expected_str = repo.git_output(&["rev-parse", "v1.1.0"]);
        let expected = gix_hash::ObjectId::from_hex(expected_str.as_bytes())
            .expect("failed to parse expected id");

        let result = ops::resolve_revision(&handle, "v1.1.0").expect("failed to resolve tag");
        assert_eq!(result, expected);
    }

    #[test]
    fn lightweight_tag() {
        let repo = TestRepo::with_tags();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let expected_str = repo.git_output(&["rev-parse", "v1.0.0"]);
        let expected = gix_hash::ObjectId::from_hex(expected_str.as_bytes())
            .expect("failed to parse expected id");

        let result = ops::resolve_revision(&handle, "v1.0.0").expect("failed to resolve tag");
        assert_eq!(result, expected);
    }

    #[test]
    fn full_hash() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id =
            gix_hash::ObjectId::from_hex(head_str.as_bytes()).expect("failed to parse head id");

        let result = ops::resolve_revision(&handle, &head_str).expect("failed to resolve full hash");
        assert_eq!(result, head_id);
    }

    #[test]
    fn short_hash() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let short = repo.git_output(&["rev-parse", "--short", "HEAD"]);
        let expected_str = repo.git_output(&["rev-parse", "HEAD"]);
        let expected = gix_hash::ObjectId::from_hex(expected_str.as_bytes())
            .expect("failed to parse expected id");

        let result =
            ops::resolve_revision(&handle, &short).expect("failed to resolve short hash");
        assert_eq!(result, expected);
    }

    #[test]
    fn tree_suffix() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let expected_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let expected = gix_hash::ObjectId::from_hex(expected_str.as_bytes())
            .expect("failed to parse expected id");

        let result =
            ops::resolve_revision(&handle, "HEAD^{tree}").expect("failed to resolve tree");
        assert_eq!(result, expected);
    }

    #[test]
    fn tag_peeled_to_commit() {
        let repo = TestRepo::with_tags();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let expected_str = repo.git_output(&["rev-parse", "v1.1.0^{commit}"]);
        let expected = gix_hash::ObjectId::from_hex(expected_str.as_bytes())
            .expect("failed to parse expected id");

        let result =
            ops::resolve_revision(&handle, "v1.1.0^{commit}").expect("failed to resolve peeled tag");
        assert_eq!(result, expected);
    }

    #[test]
    fn refs_heads_prefix() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let expected_str = repo.git_output(&["rev-parse", "refs/heads/main"]);
        let expected = gix_hash::ObjectId::from_hex(expected_str.as_bytes())
            .expect("failed to parse expected id");

        let result =
            ops::resolve_revision(&handle, "refs/heads/main").expect("failed to resolve full ref");
        assert_eq!(result, expected);
    }

    #[test]
    fn invalid_revision_returns_error() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let result = ops::resolve_revision(&handle, "nonexistent-branch-12345");
        assert!(result.is_err());
        match result.unwrap_err() {
            SdkError::InvalidRevision(msg) => {
                assert!(msg.contains("nonexistent-branch-12345"));
            }
            other => panic!("unexpected error: {:?}", other),
        }
    }

    #[test]
    fn invalid_hash_returns_error() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let result = ops::resolve_revision(&handle, "zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz");
        assert!(result.is_err());
    }

    #[test]
    fn empty_revision_returns_error() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let result = ops::resolve_revision(&handle, "");
        assert!(result.is_err());
    }

    #[test]
    fn at_symbol_for_head() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let expected_str = repo.git_output(&["rev-parse", "HEAD"]);
        let expected = gix_hash::ObjectId::from_hex(expected_str.as_bytes())
            .expect("failed to parse expected id");

        let result = ops::resolve_revision(&handle, "@").expect("failed to resolve @");
        assert_eq!(result, expected);
    }
}

mod get_blob {
    use super::*;

    #[test]
    fn text_blob() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let blob_id_str = repo.git_output(&["rev-parse", "HEAD:README.md"]);
        let blob_id =
            gix_hash::ObjectId::from_hex(blob_id_str.as_bytes()).expect("failed to parse blob id");

        let result = ops::get_blob(&handle, blob_id).expect("failed to get blob");
        assert!(result.starts_with(b"# Test Repository"));
    }

    #[test]
    fn binary_blob() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let blob_id_str = repo.git_output(&["rev-parse", "HEAD:data.bin"]);
        let blob_id =
            gix_hash::ObjectId::from_hex(blob_id_str.as_bytes()).expect("failed to parse blob id");

        let result = ops::get_blob(&handle, blob_id).expect("failed to get blob");
        let expected: Vec<u8> = (0..=255).collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn rust_source_blob() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let blob_id_str = repo.git_output(&["rev-parse", "HEAD:src/main.rs"]);
        let blob_id =
            gix_hash::ObjectId::from_hex(blob_id_str.as_bytes()).expect("failed to parse blob id");

        let result = ops::get_blob(&handle, blob_id).expect("failed to get blob");
        let content = String::from_utf8(result).expect("failed to parse as utf8");
        assert!(content.contains("fn main()"));
        assert!(content.contains("println!"));
    }

    #[test]
    fn nonexistent_blob_returns_error() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let fake_id = gix_hash::ObjectId::from_hex(b"0000000000000000000000000000000000000000")
            .expect("valid hex");
        let result = ops::get_blob(&handle, fake_id);

        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_debug = format!("{:?}", err);
        let err_display = format!("{}", err);
        let is_not_found = matches!(&err, SdkError::ObjectNotFound(_))
            || err_debug.contains("NotFound")
            || err_display.contains("NotFound")
            || err_display.contains("not found");
        assert!(is_not_found, "expected not found error, got: {:?}", err);
    }

    #[test]
    fn tree_id_returns_wrong_type_error() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let result = ops::get_blob(&handle, tree_id);

        assert!(result.is_err());
        match result.unwrap_err() {
            SdkError::InvalidObjectType { expected, actual } => {
                assert_eq!(expected, "blob");
                assert_eq!(actual, "tree");
            }
            other => panic!("unexpected error: {:?}", other),
        }
    }

    #[test]
    fn commit_id_returns_wrong_type_error() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let result = ops::get_blob(&handle, commit_id);

        assert!(result.is_err());
        match result.unwrap_err() {
            SdkError::InvalidObjectType { expected, actual } => {
                assert_eq!(expected, "blob");
                assert_eq!(actual, "commit");
            }
            other => panic!("unexpected error: {:?}", other),
        }
    }

    #[test]
    fn tag_id_returns_wrong_type_error() {
        let repo = TestRepo::with_tags();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tag_id_str = repo.git_output(&["rev-parse", "refs/tags/v1.1.0"]);
        let tag_id =
            gix_hash::ObjectId::from_hex(tag_id_str.as_bytes()).expect("failed to parse tag id");

        let result = ops::get_blob(&handle, tag_id);

        assert!(result.is_err());
        match result.unwrap_err() {
            SdkError::InvalidObjectType { expected, actual } => {
                assert_eq!(expected, "blob");
                assert_eq!(actual, "tag");
            }
            other => panic!("unexpected error: {:?}", other),
        }
    }

    #[test]
    fn blob_with_special_characters() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let blob_id_str = repo.git_output(&["rev-parse", "HEAD:src/lib.rs"]);
        let blob_id =
            gix_hash::ObjectId::from_hex(blob_id_str.as_bytes()).expect("failed to parse blob id");

        let result = ops::get_blob(&handle, blob_id).expect("failed to get blob");
        let content = String::from_utf8(result).expect("failed to parse as utf8");
        assert!(content.contains("pub fn add"));
        assert!(content.contains("#[cfg(test)]"));
        assert!(content.contains("assert_eq!"));
    }

    #[test]
    fn blob_in_subdirectory() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let blob_id_str = repo.git_output(&["rev-parse", "HEAD:docs/guide.md"]);
        let blob_id =
            gix_hash::ObjectId::from_hex(blob_id_str.as_bytes()).expect("failed to parse blob id");

        let result = ops::get_blob(&handle, blob_id).expect("failed to get blob");
        let content = String::from_utf8(result).expect("failed to parse as utf8");
        assert!(content.contains("# User Guide"));
        assert!(content.contains("## Getting Started"));
    }
}

mod get_blob_size {
    use super::*;

    #[test]
    fn text_blob_size() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let blob_id_str = repo.git_output(&["rev-parse", "HEAD:README.md"]);
        let blob_id =
            gix_hash::ObjectId::from_hex(blob_id_str.as_bytes()).expect("failed to parse blob id");

        let result = ops::get_blob_size(&handle, blob_id).expect("failed to get blob size");
        assert!(result > 0);
    }

    #[test]
    fn binary_blob_size() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let blob_id_str = repo.git_output(&["rev-parse", "HEAD:data.bin"]);
        let blob_id =
            gix_hash::ObjectId::from_hex(blob_id_str.as_bytes()).expect("failed to parse blob id");

        let result = ops::get_blob_size(&handle, blob_id).expect("failed to get blob size");
        assert_eq!(result, 256);
    }

    #[test]
    fn nonexistent_blob_returns_error() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let fake_id = gix_hash::ObjectId::from_hex(b"0000000000000000000000000000000000000000")
            .expect("valid hex");
        let result = ops::get_blob_size(&handle, fake_id);

        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_debug = format!("{:?}", err);
        let err_display = format!("{}", err);
        let is_not_found = matches!(&err, SdkError::ObjectNotFound(_))
            || err_debug.contains("NotFound")
            || err_display.contains("NotFound")
            || err_display.contains("not found");
        assert!(is_not_found, "expected not found error, got: {:?}", err);
    }

    #[test]
    fn tree_id_returns_wrong_type_error() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let result = ops::get_blob_size(&handle, tree_id);

        assert!(result.is_err());
        match result.unwrap_err() {
            SdkError::InvalidObjectType { expected, actual } => {
                assert_eq!(expected, "blob");
                assert_eq!(actual, "tree");
            }
            other => panic!("unexpected error: {:?}", other),
        }
    }

    #[test]
    fn commit_id_returns_wrong_type_error() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let result = ops::get_blob_size(&handle, commit_id);

        assert!(result.is_err());
        match result.unwrap_err() {
            SdkError::InvalidObjectType { expected, actual } => {
                assert_eq!(expected, "blob");
                assert_eq!(actual, "commit");
            }
            other => panic!("unexpected error: {:?}", other),
        }
    }

    #[test]
    fn tag_id_returns_wrong_type_error() {
        let repo = TestRepo::with_tags();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tag_id_str = repo.git_output(&["rev-parse", "refs/tags/v1.1.0"]);
        let tag_id =
            gix_hash::ObjectId::from_hex(tag_id_str.as_bytes()).expect("failed to parse tag id");

        let result = ops::get_blob_size(&handle, tag_id);

        assert!(result.is_err());
        match result.unwrap_err() {
            SdkError::InvalidObjectType { expected, actual } => {
                assert_eq!(expected, "blob");
                assert_eq!(actual, "tag");
            }
            other => panic!("unexpected error: {:?}", other),
        }
    }

    #[test]
    fn size_matches_content_length() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let blob_id_str = repo.git_output(&["rev-parse", "HEAD:README.md"]);
        let blob_id =
            gix_hash::ObjectId::from_hex(blob_id_str.as_bytes()).expect("failed to parse blob id");

        let size = ops::get_blob_size(&handle, blob_id).expect("failed to get blob size");
        let content = ops::get_blob(&handle, blob_id).expect("failed to get blob");

        assert_eq!(size, content.len());
    }

    #[test]
    fn size_matches_object_header_size() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let blob_id_str = repo.git_output(&["rev-parse", "HEAD:src/main.rs"]);
        let blob_id =
            gix_hash::ObjectId::from_hex(blob_id_str.as_bytes()).expect("failed to parse blob id");

        let size = ops::get_blob_size(&handle, blob_id).expect("failed to get blob size");
        let header = ops::get_object_header(&handle, blob_id).expect("failed to get header");

        assert_eq!(size, header.size);
    }

    #[test]
    fn multiple_blobs_different_sizes() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let readme_id_str = repo.git_output(&["rev-parse", "HEAD:README.md"]);
        let readme_id = gix_hash::ObjectId::from_hex(readme_id_str.as_bytes())
            .expect("failed to parse blob id");

        let lib_id_str = repo.git_output(&["rev-parse", "HEAD:src/lib.rs"]);
        let lib_id =
            gix_hash::ObjectId::from_hex(lib_id_str.as_bytes()).expect("failed to parse blob id");

        let data_id_str = repo.git_output(&["rev-parse", "HEAD:data.bin"]);
        let data_id =
            gix_hash::ObjectId::from_hex(data_id_str.as_bytes()).expect("failed to parse blob id");

        let readme_size = ops::get_blob_size(&handle, readme_id).expect("failed to get size");
        let lib_size = ops::get_blob_size(&handle, lib_id).expect("failed to get size");
        let data_size = ops::get_blob_size(&handle, data_id).expect("failed to get size");

        assert!(readme_size > 0);
        assert!(lib_size > 0);
        assert_eq!(data_size, 256);

        assert_ne!(readme_size, lib_size);
    }
}

mod packed_objects {
    use super::*;

    #[test]
    fn get_object_header_from_packed() {
        let repo = TestRepo::with_packed_objects();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let result = ops::get_object_header(&handle, commit_id).expect("failed to get header");

        assert_eq!(result.id, commit_id);
        assert_eq!(result.kind, ops::ObjectKind::Commit);
        assert!(result.size > 0);
    }

    #[test]
    fn get_blob_size_from_packed() {
        let repo = TestRepo::with_packed_objects();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let blob_id_str = repo.git_output(&["rev-parse", "HEAD:README.md"]);
        let blob_id =
            gix_hash::ObjectId::from_hex(blob_id_str.as_bytes()).expect("failed to parse blob id");

        let result = ops::get_blob_size(&handle, blob_id).expect("failed to get blob size");
        assert!(result > 0);
    }

    #[test]
    fn get_object_from_packed() {
        let repo = TestRepo::with_packed_objects();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let blob_id_str = repo.git_output(&["rev-parse", "HEAD:README.md"]);
        let blob_id =
            gix_hash::ObjectId::from_hex(blob_id_str.as_bytes()).expect("failed to parse blob id");

        let result = ops::get_object(&handle, blob_id).expect("failed to get object");
        assert_eq!(result.id, blob_id);
        assert_eq!(result.kind, ops::ObjectKind::Blob);
    }

    #[test]
    fn object_exists_in_packed() {
        let repo = TestRepo::with_packed_objects();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        assert!(ops::object_exists(&handle, &tree_id));
    }

    #[test]
    fn get_blob_from_packed() {
        let repo = TestRepo::with_packed_objects();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let blob_id_str = repo.git_output(&["rev-parse", "HEAD:README.md"]);
        let blob_id =
            gix_hash::ObjectId::from_hex(blob_id_str.as_bytes()).expect("failed to parse blob id");

        let result = ops::get_blob(&handle, blob_id).expect("failed to get blob");
        assert!(result.starts_with(b"# Test Repository"));
    }
}

mod only_packed_objects {
    use super::*;

    fn has_loose_objects(path: &std::path::Path) -> bool {
        let objects_dir = path.join(".git/objects");
        if let Ok(entries) = std::fs::read_dir(&objects_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.len() == 2 && name.chars().all(|c| c.is_ascii_hexdigit()) {
                    if let Ok(inner) = std::fs::read_dir(entry.path()) {
                        if inner.count() > 0 {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    fn has_pack_files(path: &std::path::Path) -> bool {
        let pack_dir = path.join(".git/objects/pack");
        if let Ok(entries) = std::fs::read_dir(&pack_dir) {
            for entry in entries.flatten() {
                if entry.path().extension().map(|e| e == "pack").unwrap_or(false) {
                    return true;
                }
            }
        }
        false
    }

    #[test]
    fn verify_no_loose_objects_exist() {
        let repo = TestRepo::with_only_packed_objects();
        assert!(
            !has_loose_objects(&repo.path),
            "expected no loose objects after gc"
        );
        assert!(
            has_pack_files(&repo.path),
            "expected pack files to exist"
        );
    }

    #[test]
    fn get_object_header_from_only_packed() {
        let repo = TestRepo::with_only_packed_objects();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let result = ops::get_object_header(&handle, commit_id).expect("failed to get header");

        assert_eq!(result.id, commit_id);
        assert_eq!(result.kind, ops::ObjectKind::Commit);
        assert!(result.size > 0);
    }

    #[test]
    fn get_blob_size_from_only_packed() {
        let repo = TestRepo::with_only_packed_objects();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let blob_id_str = repo.git_output(&["rev-parse", "HEAD:README.md"]);
        let blob_id =
            gix_hash::ObjectId::from_hex(blob_id_str.as_bytes()).expect("failed to parse blob id");

        let result = ops::get_blob_size(&handle, blob_id).expect("failed to get blob size");
        assert!(result > 0);
    }

    #[test]
    fn get_object_from_only_packed() {
        let repo = TestRepo::with_only_packed_objects();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let blob_id_str = repo.git_output(&["rev-parse", "HEAD:README.md"]);
        let blob_id =
            gix_hash::ObjectId::from_hex(blob_id_str.as_bytes()).expect("failed to parse blob id");

        let result = ops::get_object(&handle, blob_id).expect("failed to get object");
        assert_eq!(result.id, blob_id);
        assert_eq!(result.kind, ops::ObjectKind::Blob);
    }

    #[test]
    fn get_blob_from_only_packed() {
        let repo = TestRepo::with_only_packed_objects();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let blob_id_str = repo.git_output(&["rev-parse", "HEAD:README.md"]);
        let blob_id =
            gix_hash::ObjectId::from_hex(blob_id_str.as_bytes()).expect("failed to parse blob id");

        let result = ops::get_blob(&handle, blob_id).expect("failed to get blob");
        assert!(result.starts_with(b"# Test Repository"));
    }

    #[test]
    fn get_tree_header_from_only_packed() {
        let repo = TestRepo::with_only_packed_objects();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let result = ops::get_object_header(&handle, tree_id).expect("failed to get header");

        assert_eq!(result.id, tree_id);
        assert_eq!(result.kind, ops::ObjectKind::Tree);
        assert!(result.size > 0);
    }
}

mod corrupted_objects {
    use super::*;

    #[test]
    fn corrupted_loose_object_returns_git_error() {
        let repo = TestRepo::with_corrupted_loose_object();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let objects_dir = repo.path.join(".git/objects");
        let mut corrupted_id = None;
        for entry in std::fs::read_dir(&objects_dir).expect("read objects dir") {
            let entry = entry.expect("read entry");
            let dir_name = entry.file_name().to_string_lossy().to_string();
            if dir_name.len() == 2 && dir_name.chars().all(|c| c.is_ascii_hexdigit()) {
                if let Ok(inner_entries) = std::fs::read_dir(entry.path()) {
                    for inner in inner_entries.flatten() {
                        let file_name = inner.file_name().to_string_lossy().to_string();
                        let full_hex = format!("{}{}", dir_name, file_name);
                        if let Ok(id) = gix_hash::ObjectId::from_hex(full_hex.as_bytes()) {
                            corrupted_id = Some(id);
                            break;
                        }
                    }
                }
                if corrupted_id.is_some() {
                    break;
                }
            }
        }

        if let Some(id) = corrupted_id {
            let result = ops::get_object(&handle, id);
            assert!(result.is_err(), "expected error for corrupted object");
            let err = result.unwrap_err();
            let err_str = err.to_string();
            assert!(
                !err_str.contains("not found"),
                "error should not be a 'not found' error, got: {}",
                err_str
            );
            assert!(
                matches!(err, SdkError::Git(_)),
                "expected Git error variant, got: {:?}",
                err
            );
        }
    }

    #[test]
    fn corrupted_loose_object_header_returns_git_error() {
        let repo = TestRepo::with_corrupted_loose_object();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let objects_dir = repo.path.join(".git/objects");
        let mut corrupted_id = None;
        for entry in std::fs::read_dir(&objects_dir).expect("read objects dir") {
            let entry = entry.expect("read entry");
            let dir_name = entry.file_name().to_string_lossy().to_string();
            if dir_name.len() == 2 && dir_name.chars().all(|c| c.is_ascii_hexdigit()) {
                if let Ok(inner_entries) = std::fs::read_dir(entry.path()) {
                    for inner in inner_entries.flatten() {
                        let file_name = inner.file_name().to_string_lossy().to_string();
                        let full_hex = format!("{}{}", dir_name, file_name);
                        if let Ok(id) = gix_hash::ObjectId::from_hex(full_hex.as_bytes()) {
                            corrupted_id = Some(id);
                            break;
                        }
                    }
                }
                if corrupted_id.is_some() {
                    break;
                }
            }
        }

        if let Some(id) = corrupted_id {
            let result = ops::get_object_header(&handle, id);
            assert!(result.is_err(), "expected error for corrupted object");
            let err = result.unwrap_err();
            let err_str = err.to_string();
            assert!(
                !err_str.contains("not found"),
                "error should not be a 'not found' error, got: {}",
                err_str
            );
            assert!(
                matches!(err, SdkError::Git(_)),
                "expected Git error variant, got: {:?}",
                err
            );
        }
    }

    #[test]
    fn corrupted_loose_blob_returns_git_error() {
        let repo = TestRepo::with_corrupted_loose_object();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let objects_dir = repo.path.join(".git/objects");
        let mut corrupted_id = None;
        for entry in std::fs::read_dir(&objects_dir).expect("read objects dir") {
            let entry = entry.expect("read entry");
            let dir_name = entry.file_name().to_string_lossy().to_string();
            if dir_name.len() == 2 && dir_name.chars().all(|c| c.is_ascii_hexdigit()) {
                if let Ok(inner_entries) = std::fs::read_dir(entry.path()) {
                    for inner in inner_entries.flatten() {
                        let file_name = inner.file_name().to_string_lossy().to_string();
                        let full_hex = format!("{}{}", dir_name, file_name);
                        if let Ok(id) = gix_hash::ObjectId::from_hex(full_hex.as_bytes()) {
                            corrupted_id = Some(id);
                            break;
                        }
                    }
                }
                if corrupted_id.is_some() {
                    break;
                }
            }
        }

        if let Some(id) = corrupted_id {
            let result = ops::get_blob(&handle, id);
            assert!(result.is_err(), "expected error for corrupted object");
            let err = result.unwrap_err();
            let err_str = err.to_string();
            assert!(
                !err_str.contains("not found"),
                "error should not be a 'not found' error, got: {}",
                err_str
            );
            assert!(
                matches!(err, SdkError::Git(_)),
                "expected Git error variant, got: {:?}",
                err
            );
        }
    }

    #[test]
    fn corrupted_loose_blob_size_returns_git_error() {
        let repo = TestRepo::with_corrupted_loose_object();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let objects_dir = repo.path.join(".git/objects");
        let mut corrupted_id = None;
        for entry in std::fs::read_dir(&objects_dir).expect("read objects dir") {
            let entry = entry.expect("read entry");
            let dir_name = entry.file_name().to_string_lossy().to_string();
            if dir_name.len() == 2 && dir_name.chars().all(|c| c.is_ascii_hexdigit()) {
                if let Ok(inner_entries) = std::fs::read_dir(entry.path()) {
                    for inner in inner_entries.flatten() {
                        let file_name = inner.file_name().to_string_lossy().to_string();
                        let full_hex = format!("{}{}", dir_name, file_name);
                        if let Ok(id) = gix_hash::ObjectId::from_hex(full_hex.as_bytes()) {
                            corrupted_id = Some(id);
                            break;
                        }
                    }
                }
                if corrupted_id.is_some() {
                    break;
                }
            }
        }

        if let Some(id) = corrupted_id {
            let result = ops::get_blob_size(&handle, id);
            assert!(result.is_err(), "expected error for corrupted object");
            let err = result.unwrap_err();
            let err_str = err.to_string();
            assert!(
                !err_str.contains("not found"),
                "error should not be a 'not found' error, got: {}",
                err_str
            );
            assert!(
                matches!(err, SdkError::Git(_)),
                "expected Git error variant, got: {:?}",
                err
            );
        }
    }
}
