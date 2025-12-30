mod fixtures;

use fixtures::TestRepo;
use gix_server_sdk::{ops, RepoPool, SdkConfig, SdkError};

fn create_pool() -> RepoPool {
    RepoPool::new(SdkConfig::default())
}

fn is_tree_mode(mode: &impl std::fmt::Display) -> bool {
    mode.to_string() == "tree"
}

fn is_blob_mode(mode: &impl std::fmt::Display) -> bool {
    mode.to_string() == "blob"
}

mod get_tree {
    use super::*;

    #[test]
    fn root_tree() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let entries = ops::get_tree(&handle, tree_id).expect("failed to get tree");

        assert!(!entries.is_empty());

        let names: Vec<_> = entries.iter().map(|e| e.name.to_string()).collect();
        assert!(names.contains(&"README.md".to_string()));
        assert!(names.contains(&"src".to_string()));
        assert!(names.contains(&"docs".to_string()));
        assert!(names.contains(&"data.bin".to_string()));
    }

    #[test]
    fn nested_tree() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD:src"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let entries = ops::get_tree(&handle, tree_id).expect("failed to get nested tree");

        let names: Vec<_> = entries.iter().map(|e| e.name.to_string()).collect();
        assert!(names.contains(&"main.rs".to_string()));
        assert!(names.contains(&"lib.rs".to_string()));
    }

    #[test]
    fn docs_tree() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD:docs"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let entries = ops::get_tree(&handle, tree_id).expect("failed to get docs tree");

        let names: Vec<_> = entries.iter().map(|e| e.name.to_string()).collect();
        assert!(names.contains(&"guide.md".to_string()));
    }

    #[test]
    fn entry_modes_correct() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let entries = ops::get_tree(&handle, tree_id).expect("failed to get tree");

        let src_entry = entries.iter().find(|e| e.name == "src").expect("src not found");
        assert!(is_tree_mode(&src_entry.mode));

        let readme_entry = entries.iter().find(|e| e.name == "README.md").expect("README.md not found");
        assert!(is_blob_mode(&readme_entry.mode));
    }

    #[test]
    fn nonexistent_tree_returns_error() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let fake_id = gix_hash::ObjectId::from_hex(b"0000000000000000000000000000000000000000")
            .expect("valid hex");

        let result = ops::get_tree(&handle, fake_id);
        assert!(result.is_err());
        match result.unwrap_err() {
            SdkError::ObjectNotFound(_) => {}
            other => panic!("expected ObjectNotFound, got: {:?}", other),
        }
    }

    #[test]
    fn blob_id_returns_invalid_object_type() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let blob_id_str = repo.git_output(&["rev-parse", "HEAD:README.md"]);
        let blob_id =
            gix_hash::ObjectId::from_hex(blob_id_str.as_bytes()).expect("failed to parse blob id");

        let result = ops::get_tree(&handle, blob_id);
        assert!(result.is_err());
        match result.unwrap_err() {
            SdkError::InvalidObjectType { expected, actual } => {
                assert_eq!(expected, "tree");
                assert_eq!(actual, "blob");
            }
            other => panic!("expected InvalidObjectType, got: {:?}", other),
        }
    }

    #[test]
    fn commit_id_returns_invalid_object_type() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let result = ops::get_tree(&handle, commit_id);
        assert!(result.is_err());
        match result.unwrap_err() {
            SdkError::InvalidObjectType { expected, actual } => {
                assert_eq!(expected, "tree");
                assert_eq!(actual, "commit");
            }
            other => panic!("expected InvalidObjectType, got: {:?}", other),
        }
    }

    #[test]
    fn tag_id_returns_invalid_object_type() {
        let repo = TestRepo::with_tags();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tag_id_str = repo.git_output(&["rev-parse", "refs/tags/v1.1.0"]);
        let tag_id =
            gix_hash::ObjectId::from_hex(tag_id_str.as_bytes()).expect("failed to parse tag id");

        let result = ops::get_tree(&handle, tag_id);
        assert!(result.is_err());
        match result.unwrap_err() {
            SdkError::InvalidObjectType { expected, actual } => {
                assert_eq!(expected, "tree");
                assert_eq!(actual, "tag");
            }
            other => panic!("expected InvalidObjectType, got: {:?}", other),
        }
    }

    #[test]
    fn tree_with_submodule() {
        let repo = TestRepo::with_submodules();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let entries = ops::get_tree(&handle, tree_id).expect("failed to get tree");

        let names: Vec<_> = entries.iter().map(|e| e.name.to_string()).collect();
        assert!(names.contains(&"vendor".to_string()) || names.contains(&"main.rs".to_string()));
    }

    #[test]
    fn entries_have_valid_ids() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let entries = ops::get_tree(&handle, tree_id).expect("failed to get tree");

        for entry in entries {
            assert!(!entry.id.is_null());
        }
    }
}

mod get_tree_entry {
    use super::*;

    #[test]
    fn file_at_root() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let entry = ops::get_tree_entry(&handle, tree_id, "README.md")
            .expect("failed to get tree entry");

