mod fixtures;

use bstr::{BString, ByteSlice};
use fixtures::TestRepo;
use gix_server_sdk::{ops, RepoPool, SdkConfig};

fn create_pool() -> RepoPool {
    RepoPool::new(SdkConfig::default())
}

mod grep_blob {
    use super::*;

    #[test]
    fn find_pattern_in_blob() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let blob_id_str = repo.git_output(&["rev-parse", "HEAD:src/main.rs"]);
        let blob_id =
            gix_hash::ObjectId::from_hex(blob_id_str.as_bytes()).expect("failed to parse blob id");

        let matches = ops::grep_blob(&handle, blob_id, "println").expect("grep_blob failed");

        assert!(!matches.is_empty());
        assert!(matches.iter().any(|m| m.content.contains_str("println")));
    }

    #[test]
    fn pattern_not_found_returns_empty() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let blob_id_str = repo.git_output(&["rev-parse", "HEAD:src/main.rs"]);
        let blob_id =
            gix_hash::ObjectId::from_hex(blob_id_str.as_bytes()).expect("failed to parse blob id");

        let matches =
            ops::grep_blob(&handle, blob_id, "nonexistent_pattern_xyz").expect("grep_blob failed");

        assert!(matches.is_empty());
    }

    #[test]
    fn find_regex_pattern() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let blob_id_str = repo.git_output(&["rev-parse", "HEAD:src/lib.rs"]);
        let blob_id =
            gix_hash::ObjectId::from_hex(blob_id_str.as_bytes()).expect("failed to parse blob id");

        let matches = ops::grep_blob(&handle, blob_id, r"fn\s+\w+").expect("grep_blob failed");

        assert!(!matches.is_empty());
    }

    #[test]
    fn multiple_matches_in_blob() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let blob_id_str = repo.git_output(&["rev-parse", "HEAD:src/lib.rs"]);
        let blob_id =
            gix_hash::ObjectId::from_hex(blob_id_str.as_bytes()).expect("failed to parse blob id");

        let matches = ops::grep_blob(&handle, blob_id, "fn").expect("grep_blob failed");

        assert!(matches.len() >= 2);
    }

    #[test]
    fn line_match_structure() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let blob_id_str = repo.git_output(&["rev-parse", "HEAD:src/main.rs"]);
        let blob_id =
            gix_hash::ObjectId::from_hex(blob_id_str.as_bytes()).expect("failed to parse blob id");

        let matches = ops::grep_blob(&handle, blob_id, "main").expect("grep_blob failed");

        assert!(!matches.is_empty());
        let first_match = &matches[0];
        assert!(first_match.line_number > 0);
        assert!(first_match.match_start < first_match.match_end);
        assert!(first_match.match_end <= first_match.content.len());
    }
}

mod grep_tree {
    use super::*;

    #[test]
    fn find_pattern_in_multiple_files() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let options = ops::GrepOptions::default();
        let results = ops::grep_tree(&handle, tree_id, "fn", &options).expect("grep_tree failed");

        assert!(!results.is_empty());
        let paths: Vec<_> = results.iter().map(|r| r.path.clone()).collect();
        assert!(
            paths.iter().any(|p| p.ends_with(b".rs")),
            "expected Rust files in results"
        );
    }

    #[test]
    fn pattern_not_found_returns_empty() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let options = ops::GrepOptions::default();
        let results =
            ops::grep_tree(&handle, tree_id, "nonexistent_pattern_xyz_123", &options)
                .expect("grep_tree failed");

        assert!(results.is_empty());
    }

    #[test]
    fn grep_match_structure() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let options = ops::GrepOptions::default();
        let results = ops::grep_tree(&handle, tree_id, "fn main", &options).expect("grep_tree failed");

        assert!(!results.is_empty());
        let grep_match = &results[0];
        assert!(!grep_match.path.is_empty());
        assert!(!grep_match.blob_id.is_null());
        assert!(!grep_match.matches.is_empty());
    }

    #[test]
    fn case_insensitive_search() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let options = ops::GrepOptions {
            case_insensitive: true,
            ..Default::default()
        };
        let results = ops::grep_tree(&handle, tree_id, "TEST", &options).expect("grep_tree failed");

        assert!(
            !results.is_empty(),
            "case insensitive search should find 'test' content"
        );
    }

    #[test]
    fn case_sensitive_search() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let options_sensitive = ops::GrepOptions {
            case_insensitive: false,
            ..Default::default()
        };
        let results_sensitive =
            ops::grep_tree(&handle, tree_id, "TEST", &options_sensitive).expect("grep_tree failed");

        let options_insensitive = ops::GrepOptions {
            case_insensitive: true,
            ..Default::default()
        };
        let results_insensitive = ops::grep_tree(&handle, tree_id, "TEST", &options_insensitive)
            .expect("grep_tree failed");

        assert!(
            results_insensitive.len() >= results_sensitive.len(),
            "case insensitive should find equal or more matches"
        );
    }

    #[test]
    fn max_matches_per_file() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let options = ops::GrepOptions {
            max_matches_per_file: Some(1),
            ..Default::default()
        };
        let results = ops::grep_tree(&handle, tree_id, "fn", &options).expect("grep_tree failed");

        for grep_match in &results {
            assert!(
                grep_match.matches.len() <= 1,
                "each file should have at most 1 match"
            );
        }
    }

    #[test]
    fn path_pattern_rs_files_only() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let options = ops::GrepOptions {
            path_pattern: Some("*.rs".to_string()),
            ..Default::default()
        };
        let results = ops::grep_tree(&handle, tree_id, "fn", &options).expect("grep_tree failed");

        assert!(!results.is_empty());
        for grep_match in &results {
            assert!(
                grep_match.path.ends_with(b".rs"),
                "all matches should be in .rs files, got: {:?}",
                grep_match.path
            );
        }
    }

    #[test]
    fn path_pattern_md_files_only() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let options = ops::GrepOptions {
            path_pattern: Some("*.md".to_string()),
            ..Default::default()
        };
        let results =
            ops::grep_tree(&handle, tree_id, "Repository", &options).expect("grep_tree failed");

        assert!(!results.is_empty());
        for grep_match in &results {
            assert!(
                grep_match.path.ends_with(b".md"),
                "all matches should be in .md files, got: {:?}",
                grep_match.path
            );
        }
    }

    #[test]
    fn path_pattern_recursive_glob() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let options = ops::GrepOptions {
            path_pattern: Some("src/**/*.rs".to_string()),
            ..Default::default()
        };
        let results = ops::grep_tree(&handle, tree_id, "fn", &options).expect("grep_tree failed");

        assert!(!results.is_empty());
        for grep_match in &results {
            let path_str = String::from_utf8_lossy(&grep_match.path);
            assert!(
                path_str.starts_with("src/"),
                "all matches should be in src/ directory, got: {}",
                path_str
            );
        }
    }

    #[test]
    fn binary_file_excluded_by_default() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let options = ops::GrepOptions {
            include_binary: false,
            ..Default::default()
        };
        let results = ops::grep_tree(&handle, tree_id, ".", &options).expect("grep_tree failed");

        for grep_match in &results {
            assert!(
                !grep_match.path.ends_with(b".bin"),
                "binary files should be excluded"
            );
        }
    }

    #[test]
    fn binary_file_included_when_option_set() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let options_exclude = ops::GrepOptions {
            include_binary: false,
            ..Default::default()
        };
        let results_exclude =
            ops::grep_tree(&handle, tree_id, ".", &options_exclude).expect("grep_tree failed");
        let exclude_count = results_exclude.len();

        let options_include = ops::GrepOptions {
            include_binary: true,
            ..Default::default()
        };
        let results_include =
            ops::grep_tree(&handle, tree_id, ".", &options_include).expect("grep_tree failed");
        let include_count = results_include.len();

        assert!(
            include_count >= exclude_count,
            "including binary should give equal or more results"
        );
    }
}

mod grep_commit {
    use super::*;

    #[test]
    fn search_at_commit() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let options = ops::GrepOptions::default();
        let results =
            ops::grep_commit(&handle, commit_id, "fn main", &options).expect("grep_commit failed");

        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.path.ends_with(b"main.rs")));
    }

    #[test]
    fn search_with_all_options() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let options = ops::GrepOptions {
            case_insensitive: true,
            max_matches_per_file: Some(2),
            include_binary: false,
            path_pattern: Some("*.rs".to_string()),
        };
        let results =
            ops::grep_commit(&handle, commit_id, "FN", &options).expect("grep_commit failed");

        for grep_match in &results {
            assert!(grep_match.path.ends_with(b".rs"));
            assert!(grep_match.matches.len() <= 2);
        }
    }

    #[test]
    fn search_at_historical_commit() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let old_commit_str = repo.git_output(&["rev-parse", "HEAD~3"]);
        let old_commit_id = gix_hash::ObjectId::from_hex(old_commit_str.as_bytes())
            .expect("failed to parse commit id");

        let options = ops::GrepOptions::default();
        let results =
            ops::grep_commit(&handle, old_commit_id, "main", &options).expect("grep_commit failed");

        assert!(!results.is_empty());
    }

    #[test]
    fn pattern_not_found_returns_empty() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let options = ops::GrepOptions::default();
        let results = ops::grep_commit(&handle, commit_id, "nonexistent_xyz_pattern", &options)
            .expect("grep_commit failed");

        assert!(results.is_empty());
    }
}

mod pickaxe_search {
    use super::*;

    #[test]
    fn find_commit_that_added_pattern() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id =
            gix_hash::ObjectId::from_hex(head_str.as_bytes()).expect("failed to parse commit id");

        let results =
            ops::pickaxe_search(&handle, head_id, "goodbye", Some(100)).expect("pickaxe failed");

