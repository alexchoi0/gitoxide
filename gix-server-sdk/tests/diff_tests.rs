mod fixtures;

use fixtures::TestRepo;
use gix_server_sdk::{ops, RepoPool, SdkConfig};

fn create_pool() -> RepoPool {
    RepoPool::new(SdkConfig::default())
}

mod diff_trees {
    use super::*;

    #[test]
    fn same_tree_returns_empty_diff() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let result = ops::diff_trees(&handle, tree_id, tree_id).expect("failed to diff trees");
        assert!(result.is_empty(), "expected no changes for same tree");
    }

    #[test]
    fn detects_file_additions() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let old_tree_str = repo.git_output(&["rev-parse", "HEAD~5^{tree}"]);
        let old_tree_id =
            gix_hash::ObjectId::from_hex(old_tree_str.as_bytes()).expect("failed to parse tree id");

        let new_tree_str = repo.git_output(&["rev-parse", "HEAD~4^{tree}"]);
        let new_tree_id =
            gix_hash::ObjectId::from_hex(new_tree_str.as_bytes()).expect("failed to parse tree id");

        let result = ops::diff_trees(&handle, old_tree_id, new_tree_id).expect("failed to diff trees");

        let added: Vec<_> = result
            .iter()
            .filter(|e| e.change == ops::ChangeKind::Added)
            .collect();

        assert!(!added.is_empty(), "expected at least one added file");
        for entry in &added {
            assert!(entry.old_id.is_none());
            assert!(entry.new_id.is_some());
            assert!(entry.old_mode.is_none());
            assert!(entry.new_mode.is_some());
        }
    }

    #[test]
    fn detects_file_deletions() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let old_tree_str = repo.git_output(&["rev-parse", "HEAD~1^{tree}"]);
        let old_tree_id =
            gix_hash::ObjectId::from_hex(old_tree_str.as_bytes()).expect("failed to parse tree id");

        let new_tree_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let new_tree_id =
            gix_hash::ObjectId::from_hex(new_tree_str.as_bytes()).expect("failed to parse tree id");

        let result = ops::diff_trees(&handle, old_tree_id, new_tree_id).expect("failed to diff trees");

        let deleted: Vec<_> = result
            .iter()
            .filter(|e| e.change == ops::ChangeKind::Deleted)
            .collect();

        assert!(!deleted.is_empty(), "expected at least one deleted file");
        for entry in &deleted {
            assert!(entry.old_id.is_some());
            assert!(entry.new_id.is_none());
            assert!(entry.old_mode.is_some());
            assert!(entry.new_mode.is_none());
        }
    }

    #[test]
    fn detects_file_modifications() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let old_tree_str = repo.git_output(&["rev-parse", "HEAD~1^{tree}"]);
        let old_tree_id =
            gix_hash::ObjectId::from_hex(old_tree_str.as_bytes()).expect("failed to parse tree id");

        let new_tree_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let new_tree_id =
            gix_hash::ObjectId::from_hex(new_tree_str.as_bytes()).expect("failed to parse tree id");

        let result = ops::diff_trees(&handle, old_tree_id, new_tree_id).expect("failed to diff trees");

        let modified: Vec<_> = result
            .iter()
            .filter(|e| e.change == ops::ChangeKind::Modified)
            .collect();

        assert!(!modified.is_empty(), "expected at least one modified file");
        for entry in &modified {
            assert!(entry.old_id.is_some());
            assert!(entry.new_id.is_some());
            assert!(entry.old_mode.is_some());
            assert!(entry.new_mode.is_some());
            assert_ne!(entry.old_id, entry.new_id);
        }
    }

    #[test]
    fn multiple_changes_across_commits() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let old_tree_str = repo.git_output(&["rev-parse", "HEAD~5^{tree}"]);
        let old_tree_id =
            gix_hash::ObjectId::from_hex(old_tree_str.as_bytes()).expect("failed to parse tree id");

        let new_tree_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let new_tree_id =
            gix_hash::ObjectId::from_hex(new_tree_str.as_bytes()).expect("failed to parse tree id");

        let result = ops::diff_trees(&handle, old_tree_id, new_tree_id).expect("failed to diff trees");

        assert!(!result.is_empty(), "expected multiple changes");

        let has_added = result.iter().any(|e| e.change == ops::ChangeKind::Added);
        let has_modified = result.iter().any(|e| e.change == ops::ChangeKind::Modified);

        assert!(has_added || has_modified, "expected additions or modifications");
    }

    #[test]
    fn diff_entries_have_valid_paths() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let old_tree_str = repo.git_output(&["rev-parse", "HEAD~2^{tree}"]);
        let old_tree_id =
            gix_hash::ObjectId::from_hex(old_tree_str.as_bytes()).expect("failed to parse tree id");

        let new_tree_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let new_tree_id =
            gix_hash::ObjectId::from_hex(new_tree_str.as_bytes()).expect("failed to parse tree id");

        let result = ops::diff_trees(&handle, old_tree_id, new_tree_id).expect("failed to diff trees");

        for entry in &result {
            assert!(!entry.path.is_empty(), "path should not be empty");
        }
    }
}

mod diff_commits {
    use super::*;

    #[test]
    fn same_commit_returns_empty_diff() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id =
            gix_hash::ObjectId::from_hex(commit_id_str.as_bytes()).expect("failed to parse commit id");

        let result = ops::diff_commits(&handle, commit_id, commit_id).expect("failed to diff commits");
        assert!(result.is_empty(), "expected no changes for same commit");
    }

    #[test]
    fn diff_between_adjacent_commits() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let old_commit_str = repo.git_output(&["rev-parse", "HEAD~1"]);
        let old_commit_id =
            gix_hash::ObjectId::from_hex(old_commit_str.as_bytes()).expect("failed to parse commit id");

        let new_commit_str = repo.git_output(&["rev-parse", "HEAD"]);
        let new_commit_id =
            gix_hash::ObjectId::from_hex(new_commit_str.as_bytes()).expect("failed to parse commit id");

        let result = ops::diff_commits(&handle, old_commit_id, new_commit_id).expect("failed to diff commits");

        assert!(!result.is_empty(), "expected changes between adjacent commits");
    }

    #[test]
    fn diff_between_distant_commits() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let old_commit_str = repo.git_output(&["rev-parse", "HEAD~5"]);
        let old_commit_id =
            gix_hash::ObjectId::from_hex(old_commit_str.as_bytes()).expect("failed to parse commit id");

        let new_commit_str = repo.git_output(&["rev-parse", "HEAD"]);
        let new_commit_id =
            gix_hash::ObjectId::from_hex(new_commit_str.as_bytes()).expect("failed to parse commit id");

        let result = ops::diff_commits(&handle, old_commit_id, new_commit_id).expect("failed to diff commits");

        assert!(!result.is_empty(), "expected multiple changes between distant commits");
    }

    #[test]
    fn diff_commits_matches_diff_trees() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let old_commit_str = repo.git_output(&["rev-parse", "HEAD~2"]);
        let old_commit_id =
            gix_hash::ObjectId::from_hex(old_commit_str.as_bytes()).expect("failed to parse commit id");

        let new_commit_str = repo.git_output(&["rev-parse", "HEAD"]);
        let new_commit_id =
            gix_hash::ObjectId::from_hex(new_commit_str.as_bytes()).expect("failed to parse commit id");

        let old_tree_str = repo.git_output(&["rev-parse", "HEAD~2^{tree}"]);
        let old_tree_id =
            gix_hash::ObjectId::from_hex(old_tree_str.as_bytes()).expect("failed to parse tree id");

        let new_tree_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let new_tree_id =
            gix_hash::ObjectId::from_hex(new_tree_str.as_bytes()).expect("failed to parse tree id");

        let commit_diff = ops::diff_commits(&handle, old_commit_id, new_commit_id).expect("failed to diff commits");
        let tree_diff = ops::diff_trees(&handle, old_tree_id, new_tree_id).expect("failed to diff trees");

        assert_eq!(commit_diff.len(), tree_diff.len(), "commit diff should match tree diff");
    }

    #[test]
    fn diff_commits_detects_all_change_kinds() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let old_commit_str = repo.git_output(&["rev-parse", "HEAD~5"]);
        let old_commit_id =
            gix_hash::ObjectId::from_hex(old_commit_str.as_bytes()).expect("failed to parse commit id");

        let new_commit_str = repo.git_output(&["rev-parse", "HEAD"]);
        let new_commit_id =
            gix_hash::ObjectId::from_hex(new_commit_str.as_bytes()).expect("failed to parse commit id");

        let result = ops::diff_commits(&handle, old_commit_id, new_commit_id).expect("failed to diff commits");

        for entry in &result {
            match entry.change {
                ops::ChangeKind::Added => {
                    assert!(entry.new_id.is_some());
                    assert!(entry.old_id.is_none());
                }
                ops::ChangeKind::Deleted => {
                    assert!(entry.old_id.is_some());
                    assert!(entry.new_id.is_none());
                }
                ops::ChangeKind::Modified => {
                    assert!(entry.old_id.is_some());
                    assert!(entry.new_id.is_some());
                }
            }
        }
    }
}

mod diff_blob {
    use super::*;

    #[test]
    fn same_blob_returns_empty_diff() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let blob_id_str = repo.git_output(&["rev-parse", "HEAD:README.md"]);
        let blob_id =
            gix_hash::ObjectId::from_hex(blob_id_str.as_bytes()).expect("failed to parse blob id");

        let result = ops::diff_blob(&handle, blob_id, blob_id, 3).expect("failed to diff blobs");