        assert_eq!(entry.name.to_string(), "README.md");
        assert!(is_blob_mode(&entry.mode));
    }

    #[test]
    fn directory_at_root() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let entry = ops::get_tree_entry(&handle, tree_id, "src")
            .expect("failed to get tree entry");

        assert_eq!(entry.name.to_string(), "src");
        assert!(is_tree_mode(&entry.mode));
    }

    #[test]
    fn file_in_nested_directory() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let entry = ops::get_tree_entry(&handle, tree_id, "src/main.rs")
            .expect("failed to get tree entry");

        assert_eq!(entry.name.to_string(), "main.rs");
        assert!(is_blob_mode(&entry.mode));
    }

    #[test]
    fn file_in_docs_directory() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let entry = ops::get_tree_entry(&handle, tree_id, "docs/guide.md")
            .expect("failed to get tree entry");

        assert_eq!(entry.name.to_string(), "guide.md");
        assert!(is_blob_mode(&entry.mode));
    }

    #[test]
    fn path_with_leading_slash() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let entry = ops::get_tree_entry(&handle, tree_id, "/README.md")
            .expect("failed to get tree entry");

        assert_eq!(entry.name.to_string(), "README.md");
    }

    #[test]
    fn path_with_trailing_slash() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let entry = ops::get_tree_entry(&handle, tree_id, "src/main.rs/")
            .expect("failed to get tree entry");

        assert_eq!(entry.name.to_string(), "main.rs");
    }

    #[test]
    fn nonexistent_path_returns_error() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let result = ops::get_tree_entry(&handle, tree_id, "nonexistent.txt");
        assert!(result.is_err());
        match result.unwrap_err() {
            SdkError::TreeEntryNotFound(path) => {
                assert_eq!(path, "nonexistent.txt");
            }
            other => panic!("expected TreeEntryNotFound, got: {:?}", other),
        }
    }

    #[test]
    fn nonexistent_nested_path_returns_error() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let result = ops::get_tree_entry(&handle, tree_id, "src/nonexistent.rs");
        assert!(result.is_err());
        match result.unwrap_err() {
            SdkError::TreeEntryNotFound(path) => {
                assert_eq!(path, "src/nonexistent.rs");
            }
            other => panic!("expected TreeEntryNotFound, got: {:?}", other),
        }
    }

    #[test]
    fn nonexistent_directory_in_path_returns_error() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let result = ops::get_tree_entry(&handle, tree_id, "nonexistent/file.rs");
        assert!(result.is_err());
        match result.unwrap_err() {
            SdkError::TreeEntryNotFound(path) => {
                assert_eq!(path, "nonexistent/file.rs");
            }
            other => panic!("expected TreeEntryNotFound, got: {:?}", other),
        }
    }

    #[test]
    fn file_as_directory_returns_error() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let result = ops::get_tree_entry(&handle, tree_id, "README.md/something");
        assert!(result.is_err());
        match result.unwrap_err() {
            SdkError::TreeEntryNotFound(path) => {
                assert_eq!(path, "README.md/something");
            }
            other => panic!("expected TreeEntryNotFound, got: {:?}", other),
        }
    }

    #[test]
    fn empty_path_returns_error() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let result = ops::get_tree_entry(&handle, tree_id, "");
        assert!(result.is_err());
        match result.unwrap_err() {
            SdkError::TreeEntryNotFound(path) => {
                assert_eq!(path, "");
            }
            other => panic!("expected TreeEntryNotFound, got: {:?}", other),
        }
    }

    #[test]
    fn slash_only_path_returns_error() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let result = ops::get_tree_entry(&handle, tree_id, "/");
        assert!(result.is_err());
        match result.unwrap_err() {
            SdkError::TreeEntryNotFound(path) => {
                assert_eq!(path, "/");
            }
            other => panic!("expected TreeEntryNotFound, got: {:?}", other),
        }
    }

    #[test]
    fn blob_id_returns_invalid_object_type() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let blob_id_str = repo.git_output(&["rev-parse", "HEAD:README.md"]);
        let blob_id =
            gix_hash::ObjectId::from_hex(blob_id_str.as_bytes()).expect("failed to parse blob id");

        let result = ops::get_tree_entry(&handle, blob_id, "something");
        assert!(result.is_err());
        match result.unwrap_err() {
            SdkError::InvalidObjectType { expected, actual } => {
                assert_eq!(expected, "tree");
                assert_eq!(actual, "blob");
            }
            other => panic!("expected InvalidObjectType, got: {:?}", other),
        }
    }

    #[test]
    fn nonexistent_tree_returns_object_not_found() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let fake_id = gix_hash::ObjectId::from_hex(b"0000000000000000000000000000000000000000")
            .expect("valid hex");

        let result = ops::get_tree_entry(&handle, fake_id, "something");
        assert!(result.is_err());
        match result.unwrap_err() {
            SdkError::ObjectNotFound(_) => {}
            other => panic!("expected ObjectNotFound, got: {:?}", other),
        }
    }

    #[test]
    fn binary_file_entry() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let entry = ops::get_tree_entry(&handle, tree_id, "data.bin")
            .expect("failed to get tree entry");

        assert_eq!(entry.name.to_string(), "data.bin");
        assert!(is_blob_mode(&entry.mode));
    }

    #[test]
    fn entry_id_matches_git() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let expected_id_str = repo.git_output(&["rev-parse", "HEAD:README.md"]);
        let expected_id =
            gix_hash::ObjectId::from_hex(expected_id_str.as_bytes()).expect("failed to parse expected id");

        let entry = ops::get_tree_entry(&handle, tree_id, "README.md")
            .expect("failed to get tree entry");

        assert_eq!(entry.id, expected_id);
    }
}

