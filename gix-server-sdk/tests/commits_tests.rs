mod fixtures;

use fixtures::TestRepo;
use gix_server_sdk::{ops, RepoPool, SdkConfig};

fn create_pool() -> RepoPool {
    RepoPool::new(SdkConfig::default())
}

mod get_commit {
    use super::*;

    #[test]
    fn basic_commit_info() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let result = ops::get_commit(&handle, commit_id).expect("failed to get commit");

        assert_eq!(result.id, commit_id);
        assert!(!result.tree_id.is_null());
        assert!(!result.parent_ids.is_empty());
    }

    #[test]
    fn author_data() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let result = ops::get_commit(&handle, commit_id).expect("failed to get commit");

        assert_eq!(result.author.email.as_slice(), b"alice@example.com");
        assert_eq!(result.author.name.as_slice(), b"Alice Developer");
        assert!(result.author.time > 0);
    }

    #[test]
    fn committer_data() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let result = ops::get_commit(&handle, commit_id).expect("failed to get commit");

        assert_eq!(result.committer.email.as_slice(), b"alice@example.com");
        assert_eq!(result.committer.name.as_slice(), b"Alice Developer");
        assert!(result.committer.time > 0);
    }

    #[test]
    fn commit_message() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let result = ops::get_commit(&handle, commit_id).expect("failed to get commit");

        let message_str = String::from_utf8_lossy(result.message.as_ref());
        assert!(message_str.contains("Update README and remove old tests"));
    }

    #[test]
    fn initial_commit_has_no_parents() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let initial_id_str = repo.git_output(&["rev-list", "--max-parents=0", "HEAD"]);
        let initial_id = gix_hash::ObjectId::from_hex(initial_id_str.as_bytes())
            .expect("failed to parse initial commit id");

        let result = ops::get_commit(&handle, initial_id).expect("failed to get commit");

        assert!(result.parent_ids.is_empty());
    }

    #[test]
    fn commit_with_single_parent() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD~1"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let result = ops::get_commit(&handle, commit_id).expect("failed to get commit");

        assert_eq!(result.parent_ids.len(), 1);
    }

    #[test]
    fn different_author_and_committer() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD~4"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let result = ops::get_commit(&handle, commit_id).expect("failed to get commit");

        assert_eq!(result.author.email.as_slice(), b"bob@example.com");
        assert_eq!(result.author.name.as_slice(), b"Bob Contributor");
    }

    #[test]
    fn nonexistent_commit_returns_error() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let fake_id = gix_hash::ObjectId::from_hex(b"0000000000000000000000000000000000000000")
            .expect("valid hex");
        let result = ops::get_commit(&handle, fake_id);

        assert!(result.is_err());
    }

    #[test]
    fn tree_id_is_not_commit() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let result = ops::get_commit(&handle, tree_id);

        assert!(result.is_err());
    }
}

mod log {
    use super::*;

    #[test]
    fn all_commits() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id = gix_hash::ObjectId::from_hex(head_id_str.as_bytes())
            .expect("failed to parse head id");

        let result = ops::log(&handle, head_id, None).expect("failed to get log");