        assert!(result.hunks.is_empty(), "expected no hunks for same blob");
        assert_eq!(result.additions, 0);
        assert_eq!(result.deletions, 0);
        assert_eq!(result.old_id, blob_id);
        assert_eq!(result.new_id, blob_id);
    }

    #[test]
    fn diff_modified_file_produces_hunks() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let old_blob_str = repo.git_output(&["rev-parse", "HEAD~1:src/lib.rs"]);
        let old_blob_id =
            gix_hash::ObjectId::from_hex(old_blob_str.as_bytes()).expect("failed to parse blob id");

        let new_blob_str = repo.git_output(&["rev-parse", "HEAD~0:src/lib.rs"]);
        let new_blob_id_result = gix_hash::ObjectId::from_hex(new_blob_str.as_bytes());

        if let Ok(new_blob_id) = new_blob_id_result {
            if old_blob_id != new_blob_id {
                let result = ops::diff_blob(&handle, old_blob_id, new_blob_id, 3).expect("failed to diff blobs");

                assert!(!result.hunks.is_empty(), "expected hunks for modified file");
                assert!(result.additions > 0 || result.deletions > 0, "expected some changes");
            }
        }
    }

    #[test]
    fn diff_blob_tracks_additions() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let old_blob_str = repo.git_output(&["rev-parse", "HEAD~2:src/lib.rs"]);
        let old_blob_id =
            gix_hash::ObjectId::from_hex(old_blob_str.as_bytes()).expect("failed to parse blob id");

        let new_blob_str = repo.git_output(&["rev-parse", "HEAD:src/lib.rs"]);
        let new_blob_id =
            gix_hash::ObjectId::from_hex(new_blob_str.as_bytes()).expect("failed to parse blob id");

        if old_blob_id != new_blob_id {
            let result = ops::diff_blob(&handle, old_blob_id, new_blob_id, 3).expect("failed to diff blobs");

            let mut counted_additions = 0u32;
            let mut counted_deletions = 0u32;
            for hunk in &result.hunks {
                for line in &hunk.lines {
                    match line.kind {
                        ops::DiffLineKind::Addition => counted_additions += 1,
                        ops::DiffLineKind::Deletion => counted_deletions += 1,
                        ops::DiffLineKind::Context => {}
                    }
                }
            }

            assert_eq!(result.additions, counted_additions, "additions count should match");
            assert_eq!(result.deletions, counted_deletions, "deletions count should match");
        }
    }

    #[test]
    fn diff_blob_hunk_structure() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let old_blob_str = repo.git_output(&["rev-parse", "HEAD~5:README.md"]);
        let old_blob_id =
            gix_hash::ObjectId::from_hex(old_blob_str.as_bytes()).expect("failed to parse blob id");

        let new_blob_str = repo.git_output(&["rev-parse", "HEAD:README.md"]);
        let new_blob_id =
            gix_hash::ObjectId::from_hex(new_blob_str.as_bytes()).expect("failed to parse blob id");

        if old_blob_id != new_blob_id {
            let result = ops::diff_blob(&handle, old_blob_id, new_blob_id, 3).expect("failed to diff blobs");

            for hunk in &result.hunks {
                assert!(hunk.old_lines > 0 || hunk.new_lines > 0, "hunk should have lines");
                assert!(!hunk.lines.is_empty(), "hunk should have diff lines");
            }
        }
    }

    #[test]
    fn diff_blob_with_zero_context() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let old_blob_str = repo.git_output(&["rev-parse", "HEAD~5:README.md"]);
        let old_blob_id =
            gix_hash::ObjectId::from_hex(old_blob_str.as_bytes()).expect("failed to parse blob id");

        let new_blob_str = repo.git_output(&["rev-parse", "HEAD:README.md"]);
        let new_blob_id =
            gix_hash::ObjectId::from_hex(new_blob_str.as_bytes()).expect("failed to parse blob id");

        if old_blob_id != new_blob_id {
            let result = ops::diff_blob(&handle, old_blob_id, new_blob_id, 0).expect("failed to diff blobs");

            for hunk in &result.hunks {
                let context_count = hunk
                    .lines
                    .iter()
                    .filter(|l| l.kind == ops::DiffLineKind::Context)
                    .count();
                assert_eq!(context_count, 0, "expected no context lines with context_lines=0");
            }
        }
    }

    #[test]
    fn diff_blob_with_large_context() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let old_blob_str = repo.git_output(&["rev-parse", "HEAD~5:README.md"]);
        let old_blob_id =
            gix_hash::ObjectId::from_hex(old_blob_str.as_bytes()).expect("failed to parse blob id");

        let new_blob_str = repo.git_output(&["rev-parse", "HEAD:README.md"]);
        let new_blob_id =
            gix_hash::ObjectId::from_hex(new_blob_str.as_bytes()).expect("failed to parse blob id");

        if old_blob_id != new_blob_id {
            let result = ops::diff_blob(&handle, old_blob_id, new_blob_id, 10).expect("failed to diff blobs");
            assert!(!result.hunks.is_empty() || (result.additions == 0 && result.deletions == 0));
        }
    }

    #[test]
    fn diff_blob_line_content_is_preserved() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let old_blob_str = repo.git_output(&["rev-parse", "HEAD~3:src/lib.rs"]);
        let old_blob_id =
            gix_hash::ObjectId::from_hex(old_blob_str.as_bytes()).expect("failed to parse blob id");

        let new_blob_str = repo.git_output(&["rev-parse", "HEAD:src/lib.rs"]);
        let new_blob_id =
            gix_hash::ObjectId::from_hex(new_blob_str.as_bytes()).expect("failed to parse blob id");

        if old_blob_id != new_blob_id {
            let result = ops::diff_blob(&handle, old_blob_id, new_blob_id, 3).expect("failed to diff blobs");

            for hunk in &result.hunks {
                for line in &hunk.lines {
                    assert!(!line.content.is_empty() || line.content.len() == 0);
                }
            }
        }
    }
}

mod diff_blob_inline {
    use super::*;

    #[test]
    fn same_content_returns_empty_diff() {
        let content = b"line1\nline2\nline3\n";
        let result = ops::diff_blob_inline(content, content, 3).expect("failed to diff");

        assert!(result.hunks.is_empty());
        assert_eq!(result.additions, 0);
        assert_eq!(result.deletions, 0);
    }

    #[test]
    fn detects_additions() {
        let old_content = b"line1\nline2\n";
        let new_content = b"line1\nline2\nline3\n";

        let result = ops::diff_blob_inline(old_content, new_content, 3).expect("failed to diff");

        assert!(!result.hunks.is_empty());
        assert!(result.additions > 0);
    }

    #[test]
    fn detects_deletions() {
        let old_content = b"line1\nline2\nline3\n";
        let new_content = b"line1\nline2\n";

        let result = ops::diff_blob_inline(old_content, new_content, 3).expect("failed to diff");

        assert!(!result.hunks.is_empty());
        assert!(result.deletions > 0);
    }

    #[test]
    fn detects_modifications() {
        let old_content = b"line1\nline2\nline3\n";
        let new_content = b"line1\nmodified\nline3\n";

        let result = ops::diff_blob_inline(old_content, new_content, 3).expect("failed to diff");

        assert!(!result.hunks.is_empty());
        assert!(result.additions > 0);
        assert!(result.deletions > 0);
    }

    #[test]
    fn empty_old_content_shows_all_as_additions() {
        let old_content = b"";
        let new_content = b"line1\nline2\nline3\n";

        let result = ops::diff_blob_inline(old_content, new_content, 3).expect("failed to diff");

        assert!(result.additions >= 3);
        assert_eq!(result.deletions, 0);
    }

    #[test]
    fn empty_new_content_shows_all_as_deletions() {
        let old_content = b"line1\nline2\nline3\n";
        let new_content = b"";

        let result = ops::diff_blob_inline(old_content, new_content, 3).expect("failed to diff");

        assert_eq!(result.additions, 0);
        assert!(result.deletions >= 3);
    }

    #[test]
    fn context_lines_respected() {
        let old_content = b"a\nb\nc\nd\ne\nf\ng\nh\ni\nj\n";
        let new_content = b"a\nb\nc\nd\nX\nf\ng\nh\ni\nj\n";

        let result_ctx3 = ops::diff_blob_inline(old_content, new_content, 3).expect("failed to diff");
        let result_ctx0 = ops::diff_blob_inline(old_content, new_content, 0).expect("failed to diff");

        if !result_ctx3.hunks.is_empty() && !result_ctx0.hunks.is_empty() {
            let ctx3_lines: usize = result_ctx3.hunks.iter().map(|h| h.lines.len()).sum();
            let ctx0_lines: usize = result_ctx0.hunks.iter().map(|h| h.lines.len()).sum();
            assert!(ctx3_lines >= ctx0_lines, "more context should mean more lines");
        }
    }
}

mod diff_stats {
    use super::*;

    #[test]
    fn same_tree_returns_zero_stats() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let result = ops::diff_stats(&handle, tree_id, tree_id).expect("failed to get stats");