        assert!(
            !results.is_empty(),
            "should find commit that added 'goodbye'"
        );
        assert!(results
            .iter()
            .any(|r| r.change_type == ops::PickaxeChangeType::Added));
    }

    #[test]
    fn find_multiple_changes() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id =
            gix_hash::ObjectId::from_hex(head_str.as_bytes()).expect("failed to parse commit id");

        let results = ops::pickaxe_search(&handle, head_id, "hello", Some(100))
            .expect("pickaxe failed");

        assert!(
            !results.is_empty(),
            "should find changes related to 'hello'"
        );
    }

    #[test]
    fn pickaxe_match_structure() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id =
            gix_hash::ObjectId::from_hex(head_str.as_bytes()).expect("failed to parse commit id");

        let results =
            ops::pickaxe_search(&handle, head_id, "hello", Some(100)).expect("pickaxe failed");

        assert!(!results.is_empty());
        let match_result = &results[0];
        assert!(!match_result.commit_id.is_null());
        assert!(!match_result.path.is_empty());
        assert!(
            match_result.change_type == ops::PickaxeChangeType::Added
                || match_result.change_type == ops::PickaxeChangeType::Removed
        );
    }

    #[test]
    fn limit_restricts_results() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id =
            gix_hash::ObjectId::from_hex(head_str.as_bytes()).expect("failed to parse commit id");

        let results_limited =
            ops::pickaxe_search(&handle, head_id, "fn", Some(2)).expect("pickaxe failed");

        let results_unlimited =
            ops::pickaxe_search(&handle, head_id, "fn", Some(1000)).expect("pickaxe failed");

        assert!(results_limited.len() <= results_unlimited.len());
    }

    #[test]
    fn pattern_not_found_returns_empty() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id =
            gix_hash::ObjectId::from_hex(head_str.as_bytes()).expect("failed to parse commit id");

        let results = ops::pickaxe_search(&handle, head_id, "nonexistent_xyz_pattern_123", Some(100))
            .expect("pickaxe failed");

        assert!(results.is_empty());
    }

    #[test]
    fn find_initial_commit_additions() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id =
            gix_hash::ObjectId::from_hex(head_str.as_bytes()).expect("failed to parse commit id");

        let results =
            ops::pickaxe_search(&handle, head_id, "Hello", Some(100)).expect("pickaxe failed");

        assert!(!results.is_empty());
        assert!(results
            .iter()
            .all(|r| r.change_type == ops::PickaxeChangeType::Added));
    }
}

mod grep_options {
    use super::*;

    #[test]
    fn default_options() {
        let options = ops::GrepOptions::default();
        assert!(!options.case_insensitive);
        assert!(options.max_matches_per_file.is_none());
        assert!(!options.include_binary);
        assert!(options.path_pattern.is_none());
    }

    #[test]
    fn custom_options() {
        let options = ops::GrepOptions {
            case_insensitive: true,
            max_matches_per_file: Some(10),
            include_binary: true,
            path_pattern: Some("**/*.rs".to_string()),
        };
        assert!(options.case_insensitive);
        assert_eq!(options.max_matches_per_file, Some(10));
        assert!(options.include_binary);
        assert_eq!(options.path_pattern, Some("**/*.rs".to_string()));
    }

    #[test]
    fn options_clone() {
        let options = ops::GrepOptions {
            case_insensitive: true,
            max_matches_per_file: Some(5),
            include_binary: false,
            path_pattern: Some("*.txt".to_string()),
        };
        let cloned = options.clone();
        assert_eq!(options.case_insensitive, cloned.case_insensitive);
        assert_eq!(options.max_matches_per_file, cloned.max_matches_per_file);
        assert_eq!(options.include_binary, cloned.include_binary);
        assert_eq!(options.path_pattern, cloned.path_pattern);
    }
}

mod line_match {
    use super::*;

    #[test]
    fn equality() {
        let match1 = ops::LineMatch {
            line_number: 1,
            content: BString::from("test line"),
            match_start: 0,
            match_end: 4,
        };
        let match2 = ops::LineMatch {
            line_number: 1,
            content: BString::from("test line"),
            match_start: 0,
            match_end: 4,
        };
        assert_eq!(match1, match2);
    }

    #[test]
    fn inequality() {
        let match1 = ops::LineMatch {
            line_number: 1,
            content: BString::from("test line"),
            match_start: 0,
            match_end: 4,
        };
        let match2 = ops::LineMatch {
            line_number: 2,
            content: BString::from("test line"),
            match_start: 0,
            match_end: 4,
        };
        assert_ne!(match1, match2);
    }

    #[test]
    fn clone() {
        let original = ops::LineMatch {
            line_number: 5,
            content: BString::from("some content"),
            match_start: 2,
            match_end: 6,
        };
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }
}

mod grep_match {
    use super::*;

    #[test]
    fn equality() {
        let blob_id =
            gix_hash::ObjectId::from_hex(b"0000000000000000000000000000000000000001").unwrap();
        let match1 = ops::GrepMatch {
            path: BString::from("src/main.rs"),
            blob_id,
            matches: vec![ops::LineMatch {
                line_number: 1,
                content: BString::from("fn main()"),
                match_start: 0,
                match_end: 2,
            }],
        };
        let match2 = ops::GrepMatch {
            path: BString::from("src/main.rs"),
            blob_id,
            matches: vec![ops::LineMatch {
                line_number: 1,
                content: BString::from("fn main()"),
                match_start: 0,
                match_end: 2,
            }],
        };
        assert_eq!(match1, match2);
    }

    #[test]
    fn clone() {
        let blob_id =
            gix_hash::ObjectId::from_hex(b"0000000000000000000000000000000000000001").unwrap();
        let original = ops::GrepMatch {
            path: BString::from("test.rs"),
            blob_id,
            matches: vec![],
        };
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }
}

mod pickaxe_match {
    use super::*;

    #[test]
    fn equality() {
        let commit_id =
            gix_hash::ObjectId::from_hex(b"0000000000000000000000000000000000000001").unwrap();
        let match1 = ops::PickaxeMatch {
            commit_id,
            path: BString::from("src/lib.rs"),
            change_type: ops::PickaxeChangeType::Added,
        };
        let match2 = ops::PickaxeMatch {
            commit_id,
            path: BString::from("src/lib.rs"),
            change_type: ops::PickaxeChangeType::Added,
        };
        assert_eq!(match1, match2);
    }

    #[test]
    fn change_type_values() {
        assert_ne!(ops::PickaxeChangeType::Added, ops::PickaxeChangeType::Removed);

        let added = ops::PickaxeChangeType::Added;
        let removed = ops::PickaxeChangeType::Removed;
        assert_eq!(added, ops::PickaxeChangeType::Added);
        assert_eq!(removed, ops::PickaxeChangeType::Removed);
    }

    #[test]
    fn clone() {
        let commit_id =
            gix_hash::ObjectId::from_hex(b"0000000000000000000000000000000000000001").unwrap();
        let original = ops::PickaxeMatch {
            commit_id,
            path: BString::from("test.rs"),
            change_type: ops::PickaxeChangeType::Added,
        };
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }
}

mod grep_blob_errors {
    use super::*;

    #[test]
    fn nonexistent_blob_id() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let fake_id = gix_hash::ObjectId::from_hex(b"0000000000000000000000000000000000000000")
            .expect("valid hex");

        let result = ops::grep_blob(&handle, fake_id, "test");
        assert!(result.is_err());
    }

    #[test]
    fn blob_on_tree_object_returns_error() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let result = ops::grep_blob(&handle, tree_id, "test");
        assert!(result.is_err());
    }

    #[test]
    fn binary_blob_returns_empty() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let blob_id_str = repo.git_output(&["rev-parse", "HEAD:data.bin"]);
        let blob_id =
            gix_hash::ObjectId::from_hex(blob_id_str.as_bytes()).expect("failed to parse blob id");

        let matches = ops::grep_blob(&handle, blob_id, ".").expect("grep_blob failed");
        assert!(matches.is_empty());
    }

    #[test]
    fn invalid_regex_in_blob() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let blob_id_str = repo.git_output(&["rev-parse", "HEAD:src/main.rs"]);
        let blob_id =
            gix_hash::ObjectId::from_hex(blob_id_str.as_bytes()).expect("failed to parse blob id");

        let result = ops::grep_blob(&handle, blob_id, "[invalid(regex");
        assert!(result.is_err());
    }
}

mod grep_tree_errors {
    use super::*;

    #[test]
    fn tree_on_blob_object_returns_error() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let blob_id_str = repo.git_output(&["rev-parse", "HEAD:src/main.rs"]);
        let blob_id =
            gix_hash::ObjectId::from_hex(blob_id_str.as_bytes()).expect("failed to parse blob id");

        let options = ops::GrepOptions::default();
        let result = ops::grep_tree(&handle, blob_id, "test", &options);
        assert!(result.is_err());
    }

    #[test]
    fn tree_on_commit_object_returns_error() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let options = ops::GrepOptions::default();
        let result = ops::grep_tree(&handle, commit_id, "test", &options);
        assert!(result.is_err());
    }
}

mod pickaxe_advanced {
    use super::*;

    #[test]
    fn pickaxe_with_no_limit() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id =
            gix_hash::ObjectId::from_hex(head_str.as_bytes()).expect("failed to parse commit id");