mod list_tree_recursive {
    use super::*;

    #[test]
    fn lists_all_entries() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let entries = ops::list_tree_recursive(&handle, tree_id, None)
            .expect("failed to list tree");

        let paths: Vec<_> = entries.iter().map(|e| e.path.to_string()).collect();

        assert!(paths.iter().any(|p| p == "README.md" || p.ends_with("/README.md") || p.contains("README.md")));
        assert!(paths.iter().any(|p| p.contains("main.rs")));
        assert!(paths.iter().any(|p| p.contains("lib.rs")));
        assert!(paths.iter().any(|p| p.contains("guide.md")));
        assert!(paths.iter().any(|p| p.contains("data.bin")));
    }

    #[test]
    fn includes_directories() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let entries = ops::list_tree_recursive(&handle, tree_id, None)
            .expect("failed to list tree");

        let tree_entries: Vec<_> = entries.iter()
            .filter(|e| is_tree_mode(&e.entry.mode))
            .collect();

        assert!(!tree_entries.is_empty());
    }

    #[test]
    fn max_depth_zero_returns_root_only() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let entries = ops::list_tree_recursive(&handle, tree_id, Some(0))
            .expect("failed to list tree");

        for entry in &entries {
            let depth = if entry.path.is_empty() { 0 } else { entry.path.to_string().matches('/').count() + 1 };
            assert!(depth <= 1, "entry {:?} exceeds max_depth 0", entry.path);
        }
    }

    #[test]
    fn max_depth_one_limits_nesting() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let entries = ops::list_tree_recursive(&handle, tree_id, Some(1))
            .expect("failed to list tree");

        for entry in &entries {
            let depth = if entry.path.is_empty() { 0 } else { entry.path.to_string().matches('/').count() + 1 };
            assert!(depth <= 2, "entry {:?} exceeds max_depth 1", entry.path);
        }
    }

    #[test]
    fn unlimited_depth_gets_all_nested() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let limited = ops::list_tree_recursive(&handle, tree_id, Some(0))
            .expect("failed to list tree with limit");
        let unlimited = ops::list_tree_recursive(&handle, tree_id, None)
            .expect("failed to list tree without limit");

        assert!(unlimited.len() >= limited.len());
    }

    #[test]
    fn nonexistent_tree_returns_error() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let fake_id = gix_hash::ObjectId::from_hex(b"0000000000000000000000000000000000000000")
            .expect("valid hex");

        let result = ops::list_tree_recursive(&handle, fake_id, None);
        assert!(result.is_err());
        match result.unwrap_err() {
            SdkError::ObjectNotFound(_) => {}
            other => panic!("expected ObjectNotFound, got: {:?}", other),
        }
    }

    #[test]
    fn blob_id_returns_invalid_object_type() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let blob_id_str = repo.git_output(&["rev-parse", "HEAD:README.md"]);
        let blob_id =
            gix_hash::ObjectId::from_hex(blob_id_str.as_bytes()).expect("failed to parse blob id");

        let result = ops::list_tree_recursive(&handle, blob_id, None);
        assert!(result.is_err());
        match result.unwrap_err() {
            SdkError::InvalidObjectType { expected, actual } => {
                assert_eq!(expected, "tree");
                assert_eq!(actual, "blob");
            }
            other => panic!("expected InvalidObjectType, got: {:?}", other),
        }
    }

    #[test]
    fn commit_id_returns_invalid_object_type() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let result = ops::list_tree_recursive(&handle, commit_id, None);
        assert!(result.is_err());
        match result.unwrap_err() {
            SdkError::InvalidObjectType { expected, actual } => {
                assert_eq!(expected, "tree");
                assert_eq!(actual, "commit");
            }
            other => panic!("expected InvalidObjectType, got: {:?}", other),
        }
    }

    #[test]
    fn subtree_recursive() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD:src"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let entries = ops::list_tree_recursive(&handle, tree_id, None)
            .expect("failed to list subtree");

        let names: Vec<_> = entries.iter().map(|e| e.entry.name.to_string()).collect();
        assert!(names.contains(&"main.rs".to_string()));
        assert!(names.contains(&"lib.rs".to_string()));
    }

    #[test]
    fn entry_paths_are_correct() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let entries = ops::list_tree_recursive(&handle, tree_id, None)
            .expect("failed to list tree");

        for entry in &entries {
            let path_str = entry.path.to_string();
            let name_str = entry.entry.name.to_string();
            if !path_str.is_empty() {
                assert!(path_str.ends_with(&name_str) || path_str == name_str,
                    "path {:?} should end with name {:?}", path_str, name_str);
            }
        }
    }

    #[test]
    fn deep_tree_with_history() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let entries = ops::list_tree_recursive(&handle, tree_id, None)
            .expect("failed to list tree");

        assert!(!entries.is_empty());
    }

    #[test]
    fn different_max_depths_return_different_counts() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let depth_0 = ops::list_tree_recursive(&handle, tree_id, Some(0))
            .expect("failed to list tree");
        let depth_1 = ops::list_tree_recursive(&handle, tree_id, Some(1))
            .expect("failed to list tree");
        let depth_none = ops::list_tree_recursive(&handle, tree_id, None)
            .expect("failed to list tree");

        assert!(depth_1.len() >= depth_0.len());
        assert!(depth_none.len() >= depth_1.len());
    }
}