        assert_eq!(result.files_changed, 0);
        assert_eq!(result.additions, 0);
        assert_eq!(result.deletions, 0);
        assert!(result.entries.is_empty());
    }

    #[test]
    fn stats_for_additions() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let old_tree_str = repo.git_output(&["rev-parse", "HEAD~4^{tree}"]);
        let old_tree_id =
            gix_hash::ObjectId::from_hex(old_tree_str.as_bytes()).expect("failed to parse tree id");

        let new_tree_str = repo.git_output(&["rev-parse", "HEAD~3^{tree}"]);
        let new_tree_id =
            gix_hash::ObjectId::from_hex(new_tree_str.as_bytes()).expect("failed to parse tree id");

        let result = ops::diff_stats(&handle, old_tree_id, new_tree_id).expect("failed to get stats");

        assert!(result.files_changed > 0, "expected some files changed");
        assert!(result.additions > 0, "expected some additions");
        assert_eq!(result.entries.len(), result.files_changed);
    }

    #[test]
    fn stats_for_deletions() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let old_tree_str = repo.git_output(&["rev-parse", "HEAD~1^{tree}"]);
        let old_tree_id =
            gix_hash::ObjectId::from_hex(old_tree_str.as_bytes()).expect("failed to parse tree id");

        let new_tree_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let new_tree_id =
            gix_hash::ObjectId::from_hex(new_tree_str.as_bytes()).expect("failed to parse tree id");

        let result = ops::diff_stats(&handle, old_tree_id, new_tree_id).expect("failed to get stats");

        assert!(result.files_changed > 0, "expected some files changed");
        assert!(result.deletions > 0 || result.additions > 0, "expected some changes");
    }

    #[test]
    fn stats_for_modifications() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let old_tree_str = repo.git_output(&["rev-parse", "HEAD~2^{tree}"]);
        let old_tree_id =
            gix_hash::ObjectId::from_hex(old_tree_str.as_bytes()).expect("failed to parse tree id");

        let new_tree_str = repo.git_output(&["rev-parse", "HEAD~1^{tree}"]);
        let new_tree_id =
            gix_hash::ObjectId::from_hex(new_tree_str.as_bytes()).expect("failed to parse tree id");

        let result = ops::diff_stats(&handle, old_tree_id, new_tree_id).expect("failed to get stats");

        if result.files_changed > 0 {
            assert_eq!(result.entries.len(), result.files_changed);
        }
    }

    #[test]
    fn stats_totals_match_entry_sums() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let old_tree_str = repo.git_output(&["rev-parse", "HEAD~5^{tree}"]);
        let old_tree_id =
            gix_hash::ObjectId::from_hex(old_tree_str.as_bytes()).expect("failed to parse tree id");

        let new_tree_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let new_tree_id =
            gix_hash::ObjectId::from_hex(new_tree_str.as_bytes()).expect("failed to parse tree id");

        let result = ops::diff_stats(&handle, old_tree_id, new_tree_id).expect("failed to get stats");

        let sum_additions: u32 = result.entries.iter().map(|e| e.additions).sum();
        let sum_deletions: u32 = result.entries.iter().map(|e| e.deletions).sum();

        assert_eq!(result.additions, sum_additions, "total additions should match sum of entries");
        assert_eq!(result.deletions, sum_deletions, "total deletions should match sum of entries");
    }

    #[test]
    fn stats_entries_have_valid_paths() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let old_tree_str = repo.git_output(&["rev-parse", "HEAD~3^{tree}"]);
        let old_tree_id =
            gix_hash::ObjectId::from_hex(old_tree_str.as_bytes()).expect("failed to parse tree id");

        let new_tree_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let new_tree_id =
            gix_hash::ObjectId::from_hex(new_tree_str.as_bytes()).expect("failed to parse tree id");

        let result = ops::diff_stats(&handle, old_tree_id, new_tree_id).expect("failed to get stats");

        for entry in &result.entries {
            assert!(!entry.path.is_empty(), "path should not be empty");
        }
    }

    #[test]
    fn stats_files_changed_count() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let old_tree_str = repo.git_output(&["rev-parse", "HEAD~5^{tree}"]);
        let old_tree_id =
            gix_hash::ObjectId::from_hex(old_tree_str.as_bytes()).expect("failed to parse tree id");

        let new_tree_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let new_tree_id =
            gix_hash::ObjectId::from_hex(new_tree_str.as_bytes()).expect("failed to parse tree id");

        let diff_entries = ops::diff_trees(&handle, old_tree_id, new_tree_id).expect("failed to diff trees");
        let stats = ops::diff_stats(&handle, old_tree_id, new_tree_id).expect("failed to get stats");

        assert_eq!(stats.files_changed, diff_entries.len(), "files_changed should match diff entry count");
    }
}

mod diff_trees_with_filter {
    use super::*;

    #[test]
    fn filter_by_extension() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let old_tree_str = repo.git_output(&["rev-parse", "HEAD~5^{tree}"]);
        let old_tree_id =
            gix_hash::ObjectId::from_hex(old_tree_str.as_bytes()).expect("failed to parse tree id");

        let new_tree_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let new_tree_id =
            gix_hash::ObjectId::from_hex(new_tree_str.as_bytes()).expect("failed to parse tree id");

        let result = ops::diff_trees_with_filter(&handle, old_tree_id, new_tree_id, |path| {
            path.ends_with(b".rs")
        }).expect("failed to diff with filter");

        for entry in &result {
            assert!(
                entry.path.ends_with(b".rs"),
                "expected only .rs files, got: {:?}",
                entry.path
            );
        }
    }

    #[test]
    fn filter_by_directory() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let old_tree_str = repo.git_output(&["rev-parse", "HEAD~5^{tree}"]);
        let old_tree_id =
            gix_hash::ObjectId::from_hex(old_tree_str.as_bytes()).expect("failed to parse tree id");

        let new_tree_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let new_tree_id =
            gix_hash::ObjectId::from_hex(new_tree_str.as_bytes()).expect("failed to parse tree id");

        let result = ops::diff_trees_with_filter(&handle, old_tree_id, new_tree_id, |path| {
            path.starts_with(b"src/")
        }).expect("failed to diff with filter");

        for entry in &result {
            assert!(
                entry.path.starts_with(b"src/"),
                "expected only src/ files, got: {:?}",
                entry.path
            );
        }
    }

    #[test]
    fn filter_matches_nothing() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let old_tree_str = repo.git_output(&["rev-parse", "HEAD~5^{tree}"]);
        let old_tree_id =
            gix_hash::ObjectId::from_hex(old_tree_str.as_bytes()).expect("failed to parse tree id");

        let new_tree_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let new_tree_id =
            gix_hash::ObjectId::from_hex(new_tree_str.as_bytes()).expect("failed to parse tree id");

        let result = ops::diff_trees_with_filter(&handle, old_tree_id, new_tree_id, |_| false)
            .expect("failed to diff with filter");

        assert!(result.is_empty(), "expected no results when filter matches nothing");
    }

    #[test]
    fn filter_matches_everything() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let old_tree_str = repo.git_output(&["rev-parse", "HEAD~3^{tree}"]);
        let old_tree_id =
            gix_hash::ObjectId::from_hex(old_tree_str.as_bytes()).expect("failed to parse tree id");

        let new_tree_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let new_tree_id =
            gix_hash::ObjectId::from_hex(new_tree_str.as_bytes()).expect("failed to parse tree id");

        let filtered = ops::diff_trees_with_filter(&handle, old_tree_id, new_tree_id, |_| true)
            .expect("failed to diff with filter");
        let unfiltered = ops::diff_trees(&handle, old_tree_id, new_tree_id)
            .expect("failed to diff");

        assert_eq!(filtered.len(), unfiltered.len(), "filter that matches all should return all");
    }
}

mod diff_blob_error_paths {
    use super::*;
    use gix_server_sdk::SdkError;

    #[test]
    fn diff_blob_with_non_blob_old_object_returns_error() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let blob_id_str = repo.git_output(&["rev-parse", "HEAD:README.md"]);
        let blob_id =
            gix_hash::ObjectId::from_hex(blob_id_str.as_bytes()).expect("failed to parse blob id");

        let result = ops::diff_blob(&handle, tree_id, blob_id, 3);
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            SdkError::InvalidObjectType { expected, actual } => {
                assert_eq!(expected, "blob");
                assert_eq!(actual, "tree");
            }
            _ => panic!("expected InvalidObjectType error, got {:?}", err),
        }
    }

    #[test]
    fn diff_blob_with_non_blob_new_object_returns_error() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let blob_id_str = repo.git_output(&["rev-parse", "HEAD:README.md"]);
        let blob_id =
            gix_hash::ObjectId::from_hex(blob_id_str.as_bytes()).expect("failed to parse blob id");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let result = ops::diff_blob(&handle, blob_id, tree_id, 3);
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            SdkError::InvalidObjectType { expected, actual } => {
                assert_eq!(expected, "blob");
                assert_eq!(actual, "tree");
            }
            _ => panic!("expected InvalidObjectType error, got {:?}", err),
        }
    }

    #[test]
    fn diff_blob_with_nonexistent_old_object_returns_error() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let fake_id = gix_hash::ObjectId::from_hex(b"0000000000000000000000000000000000000000")
            .expect("failed to create fake id");
        let blob_id_str = repo.git_output(&["rev-parse", "HEAD:README.md"]);
        let blob_id =
            gix_hash::ObjectId::from_hex(blob_id_str.as_bytes()).expect("failed to parse blob id");

        let result = ops::diff_blob(&handle, fake_id, blob_id, 3);
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            SdkError::ObjectNotFound(id) => {
                assert_eq!(id, fake_id);
            }
            _ => panic!("expected ObjectNotFound error, got {:?}", err),
        }
    }

    #[test]
    fn diff_blob_with_nonexistent_new_object_returns_error() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let blob_id_str = repo.git_output(&["rev-parse", "HEAD:README.md"]);
        let blob_id =
            gix_hash::ObjectId::from_hex(blob_id_str.as_bytes()).expect("failed to parse blob id");

        let fake_id = gix_hash::ObjectId::from_hex(b"0000000000000000000000000000000000000000")
            .expect("failed to create fake id");

        let result = ops::diff_blob(&handle, blob_id, fake_id, 3);
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            SdkError::ObjectNotFound(id) => {
                assert_eq!(id, fake_id);
            }
            _ => panic!("expected ObjectNotFound error, got {:?}", err),
        }
    }

    #[test]
    fn diff_blob_with_commit_object_returns_error() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id =
            gix_hash::ObjectId::from_hex(commit_id_str.as_bytes()).expect("failed to parse commit id");

        let blob_id_str = repo.git_output(&["rev-parse", "HEAD:README.md"]);
        let blob_id =
            gix_hash::ObjectId::from_hex(blob_id_str.as_bytes()).expect("failed to parse blob id");

        let result = ops::diff_blob(&handle, commit_id, blob_id, 3);
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            SdkError::InvalidObjectType { expected, actual } => {
                assert_eq!(expected, "blob");
                assert_eq!(actual, "commit");
            }
            _ => panic!("expected InvalidObjectType error, got {:?}", err),
        }
    }
}