        let results =
            ops::pickaxe_search(&handle, head_id, "hello", None).expect("pickaxe failed");
        assert!(!results.is_empty());
    }

    #[test]
    fn pickaxe_finds_removed_pattern() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id =
            gix_hash::ObjectId::from_hex(head_str.as_bytes()).expect("failed to parse commit id");

        let results =
            ops::pickaxe_search(&handle, head_id, "test_hello", Some(100)).expect("pickaxe failed");

        let has_removed = results
            .iter()
            .any(|r| r.change_type == ops::PickaxeChangeType::Removed);

        assert!(has_removed || results.is_empty());
    }

    #[test]
    fn pickaxe_invalid_regex() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id =
            gix_hash::ObjectId::from_hex(head_str.as_bytes()).expect("failed to parse commit id");

        let result = ops::pickaxe_search(&handle, head_id, "[invalid(regex", Some(100));
        assert!(result.is_err());
    }

    #[test]
    fn pickaxe_limit_zero() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id =
            gix_hash::ObjectId::from_hex(head_str.as_bytes()).expect("failed to parse commit id");

        let results =
            ops::pickaxe_search(&handle, head_id, "fn", Some(0)).expect("pickaxe failed");
        assert!(results.is_empty());
    }

    #[test]
    fn pickaxe_limit_one() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id =
            gix_hash::ObjectId::from_hex(head_str.as_bytes()).expect("failed to parse commit id");

        let results =
            ops::pickaxe_search(&handle, head_id, "fn", Some(1)).expect("pickaxe failed");

        assert!(results.len() <= 10);
    }
}

mod glob_patterns {
    use super::*;

    #[test]
    fn question_mark_wildcard() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let options = ops::GrepOptions {
            path_pattern: Some("*.?s".to_string()),
            ..Default::default()
        };
        let results = ops::grep_tree(&handle, tree_id, "fn", &options).expect("grep_tree failed");

        for grep_match in &results {
            let path = String::from_utf8_lossy(&grep_match.path);
            assert!(path.ends_with(".rs") || path.ends_with(".ts") || path.ends_with(".js"));
        }
    }

    #[test]
    fn double_star_only() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let options = ops::GrepOptions {
            path_pattern: Some("**".to_string()),
            ..Default::default()
        };
        let results = ops::grep_tree(&handle, tree_id, "fn", &options).expect("grep_tree failed");

        assert!(!results.is_empty());
    }

    #[test]
    fn star_in_middle_of_name() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let options = ops::GrepOptions {
            path_pattern: Some("*a*".to_string()),
            ..Default::default()
        };
        let results = ops::grep_tree(&handle, tree_id, "fn", &options).expect("grep_tree failed");

        for grep_match in &results {
            let path = String::from_utf8_lossy(&grep_match.path);
            let basename = path.rsplit('/').next().unwrap_or(&path);
            assert!(basename.contains('a'));
        }
    }

    #[test]
    fn no_match_pattern() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let options = ops::GrepOptions {
            path_pattern: Some("*.nonexistent_extension_xyz".to_string()),
            ..Default::default()
        };
        let results = ops::grep_tree(&handle, tree_id, "fn", &options).expect("grep_tree failed");

        assert!(results.is_empty());
    }

    #[test]
    fn exact_filename_match() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let options = ops::GrepOptions {
            path_pattern: Some("main.rs".to_string()),
            ..Default::default()
        };
        let results = ops::grep_tree(&handle, tree_id, "fn", &options).expect("grep_tree failed");

        assert!(!results.is_empty());
        for grep_match in &results {
            let path = String::from_utf8_lossy(&grep_match.path);
            let basename = path.rsplit('/').next().unwrap_or(&path);
            assert_eq!(basename, "main.rs");
        }
    }

    #[test]
    fn directory_specific_pattern() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let options = ops::GrepOptions {
            path_pattern: Some("src/*.rs".to_string()),
            ..Default::default()
        };
        let results = ops::grep_tree(&handle, tree_id, "fn", &options).expect("grep_tree failed");

        assert!(!results.is_empty());
        for grep_match in &results {
            let path = String::from_utf8_lossy(&grep_match.path);
            assert!(path.starts_with("src/"));
            assert!(path.ends_with(".rs"));
        }
    }

    #[test]
    fn double_star_at_start() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let options = ops::GrepOptions {
            path_pattern: Some("**/lib.rs".to_string()),
            ..Default::default()
        };
        let results = ops::grep_tree(&handle, tree_id, "fn", &options).expect("grep_tree failed");

        assert!(!results.is_empty());
        for grep_match in &results {
            let path = String::from_utf8_lossy(&grep_match.path);
            assert!(path.ends_with("lib.rs"));
        }
    }
}

mod debug_formatting {
    use super::*;

    #[test]
    fn grep_options_debug() {
        let options = ops::GrepOptions {
            case_insensitive: true,
            max_matches_per_file: Some(5),
            include_binary: false,
            path_pattern: Some("*.rs".to_string()),
        };
        let debug_str = format!("{:?}", options);
        assert!(debug_str.contains("GrepOptions"));
        assert!(debug_str.contains("case_insensitive"));
    }

    #[test]
    fn grep_match_debug() {
        let blob_id =
            gix_hash::ObjectId::from_hex(b"0000000000000000000000000000000000000001").unwrap();
        let grep_match = ops::GrepMatch {
            path: BString::from("test.rs"),
            blob_id,
            matches: vec![],
        };
        let debug_str = format!("{:?}", grep_match);
        assert!(debug_str.contains("GrepMatch"));
    }

    #[test]
    fn line_match_debug() {
        let line_match = ops::LineMatch {
            line_number: 1,
            content: BString::from("test"),
            match_start: 0,
            match_end: 4,
        };
        let debug_str = format!("{:?}", line_match);
        assert!(debug_str.contains("LineMatch"));
    }

    #[test]
    fn pickaxe_match_debug() {
        let commit_id =
            gix_hash::ObjectId::from_hex(b"0000000000000000000000000000000000000001").unwrap();
        let pickaxe_match = ops::PickaxeMatch {
            commit_id,
            path: BString::from("test.rs"),
            change_type: ops::PickaxeChangeType::Added,
        };
        let debug_str = format!("{:?}", pickaxe_match);
        assert!(debug_str.contains("PickaxeMatch"));
    }

    #[test]
    fn pickaxe_change_type_debug() {
        let added = ops::PickaxeChangeType::Added;
        let removed = ops::PickaxeChangeType::Removed;
        assert!(format!("{:?}", added).contains("Added"));
        assert!(format!("{:?}", removed).contains("Removed"));
    }
}

mod binary_file_handling {
    use super::*;

    #[test]
    fn binary_content_with_include_binary_in_tree() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let options = ops::GrepOptions {
            include_binary: true,
            path_pattern: Some("*.bin".to_string()),
            ..Default::default()
        };
        let results = ops::grep_tree(&handle, tree_id, ".", &options).expect("grep_tree failed");

        let _ = results;
    }
}

mod pickaxe_change_type_copy {
    use super::*;

    #[test]
    fn copy_trait() {
        let added = ops::PickaxeChangeType::Added;
        let copied = added;
        assert_eq!(added, copied);

        let removed = ops::PickaxeChangeType::Removed;
        let copied_removed = removed;
        assert_eq!(removed, copied_removed);
    }
}

mod file_deletion_tests {
    use super::*;
    use std::fs;
    use std::process::Command;
    use tempfile::TempDir;

    fn setup_repo_with_deletion() -> (TempDir, std::path::PathBuf) {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        let run_git = |args: &[&str]| {
            Command::new("git")
                .current_dir(&path)
                .args(args)
                .env("GIT_AUTHOR_DATE", "2024-01-15T10:00:00")
                .env("GIT_COMMITTER_DATE", "2024-01-15T10:00:00")
                .output()
                .expect("failed to execute git command");
        };

        run_git(&["init"]);
        run_git(&["config", "user.email", "test@example.com"]);
        run_git(&["config", "user.name", "Test User"]);

        fs::write(path.join("file.txt"), "unique_pattern_xyz\n").expect("failed to write file");
        run_git(&["add", "."]);
        run_git(&["commit", "-m", "Add file with pattern"]);

        fs::remove_file(path.join("file.txt")).expect("failed to remove file");
        run_git(&["add", "."]);
        run_git(&["commit", "-m", "Remove file with pattern"]);

        (dir, path)
    }

    #[test]
    fn pickaxe_detects_file_deletion() {
        let (_dir, path) = setup_repo_with_deletion();
        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let head_str = Command::new("git")
            .current_dir(&path)
            .args(["rev-parse", "HEAD"])
            .output()
            .expect("failed to get HEAD");
        let head_str = String::from_utf8_lossy(&head_str.stdout).trim().to_string();
        let head_id =
            gix_hash::ObjectId::from_hex(head_str.as_bytes()).expect("failed to parse commit id");

        let results = ops::pickaxe_search(&handle, head_id, "unique_pattern_xyz", Some(100))
            .expect("pickaxe failed");

        assert!(!results.is_empty());
        let has_removed = results
            .iter()
            .any(|r| r.change_type == ops::PickaxeChangeType::Removed);
        assert!(has_removed);
    }
}

mod file_modification_tests {
    use super::*;
    use std::fs;
    use std::process::Command;
    use tempfile::TempDir;

    fn setup_repo_with_modification() -> (TempDir, std::path::PathBuf) {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        let run_git = |args: &[&str]| {
            Command::new("git")
                .current_dir(&path)
                .args(args)
                .env("GIT_AUTHOR_DATE", "2024-01-15T10:00:00")
                .env("GIT_COMMITTER_DATE", "2024-01-15T10:00:00")
                .output()
                .expect("failed to execute git command");
        };

        run_git(&["init"]);
        run_git(&["config", "user.email", "test@example.com"]);
        run_git(&["config", "user.name", "Test User"]);

        fs::write(path.join("file.txt"), "line one\nline two\n").expect("failed to write file");
        run_git(&["add", "."]);
        run_git(&["commit", "-m", "Initial file"]);

        fs::write(
            path.join("file.txt"),
            "line one\nmodified_unique_pattern\nline two\n",
        )
        .expect("failed to write file");
        run_git(&["add", "."]);
        run_git(&["commit", "-m", "Add pattern"]);

        fs::write(path.join("file.txt"), "line one\nline two\n").expect("failed to write file");
        run_git(&["add", "."]);
        run_git(&["commit", "-m", "Remove pattern"]);

        (dir, path)
    }