mod get_path_at_commit {
    use super::*;

    #[test]
    fn file_at_root() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let entry = ops::get_path_at_commit(&handle, commit_id, "README.md")
            .expect("failed to get path at commit");

        assert_eq!(entry.name.to_string(), "README.md");
        assert!(is_blob_mode(&entry.mode));
    }

    #[test]
    fn directory_at_root() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let entry = ops::get_path_at_commit(&handle, commit_id, "src")
            .expect("failed to get path at commit");

        assert_eq!(entry.name.to_string(), "src");
        assert!(is_tree_mode(&entry.mode));
    }

    #[test]
    fn file_in_nested_directory() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let entry = ops::get_path_at_commit(&handle, commit_id, "src/main.rs")
            .expect("failed to get path at commit");

        assert_eq!(entry.name.to_string(), "main.rs");
        assert!(is_blob_mode(&entry.mode));
    }

    #[test]
    fn file_at_older_commit() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD~3"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let entry = ops::get_path_at_commit(&handle, commit_id, "src/main.rs")
            .expect("failed to get path at commit");

        assert_eq!(entry.name.to_string(), "main.rs");
    }

    #[test]
    fn nonexistent_path_returns_error() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let result = ops::get_path_at_commit(&handle, commit_id, "nonexistent.txt");
        assert!(result.is_err());
        match result.unwrap_err() {
            SdkError::TreeEntryNotFound(path) => {
                assert_eq!(path, "nonexistent.txt");
            }
            other => panic!("expected TreeEntryNotFound, got: {:?}", other),
        }
    }

    #[test]
    fn nonexistent_nested_path_returns_error() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let result = ops::get_path_at_commit(&handle, commit_id, "nonexistent/path/file.txt");
        assert!(result.is_err());
        match result.unwrap_err() {
            SdkError::TreeEntryNotFound(path) => {
                assert_eq!(path, "nonexistent/path/file.txt");
            }
            other => panic!("expected TreeEntryNotFound, got: {:?}", other),
        }
    }

    #[test]
    fn nonexistent_commit_returns_error() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let fake_id = gix_hash::ObjectId::from_hex(b"0000000000000000000000000000000000000000")
            .expect("valid hex");

        let result = ops::get_path_at_commit(&handle, fake_id, "README.md");
        assert!(result.is_err());
        match result.unwrap_err() {
            SdkError::ObjectNotFound(_) => {}
            other => panic!("expected ObjectNotFound, got: {:?}", other),
        }
    }

    #[test]
    fn tree_id_returns_invalid_object_type() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let result = ops::get_path_at_commit(&handle, tree_id, "README.md");
        assert!(result.is_err());
        match result.unwrap_err() {
            SdkError::InvalidObjectType { expected, actual } => {
                assert_eq!(expected, "commit");
                assert_eq!(actual, "tree");
            }
            other => panic!("expected InvalidObjectType, got: {:?}", other),
        }
    }

    #[test]
    fn blob_id_returns_invalid_object_type() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let blob_id_str = repo.git_output(&["rev-parse", "HEAD:README.md"]);
        let blob_id =
            gix_hash::ObjectId::from_hex(blob_id_str.as_bytes()).expect("failed to parse blob id");

        let result = ops::get_path_at_commit(&handle, blob_id, "README.md");
        assert!(result.is_err());
        match result.unwrap_err() {
            SdkError::InvalidObjectType { expected, actual } => {
                assert_eq!(expected, "commit");
                assert_eq!(actual, "blob");
            }
            other => panic!("expected InvalidObjectType, got: {:?}", other),
        }
    }

    #[test]
    fn empty_path_returns_error() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let result = ops::get_path_at_commit(&handle, commit_id, "");
        assert!(result.is_err());
        match result.unwrap_err() {
            SdkError::TreeEntryNotFound(path) => {
                assert_eq!(path, "");
            }
            other => panic!("expected TreeEntryNotFound, got: {:?}", other),
        }
    }

    #[test]
    fn binary_file_path() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let entry = ops::get_path_at_commit(&handle, commit_id, "data.bin")
            .expect("failed to get path at commit");

        assert_eq!(entry.name.to_string(), "data.bin");
        assert!(is_blob_mode(&entry.mode));
    }

    #[test]
    fn file_id_matches_git() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let expected_id_str = repo.git_output(&["rev-parse", "HEAD:src/main.rs"]);
        let expected_id =
            gix_hash::ObjectId::from_hex(expected_id_str.as_bytes()).expect("failed to parse expected id");

        let entry = ops::get_path_at_commit(&handle, commit_id, "src/main.rs")
            .expect("failed to get path at commit");

        assert_eq!(entry.id, expected_id);
    }

    #[test]
    fn different_commits_same_path_may_differ() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id = gix_hash::ObjectId::from_hex(head_id_str.as_bytes())
            .expect("failed to parse commit id");

        let old_id_str = repo.git_output(&["rev-parse", "HEAD~3"]);
        let old_id = gix_hash::ObjectId::from_hex(old_id_str.as_bytes())
            .expect("failed to parse commit id");

        let head_entry = ops::get_path_at_commit(&handle, head_id, "src/main.rs")
            .expect("failed to get path at HEAD");
        let old_entry = ops::get_path_at_commit(&handle, old_id, "src/main.rs")
            .expect("failed to get path at old commit");

        assert_eq!(head_entry.name.to_string(), "main.rs");
        assert_eq!(old_entry.name.to_string(), "main.rs");
    }

    #[test]
    fn docs_guide_at_commit() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let entry = ops::get_path_at_commit(&handle, commit_id, "docs/guide.md")
            .expect("failed to get path at commit");

        assert_eq!(entry.name.to_string(), "guide.md");
        assert!(is_blob_mode(&entry.mode));
    }

    #[test]
    fn detached_head_commit() {
        let repo = TestRepo::with_detached_head();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let entry = ops::get_path_at_commit(&handle, commit_id, "README.md")
            .expect("failed to get path at commit");

        assert_eq!(entry.name.to_string(), "README.md");
    }
}