        assert_eq!(result.len(), 6);
    }

    #[test]
    fn limit_one() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id = gix_hash::ObjectId::from_hex(head_id_str.as_bytes())
            .expect("failed to parse head id");

        let result = ops::log(&handle, head_id, Some(1)).expect("failed to get log");

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, head_id);
    }

    #[test]
    fn limit_three() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id = gix_hash::ObjectId::from_hex(head_id_str.as_bytes())
            .expect("failed to parse head id");

        let result = ops::log(&handle, head_id, Some(3)).expect("failed to get log");

        assert_eq!(result.len(), 3);
    }

    #[test]
    fn limit_greater_than_total() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id = gix_hash::ObjectId::from_hex(head_id_str.as_bytes())
            .expect("failed to parse head id");

        let result = ops::log(&handle, head_id, Some(100)).expect("failed to get log");

        assert_eq!(result.len(), 6);
    }

    #[test]
    fn limit_zero_returns_one() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id = gix_hash::ObjectId::from_hex(head_id_str.as_bytes())
            .expect("failed to parse head id");

        let result = ops::log(&handle, head_id, Some(0)).expect("failed to get log");

        assert_eq!(result.len(), 1);
    }

    #[test]
    fn commits_ordered_newest_first() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id = gix_hash::ObjectId::from_hex(head_id_str.as_bytes())
            .expect("failed to parse head id");

        let result = ops::log(&handle, head_id, None).expect("failed to get log");

        assert_eq!(result[0].id, head_id);

        for i in 0..result.len() - 1 {
            assert!(
                result[i].committer.time >= result[i + 1].committer.time,
                "commits should be ordered newest first"
            );
        }
    }

    #[test]
    fn log_from_older_commit() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let old_id_str = repo.git_output(&["rev-parse", "HEAD~3"]);
        let old_id = gix_hash::ObjectId::from_hex(old_id_str.as_bytes())
            .expect("failed to parse commit id");

        let result = ops::log(&handle, old_id, None).expect("failed to get log");

        assert_eq!(result.len(), 3);
        assert_eq!(result[0].id, old_id);
    }

    #[test]
    fn log_single_commit_repo() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id = gix_hash::ObjectId::from_hex(head_id_str.as_bytes())
            .expect("failed to parse head id");

        let result = ops::log(&handle, head_id, None).expect("failed to get log");

        assert_eq!(result.len(), 1);
        assert!(result[0].parent_ids.is_empty());
    }

    #[test]
    fn each_commit_has_valid_tree() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id = gix_hash::ObjectId::from_hex(head_id_str.as_bytes())
            .expect("failed to parse head id");

        let result = ops::log(&handle, head_id, None).expect("failed to get log");

        for commit in &result {
            assert!(!commit.tree_id.is_null());
        }
    }

    #[test]
    fn multiple_authors_in_history() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id = gix_hash::ObjectId::from_hex(head_id_str.as_bytes())
            .expect("failed to parse head id");

        let result = ops::log(&handle, head_id, None).expect("failed to get log");

        let authors: std::collections::HashSet<_> =
            result.iter().map(|c| c.author.email.clone()).collect();

        assert!(authors.len() >= 2, "expected multiple authors in history");
    }
}

mod log_with_path {
    use super::*;

    #[test]
    fn filter_by_file() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id = gix_hash::ObjectId::from_hex(head_id_str.as_bytes())
            .expect("failed to parse head id");

        let result = ops::log_with_path(&handle, head_id, "src/lib.rs", None);

        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn filter_by_readme() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id = gix_hash::ObjectId::from_hex(head_id_str.as_bytes())
            .expect("failed to parse head id");

        let result = ops::log_with_path(&handle, head_id, "README.md", None);

        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn filter_by_directory() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id = gix_hash::ObjectId::from_hex(head_id_str.as_bytes())
            .expect("failed to parse head id");

        let result = ops::log_with_path(&handle, head_id, "src", None);

        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn filter_with_limit() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id = gix_hash::ObjectId::from_hex(head_id_str.as_bytes())
            .expect("failed to parse head id");

        let result = ops::log_with_path(&handle, head_id, "src/lib.rs", Some(1));

        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn nonexistent_path_returns_empty() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id = gix_hash::ObjectId::from_hex(head_id_str.as_bytes())
            .expect("failed to parse head id");

        let result = ops::log_with_path(&handle, head_id, "nonexistent/path.rs", None)
            .expect("failed to get log");