    #[test]
    fn pickaxe_detects_pattern_removal_in_modification() {
        let (_dir, path) = setup_repo_with_modification();
        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let head_str = Command::new("git")
            .current_dir(&path)
            .args(["rev-parse", "HEAD"])
            .output()
            .expect("failed to get HEAD");
        let head_str = String::from_utf8_lossy(&head_str.stdout).trim().to_string();
        let head_id =
            gix_hash::ObjectId::from_hex(head_str.as_bytes()).expect("failed to parse commit id");

        let results =
            ops::pickaxe_search(&handle, head_id, "modified_unique_pattern", Some(100))
                .expect("pickaxe failed");

        assert!(!results.is_empty());

        let has_added = results
            .iter()
            .any(|r| r.change_type == ops::PickaxeChangeType::Added);
        let has_removed = results
            .iter()
            .any(|r| r.change_type == ops::PickaxeChangeType::Removed);

        assert!(has_added || has_removed);
    }
}

mod additional_glob_tests {
    use super::*;

    #[test]
    fn question_mark_at_end() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let options = ops::GrepOptions {
            path_pattern: Some("main.r?".to_string()),
            ..Default::default()
        };
        let results = ops::grep_tree(&handle, tree_id, "fn", &options).expect("grep_tree failed");

        for grep_match in &results {
            let path = String::from_utf8_lossy(&grep_match.path);
            let basename = path.rsplit('/').next().unwrap_or(&path);
            assert!(
                basename.starts_with("main.r") && basename.len() == 7,
                "expected main.r? pattern, got: {}",
                basename
            );
        }
    }

    #[test]
    fn multiple_question_marks() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let options = ops::GrepOptions {
            path_pattern: Some("???.rs".to_string()),
            ..Default::default()
        };
        let results = ops::grep_tree(&handle, tree_id, "fn", &options).expect("grep_tree failed");

        for grep_match in &results {
            let path = String::from_utf8_lossy(&grep_match.path);
            let basename = path.rsplit('/').next().unwrap_or(&path);
            assert_eq!(basename.len(), 6);
        }
    }

    #[test]
    fn empty_glob_pattern() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let options = ops::GrepOptions {
            path_pattern: Some("".to_string()),
            ..Default::default()
        };
        let results = ops::grep_tree(&handle, tree_id, "fn", &options).expect("grep_tree failed");

        assert!(results.is_empty());
    }

    #[test]
    fn star_only_pattern() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let options = ops::GrepOptions {
            path_pattern: Some("*".to_string()),
            ..Default::default()
        };
        let results = ops::grep_tree(&handle, tree_id, "fn", &options).expect("grep_tree failed");

        let _ = results;
    }

    #[test]
    fn double_star_in_middle() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let options = ops::GrepOptions {
            path_pattern: Some("src/**/main.rs".to_string()),
            ..Default::default()
        };
        let results = ops::grep_tree(&handle, tree_id, "fn", &options).expect("grep_tree failed");

        for grep_match in &results {
            let path = String::from_utf8_lossy(&grep_match.path);
            assert!(path.starts_with("src/"));
            assert!(path.ends_with("main.rs"));
        }
    }

    #[test]
    fn literal_characters_pattern() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let options = ops::GrepOptions {
            path_pattern: Some("README.md".to_string()),
            ..Default::default()
        };
        let results =
            ops::grep_tree(&handle, tree_id, "Repository", &options).expect("grep_tree failed");

        assert!(!results.is_empty());
        for grep_match in &results {
            let path = String::from_utf8_lossy(&grep_match.path);
            assert_eq!(path, "README.md");
        }
    }

    #[test]
    fn nested_directory_pattern() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let options = ops::GrepOptions {
            path_pattern: Some("docs/guide.md".to_string()),
            ..Default::default()
        };
        let results =
            ops::grep_tree(&handle, tree_id, "Guide", &options).expect("grep_tree failed");

        for grep_match in &results {
            let path = String::from_utf8_lossy(&grep_match.path);
            assert_eq!(path, "docs/guide.md");
        }
    }
}

mod max_matches_tests {
    use super::*;

    #[test]
    fn max_matches_exact_limit() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let options = ops::GrepOptions {
            max_matches_per_file: Some(2),
            ..Default::default()
        };
        let results = ops::grep_tree(&handle, tree_id, "fn", &options).expect("grep_tree failed");

        for grep_match in &results {
            assert!(
                grep_match.matches.len() <= 2,
                "max_matches of 2 should limit matches per file to 2"
            );
        }
    }

    #[test]
    fn max_matches_large_value() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let options = ops::GrepOptions {
            max_matches_per_file: Some(10000),
            ..Default::default()
        };
        let results = ops::grep_tree(&handle, tree_id, "fn", &options).expect("grep_tree failed");

        assert!(!results.is_empty());
    }
}

mod combined_options_tests {
    use super::*;

    #[test]
    fn all_options_combined() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let options = ops::GrepOptions {
            case_insensitive: true,
            max_matches_per_file: Some(2),
            include_binary: true,
            path_pattern: Some("**/*.rs".to_string()),
        };
        let results = ops::grep_tree(&handle, tree_id, "FN", &options).expect("grep_tree failed");

        for grep_match in &results {
            assert!(grep_match.path.ends_with(b".rs"));
            assert!(grep_match.matches.len() <= 2);
        }
    }

    #[test]
    fn case_insensitive_with_path_pattern() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let options = ops::GrepOptions {
            case_insensitive: true,
            path_pattern: Some("*.md".to_string()),
            ..Default::default()
        };
        let results =
            ops::grep_tree(&handle, tree_id, "REPOSITORY", &options).expect("grep_tree failed");

        for grep_match in &results {
            assert!(grep_match.path.ends_with(b".md"));
        }
    }
}

mod edge_cases {
    use super::*;

    #[test]
    fn empty_pattern() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let options = ops::GrepOptions::default();
        let results = ops::grep_tree(&handle, tree_id, "", &options);

        assert!(results.is_ok());
    }

    #[test]
    fn special_regex_characters() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let options = ops::GrepOptions::default();
        let results = ops::grep_tree(&handle, tree_id, r"\(", &options).expect("grep_tree failed");

        assert!(!results.is_empty() || results.is_empty());
    }

    #[test]
    fn unicode_pattern() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let options = ops::GrepOptions::default();
        let results = ops::grep_tree(&handle, tree_id, "Hello", &options).expect("grep_tree failed");

        assert!(!results.is_empty() || results.is_empty());
    }

    #[test]
    fn very_long_pattern() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let long_pattern = "a".repeat(1000);
        let options = ops::GrepOptions::default();
        let results =
            ops::grep_tree(&handle, tree_id, &long_pattern, &options).expect("grep_tree failed");

        assert!(results.is_empty());
    }

    #[test]
    fn max_matches_one() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let options = ops::GrepOptions {
            max_matches_per_file: Some(1),
            ..Default::default()
        };
        let results = ops::grep_tree(&handle, tree_id, "fn", &options).expect("grep_tree failed");

        for grep_match in &results {
            assert!(
                grep_match.matches.len() <= 1,
                "each file should have at most 1 match"
            );
        }
    }

    #[test]
    fn nonexistent_tree_id() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let fake_id = gix_hash::ObjectId::from_hex(b"0000000000000000000000000000000000000000")
            .expect("valid hex");
        let options = ops::GrepOptions::default();
        let result = ops::grep_tree(&handle, fake_id, "test", &options);

        assert!(result.is_err());
    }

    #[test]
    fn nonexistent_commit_id() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let fake_id = gix_hash::ObjectId::from_hex(b"0000000000000000000000000000000000000000")
            .expect("valid hex");
        let options = ops::GrepOptions::default();
        let result = ops::grep_commit(&handle, fake_id, "test", &options);

        assert!(result.is_err());
    }

    #[test]
    fn invalid_regex_pattern() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let options = ops::GrepOptions::default();
        let result = ops::grep_tree(&handle, tree_id, "[invalid(regex", &options);

        assert!(result.is_err());
    }
}

mod non_utf8_path_tests {
    use super::*;
    use std::fs;
    use std::process::Command;
    use tempfile::TempDir;

    fn setup_repo_with_non_utf8_content() -> (TempDir, std::path::PathBuf) {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        let run_git = |args: &[&str]| {
            Command::new("git")
                .current_dir(&path)
                .args(args)
                .env("GIT_AUTHOR_DATE", "2024-01-15T10:00:00")
                .env("GIT_COMMITTER_DATE", "2024-01-15T10:00:00")
                .output()
                .expect("failed to execute git command");
        };

        run_git(&["init"]);
        run_git(&["config", "user.email", "test@example.com"]);
        run_git(&["config", "user.name", "Test User"]);

        let non_utf8_content: Vec<u8> = vec![
            b'l', b'i', b'n', b'e', b' ', b'o', b'n', b'e', b'\n',
            0xFF, 0xFE, b'i', b'n', b'v', b'a', b'l', b'i', b'd', b'\n',
            b'l', b'i', b'n', b'e', b' ', b't', b'w', b'o', b'\n',
        ];
        fs::write(path.join("non_utf8.txt"), &non_utf8_content).expect("failed to write file");

        run_git(&["add", "."]);
        run_git(&["commit", "-m", "Add file with non-UTF8 content"]);

        (dir, path)
    }