mod tree_entry_with_path {
    use super::*;

    #[test]
    fn entry_has_correct_fields() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let entries = ops::list_tree_recursive(&handle, tree_id, None)
            .expect("failed to list tree");

        for entry in entries {
            assert!(!entry.entry.id.is_null());
            assert!(!entry.entry.name.is_empty());
        }
    }

    #[test]
    fn path_and_name_consistent() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let entries = ops::list_tree_recursive(&handle, tree_id, None)
            .expect("failed to list tree");

        for entry in entries {
            let path_str = entry.path.to_string();
            let name_str = entry.entry.name.to_string();

            if !path_str.is_empty() {
                let path_name = path_str.rsplit('/').next().unwrap_or(&path_str);
                assert_eq!(path_name, name_str,
                    "path {:?} should have name {:?} as last component", path_str, name_str);
            }
        }
    }
}

mod depth_limit_edge_cases {
    use super::*;

    #[test]
    fn max_depth_two_limits_deeply_nested() {
        let repo = TestRepo::with_deep_nesting();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let entries = ops::list_tree_recursive(&handle, tree_id, Some(2))
            .expect("failed to list tree");

        for entry in &entries {
            let depth = if entry.path.is_empty() {
                0
            } else {
                entry.path.to_string().matches('/').count() + 1
            };
            assert!(depth <= 3, "entry {:?} exceeds max_depth 2", entry.path);
        }
    }

    #[test]
    fn max_depth_exactly_at_boundary() {
        let repo = TestRepo::with_deep_nesting();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let depth_2 = ops::list_tree_recursive(&handle, tree_id, Some(2))
            .expect("failed to list tree with depth 2");
        let depth_3 = ops::list_tree_recursive(&handle, tree_id, Some(3))
            .expect("failed to list tree with depth 3");
        let depth_none = ops::list_tree_recursive(&handle, tree_id, None)
            .expect("failed to list tree unlimited");

        assert!(depth_3.len() >= depth_2.len());
        assert!(depth_none.len() >= depth_3.len());
    }