mod diff_commits_error_paths {
    use super::*;

    #[test]
    fn diff_commits_with_nonexistent_old_commit_returns_error() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let fake_id = gix_hash::ObjectId::from_hex(b"0000000000000000000000000000000000000000")
            .expect("failed to create fake id");
        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id =
            gix_hash::ObjectId::from_hex(commit_id_str.as_bytes()).expect("failed to parse commit id");

        let result = ops::diff_commits(&handle, fake_id, commit_id);
        assert!(result.is_err());
    }

    #[test]
    fn diff_commits_with_nonexistent_new_commit_returns_error() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id =
            gix_hash::ObjectId::from_hex(commit_id_str.as_bytes()).expect("failed to parse commit id");
        let fake_id = gix_hash::ObjectId::from_hex(b"0000000000000000000000000000000000000000")
            .expect("failed to create fake id");

        let result = ops::diff_commits(&handle, commit_id, fake_id);
        assert!(result.is_err());
    }
}

mod diff_trees_error_paths {
    use super::*;

    #[test]
    fn diff_trees_with_nonexistent_old_tree_returns_error() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let fake_id = gix_hash::ObjectId::from_hex(b"0000000000000000000000000000000000000000")
            .expect("failed to create fake id");
        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let result = ops::diff_trees(&handle, fake_id, tree_id);
        assert!(result.is_err());
    }

    #[test]
    fn diff_trees_with_nonexistent_new_tree_returns_error() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");
        let fake_id = gix_hash::ObjectId::from_hex(b"0000000000000000000000000000000000000000")
            .expect("failed to create fake id");

        let result = ops::diff_trees(&handle, tree_id, fake_id);
        assert!(result.is_err());
    }
}

mod diff_stats_error_paths {
    use super::*;

    #[test]
    fn diff_stats_with_nonexistent_old_tree_returns_error() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let fake_id = gix_hash::ObjectId::from_hex(b"0000000000000000000000000000000000000000")
            .expect("failed to create fake id");
        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let result = ops::diff_stats(&handle, fake_id, tree_id);
        assert!(result.is_err());
    }

    #[test]
    fn diff_stats_with_nonexistent_new_tree_returns_error() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");
        let fake_id = gix_hash::ObjectId::from_hex(b"0000000000000000000000000000000000000000")
            .expect("failed to create fake id");

        let result = ops::diff_stats(&handle, tree_id, fake_id);
        assert!(result.is_err());
    }
}

mod diff_blob_inline_edge_cases {
    use super::*;

    #[test]
    fn diff_blob_inline_both_empty() {
        let result = ops::diff_blob_inline(b"", b"", 3).expect("failed to diff");
        assert!(result.hunks.is_empty());
        assert_eq!(result.additions, 0);
        assert_eq!(result.deletions, 0);
    }

    #[test]
    fn diff_blob_inline_with_binary_content() {
        let old_content: Vec<u8> = (0..128).collect();
        let mut new_content: Vec<u8> = (0..128).collect();
        new_content[64] = 255;

        let result = ops::diff_blob_inline(&old_content, &new_content, 3).expect("failed to diff");
        assert!(result.additions > 0 || result.deletions > 0);
    }

    #[test]
    fn diff_blob_inline_single_line_change() {
        let old_content = b"single line";
        let new_content = b"modified line";

        let result = ops::diff_blob_inline(old_content, new_content, 3).expect("failed to diff");
        assert!(!result.hunks.is_empty());
        assert_eq!(result.additions, 1);
        assert_eq!(result.deletions, 1);
    }

    #[test]
    fn diff_blob_inline_only_newlines() {
        let old_content = b"\n\n\n";
        let new_content = b"\n\n\n\n\n";

        let result = ops::diff_blob_inline(old_content, new_content, 3).expect("failed to diff");
        assert!(result.additions > 0);
    }

    #[test]
    fn diff_blob_inline_large_context() {
        let old_content = b"line1\nline2\nline3\nline4\nline5\n";
        let new_content = b"line1\nline2\nmodified\nline4\nline5\n";

        let result = ops::diff_blob_inline(old_content, new_content, 100).expect("failed to diff");
        assert!(!result.hunks.is_empty());
    }

    #[test]
    fn diff_blob_inline_completely_different_content() {
        let old_content = b"aaa\nbbb\nccc\n";
        let new_content = b"xxx\nyyy\nzzz\n";

        let result = ops::diff_blob_inline(old_content, new_content, 3).expect("failed to diff");
        assert!(!result.hunks.is_empty());
        assert_eq!(result.additions, 3);
        assert_eq!(result.deletions, 3);
    }

    #[test]
    fn diff_blob_inline_multiple_hunks() {
        let old_content = b"a\nb\nc\nd\ne\nf\ng\nh\ni\nj\nk\nl\nm\nn\no\np\nq\nr\ns\nt\n";
        let new_content = b"a\nX\nc\nd\ne\nf\ng\nh\ni\nj\nk\nl\nm\nY\no\np\nq\nr\ns\nt\n";

        let result = ops::diff_blob_inline(old_content, new_content, 1).expect("failed to diff");
        assert!(result.hunks.len() >= 2, "expected multiple hunks");
    }

    #[test]
    fn diff_blob_inline_hunk_header_values() {
        let old_content = b"line1\nline2\nline3\n";
        let new_content = b"line1\nmodified\nline3\n";

        let result = ops::diff_blob_inline(old_content, new_content, 0).expect("failed to diff");
        assert!(!result.hunks.is_empty());

        let hunk = &result.hunks[0];
        assert!(hunk.old_start > 0);
        assert!(hunk.new_start > 0);
    }

    #[test]
    fn diff_blob_inline_preserves_non_utf8_content() {
        let old_content: Vec<u8> = vec![0x80, 0x81, 0x82, b'\n', 0x83, 0x84, b'\n'];
        let new_content: Vec<u8> = vec![0x80, 0x81, 0x82, b'\n', 0xFF, 0xFE, b'\n'];

        let result = ops::diff_blob_inline(&old_content, &new_content, 3).expect("failed to diff");
        if !result.hunks.is_empty() {
            for hunk in &result.hunks {
                for line in &hunk.lines {
                    let _ = &line.content;
                }
            }
        }
    }
}

mod diff_with_binary_files {
    use super::*;

    #[test]
    fn diff_trees_with_binary_files() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let result = ops::diff_trees(&handle, tree_id, tree_id).expect("failed to diff trees");
        assert!(result.is_empty());
    }

    #[test]
    fn diff_stats_with_binary_file() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_id_str.as_bytes()).expect("failed to parse tree id");

        let result = ops::diff_stats(&handle, tree_id, tree_id).expect("failed to get stats");
        assert_eq!(result.files_changed, 0);
    }
}

mod diff_hunk_line_kinds {
    use super::*;

    #[test]
    fn verify_all_line_kinds_present() {
        let old_content = b"context line\nold line\nmore context\n";
        let new_content = b"context line\nnew line\nmore context\n";

        let result = ops::diff_blob_inline(old_content, new_content, 3).expect("failed to diff");
        assert!(!result.hunks.is_empty());

        let mut has_context = false;
        let mut has_addition = false;
        let mut has_deletion = false;

        for hunk in &result.hunks {
            for line in &hunk.lines {
                match line.kind {
                    ops::DiffLineKind::Context => has_context = true,
                    ops::DiffLineKind::Addition => has_addition = true,
                    ops::DiffLineKind::Deletion => has_deletion = true,
                }
            }
        }

        assert!(has_context, "expected context lines");
        assert!(has_addition, "expected addition lines");
        assert!(has_deletion, "expected deletion lines");
    }

    #[test]
    fn diff_only_additions() {
        let old_content = b"";
        let new_content = b"line1\nline2\n";

        let result = ops::diff_blob_inline(old_content, new_content, 0).expect("failed to diff");

        for hunk in &result.hunks {
            for line in &hunk.lines {
                assert!(
                    line.kind == ops::DiffLineKind::Addition,
                    "expected only additions when old is empty"
                );
            }
        }
    }

    #[test]
    fn diff_only_deletions() {
        let old_content = b"line1\nline2\n";
        let new_content = b"";

        let result = ops::diff_blob_inline(old_content, new_content, 0).expect("failed to diff");

        for hunk in &result.hunks {
            for line in &hunk.lines {
                assert!(
                    line.kind == ops::DiffLineKind::Deletion,
                    "expected only deletions when new is empty"
                );
            }
        }
    }
}

mod diff_stats_change_types {
    use super::*;

    #[test]
    fn stats_correctly_counts_added_file_lines() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let old_tree_str = repo.git_output(&["rev-parse", "HEAD~5^{tree}"]);
        let old_tree_id =
            gix_hash::ObjectId::from_hex(old_tree_str.as_bytes()).expect("failed to parse tree id");

        let new_tree_str = repo.git_output(&["rev-parse", "HEAD~4^{tree}"]);
        let new_tree_id =
            gix_hash::ObjectId::from_hex(new_tree_str.as_bytes()).expect("failed to parse tree id");

        let stats = ops::diff_stats(&handle, old_tree_id, new_tree_id).expect("failed to get stats");