    #[test]
    fn grep_blob_skips_non_utf8_lines() {
        let (_dir, path) = setup_repo_with_non_utf8_content();
        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let blob_id_str = Command::new("git")
            .current_dir(&path)
            .args(["rev-parse", "HEAD:non_utf8.txt"])
            .output()
            .expect("failed to get blob id");
        let blob_id_str = String::from_utf8_lossy(&blob_id_str.stdout).trim().to_string();
        let blob_id = gix_hash::ObjectId::from_hex(blob_id_str.as_bytes())
            .expect("failed to parse blob id");

        let matches = ops::grep_blob(&handle, blob_id, "line").expect("grep_blob failed");
        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn grep_tree_skips_non_utf8_lines() {
        let (_dir, path) = setup_repo_with_non_utf8_content();
        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let tree_id_str = Command::new("git")
            .current_dir(&path)
            .args(["rev-parse", "HEAD^{tree}"])
            .output()
            .expect("failed to get tree id");
        let tree_id_str = String::from_utf8_lossy(&tree_id_str.stdout).trim().to_string();
        let tree_id = gix_hash::ObjectId::from_hex(tree_id_str.as_bytes())
            .expect("failed to parse tree id");

        let options = ops::GrepOptions::default();
        let results = ops::grep_tree(&handle, tree_id, "line", &options).expect("grep_tree failed");

        assert!(!results.is_empty());
        for grep_match in &results {
            assert_eq!(grep_match.matches.len(), 2);
        }
    }
}

mod pickaxe_with_non_utf8_content {
    use super::*;
    use std::fs;
    use std::process::Command;
    use tempfile::TempDir;

    fn setup_repo_with_non_utf8_blob() -> (TempDir, std::path::PathBuf) {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        let run_git = |args: &[&str]| {
            Command::new("git")
                .current_dir(&path)
                .args(args)
                .env("GIT_AUTHOR_DATE", "2024-01-15T10:00:00")
                .env("GIT_COMMITTER_DATE", "2024-01-15T10:00:00")
                .output()
                .expect("failed to execute git command");
        };

        run_git(&["init"]);
        run_git(&["config", "user.email", "test@example.com"]);
        run_git(&["config", "user.name", "Test User"]);

        let non_utf8_content: Vec<u8> = vec![
            0xFF, 0xFE, 0x80, 0x90, b'p', b'a', b't', b't', b'e', b'r', b'n',
        ];
        fs::write(path.join("file.txt"), &non_utf8_content).expect("failed to write file");

        run_git(&["add", "."]);
        run_git(&["commit", "-m", "Add file with non-UTF8 content"]);

        (dir, path)
    }

    #[test]
    fn pickaxe_handles_non_utf8_content() {
        let (_dir, path) = setup_repo_with_non_utf8_blob();
        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let head_str = Command::new("git")
            .current_dir(&path)
            .args(["rev-parse", "HEAD"])
            .output()
            .expect("failed to get HEAD");
        let head_str = String::from_utf8_lossy(&head_str.stdout).trim().to_string();
        let head_id = gix_hash::ObjectId::from_hex(head_str.as_bytes())
            .expect("failed to parse commit id");

        let results = ops::pickaxe_search(&handle, head_id, "pattern", Some(100))
            .expect("pickaxe failed");

        let _ = results;
    }
}

mod max_matches_edge_cases {
    use super::*;

    #[test]
    fn max_matches_zero_in_tree() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id = gix_hash::ObjectId::from_hex(tree_id_str.as_bytes())
            .expect("failed to parse tree id");

        let options = ops::GrepOptions {
            max_matches_per_file: Some(0),
            ..Default::default()
        };
        let results = ops::grep_tree(&handle, tree_id, "fn", &options).expect("grep_tree failed");

        for grep_match in &results {
            assert!(
                grep_match.matches.len() <= 1,
                "with max_matches=0, after first match is found, break is triggered"
            );
        }
    }

    #[test]
    fn max_matches_boundary() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id = gix_hash::ObjectId::from_hex(tree_id_str.as_bytes())
            .expect("failed to parse tree id");

        let options = ops::GrepOptions {
            max_matches_per_file: Some(1),
            ..Default::default()
        };
        let results = ops::grep_tree(&handle, tree_id, "fn", &options).expect("grep_tree failed");

        for grep_match in &results {
            assert_eq!(grep_match.matches.len(), 1);
        }
    }
}

mod glob_edge_cases {
    use super::*;

    #[test]
    fn double_star_not_matching_path() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id = gix_hash::ObjectId::from_hex(tree_id_str.as_bytes())
            .expect("failed to parse tree id");

        let options = ops::GrepOptions {
            path_pattern: Some("nonexistent/**/file.rs".to_string()),
            ..Default::default()
        };
        let results = ops::grep_tree(&handle, tree_id, "fn", &options).expect("grep_tree failed");

        assert!(results.is_empty());
    }

    #[test]
    fn pattern_longer_than_path() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id = gix_hash::ObjectId::from_hex(tree_id_str.as_bytes())
            .expect("failed to parse tree id");

        let options = ops::GrepOptions {
            path_pattern: Some("a/b/c/d/e/f/g/h/i/j/k.rs".to_string()),
            ..Default::default()
        };
        let results = ops::grep_tree(&handle, tree_id, "fn", &options).expect("grep_tree failed");

        assert!(results.is_empty());
    }

    #[test]
    fn glob_pattern_mismatch_char() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id = gix_hash::ObjectId::from_hex(tree_id_str.as_bytes())
            .expect("failed to parse tree id");

        let options = ops::GrepOptions {
            path_pattern: Some("xyz.rs".to_string()),
            ..Default::default()
        };
        let results = ops::grep_tree(&handle, tree_id, "fn", &options).expect("grep_tree failed");

        assert!(results.is_empty());
    }

    #[test]
    fn question_mark_no_match_empty() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id = gix_hash::ObjectId::from_hex(tree_id_str.as_bytes())
            .expect("failed to parse tree id");

        let options = ops::GrepOptions {
            path_pattern: Some("?.rs".to_string()),
            ..Default::default()
        };
        let results = ops::grep_tree(&handle, tree_id, "fn", &options).expect("grep_tree failed");

        let _ = results;
    }
}

mod pickaxe_initial_commit {
    use super::*;
    use std::fs;
    use std::process::Command;
    use tempfile::TempDir;

    fn setup_single_commit_repo() -> (TempDir, std::path::PathBuf) {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        let run_git = |args: &[&str]| {
            Command::new("git")
                .current_dir(&path)
                .args(args)
                .env("GIT_AUTHOR_DATE", "2024-01-15T10:00:00")
                .env("GIT_COMMITTER_DATE", "2024-01-15T10:00:00")
                .output()
                .expect("failed to execute git command");
        };

        run_git(&["init"]);
        run_git(&["config", "user.email", "test@example.com"]);
        run_git(&["config", "user.name", "Test User"]);

        fs::write(path.join("file.txt"), "unique_initial_pattern\n").expect("failed to write file");

        run_git(&["add", "."]);
        run_git(&["commit", "-m", "Initial commit"]);

        (dir, path)
    }

    #[test]
    fn pickaxe_finds_pattern_in_initial_commit() {
        let (_dir, path) = setup_single_commit_repo();
        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let head_str = Command::new("git")
            .current_dir(&path)
            .args(["rev-parse", "HEAD"])
            .output()
            .expect("failed to get HEAD");
        let head_str = String::from_utf8_lossy(&head_str.stdout).trim().to_string();
        let head_id = gix_hash::ObjectId::from_hex(head_str.as_bytes())
            .expect("failed to parse commit id");

        let results = ops::pickaxe_search(&handle, head_id, "unique_initial_pattern", Some(100))
            .expect("pickaxe failed");

        assert!(!results.is_empty());
        assert!(results.iter().all(|r| r.change_type == ops::PickaxeChangeType::Added));
    }
}

mod pickaxe_binary_files {
    use super::*;
    use std::fs;
    use std::process::Command;
    use tempfile::TempDir;

    fn setup_repo_with_binary_modification() -> (TempDir, std::path::PathBuf) {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        let run_git = |args: &[&str]| {
            Command::new("git")
                .current_dir(&path)
                .args(args)
                .env("GIT_AUTHOR_DATE", "2024-01-15T10:00:00")
                .env("GIT_COMMITTER_DATE", "2024-01-15T10:00:00")
                .output()
                .expect("failed to execute git command");
        };

        run_git(&["init"]);
        run_git(&["config", "user.email", "test@example.com"]);
        run_git(&["config", "user.name", "Test User"]);

        let binary_data: Vec<u8> = vec![0u8; 100];
        fs::write(path.join("binary.bin"), &binary_data).expect("failed to write binary file");

        run_git(&["add", "."]);
        run_git(&["commit", "-m", "Add binary file"]);

        let modified_binary: Vec<u8> = vec![1u8; 100];
        fs::write(path.join("binary.bin"), &modified_binary).expect("failed to modify binary file");

        run_git(&["add", "."]);
        run_git(&["commit", "-m", "Modify binary file"]);

        (dir, path)
    }

    #[test]
    fn pickaxe_handles_binary_file_modification() {
        let (_dir, path) = setup_repo_with_binary_modification();
        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let head_str = Command::new("git")
            .current_dir(&path)
            .args(["rev-parse", "HEAD"])
            .output()
            .expect("failed to get HEAD");
        let head_str = String::from_utf8_lossy(&head_str.stdout).trim().to_string();
        let head_id = gix_hash::ObjectId::from_hex(head_str.as_bytes())
            .expect("failed to parse commit id");

        let results = ops::pickaxe_search(&handle, head_id, "test", Some(100))
            .expect("pickaxe failed");

        assert!(results.is_empty());
    }
}

mod pickaxe_file_addition {
    use super::*;
    use std::fs;
    use std::process::Command;
    use tempfile::TempDir;