    #[test]
    fn single_file_tree_no_depth() {
        let repo = TestRepo::single_file();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let entries = ops::list_tree_recursive(&handle, tree_id, None)
            .expect("failed to list tree");

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].entry.name.to_string(), "single.txt");
    }

    #[test]
    fn single_file_tree_with_depth_zero() {
        let repo = TestRepo::single_file();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let entries = ops::list_tree_recursive(&handle, tree_id, Some(0))
            .expect("failed to list tree");

        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn deeply_nested_single_path() {
        let repo = TestRepo::with_deep_single_path();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let entries_unlimited = ops::list_tree_recursive(&handle, tree_id, None)
            .expect("failed to list tree");

        let entries_limited = ops::list_tree_recursive(&handle, tree_id, Some(1))
            .expect("failed to list tree");

        assert!(entries_unlimited.len() > entries_limited.len());
    }

    #[test]
    fn traversal_visits_all_siblings() {
        let repo = TestRepo::with_many_siblings();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let entries = ops::list_tree_recursive(&handle, tree_id, None)
            .expect("failed to list tree");

        let names: Vec<_> = entries.iter().map(|e| e.entry.name.to_string()).collect();
        assert!(names.contains(&"file1.txt".to_string()));
        assert!(names.contains(&"file2.txt".to_string()));
        assert!(names.contains(&"file3.txt".to_string()));
        assert!(names.contains(&"dir1".to_string()));
        assert!(names.contains(&"dir2".to_string()));
    }

    #[test]
    fn pop_path_handles_root_level_entry() {
        let repo = TestRepo::single_file();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let entries = ops::list_tree_recursive(&handle, tree_id, None)
            .expect("failed to list tree");

        for entry in entries {
            let path_str = entry.path.to_string();
            assert!(!path_str.contains("//"), "path should not have double slashes: {}", path_str);
        }
    }

    #[test]
    fn multiple_nested_directories_with_limit() {
        let repo = TestRepo::with_multiple_nested_dirs();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let entries_0 = ops::list_tree_recursive(&handle, tree_id, Some(0))
            .expect("depth 0");
        let entries_1 = ops::list_tree_recursive(&handle, tree_id, Some(1))
            .expect("depth 1");
        let entries_2 = ops::list_tree_recursive(&handle, tree_id, Some(2))
            .expect("depth 2");

        assert!(entries_1.len() >= entries_0.len());
        assert!(entries_2.len() >= entries_1.len());
    }
}

mod get_tree_entry_edge_cases {
    use super::*;

    #[test]
    fn deeply_nested_file_path() {
        let repo = TestRepo::with_deep_nesting();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let entry = ops::get_tree_entry(&handle, tree_id, "a/b/c/deep.txt")
            .expect("failed to get deeply nested entry");

        assert_eq!(entry.name.to_string(), "deep.txt");
    }

    #[test]
    fn path_with_multiple_consecutive_slashes() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let entry = ops::get_tree_entry(&handle, tree_id, "src//main.rs")
            .expect("failed to get entry with double slash");

        assert_eq!(entry.name.to_string(), "main.rs");
    }

    #[test]
    fn intermediate_directory_entry() {
        let repo = TestRepo::with_deep_nesting();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let entry = ops::get_tree_entry(&handle, tree_id, "a/b")
            .expect("failed to get intermediate directory");

        assert_eq!(entry.name.to_string(), "b");
        assert!(is_tree_mode(&entry.mode));
    }

    #[test]
    fn path_through_nonexistent_middle_component() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let result = ops::get_tree_entry(&handle, tree_id, "src/nonexistent/main.rs");
        assert!(result.is_err());
        match result.unwrap_err() {
            SdkError::TreeEntryNotFound(path) => {
                assert_eq!(path, "src/nonexistent/main.rs");
            }
            other => panic!("expected TreeEntryNotFound, got: {:?}", other),
        }
    }
}

mod get_path_at_commit_edge_cases {
    use super::*;

    #[test]
    fn tag_id_returns_invalid_object_type() {
        let repo = TestRepo::with_tags();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tag_id_str = repo.git_output(&["rev-parse", "refs/tags/v1.1.0"]);
        let tag_id =
            gix_hash::ObjectId::from_hex(tag_id_str.as_bytes()).expect("failed to parse tag id");

        let result = ops::get_path_at_commit(&handle, tag_id, "README.md");
        assert!(result.is_err());
        match result.unwrap_err() {
            SdkError::InvalidObjectType { expected, actual } => {
                assert_eq!(expected, "commit");
                assert_eq!(actual, "tag");
            }
            other => panic!("expected InvalidObjectType, got: {:?}", other),
        }
    }

    #[test]
    fn slash_only_path_returns_error() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let result = ops::get_path_at_commit(&handle, commit_id, "/");
        assert!(result.is_err());
        match result.unwrap_err() {
            SdkError::TreeEntryNotFound(path) => {
                assert_eq!(path, "/");
            }
            other => panic!("expected TreeEntryNotFound, got: {:?}", other),
        }
    }

    #[test]
    fn path_with_leading_slash() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let entry = ops::get_path_at_commit(&handle, commit_id, "/README.md")
            .expect("failed to get path with leading slash");

        assert_eq!(entry.name.to_string(), "README.md");
    }

    #[test]
    fn deeply_nested_path_at_commit() {
        let repo = TestRepo::with_deep_nesting();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let entry = ops::get_path_at_commit(&handle, commit_id, "a/b/c/deep.txt")
            .expect("failed to get deeply nested path");

        assert_eq!(entry.name.to_string(), "deep.txt");
    }
}

mod list_tree_recursive_additional {
    use super::*;

    #[test]
    fn tree_with_subdir_containing_single_file() {
        let repo = TestRepo::with_empty_subdirs();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let entries = ops::list_tree_recursive(&handle, tree_id, None)
            .expect("failed to list tree");

        let names: Vec<_> = entries.iter().map(|e| e.entry.name.to_string()).collect();
        assert!(names.contains(&"empty_dir".to_string()));
        assert!(names.contains(&".gitkeep".to_string()));
        assert!(names.contains(&"file.txt".to_string()));
    }