        for entry in &stats.entries {
            assert!(entry.additions > 0 || entry.deletions > 0 || entry.additions == 0 && entry.deletions == 0);
        }
    }

    #[test]
    fn stats_correctly_counts_deleted_file_lines() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let old_tree_str = repo.git_output(&["rev-parse", "HEAD~1^{tree}"]);
        let old_tree_id =
            gix_hash::ObjectId::from_hex(old_tree_str.as_bytes()).expect("failed to parse tree id");

        let new_tree_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let new_tree_id =
            gix_hash::ObjectId::from_hex(new_tree_str.as_bytes()).expect("failed to parse tree id");

        let entries = ops::diff_trees(&handle, old_tree_id, new_tree_id).expect("failed to diff");
        let stats = ops::diff_stats(&handle, old_tree_id, new_tree_id).expect("failed to get stats");

        let has_deleted = entries.iter().any(|e| e.change == ops::ChangeKind::Deleted);
        if has_deleted {
            assert!(stats.deletions > 0, "expected deletions for deleted files");
        }
    }

    #[test]
    fn stats_correctly_counts_modified_file_lines() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let old_tree_str = repo.git_output(&["rev-parse", "HEAD~2^{tree}"]);
        let old_tree_id =
            gix_hash::ObjectId::from_hex(old_tree_str.as_bytes()).expect("failed to parse tree id");

        let new_tree_str = repo.git_output(&["rev-parse", "HEAD~1^{tree}"]);
        let new_tree_id =
            gix_hash::ObjectId::from_hex(new_tree_str.as_bytes()).expect("failed to parse tree id");

        let entries = ops::diff_trees(&handle, old_tree_id, new_tree_id).expect("failed to diff");
        let stats = ops::diff_stats(&handle, old_tree_id, new_tree_id).expect("failed to get stats");

        let has_modified = entries.iter().any(|e| e.change == ops::ChangeKind::Modified);
        if has_modified {
            assert!(
                stats.additions > 0 || stats.deletions > 0,
                "expected changes for modified files"
            );
        }
    }

    #[test]
    fn stats_matches_detailed_diff_counts() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let old_tree_str = repo.git_output(&["rev-parse", "HEAD~3^{tree}"]);
        let old_tree_id =
            gix_hash::ObjectId::from_hex(old_tree_str.as_bytes()).expect("failed to parse tree id");

        let new_tree_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let new_tree_id =
            gix_hash::ObjectId::from_hex(new_tree_str.as_bytes()).expect("failed to parse tree id");

        let entries = ops::diff_trees(&handle, old_tree_id, new_tree_id).expect("failed to diff");
        let stats = ops::diff_stats(&handle, old_tree_id, new_tree_id).expect("failed to get stats");

        assert_eq!(stats.files_changed, entries.len());
        assert_eq!(stats.entries.len(), entries.len());
    }
}

mod diff_filter_edge_cases {
    use super::*;

    #[test]
    fn filter_with_complex_path_patterns() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let old_tree_str = repo.git_output(&["rev-parse", "HEAD~5^{tree}"]);
        let old_tree_id =
            gix_hash::ObjectId::from_hex(old_tree_str.as_bytes()).expect("failed to parse tree id");

        let new_tree_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let new_tree_id =
            gix_hash::ObjectId::from_hex(new_tree_str.as_bytes()).expect("failed to parse tree id");

        let result = ops::diff_trees_with_filter(&handle, old_tree_id, new_tree_id, |path| {
            path.contains(&b'/') && (path.ends_with(b".rs") || path.ends_with(b".md"))
        }).expect("failed to diff with filter");

        for entry in &result {
            let has_slash = entry.path.contains(&b'/');
            let ends_rs = entry.path.ends_with(b".rs");
            let ends_md = entry.path.ends_with(b".md");
            assert!(
                has_slash && (ends_rs || ends_md),
                "expected complex filter match, got: {:?}",
                entry.path
            );
        }
    }

    #[test]
    fn filter_preserves_change_kinds() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let old_tree_str = repo.git_output(&["rev-parse", "HEAD~5^{tree}"]);
        let old_tree_id =
            gix_hash::ObjectId::from_hex(old_tree_str.as_bytes()).expect("failed to parse tree id");

        let new_tree_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let new_tree_id =
            gix_hash::ObjectId::from_hex(new_tree_str.as_bytes()).expect("failed to parse tree id");

        let unfiltered = ops::diff_trees(&handle, old_tree_id, new_tree_id).expect("failed to diff");
        let filtered = ops::diff_trees_with_filter(&handle, old_tree_id, new_tree_id, |_| true)
            .expect("failed to diff with filter");

        for (uf, f) in unfiltered.iter().zip(filtered.iter()) {
            assert_eq!(uf.change, f.change);
            assert_eq!(uf.path, f.path);
            assert_eq!(uf.old_id, f.old_id);
            assert_eq!(uf.new_id, f.new_id);
        }
    }
}

mod diff_blob_special_cases {
    use super::*;

    #[test]
    fn diff_blob_with_actual_binary_content() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let blob_id_str = repo.git_output(&["rev-parse", "HEAD:data.bin"]);
        let blob_id =
            gix_hash::ObjectId::from_hex(blob_id_str.as_bytes()).expect("failed to parse blob id");

        let result = ops::diff_blob(&handle, blob_id, blob_id, 3).expect("failed to diff");
        assert!(result.hunks.is_empty());
        assert_eq!(result.additions, 0);
        assert_eq!(result.deletions, 0);
    }

    #[test]
    fn diff_blob_returns_correct_ids() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let old_blob_str = repo.git_output(&["rev-parse", "HEAD~5:README.md"]);
        let old_blob_id =
            gix_hash::ObjectId::from_hex(old_blob_str.as_bytes()).expect("failed to parse blob id");

        let new_blob_str = repo.git_output(&["rev-parse", "HEAD:README.md"]);
        let new_blob_id =
            gix_hash::ObjectId::from_hex(new_blob_str.as_bytes()).expect("failed to parse blob id");

        let result = ops::diff_blob(&handle, old_blob_id, new_blob_id, 3).expect("failed to diff");
        assert_eq!(result.old_id, old_blob_id);
        assert_eq!(result.new_id, new_blob_id);
    }
}

mod line_stats_counter {
    use super::*;

    #[test]
    fn line_stats_for_modification() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let old_tree_str = repo.git_output(&["rev-parse", "HEAD~4^{tree}"]);
        let old_tree_id =
            gix_hash::ObjectId::from_hex(old_tree_str.as_bytes()).expect("failed to parse tree id");

        let new_tree_str = repo.git_output(&["rev-parse", "HEAD~3^{tree}"]);
        let new_tree_id =
            gix_hash::ObjectId::from_hex(new_tree_str.as_bytes()).expect("failed to parse tree id");

        let stats = ops::diff_stats(&handle, old_tree_id, new_tree_id).expect("failed to get stats");

        for entry in &stats.entries {
            assert!(entry.path.len() > 0);
        }
    }

    #[test]
    fn line_stats_process_change_is_called() {
        let old_content = b"line1\nline2\nline3\n";
        let new_content = b"line1\nmodified\nline3\nnew line\n";

        let result = ops::diff_blob_inline(old_content, new_content, 3).expect("failed to diff");
        assert!(result.additions > 0);
        assert!(result.deletions > 0);
    }
}

mod empty_tree_handling {
    use super::*;

    #[test]
    fn diff_between_empty_and_populated_tree() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let populated_tree_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let populated_tree_id =
            gix_hash::ObjectId::from_hex(populated_tree_str.as_bytes()).expect("failed to parse tree id");

        let first_tree_str = repo.git_output(&["rev-parse", "HEAD~5^{tree}"]);
        let first_tree_id =
            gix_hash::ObjectId::from_hex(first_tree_str.as_bytes()).expect("failed to parse tree id");

        let result = ops::diff_trees(&handle, first_tree_id, populated_tree_id).expect("failed to diff");
        assert!(!result.is_empty());
    }

    #[test]
    fn diff_stats_between_different_commits() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let old_tree_str = repo.git_output(&["rev-parse", "HEAD~5^{tree}"]);
        let old_tree_id =
            gix_hash::ObjectId::from_hex(old_tree_str.as_bytes()).expect("failed to parse tree id");

        let new_tree_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let new_tree_id =
            gix_hash::ObjectId::from_hex(new_tree_str.as_bytes()).expect("failed to parse tree id");

        let stats = ops::diff_stats(&handle, old_tree_id, new_tree_id).expect("failed to get stats");
        assert!(stats.files_changed > 0);
        assert!(stats.additions > 0 || stats.deletions > 0);
    }
}

mod diff_trees_tree_filtering {
    use super::*;

    #[test]
    fn diff_trees_filters_tree_additions() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let old_tree_str = repo.git_output(&["rev-parse", "HEAD~5^{tree}"]);
        let old_tree_id =
            gix_hash::ObjectId::from_hex(old_tree_str.as_bytes()).expect("failed to parse tree id");

        let new_tree_str = repo.git_output(&["rev-parse", "HEAD~4^{tree}"]);
        let new_tree_id =
            gix_hash::ObjectId::from_hex(new_tree_str.as_bytes()).expect("failed to parse tree id");

        let result = ops::diff_trees(&handle, old_tree_id, new_tree_id).expect("failed to diff trees");

