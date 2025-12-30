mod fixtures;

use fixtures::TestRepo;
use gix_server_sdk::{ops, RepoPool, SdkConfig};
use gix_server_sdk::ops::BlameOptions;

fn create_pool() -> RepoPool {
    RepoPool::new(SdkConfig::default())
}

mod blame_file {
    use super::*;

    #[test]
    fn blame_single_author_file() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let result = ops::blame_file(
            &handle,
            commit_id,
            b"README.md".as_slice().into(),
            BlameOptions::default(),
        )
        .expect("failed to blame file");

        assert!(!result.entries.is_empty());
        assert!(!result.lines.is_empty());

        for entry in &result.entries {
            assert_eq!(entry.commit_id, commit_id);
            assert!(entry.line_count > 0);
            assert!(entry.start_line >= 1);
            assert!(entry.original_start_line >= 1);
        }
    }

    #[test]
    fn blame_file_with_multiple_authors() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let result = ops::blame_file(
            &handle,
            commit_id,
            b"src/lib.rs".as_slice().into(),
            BlameOptions::default(),
        )
        .expect("failed to blame file");

        assert!(!result.entries.is_empty());
        assert!(!result.lines.is_empty());

        let unique_commits: std::collections::HashSet<_> = result.entries.iter()
            .map(|e| e.commit_id)
            .collect();

        assert!(unique_commits.len() >= 1, "expected at least 1 commit in blame");

        let total_lines: u32 = result.entries.iter().map(|e| e.line_count).sum();
        assert_eq!(total_lines as usize, result.lines.len());
    }

    #[test]
    fn blame_main_rs_with_history() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let result = ops::blame_file(
            &handle,
            commit_id,
            b"src/main.rs".as_slice().into(),
            BlameOptions::default(),
        )
        .expect("failed to blame file");

        assert!(!result.entries.is_empty());
        assert!(!result.lines.is_empty());

        assert!(result.statistics.commits_traversed > 0);
    }

    #[test]
    fn blame_readme_with_history() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let result = ops::blame_file(
            &handle,
            commit_id,
            b"README.md".as_slice().into(),
            BlameOptions::default(),
        )
        .expect("failed to blame file");

        assert!(!result.entries.is_empty());

        let content = result.lines.iter()
            .map(|l| String::from_utf8_lossy(l).to_string())
            .collect::<Vec<_>>()
            .join("");
        assert!(content.contains("# Project"));
    }

    #[test]
    fn blame_file_at_earlier_commit() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD~2"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let result = ops::blame_file(
            &handle,
            commit_id,
            b"src/lib.rs".as_slice().into(),
            BlameOptions::default(),
        )
        .expect("failed to blame file");

        assert!(!result.entries.is_empty());
    }

    #[test]
    fn blame_nonexistent_file_returns_error() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let result = ops::blame_file(
            &handle,
            commit_id,
            b"nonexistent.txt".as_slice().into(),
            BlameOptions::default(),
        );

        assert!(result.is_err());
    }

    #[test]
    fn blame_with_invalid_commit_returns_error() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let fake_id = gix_hash::ObjectId::from_hex(b"0000000000000000000000000000000000000000")
            .expect("valid hex");

        let result = ops::blame_file(
            &handle,
            fake_id,
            b"README.md".as_slice().into(),
            BlameOptions::default(),
        );

        assert!(result.is_err());
    }

    #[test]
    fn blame_file_in_subdirectory() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let result = ops::blame_file(
            &handle,
            commit_id,
            b"src/main.rs".as_slice().into(),
            BlameOptions::default(),
        )
        .expect("failed to blame file");

        assert!(!result.entries.is_empty());
        assert!(!result.lines.is_empty());
    }

    #[test]
    fn blame_entries_cover_all_lines() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let result = ops::blame_file(
            &handle,
            commit_id,
            b"src/lib.rs".as_slice().into(),
            BlameOptions::default(),
        )
        .expect("failed to blame file");

        let total_lines: u32 = result.entries.iter().map(|e| e.line_count).sum();
        assert_eq!(total_lines as usize, result.lines.len());

        let mut expected_line = 1u32;
        for entry in &result.entries {
            assert_eq!(entry.start_line, expected_line, "entries should be contiguous");
            expected_line += entry.line_count;
        }
    }

    #[test]
    fn blame_statistics_populated() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let result = ops::blame_file(
            &handle,
            commit_id,
            b"src/lib.rs".as_slice().into(),
            BlameOptions::default(),
        )
        .expect("failed to blame file");

        assert!(result.statistics.commits_traversed > 0);
    }
}

mod blame_options {
    use super::*;

    #[test]
    fn default_options() {
        let options = BlameOptions::default();
        assert!(options.range.is_none());
        assert!(options.follow_renames);
    }

    #[test]
    fn custom_options_with_range() {
        let options = BlameOptions {
            range: Some((1, 5)),
            follow_renames: true,
        };
        assert_eq!(options.range, Some((1, 5)));
        assert!(options.follow_renames);
    }

    #[test]
    fn custom_options_without_follow_renames() {
        let options = BlameOptions {
            range: None,
            follow_renames: false,
        };
        assert!(options.range.is_none());
        assert!(!options.follow_renames);
    }