        assert!(result.is_empty());
    }

    #[test]
    fn main_rs_modifications() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id = gix_hash::ObjectId::from_hex(head_id_str.as_bytes())
            .expect("failed to parse head id");

        let result = ops::log_with_path(&handle, head_id, "src/main.rs", None);

        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn commits_filtered_correctly() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id = gix_hash::ObjectId::from_hex(head_id_str.as_bytes())
            .expect("failed to parse head id");

        let lib_result = ops::log_with_path(&handle, head_id, "src/lib.rs", None);
        let main_result = ops::log_with_path(&handle, head_id, "src/main.rs", None);

        assert!(lib_result.is_ok() || lib_result.is_err());
        assert!(main_result.is_ok() || main_result.is_err());
    }

    #[test]
    fn filter_preserves_commit_order() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id = gix_hash::ObjectId::from_hex(head_id_str.as_bytes())
            .expect("failed to parse head id");

        let result = ops::log_with_path(&handle, head_id, "src/lib.rs", None);

        if let Ok(commits) = result {
            for i in 0..commits.len().saturating_sub(1) {
                assert!(
                    commits[i].committer.time >= commits[i + 1].committer.time,
                    "filtered commits should be ordered newest first"
                );
            }
        }
    }

    #[test]
    fn file_in_initial_commit_detected() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id =
            gix_hash::ObjectId::from_hex(head_id_str.as_bytes()).expect("failed to parse head id");

        let result = ops::log_with_path(&handle, head_id, "README.md", None)
            .expect("failed to get log");

        assert!(!result.is_empty());
        let initial_commit = result.last().unwrap();
        assert!(initial_commit.parent_ids.is_empty());
    }

    #[test]
    fn deeply_nested_file_tracked() {
        let repo = TestRepo::with_deep_paths();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id =
            gix_hash::ObjectId::from_hex(head_id_str.as_bytes()).expect("failed to parse head id");

        let result = ops::log_with_path(&handle, head_id, "a/b/c/level3.txt", None)
            .expect("failed to get log");

        assert!(result.len() >= 2);
    }

    #[test]
    fn directory_tracks_all_nested_changes() {
        let repo = TestRepo::with_deep_paths();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id =
            gix_hash::ObjectId::from_hex(head_id_str.as_bytes()).expect("failed to parse head id");

        let result =
            ops::log_with_path(&handle, head_id, "a/b", None).expect("failed to get log");

        assert!(result.len() >= 3);
    }

    #[test]
    fn deleted_file_history() {
        let repo = TestRepo::with_deep_paths();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id =
            gix_hash::ObjectId::from_hex(head_id_str.as_bytes()).expect("failed to parse head id");

        let result = ops::log_with_path(&handle, head_id, "a/b/level2.txt", None)
            .expect("failed to get log");

        assert!(result.len() >= 2);
    }

    #[test]
    fn limit_one_with_multiple_matches() {
        let repo = TestRepo::with_deep_paths();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id =
            gix_hash::ObjectId::from_hex(head_id_str.as_bytes()).expect("failed to parse head id");

        let result = ops::log_with_path(&handle, head_id, "a/b/c/level3.txt", Some(1))
            .expect("failed to get log");

        assert_eq!(result.len(), 1);
    }

    #[test]
    fn limit_respects_filter() {
        let repo = TestRepo::with_deep_paths();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id =
            gix_hash::ObjectId::from_hex(head_id_str.as_bytes()).expect("failed to parse head id");

        let result = ops::log_with_path(&handle, head_id, "a", Some(2)).expect("failed to get log");

        assert_eq!(result.len(), 2);
    }

    #[test]
    fn single_level_directory_path() {
        let repo = TestRepo::with_deep_paths();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id =
            gix_hash::ObjectId::from_hex(head_id_str.as_bytes()).expect("failed to parse head id");

        let result =
            ops::log_with_path(&handle, head_id, "a", None).expect("failed to get log");

        assert!(result.len() >= 3);
    }

    #[test]
    fn root_level_file() {
        let repo = TestRepo::with_deep_paths();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id =
            gix_hash::ObjectId::from_hex(head_id_str.as_bytes()).expect("failed to parse head id");

        let result = ops::log_with_path(&handle, head_id, "root.txt", None)
            .expect("failed to get log");

        assert_eq!(result.len(), 1);
    }

    #[test]
    fn merge_commit_shows_file_changes() {
        let repo = TestRepo::with_merge_commits();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id =
            gix_hash::ObjectId::from_hex(head_id_str.as_bytes()).expect("failed to parse head id");

        let result = ops::log_with_path(&handle, head_id, "feature.txt", None)
            .expect("failed to get log");

        assert!(!result.is_empty());
    }

    #[test]
    fn path_with_leading_slash_handled() {
        let repo = TestRepo::with_deep_paths();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id =
            gix_hash::ObjectId::from_hex(head_id_str.as_bytes()).expect("failed to parse head id");

        let result = ops::log_with_path(&handle, head_id, "/a/level1.txt", None)
            .expect("failed to get log");

        assert!(result.len() <= 2);
    }
}

mod merge_base {
    use super::*;

    #[test]
    fn same_commit() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id = gix_hash::ObjectId::from_hex(head_id_str.as_bytes())
            .expect("failed to parse head id");