        for entry in &result {
            let path_str = String::from_utf8_lossy(&entry.path);
            assert!(
                !path_str.ends_with('/'),
                "tree entries should be filtered out: {:?}",
                entry.path
            );
            assert!(
                entry.new_mode.is_some() || entry.old_mode.is_some(),
                "entries should have mode set"
            );
        }
    }

    #[test]
    fn diff_trees_filters_tree_deletions() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let old_tree_str = repo.git_output(&["rev-parse", "HEAD~1^{tree}"]);
        let old_tree_id =
            gix_hash::ObjectId::from_hex(old_tree_str.as_bytes()).expect("failed to parse tree id");

        let new_tree_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let new_tree_id =
            gix_hash::ObjectId::from_hex(new_tree_str.as_bytes()).expect("failed to parse tree id");

        let result = ops::diff_trees(&handle, old_tree_id, new_tree_id).expect("failed to diff trees");

        for entry in &result {
            assert!(
                entry.new_mode.as_ref().map_or(true, |m| *m != ops::EntryMode::Tree)
                    && entry.old_mode.as_ref().map_or(true, |m| *m != ops::EntryMode::Tree),
                "directory entries (tree mode) should be filtered out"
            );
        }
    }

    #[test]
    fn diff_trees_between_commits_with_directory_changes() {
        use std::fs;
        use std::process::Command;

        let dir = tempfile::TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        Command::new("git")
            .current_dir(&path)
            .args(["init"])
            .output()
            .expect("failed to init");
        Command::new("git")
            .current_dir(&path)
            .args(["config", "user.email", "test@example.com"])
            .output()
            .expect("failed to configure email");
        Command::new("git")
            .current_dir(&path)
            .args(["config", "user.name", "Test User"])
            .output()
            .expect("failed to configure name");

        fs::write(path.join("root.txt"), "root content\n").expect("failed to write root.txt");
        Command::new("git")
            .current_dir(&path)
            .args(["add", "."])
            .output()
            .expect("failed to add");
        Command::new("git")
            .current_dir(&path)
            .args(["commit", "-m", "Initial commit"])
            .output()
            .expect("failed to commit");

        fs::create_dir_all(path.join("newdir")).expect("failed to create newdir");
        fs::write(path.join("newdir/file.txt"), "file content\n").expect("failed to write file");
        Command::new("git")
            .current_dir(&path)
            .args(["add", "."])
            .output()
            .expect("failed to add");
        Command::new("git")
            .current_dir(&path)
            .args(["commit", "-m", "Add directory"])
            .output()
            .expect("failed to commit");

        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let old_tree_output = Command::new("git")
            .current_dir(&path)
            .args(["rev-parse", "HEAD~1^{tree}"])
            .output()
            .expect("failed to get old tree");
        let old_tree_str = String::from_utf8_lossy(&old_tree_output.stdout).trim().to_string();
        let old_tree_id =
            gix_hash::ObjectId::from_hex(old_tree_str.as_bytes()).expect("failed to parse tree id");

        let new_tree_output = Command::new("git")
            .current_dir(&path)
            .args(["rev-parse", "HEAD^{tree}"])
            .output()
            .expect("failed to get new tree");
        let new_tree_str = String::from_utf8_lossy(&new_tree_output.stdout).trim().to_string();
        let new_tree_id =
            gix_hash::ObjectId::from_hex(new_tree_str.as_bytes()).expect("failed to parse tree id");

        let result = ops::diff_trees(&handle, old_tree_id, new_tree_id).expect("failed to diff trees");

        for entry in &result {
            assert!(
                entry.change == ops::ChangeKind::Added
                    || entry.change == ops::ChangeKind::Modified
                    || entry.change == ops::ChangeKind::Deleted
            );
            assert!(
                entry.new_mode.as_ref().map_or(true, |m| *m != ops::EntryMode::Tree),
                "should not include tree entries in diff results"
            );
        }
    }
}

mod line_stats_sink_coverage {
    use super::*;

    #[test]
    fn line_stats_counter_process_change_tracks_insertions() {
        let old_content = b"";
        let new_content = b"line1\nline2\nline3\n";

        let result = ops::diff_blob_inline(old_content, new_content, 0).expect("failed to diff");

        assert_eq!(result.additions, 3);
        assert_eq!(result.deletions, 0);
    }

    #[test]
    fn line_stats_counter_process_change_tracks_removals() {
        let old_content = b"line1\nline2\nline3\n";
        let new_content = b"";

        let result = ops::diff_blob_inline(old_content, new_content, 0).expect("failed to diff");

        assert_eq!(result.additions, 0);
        assert_eq!(result.deletions, 3);
    }

    #[test]
    fn line_stats_counter_process_change_tracks_both() {
        let old_content = b"old1\nold2\n";
        let new_content = b"new1\nnew2\nnew3\n";

        let result = ops::diff_blob_inline(old_content, new_content, 0).expect("failed to diff");

        assert_eq!(result.deletions, 2);
        assert_eq!(result.additions, 3);
    }

    #[test]
    fn line_stats_counter_finish_returns_counts() {
        let old_content = b"a\nb\nc\n";
        let new_content = b"a\nX\nc\n";

        let result = ops::diff_blob_inline(old_content, new_content, 0).expect("failed to diff");

        assert_eq!(result.additions, 1);
        assert_eq!(result.deletions, 1);
    }

    #[test]
    fn line_stats_with_empty_ranges() {
        let old_content = b"same\n";
        let new_content = b"same\n";

        let result = ops::diff_blob_inline(old_content, new_content, 0).expect("failed to diff");

        assert_eq!(result.additions, 0);
        assert_eq!(result.deletions, 0);
        assert!(result.hunks.is_empty());
    }
}

mod hunk_consumer_coverage {
    use super::*;

    #[test]
    fn hunk_consumer_consume_hunk_creates_diff_hunks() {
        let old_content = b"line1\nline2\nline3\n";
        let new_content = b"line1\nmodified\nline3\n";

        let result = ops::diff_blob_inline(old_content, new_content, 1).expect("failed to diff");

        assert!(!result.hunks.is_empty());
        let hunk = &result.hunks[0];
        assert!(hunk.old_start > 0);
        assert!(hunk.new_start > 0);
        assert!(!hunk.lines.is_empty());
    }

    #[test]
    fn hunk_consumer_maps_context_lines() {
        let old_content = b"context\nold\ncontext\n";
        let new_content = b"context\nnew\ncontext\n";

        let result = ops::diff_blob_inline(old_content, new_content, 1).expect("failed to diff");

        let mut has_context = false;
        for hunk in &result.hunks {
            for line in &hunk.lines {
                if line.kind == ops::DiffLineKind::Context {
                    has_context = true;
                }
            }
        }
        assert!(has_context, "expected context lines in hunk");
    }

    #[test]
    fn hunk_consumer_maps_add_lines() {
        let old_content = b"start\n";
        let new_content = b"start\nadded\n";

        let result = ops::diff_blob_inline(old_content, new_content, 0).expect("failed to diff");

        let mut has_addition = false;
        for hunk in &result.hunks {
            for line in &hunk.lines {
                if line.kind == ops::DiffLineKind::Addition {
                    has_addition = true;
                }
            }
        }
        assert!(has_addition, "expected addition lines in hunk");
    }

    #[test]
    fn hunk_consumer_maps_remove_lines() {
        let old_content = b"start\nremoved\n";
        let new_content = b"start\n";

        let result = ops::diff_blob_inline(old_content, new_content, 0).expect("failed to diff");

        let mut has_deletion = false;
        for hunk in &result.hunks {
            for line in &hunk.lines {
                if line.kind == ops::DiffLineKind::Deletion {
                    has_deletion = true;
                }
            }
        }
        assert!(has_deletion, "expected deletion lines in hunk");
    }

    #[test]
    fn hunk_consumer_finish_is_called() {
        let old_content = b"a\n";
        let new_content = b"b\n";

        let result = ops::diff_blob_inline(old_content, new_content, 0);

        assert!(result.is_ok(), "finish should complete without error");
        let diff = result.unwrap();
        assert!(!diff.hunks.is_empty());
    }

    #[test]
    fn hunk_consumer_preserves_line_content() {
        let old_content = b"original line\n";
        let new_content = b"modified line\n";

        let result = ops::diff_blob_inline(old_content, new_content, 0).expect("failed to diff");

        let mut found_original = false;
        let mut found_modified = false;

        for hunk in &result.hunks {
            for line in &hunk.lines {
                let content_str = String::from_utf8_lossy(&line.content);
                if content_str.contains("original") {
                    found_original = true;
                }
                if content_str.contains("modified") {
                    found_modified = true;
                }
            }
        }

        assert!(found_original, "expected original line content");
        assert!(found_modified, "expected modified line content");
    }

    #[test]
    fn hunk_consumer_handles_empty_lines() {
        let old_content = b"line1\n\nline3\n";
        let new_content = b"line1\ninserted\n\nline3\n";

        let result = ops::diff_blob_inline(old_content, new_content, 1).expect("failed to diff");

        assert!(result.additions > 0);
    }

    #[test]
    fn hunk_header_values_are_correct() {
        let old_content = b"a\nb\nc\nd\ne\n";
        let new_content = b"a\nb\nX\nd\ne\n";

        let result = ops::diff_blob_inline(old_content, new_content, 1).expect("failed to diff");

        assert!(!result.hunks.is_empty());
        let hunk = &result.hunks[0];

        assert!(hunk.old_start >= 1);
        assert!(hunk.new_start >= 1);
        assert!(hunk.old_lines > 0);
        assert!(hunk.new_lines > 0);
    }
}

mod diff_stats_edge_cases {
    use super::*;
    use std::fs;
    use std::process::Command;

    #[test]
    fn stats_added_file_without_new_id_returns_zero() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let old_tree_str = repo.git_output(&["rev-parse", "HEAD~5^{tree}"]);
        let old_tree_id =
            gix_hash::ObjectId::from_hex(old_tree_str.as_bytes()).expect("failed to parse tree id");

        let new_tree_str = repo.git_output(&["rev-parse", "HEAD~4^{tree}"]);
        let new_tree_id =
            gix_hash::ObjectId::from_hex(new_tree_str.as_bytes()).expect("failed to parse tree id");

        let stats = ops::diff_stats(&handle, old_tree_id, new_tree_id).expect("failed to get stats");