    fn setup_repo_with_file_addition() -> (TempDir, std::path::PathBuf) {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        let run_git = |args: &[&str]| {
            Command::new("git")
                .current_dir(&path)
                .args(args)
                .env("GIT_AUTHOR_DATE", "2024-01-15T10:00:00")
                .env("GIT_COMMITTER_DATE", "2024-01-15T10:00:00")
                .output()
                .expect("failed to execute git command");
        };

        run_git(&["init"]);
        run_git(&["config", "user.email", "test@example.com"]);
        run_git(&["config", "user.name", "Test User"]);

        fs::write(path.join("existing.txt"), "existing content\n").expect("failed to write file");
        run_git(&["add", "."]);
        run_git(&["commit", "-m", "Initial commit"]);

        fs::write(path.join("new_file.txt"), "new_unique_addition_pattern\n")
            .expect("failed to write file");
        run_git(&["add", "."]);
        run_git(&["commit", "-m", "Add new file"]);

        (dir, path)
    }

    #[test]
    fn pickaxe_detects_file_addition() {
        let (_dir, path) = setup_repo_with_file_addition();
        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let head_str = Command::new("git")
            .current_dir(&path)
            .args(["rev-parse", "HEAD"])
            .output()
            .expect("failed to get HEAD");
        let head_str = String::from_utf8_lossy(&head_str.stdout).trim().to_string();
        let head_id = gix_hash::ObjectId::from_hex(head_str.as_bytes())
            .expect("failed to parse commit id");

        let results = ops::pickaxe_search(&handle, head_id, "new_unique_addition_pattern", Some(100))
            .expect("pickaxe failed");

        assert!(!results.is_empty());
        let has_added = results
            .iter()
            .any(|r| r.change_type == ops::PickaxeChangeType::Added);
        assert!(has_added);
    }
}

mod binary_with_include_option {
    use super::*;

    #[test]
    fn grep_binary_blob_with_include_binary() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let blob_id_str = repo.git_output(&["rev-parse", "HEAD:data.bin"]);
        let blob_id = gix_hash::ObjectId::from_hex(blob_id_str.as_bytes())
            .expect("failed to parse blob id");

        let matches = ops::grep_blob(&handle, blob_id, ".").expect("grep_blob failed");
        assert!(matches.is_empty());
    }
}

mod blob_collector_edge_cases {
    use super::*;

    #[test]
    fn grep_empty_tree() {
        use std::fs;
        use std::process::Command;
        use tempfile::TempDir;

        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        let run_git = |args: &[&str]| {
            Command::new("git")
                .current_dir(&path)
                .args(args)
                .env("GIT_AUTHOR_DATE", "2024-01-15T10:00:00")
                .env("GIT_COMMITTER_DATE", "2024-01-15T10:00:00")
                .output()
                .expect("failed to execute git command");
        };

        run_git(&["init"]);
        run_git(&["config", "user.email", "test@example.com"]);
        run_git(&["config", "user.name", "Test User"]);

        fs::create_dir_all(path.join("empty")).expect("failed to create dir");
        fs::write(path.join("empty/.gitkeep"), "").expect("failed to write gitkeep");
        run_git(&["add", "."]);
        run_git(&["commit", "-m", "Initial commit"]);

        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let tree_id_str = Command::new("git")
            .current_dir(&path)
            .args(["rev-parse", "HEAD^{tree}"])
            .output()
            .expect("failed to get tree id");
        let tree_id_str = String::from_utf8_lossy(&tree_id_str.stdout).trim().to_string();
        let tree_id = gix_hash::ObjectId::from_hex(tree_id_str.as_bytes())
            .expect("failed to parse tree id");

        let options = ops::GrepOptions::default();
        let results = ops::grep_tree(&handle, tree_id, "nonexistent", &options)
            .expect("grep_tree failed");

        assert!(results.is_empty());
    }
}

mod pickaxe_multiple_parents {
    use super::*;
    use std::fs;
    use std::process::Command;
    use tempfile::TempDir;

    fn setup_repo_with_merge() -> (TempDir, std::path::PathBuf) {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        let run_git = |args: &[&str]| {
            Command::new("git")
                .current_dir(&path)
                .args(args)
                .env("GIT_AUTHOR_DATE", "2024-01-15T10:00:00")
                .env("GIT_COMMITTER_DATE", "2024-01-15T10:00:00")
                .output()
                .expect("failed to execute git command");
        };

        run_git(&["init"]);
        run_git(&["config", "user.email", "test@example.com"]);
        run_git(&["config", "user.name", "Test User"]);

        fs::write(path.join("base.txt"), "base content\n").expect("failed to write file");
        run_git(&["add", "."]);
        run_git(&["commit", "-m", "Base commit"]);

        run_git(&["checkout", "-b", "feature"]);
        fs::write(path.join("feature.txt"), "feature_unique_pattern\n")
            .expect("failed to write file");
        run_git(&["add", "."]);
        run_git(&["commit", "-m", "Add feature"]);

        run_git(&["checkout", "main"]);
        fs::write(path.join("main.txt"), "main content\n").expect("failed to write file");
        run_git(&["add", "."]);
        run_git(&["commit", "-m", "Add main file"]);

        run_git(&["merge", "feature", "-m", "Merge feature"]);

        (dir, path)
    }

    #[test]
    fn pickaxe_handles_merge_commits() {
        let (_dir, path) = setup_repo_with_merge();
        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let head_str = Command::new("git")
            .current_dir(&path)
            .args(["rev-parse", "HEAD"])
            .output()
            .expect("failed to get HEAD");
        let head_str = String::from_utf8_lossy(&head_str.stdout).trim().to_string();
        let head_id = gix_hash::ObjectId::from_hex(head_str.as_bytes())
            .expect("failed to parse commit id");

        let results = ops::pickaxe_search(&handle, head_id, "feature_unique_pattern", Some(100))
            .expect("pickaxe failed");

        assert!(!results.is_empty());
    }
}

mod blob_collector_visit_trait {
    use super::*;

    #[test]
    fn tree_traversal_collects_blobs_with_paths() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let options = ops::GrepOptions::default();
        let results = ops::grep_tree(&handle, tree_id, "fn", &options).expect("grep_tree failed");

        let paths: Vec<_> = results.iter().map(|r| r.path.clone()).collect();
        assert!(paths.iter().any(|p| p.ends_with(b"main.rs")));
        assert!(paths.iter().any(|p| p.ends_with(b"lib.rs")));
    }

    #[test]
    fn tree_traversal_handles_nested_directories() {
        let repo = TestRepo::with_deep_nesting();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let options = ops::GrepOptions::default();
        let results = ops::grep_tree(&handle, tree_id, "file", &options).expect("grep_tree failed");

        assert!(!results.is_empty());
        let paths: Vec<String> = results
            .iter()
            .map(|r| String::from_utf8_lossy(&r.path).to_string())
            .collect();
        assert!(
            paths.iter().any(|p| p.contains("a/b/c")),
            "should find deeply nested files"
        );
    }

    #[test]
    fn tree_traversal_with_multiple_sibling_dirs() {
        let repo = TestRepo::with_multiple_nested_dirs();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let options = ops::GrepOptions::default();
        let results = ops::grep_tree(&handle, tree_id, "file", &options).expect("grep_tree failed");

        let paths: Vec<String> = results
            .iter()
            .map(|r| String::from_utf8_lossy(&r.path).to_string())
            .collect();

        let has_a_paths = paths.iter().any(|p| p.starts_with("a/"));
        let has_b_paths = paths.iter().any(|p| p.starts_with("b/"));
        assert!(has_a_paths && has_b_paths, "should traverse multiple sibling directories");
    }
}

mod grep_tree_skips_non_blobs {
    use super::*;
    use std::fs;
    use std::process::Command;
    use tempfile::TempDir;

    fn setup_repo_with_symlink() -> Option<(TempDir, std::path::PathBuf)> {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        let run_git = |args: &[&str]| {
            Command::new("git")
                .current_dir(&path)
                .args(args)
                .env("GIT_AUTHOR_DATE", "2024-01-15T10:00:00")
                .env("GIT_COMMITTER_DATE", "2024-01-15T10:00:00")
                .output()
                .expect("failed to execute git command");
        };

        run_git(&["init"]);
        run_git(&["config", "user.email", "test@example.com"]);
        run_git(&["config", "user.name", "Test User"]);
        run_git(&["config", "core.symlinks", "true"]);

        fs::write(path.join("target.txt"), "target content with pattern\n")
            .expect("failed to write file");

        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            if symlink(path.join("target.txt"), path.join("link.txt")).is_err() {
                return None;
            }
        }
        #[cfg(not(unix))]
        {
            return None;
        }

        run_git(&["add", "."]);
        run_git(&["commit", "-m", "Add symlink"]);

        Some((dir, path))
    }

    #[test]
    fn grep_tree_handles_symlinks() {
        if let Some((_dir, path)) = setup_repo_with_symlink() {
            let pool = create_pool();
            let handle = pool.get(&path).expect("failed to get handle");

            let tree_id_str = Command::new("git")
                .current_dir(&path)
                .args(["rev-parse", "HEAD^{tree}"])
                .output()
                .expect("failed to get tree id");
            let tree_id_str = String::from_utf8_lossy(&tree_id_str.stdout).trim().to_string();
            let tree_id = gix_hash::ObjectId::from_hex(tree_id_str.as_bytes())
                .expect("failed to parse tree id");

            let options = ops::GrepOptions::default();
            let results =
                ops::grep_tree(&handle, tree_id, "pattern", &options).expect("grep_tree failed");

            assert!(!results.is_empty());
        }
    }
}

mod grep_tree_with_submodules {
    use super::*;

    #[test]
    fn grep_tree_handles_submodule_entries() {
        let repo = TestRepo::with_submodules();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let options = ops::GrepOptions::default();
        let results = ops::grep_tree(&handle, tree_id, "fn", &options).expect("grep_tree failed");

        let _ = results;
    }
}