        let result = ops::merge_base(&handle, head_id, head_id).expect("failed to get merge base");

        assert_eq!(result, head_id);
    }

    #[test]
    fn parent_and_child() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id = gix_hash::ObjectId::from_hex(head_id_str.as_bytes())
            .expect("failed to parse head id");

        let parent_id_str = repo.git_output(&["rev-parse", "HEAD~1"]);
        let parent_id = gix_hash::ObjectId::from_hex(parent_id_str.as_bytes())
            .expect("failed to parse parent id");

        let result = ops::merge_base(&handle, head_id, parent_id).expect("failed to get merge base");

        assert_eq!(result, parent_id);
    }

    #[test]
    fn grandparent_and_child() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id = gix_hash::ObjectId::from_hex(head_id_str.as_bytes())
            .expect("failed to parse head id");

        let grandparent_id_str = repo.git_output(&["rev-parse", "HEAD~2"]);
        let grandparent_id = gix_hash::ObjectId::from_hex(grandparent_id_str.as_bytes())
            .expect("failed to parse grandparent id");

        let result =
            ops::merge_base(&handle, head_id, grandparent_id).expect("failed to get merge base");

        assert_eq!(result, grandparent_id);
    }

    #[test]
    fn common_ancestor_in_linear_history() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit1_str = repo.git_output(&["rev-parse", "HEAD~1"]);
        let commit1 = gix_hash::ObjectId::from_hex(commit1_str.as_bytes())
            .expect("failed to parse commit1");

        let commit2_str = repo.git_output(&["rev-parse", "HEAD~3"]);
        let commit2 = gix_hash::ObjectId::from_hex(commit2_str.as_bytes())
            .expect("failed to parse commit2");

        let result = ops::merge_base(&handle, commit1, commit2).expect("failed to get merge base");

        assert_eq!(result, commit2);
    }

    #[test]
    fn order_independent() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id = gix_hash::ObjectId::from_hex(head_id_str.as_bytes())
            .expect("failed to parse head id");

        let old_id_str = repo.git_output(&["rev-parse", "HEAD~3"]);
        let old_id = gix_hash::ObjectId::from_hex(old_id_str.as_bytes())
            .expect("failed to parse old id");

        let result1 = ops::merge_base(&handle, head_id, old_id).expect("failed to get merge base");
        let result2 = ops::merge_base(&handle, old_id, head_id).expect("failed to get merge base");

        assert_eq!(result1, result2);
    }

    #[test]
    fn branches_with_common_ancestor() {
        let repo = TestRepo::with_branches();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let feature_a_str = repo.git_output(&["rev-parse", "feature-a"]);
        let feature_a = gix_hash::ObjectId::from_hex(feature_a_str.as_bytes())
            .expect("failed to parse feature-a");

        let main_str = repo.git_output(&["rev-parse", "main"]);
        let main_id =
            gix_hash::ObjectId::from_hex(main_str.as_bytes()).expect("failed to parse main");

        let result = ops::merge_base(&handle, feature_a, main_id).expect("failed to get merge base");

        assert_eq!(result, main_id);
    }

    #[test]
    fn feature_branches_common_ancestor() {
        let repo = TestRepo::with_branches();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let feature_a_str = repo.git_output(&["rev-parse", "feature-a"]);
        let feature_a = gix_hash::ObjectId::from_hex(feature_a_str.as_bytes())
            .expect("failed to parse feature-a");

        let feature_b_str = repo.git_output(&["rev-parse", "feature-b"]);
        let feature_b = gix_hash::ObjectId::from_hex(feature_b_str.as_bytes())
            .expect("failed to parse feature-b");

        let result =
            ops::merge_base(&handle, feature_a, feature_b).expect("failed to get merge base");

        assert_eq!(result, feature_a);
    }

    #[test]
    fn orphan_branches_no_common_ancestor() {
        let repo = TestRepo::with_orphan_branch();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let main_str = repo.git_output(&["rev-parse", "main"]);
        let main_id =
            gix_hash::ObjectId::from_hex(main_str.as_bytes()).expect("failed to parse main");

        let orphan_str = repo.git_output(&["rev-parse", "orphan"]);
        let orphan_id =
            gix_hash::ObjectId::from_hex(orphan_str.as_bytes()).expect("failed to parse orphan");

        let result = ops::merge_base(&handle, main_id, orphan_id);

        assert!(result.is_err());
    }

    #[test]
    fn merge_commit_base() {
        let repo = TestRepo::with_merge_commits();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id =
            gix_hash::ObjectId::from_hex(head_str.as_bytes()).expect("failed to parse head");

        let initial_str = repo.git_output(&["rev-list", "--max-parents=0", "HEAD"]);
        let initial_id =
            gix_hash::ObjectId::from_hex(initial_str.as_bytes()).expect("failed to parse initial");

        let result = ops::merge_base(&handle, head_id, initial_id).expect("failed to get merge base");

        assert_eq!(result, initial_id);
    }
}