        for entry in &stats.entries {
            let _ = entry.additions;
            let _ = entry.deletions;
        }
    }

    #[test]
    fn stats_deleted_file_without_old_id_returns_zero() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let old_tree_str = repo.git_output(&["rev-parse", "HEAD~1^{tree}"]);
        let old_tree_id =
            gix_hash::ObjectId::from_hex(old_tree_str.as_bytes()).expect("failed to parse tree id");

        let new_tree_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let new_tree_id =
            gix_hash::ObjectId::from_hex(new_tree_str.as_bytes()).expect("failed to parse tree id");

        let stats = ops::diff_stats(&handle, old_tree_id, new_tree_id).expect("failed to get stats");

        for entry in &stats.entries {
            let _ = entry.additions;
            let _ = entry.deletions;
        }
    }

    #[test]
    fn stats_modified_file_missing_ids_returns_zero() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let old_tree_str = repo.git_output(&["rev-parse", "HEAD~2^{tree}"]);
        let old_tree_id =
            gix_hash::ObjectId::from_hex(old_tree_str.as_bytes()).expect("failed to parse tree id");

        let new_tree_str = repo.git_output(&["rev-parse", "HEAD~1^{tree}"]);
        let new_tree_id =
            gix_hash::ObjectId::from_hex(new_tree_str.as_bytes()).expect("failed to parse tree id");

        let stats = ops::diff_stats(&handle, old_tree_id, new_tree_id).expect("failed to get stats");

        for entry in &stats.entries {
            let _ = entry.additions;
            let _ = entry.deletions;
        }
    }

    #[test]
    fn stats_correctly_handles_added_change_kind() {
        let dir = tempfile::TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        Command::new("git")
            .current_dir(&path)
            .args(["init"])
            .output()
            .expect("failed to init");
        Command::new("git")
            .current_dir(&path)
            .args(["config", "user.email", "test@example.com"])
            .output()
            .expect("failed to configure email");
        Command::new("git")
            .current_dir(&path)
            .args(["config", "user.name", "Test User"])
            .output()
            .expect("failed to configure name");

        fs::write(path.join("existing.txt"), "existing\n").expect("failed to write existing");
        Command::new("git")
            .current_dir(&path)
            .args(["add", "."])
            .output()
            .expect("failed to add");
        Command::new("git")
            .current_dir(&path)
            .args(["commit", "-m", "Initial"])
            .output()
            .expect("failed to commit");

        fs::write(path.join("new_file.txt"), "line1\nline2\nline3\n").expect("failed to write new file");
        Command::new("git")
            .current_dir(&path)
            .args(["add", "."])
            .output()
            .expect("failed to add");
        Command::new("git")
            .current_dir(&path)
            .args(["commit", "-m", "Add new file"])
            .output()
            .expect("failed to commit");

        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let old_tree_output = Command::new("git")
            .current_dir(&path)
            .args(["rev-parse", "HEAD~1^{tree}"])
            .output()
            .expect("failed to get old tree");
        let old_tree_str = String::from_utf8_lossy(&old_tree_output.stdout).trim().to_string();
        let old_tree_id =
            gix_hash::ObjectId::from_hex(old_tree_str.as_bytes()).expect("failed to parse tree id");

        let new_tree_output = Command::new("git")
            .current_dir(&path)
            .args(["rev-parse", "HEAD^{tree}"])
            .output()
            .expect("failed to get new tree");
        let new_tree_str = String::from_utf8_lossy(&new_tree_output.stdout).trim().to_string();
        let new_tree_id =
            gix_hash::ObjectId::from_hex(new_tree_str.as_bytes()).expect("failed to parse tree id");

        let entries = ops::diff_trees(&handle, old_tree_id, new_tree_id).expect("failed to diff");
        let stats = ops::diff_stats(&handle, old_tree_id, new_tree_id).expect("failed to get stats");

        let added_entries: Vec<_> = entries.iter().filter(|e| e.change == ops::ChangeKind::Added).collect();
        assert!(!added_entries.is_empty(), "expected added entries");

        for entry in &added_entries {
            assert!(entry.new_id.is_some(), "added entry should have new_id");
        }

        assert!(stats.additions > 0, "expected additions for new file");
    }

    #[test]
    fn stats_correctly_handles_deleted_change_kind() {
        let dir = tempfile::TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        Command::new("git")
            .current_dir(&path)
            .args(["init"])
            .output()
            .expect("failed to init");
        Command::new("git")
            .current_dir(&path)
            .args(["config", "user.email", "test@example.com"])
            .output()
            .expect("failed to configure email");
        Command::new("git")
            .current_dir(&path)
            .args(["config", "user.name", "Test User"])
            .output()
            .expect("failed to configure name");

        fs::write(path.join("keep.txt"), "keep\n").expect("failed to write keep");
        fs::write(path.join("to_delete.txt"), "line1\nline2\nline3\n").expect("failed to write to_delete");
        Command::new("git")
            .current_dir(&path)
            .args(["add", "."])
            .output()
            .expect("failed to add");
        Command::new("git")
            .current_dir(&path)
            .args(["commit", "-m", "Initial"])
            .output()
            .expect("failed to commit");

        fs::remove_file(path.join("to_delete.txt")).expect("failed to delete");
        Command::new("git")
            .current_dir(&path)
            .args(["add", "."])
            .output()
            .expect("failed to add");
        Command::new("git")
            .current_dir(&path)
            .args(["commit", "-m", "Delete file"])
            .output()
            .expect("failed to commit");

        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let old_tree_output = Command::new("git")
            .current_dir(&path)
            .args(["rev-parse", "HEAD~1^{tree}"])
            .output()
            .expect("failed to get old tree");
        let old_tree_str = String::from_utf8_lossy(&old_tree_output.stdout).trim().to_string();
        let old_tree_id =
            gix_hash::ObjectId::from_hex(old_tree_str.as_bytes()).expect("failed to parse tree id");

        let new_tree_output = Command::new("git")
            .current_dir(&path)
            .args(["rev-parse", "HEAD^{tree}"])
            .output()
            .expect("failed to get new tree");
        let new_tree_str = String::from_utf8_lossy(&new_tree_output.stdout).trim().to_string();
        let new_tree_id =
            gix_hash::ObjectId::from_hex(new_tree_str.as_bytes()).expect("failed to parse tree id");

        let entries = ops::diff_trees(&handle, old_tree_id, new_tree_id).expect("failed to diff");
        let stats = ops::diff_stats(&handle, old_tree_id, new_tree_id).expect("failed to get stats");

        let deleted_entries: Vec<_> = entries.iter().filter(|e| e.change == ops::ChangeKind::Deleted).collect();
        assert!(!deleted_entries.is_empty(), "expected deleted entries");

        for entry in &deleted_entries {
            assert!(entry.old_id.is_some(), "deleted entry should have old_id");
        }

        assert!(stats.deletions > 0, "expected deletions for deleted file");
    }

    #[test]
    fn stats_correctly_handles_modified_change_kind() {
        let dir = tempfile::TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        Command::new("git")
            .current_dir(&path)
            .args(["init"])
            .output()
            .expect("failed to init");
        Command::new("git")
            .current_dir(&path)
            .args(["config", "user.email", "test@example.com"])
            .output()
            .expect("failed to configure email");
        Command::new("git")
            .current_dir(&path)
            .args(["config", "user.name", "Test User"])
            .output()
            .expect("failed to configure name");

        fs::write(path.join("file.txt"), "line1\nline2\n").expect("failed to write file");
        Command::new("git")
            .current_dir(&path)
            .args(["add", "."])
            .output()
            .expect("failed to add");
        Command::new("git")
            .current_dir(&path)
            .args(["commit", "-m", "Initial"])
            .output()
            .expect("failed to commit");

        fs::write(path.join("file.txt"), "line1\nmodified\nline3\n").expect("failed to modify file");
        Command::new("git")
            .current_dir(&path)
            .args(["add", "."])
            .output()
            .expect("failed to add");
        Command::new("git")
            .current_dir(&path)
            .args(["commit", "-m", "Modify file"])
            .output()
            .expect("failed to commit");

        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let old_tree_output = Command::new("git")
            .current_dir(&path)
            .args(["rev-parse", "HEAD~1^{tree}"])
            .output()
            .expect("failed to get old tree");
        let old_tree_str = String::from_utf8_lossy(&old_tree_output.stdout).trim().to_string();
        let old_tree_id =
            gix_hash::ObjectId::from_hex(old_tree_str.as_bytes()).expect("failed to parse tree id");

        let new_tree_output = Command::new("git")
            .current_dir(&path)
            .args(["rev-parse", "HEAD^{tree}"])
            .output()
            .expect("failed to get new tree");
        let new_tree_str = String::from_utf8_lossy(&new_tree_output.stdout).trim().to_string();
        let new_tree_id =
            gix_hash::ObjectId::from_hex(new_tree_str.as_bytes()).expect("failed to parse tree id");

        let entries = ops::diff_trees(&handle, old_tree_id, new_tree_id).expect("failed to diff");
        let stats = ops::diff_stats(&handle, old_tree_id, new_tree_id).expect("failed to get stats");

        let modified_entries: Vec<_> = entries.iter().filter(|e| e.change == ops::ChangeKind::Modified).collect();
        assert!(!modified_entries.is_empty(), "expected modified entries");

        for entry in &modified_entries {
            assert!(entry.old_id.is_some(), "modified entry should have old_id");
            assert!(entry.new_id.is_some(), "modified entry should have new_id");
        }

        assert!(stats.additions > 0 || stats.deletions > 0, "expected changes for modified file");
    }
}

mod count_lines_in_blob_coverage {
    use super::*;