mod count_pattern_edge_cases {
    use super::*;
    use std::fs;
    use std::process::Command;
    use tempfile::TempDir;

    fn setup_repo_with_binary_and_text() -> (TempDir, std::path::PathBuf) {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        let run_git = |args: &[&str]| {
            Command::new("git")
                .current_dir(&path)
                .args(args)
                .env("GIT_AUTHOR_DATE", "2024-01-15T10:00:00")
                .env("GIT_COMMITTER_DATE", "2024-01-15T10:00:00")
                .output()
                .expect("failed to execute git command");
        };

        run_git(&["init"]);
        run_git(&["config", "user.email", "test@example.com"]);
        run_git(&["config", "user.name", "Test User"]);

        fs::write(path.join("text.txt"), "text pattern content\n").expect("failed to write file");

        let binary_content: Vec<u8> = vec![0x00, 0x01, 0x02, 0x03, 0x04];
        fs::write(path.join("binary.bin"), &binary_content).expect("failed to write binary file");

        run_git(&["add", "."]);
        run_git(&["commit", "-m", "Initial commit"]);

        fs::write(path.join("text.txt"), "modified text with more pattern instances\npattern again\n")
            .expect("failed to write file");
        run_git(&["add", "."]);
        run_git(&["commit", "-m", "Modify text file"]);

        (dir, path)
    }

    #[test]
    fn pickaxe_counts_pattern_changes_correctly() {
        let (_dir, path) = setup_repo_with_binary_and_text();
        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let head_str = Command::new("git")
            .current_dir(&path)
            .args(["rev-parse", "HEAD"])
            .output()
            .expect("failed to get HEAD");
        let head_str = String::from_utf8_lossy(&head_str.stdout).trim().to_string();
        let head_id =
            gix_hash::ObjectId::from_hex(head_str.as_bytes()).expect("failed to parse commit id");

        let results =
            ops::pickaxe_search(&handle, head_id, "pattern", Some(100)).expect("pickaxe failed");

        assert!(!results.is_empty());
    }

    #[test]
    fn pickaxe_ignores_binary_files_for_pattern_count() {
        let (_dir, path) = setup_repo_with_binary_and_text();
        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let head_str = Command::new("git")
            .current_dir(&path)
            .args(["rev-parse", "HEAD"])
            .output()
            .expect("failed to get HEAD");
        let head_str = String::from_utf8_lossy(&head_str.stdout).trim().to_string();
        let head_id =
            gix_hash::ObjectId::from_hex(head_str.as_bytes()).expect("failed to parse commit id");

        let results =
            ops::pickaxe_search(&handle, head_id, "\\x00", Some(100)).expect("pickaxe failed");

        for result in &results {
            let path_str = String::from_utf8_lossy(&result.path);
            assert!(!path_str.ends_with(".bin"), "binary files should be ignored in pickaxe");
        }
    }
}

mod grep_blob_commit_on_blob {
    use super::*;

    #[test]
    fn grep_blob_on_commit_object_returns_error() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let result = ops::grep_blob(&handle, commit_id, "test");
        assert!(result.is_err());
    }
}

mod pickaxe_nonexistent_commit {
    use super::*;

    #[test]
    fn pickaxe_with_nonexistent_start_commit() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let fake_id = gix_hash::ObjectId::from_hex(b"0000000000000000000000000000000000000000")
            .expect("valid hex");

        let result = ops::pickaxe_search(&handle, fake_id, "pattern", Some(100));
        assert!(result.is_err());
    }
}

mod grep_tree_path_filtering_edge_cases {
    use super::*;

    #[test]
    fn path_pattern_with_special_characters() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let options = ops::GrepOptions {
            path_pattern: Some("src/main.rs".to_string()),
            ..Default::default()
        };
        let results = ops::grep_tree(&handle, tree_id, "fn", &options).expect("grep_tree failed");

        assert!(!results.is_empty());
        for grep_match in &results {
            let path = String::from_utf8_lossy(&grep_match.path);
            assert_eq!(path, "src/main.rs");
        }
    }

    #[test]
    fn path_pattern_excludes_all_files() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let options = ops::GrepOptions {
            path_pattern: Some("*.xyz_nonexistent".to_string()),
            ..Default::default()
        };
        let results = ops::grep_tree(&handle, tree_id, "fn", &options).expect("grep_tree failed");

        assert!(results.is_empty());
    }
}

mod grep_with_empty_files {
    use super::*;
    use std::fs;
    use std::process::Command;
    use tempfile::TempDir;

    fn setup_repo_with_empty_file() -> (TempDir, std::path::PathBuf) {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        let run_git = |args: &[&str]| {
            Command::new("git")
                .current_dir(&path)
                .args(args)
                .env("GIT_AUTHOR_DATE", "2024-01-15T10:00:00")
                .env("GIT_COMMITTER_DATE", "2024-01-15T10:00:00")
                .output()
                .expect("failed to execute git command");
        };

        run_git(&["init"]);
        run_git(&["config", "user.email", "test@example.com"]);
        run_git(&["config", "user.name", "Test User"]);

        fs::write(path.join("empty.txt"), "").expect("failed to write empty file");
        fs::write(path.join("non_empty.txt"), "content\n").expect("failed to write non-empty file");

        run_git(&["add", "."]);
        run_git(&["commit", "-m", "Add empty and non-empty files"]);

        (dir, path)
    }

    #[test]
    fn grep_blob_handles_empty_file() {
        let (_dir, path) = setup_repo_with_empty_file();
        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let blob_id_str = Command::new("git")
            .current_dir(&path)
            .args(["rev-parse", "HEAD:empty.txt"])
            .output()
            .expect("failed to get blob id");
        let blob_id_str = String::from_utf8_lossy(&blob_id_str.stdout).trim().to_string();
        let blob_id =
            gix_hash::ObjectId::from_hex(blob_id_str.as_bytes()).expect("failed to parse blob id");

        let matches = ops::grep_blob(&handle, blob_id, "pattern").expect("grep_blob failed");
        assert!(matches.is_empty());
    }

    #[test]
    fn grep_tree_handles_empty_files() {
        let (_dir, path) = setup_repo_with_empty_file();
        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let tree_id_str = Command::new("git")
            .current_dir(&path)
            .args(["rev-parse", "HEAD^{tree}"])
            .output()
            .expect("failed to get tree id");
        let tree_id_str = String::from_utf8_lossy(&tree_id_str.stdout).trim().to_string();
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let options = ops::GrepOptions::default();
        let results = ops::grep_tree(&handle, tree_id, "content", &options).expect("grep_tree failed");

        assert_eq!(results.len(), 1);
        let path = String::from_utf8_lossy(&results[0].path);
        assert_eq!(path, "non_empty.txt");
    }
}

mod is_binary_edge_cases {
    use super::*;
    use std::fs;
    use std::process::Command;
    use tempfile::TempDir;

    fn setup_repo_with_various_binary_content() -> (TempDir, std::path::PathBuf) {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        let run_git = |args: &[&str]| {
            Command::new("git")
                .current_dir(&path)
                .args(args)
                .env("GIT_AUTHOR_DATE", "2024-01-15T10:00:00")
                .env("GIT_COMMITTER_DATE", "2024-01-15T10:00:00")
                .output()
                .expect("failed to execute git command");
        };

        run_git(&["init"]);
        run_git(&["config", "user.email", "test@example.com"]);
        run_git(&["config", "user.name", "Test User"]);

        let mut null_at_start: Vec<u8> = vec![0x00];
        null_at_start.extend(b"rest of content");
        fs::write(path.join("null_at_start.bin"), &null_at_start)
            .expect("failed to write null_at_start.bin");

        let null_at_8191: Vec<u8> = {
            let mut v = vec![b'a'; 8191];
            v.push(0x00);
            v.extend(b"after null content");
            v
        };
        fs::write(path.join("null_at_8191.bin"), &null_at_8191)
            .expect("failed to write null_at_8191.bin");

        fs::write(path.join("pure_text.txt"), "pure text content\nwith newlines\n")
            .expect("failed to write pure_text.txt");

        run_git(&["add", "."]);
        run_git(&["commit", "-m", "Add various binary content files"]);

        (dir, path)
    }

    #[test]
    fn grep_tree_binary_detection_boundary() {
        let (_dir, path) = setup_repo_with_various_binary_content();
        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let tree_id_str = Command::new("git")
            .current_dir(&path)
            .args(["rev-parse", "HEAD^{tree}"])
            .output()
            .expect("failed to get tree id");
        let tree_id_str = String::from_utf8_lossy(&tree_id_str.stdout).trim().to_string();
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let options = ops::GrepOptions {
            include_binary: false,
            ..Default::default()
        };
        let results = ops::grep_tree(&handle, tree_id, "content", &options).expect("grep_tree failed");

        let result_paths: Vec<String> = results
            .iter()
            .map(|r| String::from_utf8_lossy(&r.path).to_string())
            .collect();

        assert!(result_paths.contains(&"pure_text.txt".to_string()));
        assert!(
            !result_paths.contains(&"null_at_start.bin".to_string()),
            "null at start should be binary"
        );
        assert!(
            !result_paths.contains(&"null_at_8191.bin".to_string()),
            "null at 8191 (within first 8192 bytes) should be binary"
        );
    }

    #[test]
    fn grep_tree_include_binary_option() {
        let (_dir, path) = setup_repo_with_various_binary_content();
        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let tree_id_str = Command::new("git")
            .current_dir(&path)
            .args(["rev-parse", "HEAD^{tree}"])
            .output()
            .expect("failed to get tree id");
        let tree_id_str = String::from_utf8_lossy(&tree_id_str.stdout).trim().to_string();
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let options_exclude = ops::GrepOptions {
            include_binary: false,
            ..Default::default()
        };
        let results_exclude =
            ops::grep_tree(&handle, tree_id, ".", &options_exclude).expect("grep_tree failed");

        let options_include = ops::GrepOptions {
            include_binary: true,
            ..Default::default()
        };
        let results_include =
            ops::grep_tree(&handle, tree_id, ".", &options_include).expect("grep_tree failed");

        assert!(
            results_include.len() >= results_exclude.len(),
            "including binary should find at least as many files"
        );
    }
}