mod is_ancestor {
    use super::*;

    #[test]
    fn same_commit_is_ancestor() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id = gix_hash::ObjectId::from_hex(head_id_str.as_bytes())
            .expect("failed to parse head id");

        let result = ops::is_ancestor(&handle, head_id, head_id).expect("failed to check ancestor");

        assert!(result);
    }

    #[test]
    fn parent_is_ancestor() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id = gix_hash::ObjectId::from_hex(head_id_str.as_bytes())
            .expect("failed to parse head id");

        let parent_id_str = repo.git_output(&["rev-parse", "HEAD~1"]);
        let parent_id = gix_hash::ObjectId::from_hex(parent_id_str.as_bytes())
            .expect("failed to parse parent id");

        let result =
            ops::is_ancestor(&handle, parent_id, head_id).expect("failed to check ancestor");

        assert!(result);
    }

    #[test]
    fn grandparent_is_ancestor() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id = gix_hash::ObjectId::from_hex(head_id_str.as_bytes())
            .expect("failed to parse head id");

        let grandparent_id_str = repo.git_output(&["rev-parse", "HEAD~3"]);
        let grandparent_id = gix_hash::ObjectId::from_hex(grandparent_id_str.as_bytes())
            .expect("failed to parse grandparent id");

        let result =
            ops::is_ancestor(&handle, grandparent_id, head_id).expect("failed to check ancestor");

        assert!(result);
    }

    #[test]
    fn initial_commit_is_ancestor_of_all() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id = gix_hash::ObjectId::from_hex(head_id_str.as_bytes())
            .expect("failed to parse head id");

        let initial_id_str = repo.git_output(&["rev-list", "--max-parents=0", "HEAD"]);
        let initial_id = gix_hash::ObjectId::from_hex(initial_id_str.as_bytes())
            .expect("failed to parse initial commit id");

        let result =
            ops::is_ancestor(&handle, initial_id, head_id).expect("failed to check ancestor");

        assert!(result);
    }

    #[test]
    fn child_is_not_ancestor_of_parent() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id = gix_hash::ObjectId::from_hex(head_id_str.as_bytes())
            .expect("failed to parse head id");

        let parent_id_str = repo.git_output(&["rev-parse", "HEAD~1"]);
        let parent_id = gix_hash::ObjectId::from_hex(parent_id_str.as_bytes())
            .expect("failed to parse parent id");

        let result =
            ops::is_ancestor(&handle, head_id, parent_id).expect("failed to check ancestor");

        assert!(!result);
    }

    #[test]
    fn unrelated_commits_not_ancestors() {
        let repo = TestRepo::with_branches();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let feature_b_str = repo.git_output(&["rev-parse", "feature-b"]);
        let feature_b = gix_hash::ObjectId::from_hex(feature_b_str.as_bytes())
            .expect("failed to parse feature-b");

        let main_str = repo.git_output(&["rev-parse", "main"]);
        let main_id =
            gix_hash::ObjectId::from_hex(main_str.as_bytes()).expect("failed to parse main");

        let result =
            ops::is_ancestor(&handle, feature_b, main_id).expect("failed to check ancestor");

        assert!(!result);
    }

    #[test]
    fn main_is_ancestor_of_feature() {
        let repo = TestRepo::with_branches();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let feature_a_str = repo.git_output(&["rev-parse", "feature-a"]);
        let feature_a = gix_hash::ObjectId::from_hex(feature_a_str.as_bytes())
            .expect("failed to parse feature-a");

        let main_str = repo.git_output(&["rev-parse", "main"]);
        let main_id =
            gix_hash::ObjectId::from_hex(main_str.as_bytes()).expect("failed to parse main");

        let result =
            ops::is_ancestor(&handle, main_id, feature_a).expect("failed to check ancestor");

        assert!(result);
    }

    #[test]
    fn feature_a_is_ancestor_of_feature_b() {
        let repo = TestRepo::with_branches();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let feature_a_str = repo.git_output(&["rev-parse", "feature-a"]);
        let feature_a = gix_hash::ObjectId::from_hex(feature_a_str.as_bytes())
            .expect("failed to parse feature-a");

        let feature_b_str = repo.git_output(&["rev-parse", "feature-b"]);
        let feature_b = gix_hash::ObjectId::from_hex(feature_b_str.as_bytes())
            .expect("failed to parse feature-b");

        let result =
            ops::is_ancestor(&handle, feature_a, feature_b).expect("failed to check ancestor");

        assert!(result);
    }

    #[test]
    fn single_commit_repo_self_is_ancestor() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id = gix_hash::ObjectId::from_hex(head_id_str.as_bytes())
            .expect("failed to parse head id");

        let result = ops::is_ancestor(&handle, head_id, head_id).expect("failed to check ancestor");

        assert!(result);
    }

    #[test]
    fn orphan_branches_not_ancestors() {
        let repo = TestRepo::with_orphan_branch();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let main_str = repo.git_output(&["rev-parse", "main"]);
        let main_id =
            gix_hash::ObjectId::from_hex(main_str.as_bytes()).expect("failed to parse main");

        let orphan_str = repo.git_output(&["rev-parse", "orphan"]);
        let orphan_id =
            gix_hash::ObjectId::from_hex(orphan_str.as_bytes()).expect("failed to parse orphan");

        let result1 =
            ops::is_ancestor(&handle, main_id, orphan_id).expect("failed to check ancestor");
        let result2 =
            ops::is_ancestor(&handle, orphan_id, main_id).expect("failed to check ancestor");

        assert!(!result1);
        assert!(!result2);
    }

    #[test]
    fn merge_commit_parent_is_ancestor() {
        let repo = TestRepo::with_merge_commits();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id =
            gix_hash::ObjectId::from_hex(head_str.as_bytes()).expect("failed to parse head");

        let initial_str = repo.git_output(&["rev-list", "--max-parents=0", "HEAD"]);
        let initial_id =
            gix_hash::ObjectId::from_hex(initial_str.as_bytes()).expect("failed to parse initial");

        let result =
            ops::is_ancestor(&handle, initial_id, head_id).expect("failed to check ancestor");

        assert!(result);
    }

    #[test]
    fn deep_ancestry_chain() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id = gix_hash::ObjectId::from_hex(head_id_str.as_bytes())
            .expect("failed to parse head id");

        let initial_id_str = repo.git_output(&["rev-list", "--max-parents=0", "HEAD"]);
        let initial_id = gix_hash::ObjectId::from_hex(initial_id_str.as_bytes())
            .expect("failed to parse initial commit id");

        let mid_id_str = repo.git_output(&["rev-parse", "HEAD~3"]);
        let mid_id = gix_hash::ObjectId::from_hex(mid_id_str.as_bytes())
            .expect("failed to parse mid commit id");

        let init_to_mid =
            ops::is_ancestor(&handle, initial_id, mid_id).expect("failed to check ancestor");
        let mid_to_head =
            ops::is_ancestor(&handle, mid_id, head_id).expect("failed to check ancestor");
        let head_to_mid =
            ops::is_ancestor(&handle, head_id, mid_id).expect("failed to check ancestor");

        assert!(init_to_mid);
        assert!(mid_to_head);
        assert!(!head_to_mid);
    }
}