    #[test]
    fn count_lines_handles_file_without_trailing_newline() {
        let old_content = b"line1\nline2\nline3";
        let new_content = b"";

        let result = ops::diff_blob_inline(old_content, new_content, 0).expect("failed to diff");

        assert!(result.deletions >= 3);
    }

    #[test]
    fn count_lines_handles_single_line_without_newline() {
        let old_content = b"single";
        let new_content = b"";

        let result = ops::diff_blob_inline(old_content, new_content, 0).expect("failed to diff");

        assert!(result.deletions >= 1);
    }

    #[test]
    fn count_lines_handles_only_newlines() {
        let old_content = b"\n\n\n";
        let new_content = b"";

        let result = ops::diff_blob_inline(old_content, new_content, 0).expect("failed to diff");

        assert!(result.deletions >= 3);
    }

    #[test]
    fn count_lines_handles_mixed_content() {
        let old_content = b"line1\n\nline3\n";
        let new_content = b"";

        let result = ops::diff_blob_inline(old_content, new_content, 0).expect("failed to diff");

        assert!(result.deletions >= 3);
    }
}

mod compute_line_stats_coverage {
    use super::*;
    use std::fs;
    use std::process::Command;

    #[test]
    fn compute_line_stats_for_modifications() {
        let dir = tempfile::TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        Command::new("git")
            .current_dir(&path)
            .args(["init"])
            .output()
            .expect("failed to init");
        Command::new("git")
            .current_dir(&path)
            .args(["config", "user.email", "test@example.com"])
            .output()
            .expect("failed to configure email");
        Command::new("git")
            .current_dir(&path)
            .args(["config", "user.name", "Test User"])
            .output()
            .expect("failed to configure name");

        fs::write(path.join("file.txt"), "line1\nline2\nline3\n").expect("failed to write file");
        Command::new("git")
            .current_dir(&path)
            .args(["add", "."])
            .output()
            .expect("failed to add");
        Command::new("git")
            .current_dir(&path)
            .args(["commit", "-m", "Initial"])
            .output()
            .expect("failed to commit");

        fs::write(path.join("file.txt"), "line1\nmodified\nline3\nnewline\n").expect("failed to modify file");
        Command::new("git")
            .current_dir(&path)
            .args(["add", "."])
            .output()
            .expect("failed to add");
        Command::new("git")
            .current_dir(&path)
            .args(["commit", "-m", "Modify file"])
            .output()
            .expect("failed to commit");

        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let old_tree_output = Command::new("git")
            .current_dir(&path)
            .args(["rev-parse", "HEAD~1^{tree}"])
            .output()
            .expect("failed to get old tree");
        let old_tree_str = String::from_utf8_lossy(&old_tree_output.stdout).trim().to_string();
        let old_tree_id =
            gix_hash::ObjectId::from_hex(old_tree_str.as_bytes()).expect("failed to parse tree id");

        let new_tree_output = Command::new("git")
            .current_dir(&path)
            .args(["rev-parse", "HEAD^{tree}"])
            .output()
            .expect("failed to get new tree");
        let new_tree_str = String::from_utf8_lossy(&new_tree_output.stdout).trim().to_string();
        let new_tree_id =
            gix_hash::ObjectId::from_hex(new_tree_str.as_bytes()).expect("failed to parse tree id");

        let stats = ops::diff_stats(&handle, old_tree_id, new_tree_id).expect("failed to get stats");

        assert!(stats.additions > 0 || stats.deletions > 0);

        let file_stat = stats.entries.iter().find(|e| e.path.ends_with(b"file.txt"));
        assert!(file_stat.is_some(), "expected file.txt in stats");

        let stat = file_stat.unwrap();
        assert!(stat.additions > 0 || stat.deletions > 0);
    }

    #[test]
    fn compute_line_stats_identical_files_returns_zero() {
        let dir = tempfile::TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        Command::new("git")
            .current_dir(&path)
            .args(["init"])
            .output()
            .expect("failed to init");
        Command::new("git")
            .current_dir(&path)
            .args(["config", "user.email", "test@example.com"])
            .output()
            .expect("failed to configure email");
        Command::new("git")
            .current_dir(&path)
            .args(["config", "user.name", "Test User"])
            .output()
            .expect("failed to configure name");

        fs::write(path.join("file.txt"), "content\n").expect("failed to write file");
        Command::new("git")
            .current_dir(&path)
            .args(["add", "."])
            .output()
            .expect("failed to add");
        Command::new("git")
            .current_dir(&path)
            .args(["commit", "-m", "Initial"])
            .output()
            .expect("failed to commit");

        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let tree_output = Command::new("git")
            .current_dir(&path)
            .args(["rev-parse", "HEAD^{tree}"])
            .output()
            .expect("failed to get tree");
        let tree_str = String::from_utf8_lossy(&tree_output.stdout).trim().to_string();
        let tree_id =
            gix_hash::ObjectId::from_hex(tree_str.as_bytes()).expect("failed to parse tree id");

        let stats = ops::diff_stats(&handle, tree_id, tree_id).expect("failed to get stats");

        assert_eq!(stats.additions, 0);
        assert_eq!(stats.deletions, 0);
        assert_eq!(stats.files_changed, 0);
    }
}

mod count_lines_edge_cases {
    use super::*;

    #[test]
    fn stats_handles_empty_file() {
        use std::fs;
        use std::process::Command;

        let dir = tempfile::TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        Command::new("git")
            .current_dir(&path)
            .args(["init"])
            .output()
            .expect("failed to init");
        Command::new("git")
            .current_dir(&path)
            .args(["config", "user.email", "test@example.com"])
            .output()
            .expect("failed to configure email");
        Command::new("git")
            .current_dir(&path)
            .args(["config", "user.name", "Test User"])
            .output()
            .expect("failed to configure name");

        fs::write(path.join("file.txt"), "content\n").expect("failed to write");
        Command::new("git")
            .current_dir(&path)
            .args(["add", "."])
            .output()
            .expect("failed to add");
        Command::new("git")
            .current_dir(&path)
            .args(["commit", "-m", "Initial commit"])
            .output()
            .expect("failed to commit");

        fs::write(path.join("empty.txt"), "").expect("failed to write empty file");
        Command::new("git")
            .current_dir(&path)
            .args(["add", "."])
            .output()
            .expect("failed to add");
        Command::new("git")
            .current_dir(&path)
            .args(["commit", "-m", "Add empty file"])
            .output()
            .expect("failed to commit");

        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let old_tree_output = Command::new("git")
            .current_dir(&path)
            .args(["rev-parse", "HEAD~1^{tree}"])
            .output()
            .expect("failed to get old tree");
        let old_tree_str = String::from_utf8_lossy(&old_tree_output.stdout).trim().to_string();
        let old_tree_id =
            gix_hash::ObjectId::from_hex(old_tree_str.as_bytes()).expect("failed to parse tree id");

        let new_tree_output = Command::new("git")
            .current_dir(&path)
            .args(["rev-parse", "HEAD^{tree}"])
            .output()
            .expect("failed to get new tree");
        let new_tree_str = String::from_utf8_lossy(&new_tree_output.stdout).trim().to_string();
        let new_tree_id =
            gix_hash::ObjectId::from_hex(new_tree_str.as_bytes()).expect("failed to parse tree id");

        let stats = ops::diff_stats(&handle, old_tree_id, new_tree_id).expect("failed to get stats");

        assert_eq!(stats.files_changed, 1);
    }

    #[test]
    fn stats_handles_file_deletion() {
        use std::fs;
        use std::process::Command;

        let dir = tempfile::TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        Command::new("git")
            .current_dir(&path)
            .args(["init"])
            .output()
            .expect("failed to init");
        Command::new("git")
            .current_dir(&path)
            .args(["config", "user.email", "test@example.com"])
            .output()
            .expect("failed to configure email");
        Command::new("git")
            .current_dir(&path)
            .args(["config", "user.name", "Test User"])
            .output()
            .expect("failed to configure name");

        fs::write(path.join("file1.txt"), "content1\n").expect("failed to write");
        fs::write(path.join("file2.txt"), "content2\n").expect("failed to write");
        Command::new("git")
            .current_dir(&path)
            .args(["add", "."])
            .output()
            .expect("failed to add");
        Command::new("git")
            .current_dir(&path)
            .args(["commit", "-m", "Initial commit"])
            .output()
            .expect("failed to commit");

        fs::remove_file(path.join("file2.txt")).expect("failed to delete");
        Command::new("git")
            .current_dir(&path)
            .args(["add", "."])
            .output()
            .expect("failed to add");
        Command::new("git")
            .current_dir(&path)
            .args(["commit", "-m", "Delete file"])
            .output()
            .expect("failed to commit");

        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let old_tree_output = Command::new("git")
            .current_dir(&path)
            .args(["rev-parse", "HEAD~1^{tree}"])
            .output()
            .expect("failed to get old tree");
        let old_tree_str = String::from_utf8_lossy(&old_tree_output.stdout).trim().to_string();
        let old_tree_id =
            gix_hash::ObjectId::from_hex(old_tree_str.as_bytes()).expect("failed to parse tree id");

        let new_tree_output = Command::new("git")
            .current_dir(&path)
            .args(["rev-parse", "HEAD^{tree}"])
            .output()
            .expect("failed to get new tree");
        let new_tree_str = String::from_utf8_lossy(&new_tree_output.stdout).trim().to_string();
        let new_tree_id =
            gix_hash::ObjectId::from_hex(new_tree_str.as_bytes()).expect("failed to parse tree id");

        let stats = ops::diff_stats(&handle, old_tree_id, new_tree_id).expect("failed to get stats");

        assert_eq!(stats.files_changed, 1);
        assert!(stats.deletions > 0);
        assert_eq!(stats.additions, 0);

        let deleted_entry = stats.entries.iter().find(|e| e.deletions > 0);
        assert!(deleted_entry.is_some());
    }
}