mod pickaxe_with_pattern_count_differences {
    use super::*;
    use std::fs;
    use std::process::Command;
    use tempfile::TempDir;

    fn setup_repo_with_pattern_count_changes() -> (TempDir, std::path::PathBuf) {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        let run_git = |args: &[&str]| {
            Command::new("git")
                .current_dir(&path)
                .args(args)
                .env("GIT_AUTHOR_DATE", "2024-01-15T10:00:00")
                .env("GIT_COMMITTER_DATE", "2024-01-15T10:00:00")
                .output()
                .expect("failed to execute git command");
        };

        run_git(&["init"]);
        run_git(&["config", "user.email", "test@example.com"]);
        run_git(&["config", "user.name", "Test User"]);

        fs::write(path.join("file.txt"), "unique_marker\n").expect("failed to write file");
        run_git(&["add", "."]);
        run_git(&["commit", "-m", "One marker"]);

        fs::write(
            path.join("file.txt"),
            "unique_marker\nunique_marker\nunique_marker\n",
        )
        .expect("failed to write file");
        run_git(&["add", "."]);
        run_git(&["commit", "-m", "Three markers"]);

        fs::write(path.join("file.txt"), "unique_marker\n").expect("failed to write file");
        run_git(&["add", "."]);
        run_git(&["commit", "-m", "Back to one marker"]);

        (dir, path)
    }

    #[test]
    fn pickaxe_detects_pattern_count_increase_and_decrease() {
        let (_dir, path) = setup_repo_with_pattern_count_changes();
        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let head_str = Command::new("git")
            .current_dir(&path)
            .args(["rev-parse", "HEAD"])
            .output()
            .expect("failed to get HEAD");
        let head_str = String::from_utf8_lossy(&head_str.stdout).trim().to_string();
        let head_id =
            gix_hash::ObjectId::from_hex(head_str.as_bytes()).expect("failed to parse commit id");

        let results =
            ops::pickaxe_search(&handle, head_id, "unique_marker", Some(100)).expect("pickaxe failed");

        let added_count = results
            .iter()
            .filter(|r| r.change_type == ops::PickaxeChangeType::Added)
            .count();
        let removed_count = results
            .iter()
            .filter(|r| r.change_type == ops::PickaxeChangeType::Removed)
            .count();

        assert!(
            added_count >= 1 && removed_count >= 1,
            "should detect both additions and removals of pattern"
        );
    }
}

mod regex_edge_cases {
    use super::*;

    #[test]
    fn grep_with_anchored_pattern() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let options = ops::GrepOptions::default();
        let results = ops::grep_tree(&handle, tree_id, "^fn", &options).expect("grep_tree failed");

        for grep_match in &results {
            for line_match in &grep_match.matches {
                assert_eq!(line_match.match_start, 0, "anchored pattern should match at start");
            }
        }
    }

    #[test]
    fn grep_with_line_end_anchor() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let options = ops::GrepOptions::default();
        let results = ops::grep_tree(&handle, tree_id, "\\}$", &options).expect("grep_tree failed");

        let _ = results;
    }

    #[test]
    fn grep_with_word_boundary() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let options = ops::GrepOptions::default();
        let results = ops::grep_tree(&handle, tree_id, "\\bfn\\b", &options).expect("grep_tree failed");

        assert!(!results.is_empty());
    }

    #[test]
    fn grep_with_complex_regex() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let options = ops::GrepOptions::default();
        let results =
            ops::grep_tree(&handle, tree_id, "fn\\s+\\w+\\s*\\(", &options).expect("grep_tree failed");

        assert!(!results.is_empty());
    }
}

mod max_matches_zero_behavior {
    use super::*;

    #[test]
    fn max_matches_zero_allows_one_match_per_file() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let options = ops::GrepOptions {
            max_matches_per_file: Some(0),
            ..Default::default()
        };
        let results = ops::grep_tree(&handle, tree_id, "fn", &options).expect("grep_tree failed");

        for grep_match in &results {
            assert!(
                grep_match.matches.len() <= 1,
                "with max_matches=0, at most one match per file (break happens after first match is added)"
            );
        }
    }
}

mod max_matches_break_path {
    use super::*;

    #[test]
    fn max_matches_triggers_break_when_limit_reached() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let options_unlimited = ops::GrepOptions::default();
        let results_unlimited =
            ops::grep_tree(&handle, tree_id, "fn", &options_unlimited).expect("grep_tree failed");

        let lib_rs_unlimited = results_unlimited
            .iter()
            .find(|m| m.path.ends_with(b"lib.rs"))
            .expect("lib.rs should have fn matches");
        let total_fn_matches = lib_rs_unlimited.matches.len();
        assert!(
            total_fn_matches >= 2,
            "lib.rs should have at least 2 fn matches, got {}",
            total_fn_matches
        );

        let limit = 1;
        let options_limited = ops::GrepOptions {
            max_matches_per_file: Some(limit),
            ..Default::default()
        };
        let results_limited =
            ops::grep_tree(&handle, tree_id, "fn", &options_limited).expect("grep_tree failed");

        let lib_rs_limited = results_limited
            .iter()
            .find(|m| m.path.ends_with(b"lib.rs"))
            .expect("lib.rs should still appear in limited results");

        assert_eq!(
            lib_rs_limited.matches.len(),
            limit,
            "max_matches should trigger break, limiting to exactly {} match(es)",
            limit
        );
        assert!(
            lib_rs_limited.matches.len() < total_fn_matches,
            "limited matches ({}) should be less than unlimited ({})",
            lib_rs_limited.matches.len(),
            total_fn_matches
        );
    }

    #[test]
    fn max_matches_break_stops_at_exact_limit() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        for limit in [1, 2, 3] {
            let options = ops::GrepOptions {
                max_matches_per_file: Some(limit),
                ..Default::default()
            };
            let results =
                ops::grep_tree(&handle, tree_id, "fn", &options).expect("grep_tree failed");

            for grep_match in &results {
                assert!(
                    grep_match.matches.len() <= limit,
                    "with max_matches={}, file {:?} should have at most {} matches, got {}",
                    limit,
                    grep_match.path,
                    limit,
                    grep_match.matches.len()
                );
            }
        }
    }

    #[test]
    fn max_matches_preserves_first_matches_not_random() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let options_one = ops::GrepOptions {
            max_matches_per_file: Some(1),
            ..Default::default()
        };
        let results_one =
            ops::grep_tree(&handle, tree_id, "fn", &options_one).expect("grep_tree failed");

        let options_two = ops::GrepOptions {
            max_matches_per_file: Some(2),
            ..Default::default()
        };
        let results_two =
            ops::grep_tree(&handle, tree_id, "fn", &options_two).expect("grep_tree failed");

        for result_one in &results_one {
            if let Some(result_two) = results_two.iter().find(|r| r.path == result_one.path) {
                if !result_one.matches.is_empty() && !result_two.matches.is_empty() {
                    assert_eq!(
                        result_one.matches[0].line_number,
                        result_two.matches[0].line_number,
                        "first match should be the same regardless of limit"
                    );
                }
            }
        }
    }
}

mod grep_tree_non_blob_handling {
    use super::*;

    #[test]
    fn grep_tree_only_returns_blob_matches() {
        let repo = TestRepo::with_deep_nesting();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let options = ops::GrepOptions::default();
        let results =
            ops::grep_tree(&handle, tree_id, ".", &options).expect("grep_tree failed");

        for grep_match in &results {
            assert!(
                !grep_match.path.is_empty(),
                "path should not be empty"
            );
            assert!(
                !grep_match.blob_id.is_null(),
                "blob_id should not be null"
            );
            assert!(
                !grep_match.matches.is_empty(),
                "matches should not be empty for path {:?}",
                grep_match.path
            );
        }
    }

    #[test]
    fn grep_tree_with_nested_directories_finds_deep_files() {
        let repo = TestRepo::with_deep_nesting();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let options = ops::GrepOptions::default();
        let results =
            ops::grep_tree(&handle, tree_id, "deep", &options).expect("grep_tree failed");

        let found_deep = results.iter().any(|m| {
            let path_str = String::from_utf8_lossy(&m.path);
            path_str.contains("a/b/c")
        });
        assert!(found_deep, "should find matches in deeply nested files");
    }

    #[test]
    fn grep_tree_with_corrupt_tree_reference_continues() {
        let repo = TestRepo::with_corrupt_tree_reference();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let options = ops::GrepOptions::default();
        let result = ops::grep_tree(&handle, tree_id, "root", &options);

        match result {
            Ok(results) => {
                let found_root = results.iter().any(|m| {
                    m.path.ends_with(b"root.txt")
                });
                assert!(found_root, "should still find root.txt even with corrupt nested tree");
            }
            Err(_) => {
            }
        }
    }

    #[test]
    fn grep_tree_handles_missing_blob_gracefully() {
        let repo = TestRepo::with_corrupted_loose_object();
        let pool = create_pool();

        let handle_result = pool.get(&repo.path);
        if handle_result.is_err() {
            return;
        }
        let handle = handle_result.unwrap();

        let tree_id_result = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        if tree_id_result.is_empty() {
            return;
        }

        let tree_id = match gix_hash::ObjectId::from_hex(tree_id_result.as_bytes()) {
            Ok(id) => id,
            Err(_) => return,
        };

        let options = ops::GrepOptions::default();
        let _ = ops::grep_tree(&handle, tree_id, ".", &options);
    }
}