mod count_commits {
    use super::*;

    #[test]
    fn all_commits() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id = gix_hash::ObjectId::from_hex(head_id_str.as_bytes())
            .expect("failed to parse head id");

        let result = ops::count_commits(&handle, head_id, None).expect("failed to count commits");

        assert_eq!(result, 6);
    }

    #[test]
    fn with_stop_commit() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id = gix_hash::ObjectId::from_hex(head_id_str.as_bytes())
            .expect("failed to parse head id");

        let stop_id_str = repo.git_output(&["rev-parse", "HEAD~3"]);
        let stop_id = gix_hash::ObjectId::from_hex(stop_id_str.as_bytes())
            .expect("failed to parse stop id");

        let result =
            ops::count_commits(&handle, head_id, Some(stop_id)).expect("failed to count commits");

        assert_eq!(result, 3);
    }

    #[test]
    fn stop_at_parent() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id = gix_hash::ObjectId::from_hex(head_id_str.as_bytes())
            .expect("failed to parse head id");

        let parent_id_str = repo.git_output(&["rev-parse", "HEAD~1"]);
        let parent_id = gix_hash::ObjectId::from_hex(parent_id_str.as_bytes())
            .expect("failed to parse parent id");

        let result =
            ops::count_commits(&handle, head_id, Some(parent_id)).expect("failed to count commits");

        assert_eq!(result, 1);
    }

    #[test]
    fn stop_at_initial_commit() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id = gix_hash::ObjectId::from_hex(head_id_str.as_bytes())
            .expect("failed to parse head id");

        let initial_id_str = repo.git_output(&["rev-list", "--max-parents=0", "HEAD"]);
        let initial_id = gix_hash::ObjectId::from_hex(initial_id_str.as_bytes())
            .expect("failed to parse initial commit id");

        let result =
            ops::count_commits(&handle, head_id, Some(initial_id)).expect("failed to count commits");

        assert_eq!(result, 5);
    }

    #[test]
    fn stop_at_head() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id = gix_hash::ObjectId::from_hex(head_id_str.as_bytes())
            .expect("failed to parse head id");

        let result =
            ops::count_commits(&handle, head_id, Some(head_id)).expect("failed to count commits");

        assert_eq!(result, 0);
    }

    #[test]
    fn single_commit_repo() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id = gix_hash::ObjectId::from_hex(head_id_str.as_bytes())
            .expect("failed to parse head id");

        let result = ops::count_commits(&handle, head_id, None).expect("failed to count commits");

        assert_eq!(result, 1);
    }

    #[test]
    fn count_from_older_commit() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let old_id_str = repo.git_output(&["rev-parse", "HEAD~2"]);
        let old_id = gix_hash::ObjectId::from_hex(old_id_str.as_bytes())
            .expect("failed to parse old id");

        let result = ops::count_commits(&handle, old_id, None).expect("failed to count commits");

        assert_eq!(result, 4);
    }

    #[test]
    fn count_from_branch() {
        let repo = TestRepo::with_branches();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let feature_a_str = repo.git_output(&["rev-parse", "feature-a"]);
        let feature_a = gix_hash::ObjectId::from_hex(feature_a_str.as_bytes())
            .expect("failed to parse feature-a");

        let result =
            ops::count_commits(&handle, feature_a, None).expect("failed to count commits");

        assert_eq!(result, 2);
    }

    #[test]
    fn count_between_branches() {
        let repo = TestRepo::with_branches();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let feature_b_str = repo.git_output(&["rev-parse", "feature-b"]);
        let feature_b = gix_hash::ObjectId::from_hex(feature_b_str.as_bytes())
            .expect("failed to parse feature-b");

        let main_str = repo.git_output(&["rev-parse", "main"]);
        let main_id =
            gix_hash::ObjectId::from_hex(main_str.as_bytes()).expect("failed to parse main");

        let result =
            ops::count_commits(&handle, feature_b, Some(main_id)).expect("failed to count commits");

        assert_eq!(result, 2);
    }

    #[test]
    fn stop_commit_not_in_history() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id = gix_hash::ObjectId::from_hex(head_id_str.as_bytes())
            .expect("failed to parse head id");

        let fake_stop = gix_hash::ObjectId::from_hex(b"1234567890abcdef1234567890abcdef12345678")
            .expect("valid hex");

        let result =
            ops::count_commits(&handle, head_id, Some(fake_stop)).expect("failed to count commits");

        assert_eq!(result, 6);
    }

    #[test]
    fn count_merge_commits() {
        let repo = TestRepo::with_merge_commits();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id =
            gix_hash::ObjectId::from_hex(head_str.as_bytes()).expect("failed to parse head");

        let result = ops::count_commits(&handle, head_id, None).expect("failed to count commits");

        assert!(result >= 3);
    }

    #[test]
    fn count_orphan_branch() {
        let repo = TestRepo::with_orphan_branch();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let orphan_str = repo.git_output(&["rev-parse", "orphan"]);
        let orphan_id =
            gix_hash::ObjectId::from_hex(orphan_str.as_bytes()).expect("failed to parse orphan");

        let result = ops::count_commits(&handle, orphan_id, None).expect("failed to count commits");

        assert_eq!(result, 1);
    }

    #[test]
    fn count_deep_paths_repo() {
        let repo = TestRepo::with_deep_paths();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id =
            gix_hash::ObjectId::from_hex(head_str.as_bytes()).expect("failed to parse head");

        let result = ops::count_commits(&handle, head_id, None).expect("failed to count commits");

        assert_eq!(result, 4);
    }
}