    #[test]
    fn traversal_with_large_depth_limit() {
        let repo = TestRepo::with_deep_single_path();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let entries = ops::list_tree_recursive(&handle, tree_id, Some(100))
            .expect("failed to list tree");

        let names: Vec<_> = entries.iter().map(|e| e.entry.name.to_string()).collect();
        assert!(names.contains(&"deepest.txt".to_string()));
    }

    #[test]
    fn traversal_with_depth_exactly_matching_structure() {
        let repo = TestRepo::with_deep_nesting();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let entries_3 = ops::list_tree_recursive(&handle, tree_id, Some(3))
            .expect("failed to list tree");
        let entries_4 = ops::list_tree_recursive(&handle, tree_id, Some(4))
            .expect("failed to list tree");

        assert!(entries_4.len() >= entries_3.len());
    }

    #[test]
    fn all_entries_have_non_empty_names() {
        let repo = TestRepo::with_many_siblings();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let entries = ops::list_tree_recursive(&handle, tree_id, None)
            .expect("failed to list tree");

        for entry in entries {
            assert!(!entry.entry.name.is_empty(), "entry name should not be empty");
        }
    }
}

mod list_tree_recursive_breadthfirst {
    use super::*;

    #[test]
    fn lists_all_entries_breadthfirst() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let entries = ops::list_tree_recursive_breadthfirst(&handle, tree_id, None)
            .expect("failed to list tree");

        let paths: Vec<_> = entries.iter().map(|e| e.path.to_string()).collect();

        assert!(paths.iter().any(|p| p == "README.md" || p.ends_with("/README.md") || p.contains("README.md")));
        assert!(paths.iter().any(|p| p.contains("main.rs")));
    }

    #[test]
    fn breadthfirst_with_depth_limit() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let entries = ops::list_tree_recursive_breadthfirst(&handle, tree_id, Some(1))
            .expect("failed to list tree");

        for entry in &entries {
            let depth = if entry.path.is_empty() { 0 } else { entry.path.to_string().matches('/').count() + 1 };
            assert!(depth <= 2, "entry {:?} exceeds max_depth 1", entry.path);
        }
    }

    #[test]
    fn breadthfirst_matches_depthfirst_entries() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let bf_entries = ops::list_tree_recursive_breadthfirst(&handle, tree_id, None)
            .expect("failed to list tree breadthfirst");
        let df_entries = ops::list_tree_recursive(&handle, tree_id, None)
            .expect("failed to list tree depthfirst");

        let mut bf_names: Vec<_> = bf_entries.iter().map(|e| e.entry.name.to_string()).collect();
        let mut df_names: Vec<_> = df_entries.iter().map(|e| e.entry.name.to_string()).collect();

        bf_names.sort();
        df_names.sort();

        assert_eq!(bf_names, df_names);
    }

    #[test]
    fn breadthfirst_nonexistent_tree_returns_error() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let fake_id = gix_hash::ObjectId::from_hex(b"0000000000000000000000000000000000000000")
            .expect("valid hex");

        let result = ops::list_tree_recursive_breadthfirst(&handle, fake_id, None);
        assert!(result.is_err());
        match result.unwrap_err() {
            SdkError::ObjectNotFound(_) => {}
            other => panic!("expected ObjectNotFound, got: {:?}", other),
        }
    }

    #[test]
    fn breadthfirst_blob_id_returns_invalid_object_type() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let blob_id_str = repo.git_output(&["rev-parse", "HEAD:README.md"]);
        let blob_id =
            gix_hash::ObjectId::from_hex(blob_id_str.as_bytes()).expect("failed to parse blob id");

        let result = ops::list_tree_recursive_breadthfirst(&handle, blob_id, None);
        assert!(result.is_err());
        match result.unwrap_err() {
            SdkError::InvalidObjectType { expected, actual } => {
                assert_eq!(expected, "tree");
                assert_eq!(actual, "blob");
            }
            other => panic!("expected InvalidObjectType, got: {:?}", other),
        }
    }

    #[test]
    fn breadthfirst_deeply_nested() {
        let repo = TestRepo::with_deep_nesting();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let entries = ops::list_tree_recursive_breadthfirst(&handle, tree_id, None)
            .expect("failed to list tree");

        let names: Vec<_> = entries.iter().map(|e| e.entry.name.to_string()).collect();
        assert!(names.contains(&"deep.txt".to_string()));
    }
}

mod get_tree_additional {
    use super::*;

    #[test]
    fn tree_with_gitkeep_file() {
        let repo = TestRepo::with_empty_subdirs();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD:empty_dir"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let entries = ops::get_tree(&handle, tree_id).expect("failed to get tree");

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name.to_string(), ".gitkeep");
    }

    #[test]
    fn deeply_nested_subtree() {
        let repo = TestRepo::with_deep_single_path();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD:level1/level2/level3"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let entries = ops::get_tree(&handle, tree_id).expect("failed to get subtree");

        let names: Vec<_> = entries.iter().map(|e| e.name.to_string()).collect();
        assert!(names.contains(&"level4".to_string()));
    }
}