    #[test]
    fn blame_with_line_range() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let options = BlameOptions {
            range: Some((1, 3)),
            follow_renames: true,
        };

        let result = ops::blame_file(
            &handle,
            commit_id,
            b"src/lib.rs".as_slice().into(),
            options,
        )
        .expect("failed to blame file with range");

        assert!(!result.entries.is_empty());

        let total_lines: u32 = result.entries.iter().map(|e| e.line_count).sum();
        assert!(total_lines <= 3, "line range should limit blame output");
    }

    #[test]
    fn blame_with_single_line_range() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let options = BlameOptions {
            range: Some((1, 1)),
            follow_renames: true,
        };

        let result = ops::blame_file(
            &handle,
            commit_id,
            b"README.md".as_slice().into(),
            options,
        )
        .expect("failed to blame single line");

        assert!(!result.entries.is_empty());
        let total_lines: u32 = result.entries.iter().map(|e| e.line_count).sum();
        assert_eq!(total_lines, 1);
    }

    #[test]
    fn blame_without_follow_renames() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let options = BlameOptions {
            range: None,
            follow_renames: false,
        };

        let result = ops::blame_file(
            &handle,
            commit_id,
            b"src/lib.rs".as_slice().into(),
            options,
        )
        .expect("failed to blame without follow renames");

        assert!(!result.entries.is_empty());
    }

    #[test]
    fn blame_with_range_middle_of_file() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let options = BlameOptions {
            range: Some((2, 4)),
            follow_renames: true,
        };

        let result = ops::blame_file(
            &handle,
            commit_id,
            b"src/lib.rs".as_slice().into(),
            options,
        )
        .expect("failed to blame middle lines");

        assert!(!result.entries.is_empty());
        let total_lines: u32 = result.entries.iter().map(|e| e.line_count).sum();
        assert!(total_lines <= 3);
    }

    #[test]
    fn compare_with_and_without_renames() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let options_with_renames = BlameOptions {
            range: None,
            follow_renames: true,
        };

        let options_without_renames = BlameOptions {
            range: None,
            follow_renames: false,
        };

        let result_with = ops::blame_file(
            &handle,
            commit_id,
            b"src/lib.rs".as_slice().into(),
            options_with_renames,
        )
        .expect("failed to blame with renames");

        let result_without = ops::blame_file(
            &handle,
            commit_id,
            b"src/lib.rs".as_slice().into(),
            options_without_renames,
        )
        .expect("failed to blame without renames");

        assert_eq!(result_with.lines.len(), result_without.lines.len());
    }
}

mod blame_error_paths {
    use super::*;

    #[test]
    fn blame_with_invalid_zero_based_range_returns_error() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let options = BlameOptions {
            range: Some((0, 5)),
            follow_renames: true,
        };

        let result = ops::blame_file(
            &handle,
            commit_id,
            b"README.md".as_slice().into(),
            options,
        );

        assert!(result.is_err());
    }
}

mod blame_without_index {
    use super::*;

    #[test]
    fn blame_file_without_index() {
        let repo = TestRepo::without_index();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let result = ops::blame_file(
            &handle,
            commit_id,
            b"README.md".as_slice().into(),
            BlameOptions::default(),
        )
        .expect("blame should work without index");

        assert!(!result.entries.is_empty());
        assert!(!result.lines.is_empty());
    }

    #[test]
    fn blame_file_in_bare_repo() {
        let repo = TestRepo::bare();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let result = ops::blame_file(
            &handle,
            commit_id,
            b"README.md".as_slice().into(),
            BlameOptions::default(),
        )
        .expect("blame should work in bare repo");

        assert!(!result.entries.is_empty());
        assert!(!result.lines.is_empty());
    }
}

mod blame_result_structure {
    use super::*;

    #[test]
    fn entries_have_valid_line_numbers() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let result = ops::blame_file(
            &handle,
            commit_id,
            b"README.md".as_slice().into(),
            BlameOptions::default(),
        )
        .expect("failed to blame file");

        for entry in &result.entries {
            assert!(entry.start_line >= 1, "start_line should be 1-based");
            assert!(entry.original_start_line >= 1, "original_start_line should be 1-based");
            assert!(entry.line_count > 0, "line_count should be positive");
        }
    }

    #[test]
    fn lines_content_is_readable() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let result = ops::blame_file(
            &handle,
            commit_id,
            b"README.md".as_slice().into(),
            BlameOptions::default(),
        )
        .expect("failed to blame file");

        assert!(!result.lines.is_empty());

        let first_line = String::from_utf8_lossy(&result.lines[0]);
        assert!(first_line.contains("# Test Repository") || first_line.starts_with("#"));
    }

    #[test]
    fn commit_ids_are_valid() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let result = ops::blame_file(
            &handle,
            commit_id,
            b"README.md".as_slice().into(),
            BlameOptions::default(),
        )
        .expect("failed to blame file");

        for entry in &result.entries {
            assert!(!entry.commit_id.is_null(), "commit_id should not be null");
        }
    }
}