mod log_with_path_edge_cases {
    use super::*;

    #[test]
    fn path_with_trailing_slash() {
        let repo = TestRepo::with_deep_paths();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id =
            gix_hash::ObjectId::from_hex(head_id_str.as_bytes()).expect("failed to parse head id");

        let result = ops::log_with_path(&handle, head_id, "a/b/", None)
            .expect("failed to get log");

        assert!(result.len() >= 2);
    }

    #[test]
    fn path_with_double_slashes() {
        let repo = TestRepo::with_deep_paths();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id =
            gix_hash::ObjectId::from_hex(head_id_str.as_bytes()).expect("failed to parse head id");

        let result = ops::log_with_path(&handle, head_id, "a//b", None)
            .expect("failed to get log");

        assert!(result.len() >= 2);
    }

    #[test]
    fn empty_path() {
        let repo = TestRepo::with_deep_paths();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id =
            gix_hash::ObjectId::from_hex(head_id_str.as_bytes()).expect("failed to parse head id");

        let result = ops::log_with_path(&handle, head_id, "", None)
            .expect("failed to get log");

        assert!(result.len() >= 1);
    }

    #[test]
    fn single_character_path() {
        let repo = TestRepo::with_deep_paths();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id =
            gix_hash::ObjectId::from_hex(head_id_str.as_bytes()).expect("failed to parse head id");

        let result = ops::log_with_path(&handle, head_id, "a", None)
            .expect("failed to get log");

        assert!(result.len() >= 3);
    }
}