mod traversal_error_paths {
    use super::*;

    #[test]
    fn depthfirst_traversal_with_corrupt_nested_tree() {
        let repo = TestRepo::with_corrupt_tree_reference();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let result = ops::list_tree_recursive(&handle, tree_id, None);
        assert!(result.is_err());
        match result.unwrap_err() {
            SdkError::Git(_) => {}
            other => panic!("expected Git error, got: {:?}", other),
        }
    }

    #[test]
    fn breadthfirst_traversal_with_corrupt_nested_tree() {
        let repo = TestRepo::with_corrupt_tree_reference();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let result = ops::list_tree_recursive_breadthfirst(&handle, tree_id, None);
        assert!(result.is_err());
        match result.unwrap_err() {
            SdkError::Git(_) => {}
            other => panic!("expected Git error, got: {:?}", other),
        }
    }
}

mod visit_trait_coverage {
    use super::*;

    #[test]
    fn pop_back_tracked_path_with_empty_deque() {
        let repo = TestRepo::single_file();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let entries = ops::list_tree_recursive(&handle, tree_id, None)
            .expect("failed to list tree");

        assert_eq!(entries.len(), 1);
        assert!(entries[0].path.to_string() == "single.txt" || entries[0].entry.name.to_string() == "single.txt");
    }

    #[test]
    fn multiple_directory_traversal_exercises_path_tracking() {
        let repo = TestRepo::with_multiple_nested_dirs();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let entries = ops::list_tree_recursive(&handle, tree_id, None)
            .expect("failed to list tree");

        let paths: Vec<_> = entries.iter().map(|e| e.path.to_string()).collect();
        assert!(paths.iter().any(|p| p.contains("aa_file.txt")));
        assert!(paths.iter().any(|p| p.contains("bb_file.txt")));
    }

    #[test]
    fn breadthfirst_exercises_pop_front_tracked_path() {
        let repo = TestRepo::with_deep_nesting();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let entries = ops::list_tree_recursive_breadthfirst(&handle, tree_id, None)
            .expect("failed to list tree");

        let names: Vec<_> = entries.iter().map(|e| e.entry.name.to_string()).collect();
        assert!(names.contains(&"deep.txt".to_string()));
        assert!(names.contains(&"a".to_string()));
        assert!(names.contains(&"b".to_string()));
        assert!(names.contains(&"c".to_string()));
    }

    #[test]
    fn depthfirst_exercises_pop_back_tracked_path() {
        let repo = TestRepo::with_many_siblings();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let entries = ops::list_tree_recursive(&handle, tree_id, None)
            .expect("failed to list tree");

        let paths: Vec<_> = entries.iter().map(|e| e.path.to_string()).collect();
        assert!(paths.iter().any(|p| p.contains("nested1.txt")));
        assert!(paths.iter().any(|p| p.contains("nested2.txt")));
    }

    #[test]
    fn visit_tree_action_skip_at_max_depth() {
        let repo = TestRepo::with_deep_nesting();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let depth_0 = ops::list_tree_recursive(&handle, tree_id, Some(0))
            .expect("failed with depth 0");
        let depth_1 = ops::list_tree_recursive(&handle, tree_id, Some(1))
            .expect("failed with depth 1");
        let unlimited = ops::list_tree_recursive(&handle, tree_id, None)
            .expect("failed with unlimited depth");

        assert!(depth_1.len() >= depth_0.len(), "depth 1 should have at least as many entries as depth 0");
        assert!(unlimited.len() >= depth_1.len(), "unlimited should have at least as many entries as depth 1");

        for entry in &depth_0 {
            let depth = entry.path.to_string().matches('/').count();
            assert!(depth <= 1, "depth 0 should not have entries deeper than 1 level");
        }
    }

    #[test]
    fn visit_nontree_returns_action_continue() {
        let repo = TestRepo::with_many_siblings();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let entries = ops::list_tree_recursive(&handle, tree_id, None)
            .expect("failed to list tree");

        let blob_entries: Vec<_> = entries.iter()
            .filter(|e| is_blob_mode(&e.entry.mode))
            .collect();

        assert!(blob_entries.len() >= 3);
        assert!(blob_entries.iter().any(|e| e.entry.name.to_string() == "file1.txt"));
        assert!(blob_entries.iter().any(|e| e.entry.name.to_string() == "file2.txt"));
        assert!(blob_entries.iter().any(|e| e.entry.name.to_string() == "file3.txt"));
    }

    #[test]
    fn push_and_pop_path_component_consistency() {
        let repo = TestRepo::with_deep_single_path();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let entries = ops::list_tree_recursive(&handle, tree_id, None)
            .expect("failed to list tree");

        for entry in &entries {
            let path_str = entry.path.to_string();
            let name_str = entry.entry.name.to_string();
            if !path_str.is_empty() {
                let last_component = path_str.rsplit('/').next().unwrap_or(&path_str);
                assert_eq!(last_component, name_str,
                    "path {} should end with name {}", path_str, name_str);
            }
        }
    }
}