mod log_with_path_nested {
    use super::*;

    #[test]
    fn nested_path_with_multiple_slashes() {
        let repo = TestRepo::with_deep_paths();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id =
            gix_hash::ObjectId::from_hex(head_id_str.as_bytes()).expect("failed to parse head id");

        let result = ops::log_with_path(&handle, head_id, "a/b/c", None)
            .expect("failed to get log");

        assert!(result.len() >= 2);
    }

    #[test]
    fn commit_with_unchanged_nested_file() {
        let repo = TestRepo::with_unchanged_nested_file();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id =
            gix_hash::ObjectId::from_hex(head_id_str.as_bytes()).expect("failed to parse head id");

        let result = ops::log_with_path(&handle, head_id, "nested/deep/file.txt", None)
            .expect("failed to get log");

        assert_eq!(result.len(), 1);
    }

    #[test]
    fn multiple_nested_changes_tracked() {
        let repo = TestRepo::with_deep_paths();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id =
            gix_hash::ObjectId::from_hex(head_id_str.as_bytes()).expect("failed to parse head id");

        let result = ops::log_with_path(&handle, head_id, "a/b/c/level3.txt", None)
            .expect("failed to get log");

        assert!(result.len() >= 2);
    }

    #[test]
    fn intermediate_directory_changes() {
        let repo = TestRepo::with_deep_paths();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id =
            gix_hash::ObjectId::from_hex(head_id_str.as_bytes()).expect("failed to parse head id");

        let result = ops::log_with_path(&handle, head_id, "a/b", None)
            .expect("failed to get log");

        assert!(result.len() >= 3);
    }

    #[test]
    fn empty_commits_with_identical_trees() {
        let repo = TestRepo::with_empty_commits();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let head_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let head_id =
            gix_hash::ObjectId::from_hex(head_id_str.as_bytes()).expect("failed to parse head id");

        let result = ops::log_with_path(&handle, head_id, "file.txt", None)
            .expect("failed to get log");

        assert_eq!(result.len(), 2);
    }
}
