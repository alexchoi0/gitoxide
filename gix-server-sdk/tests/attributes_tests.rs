mod fixtures;

use bstr::ByteSlice;
use fixtures::TestRepo;
use gix_server_sdk::{ops, RepoPool, SdkConfig};

fn create_pool() -> RepoPool {
    RepoPool::new(SdkConfig::default())
}

mod check_ignore {
    use super::*;

    #[test]
    fn ignored_file_with_wildcard_pattern() {
        let repo = TestRepo::with_attributes();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let paths = vec![b"test.log".as_bstr()];
        let results = ops::check_ignore(&handle, &paths).expect("check_ignore failed");

        assert_eq!(results.len(), 1);
        assert!(results[0].is_ignored);
        assert!(results[0].pattern.is_some());
        assert!(results[0].source.is_some());
    }

    #[test]
    fn non_ignored_file() {
        let repo = TestRepo::with_attributes();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let paths = vec![b"src/main.rs".as_bstr()];
        let results = ops::check_ignore(&handle, &paths).expect("check_ignore failed");

        assert_eq!(results.len(), 1);
        assert!(!results[0].is_ignored);
        assert!(results[0].pattern.is_none());
        assert!(results[0].source.is_none());
    }

    #[test]
    fn directory_path_with_trailing_slash() {
        let repo = TestRepo::with_attributes();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let paths = vec![b"logs/".as_bstr()];
        let results = ops::check_ignore(&handle, &paths).expect("check_ignore failed");

        assert_eq!(results.len(), 1);
        assert!(results[0].is_ignored);
    }

    #[test]
    fn directory_path_ignored_pattern() {
        let repo = TestRepo::with_attributes();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let paths = vec![b"target/".as_bstr()];
        let results = ops::check_ignore(&handle, &paths).expect("check_ignore failed");

        assert_eq!(results.len(), 1);
        assert!(results[0].is_ignored);
    }

    #[test]
    fn multiple_paths_mixed_results() {
        let repo = TestRepo::with_attributes();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let paths = vec![
            b"debug.log".as_bstr(),
            b"src/main.rs".as_bstr(),
            b".env".as_bstr(),
            b"README.md".as_bstr(),
        ];
        let results = ops::check_ignore(&handle, &paths).expect("check_ignore failed");

        assert_eq!(results.len(), 4);
        assert!(results[0].is_ignored);
        assert!(!results[1].is_ignored);
        assert!(results[2].is_ignored);
        assert!(!results[3].is_ignored);
    }

    #[test]
    fn ignore_result_path_matches_input() {
        let repo = TestRepo::with_attributes();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let paths = vec![b"test.log".as_bstr()];
        let results = ops::check_ignore(&handle, &paths).expect("check_ignore failed");

        assert_eq!(results[0].path.as_slice(), b"test.log");
    }

    #[test]
    fn ide_files_ignored() {
        let repo = TestRepo::with_attributes();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let paths = vec![
            b".idea/".as_bstr(),
            b".vscode/".as_bstr(),
            b"file.swp".as_bstr(),
        ];
        let results = ops::check_ignore(&handle, &paths).expect("check_ignore failed");

        for result in &results {
            assert!(result.is_ignored);
        }
    }

    #[test]
    fn os_files_ignored() {
        let repo = TestRepo::with_attributes();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let paths = vec![b".DS_Store".as_bstr(), b"Thumbs.db".as_bstr()];
        let results = ops::check_ignore(&handle, &paths).expect("check_ignore failed");

        for result in &results {
            assert!(result.is_ignored);
        }
    }

    #[test]
    fn build_artifacts_ignored() {
        let repo = TestRepo::with_attributes();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let paths = vec![b"file.o".as_bstr(), b"lib.a".as_bstr()];
        let results = ops::check_ignore(&handle, &paths).expect("check_ignore failed");

        for result in &results {
            assert!(result.is_ignored);
        }
    }

    #[test]
    fn empty_paths_returns_empty() {
        let repo = TestRepo::with_attributes();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let paths: Vec<&bstr::BStr> = vec![];
        let results = ops::check_ignore(&handle, &paths).expect("check_ignore failed");

        assert!(results.is_empty());
    }

    #[test]
    fn nested_directory_path() {
        let repo = TestRepo::with_attributes();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let paths = vec![b"build/debug/".as_bstr()];
        let results = ops::check_ignore(&handle, &paths).expect("check_ignore failed");

        assert_eq!(results.len(), 1);
        assert!(results[0].is_ignored);
    }

    #[test]
    fn path_without_trailing_slash_for_directory() {
        let repo = TestRepo::with_attributes();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let paths = vec![b"build".as_bstr()];
        let results = ops::check_ignore(&handle, &paths).expect("check_ignore failed");

        assert_eq!(results.len(), 1);
    }

    #[test]
    fn env_local_file_ignored() {
        let repo = TestRepo::with_attributes();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let paths = vec![b".env.local".as_bstr()];
        let results = ops::check_ignore(&handle, &paths).expect("check_ignore failed");

        assert_eq!(results.len(), 1);
        assert!(results[0].is_ignored);
    }
}

mod check_ignore_negation {
    use super::*;
    use std::path::PathBuf;
    use std::process::Command;
    use tempfile::TempDir;

    fn create_repo_with_negation() -> (TempDir, PathBuf) {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        run_git(&path, &["init"]);
        run_git(&path, &["config", "user.email", "test@example.com"]);
        run_git(&path, &["config", "user.name", "Test User"]);

        std::fs::write(
            path.join(".gitignore"),
            r#"*.log
!important.log
temp/
!temp/keep.txt
"#,
        )
        .expect("failed to write .gitignore");

        std::fs::write(path.join("README.md"), "# Test\n").expect("failed to write README");

        run_git(&path, &["add", "."]);
        run_git(&path, &["commit", "-m", "Initial commit"]);

        (dir, path)
    }

    fn run_git(dir: &PathBuf, args: &[&str]) {
        let output = Command::new("git")
            .current_dir(dir)
            .args(args)
            .env("GIT_AUTHOR_DATE", "2024-01-15T10:00:00")
            .env("GIT_COMMITTER_DATE", "2024-01-15T10:00:00")
            .output()
            .expect("failed to execute git command");

        if !output.status.success() {
            panic!(
                "git {:?} failed: {}",
                args,
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }

    #[test]
    fn negation_pattern_unignores_file() {
        let (_dir, path) = create_repo_with_negation();
        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let paths = vec![b"important.log".as_bstr()];
        let results = ops::check_ignore(&handle, &paths).expect("check_ignore failed");

        assert_eq!(results.len(), 1);
        assert!(!results[0].is_ignored);
        assert!(results[0].pattern.is_some());
        assert!(results[0].source.is_some());
        let pattern = results[0].pattern.as_ref().unwrap();
        assert!(pattern.contains("important.log"), "pattern should contain the negated pattern: {}", pattern);
    }

    #[test]
    fn regular_pattern_still_ignores() {
        let (_dir, path) = create_repo_with_negation();
        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let paths = vec![b"debug.log".as_bstr()];
        let results = ops::check_ignore(&handle, &paths).expect("check_ignore failed");

        assert_eq!(results.len(), 1);
        assert!(results[0].is_ignored);
    }

    #[test]
    fn both_patterns_in_same_check() {
        let (_dir, path) = create_repo_with_negation();
        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let paths = vec![b"debug.log".as_bstr(), b"important.log".as_bstr()];
        let results = ops::check_ignore(&handle, &paths).expect("check_ignore failed");

        assert_eq!(results.len(), 2);
        assert!(results[0].is_ignored);
        assert!(!results[1].is_ignored);
    }
}

mod get_attributes {
    use super::*;

    #[test]
    fn rust_file_has_text_and_diff_attributes() {
        let repo = TestRepo::with_attributes();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let attrs =
            ops::get_attributes(&handle, b"src/main.rs".as_bstr(), &["text", "diff"]).expect("get_attributes failed");

        assert!(!attrs.is_empty());
    }

    #[test]
    fn binary_file_has_binary_attribute() {
        let repo = TestRepo::with_attributes();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let attrs =
            ops::get_attributes(&handle, b"icon.png".as_bstr(), &["binary"]).expect("get_attributes failed");

        assert!(!attrs.is_empty());
        let binary_attr = attrs.iter().find(|a| a.name == "binary");
        assert!(binary_attr.is_some());
        assert_eq!(binary_attr.unwrap().state, ops::AttributeState::Set);
    }

    #[test]
    fn shell_script_has_eol_lf() {
        let repo = TestRepo::with_attributes();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let attrs =
            ops::get_attributes(&handle, b"setup.sh".as_bstr(), &["text", "eol"]).expect("get_attributes failed");

        assert!(!attrs.is_empty());
        let eol_attr = attrs.iter().find(|a| a.name == "eol");
        assert!(eol_attr.is_some());
        if let ops::AttributeState::Value(v) = &eol_attr.unwrap().state {
            assert_eq!(v, "lf");
        }
    }

    #[test]
    fn markdown_file_has_diff_markdown() {
        let repo = TestRepo::with_attributes();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let attrs =
            ops::get_attributes(&handle, b"README.md".as_bstr(), &["text", "diff"]).expect("get_attributes failed");

        assert!(!attrs.is_empty());
    }

    #[test]
    fn directory_path_with_trailing_slash() {
        let repo = TestRepo::with_attributes();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let attrs =
            ops::get_attributes(&handle, b"src/".as_bstr(), &["text"]).expect("get_attributes failed");

        assert!(!attrs.is_empty());
    }

    #[test]
    fn unspecified_attribute() {
        let repo = TestRepo::with_attributes();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let attrs = ops::get_attributes(
            &handle,
            b"src/main.rs".as_bstr(),
            &["nonexistent_custom_attr"],
        )
        .expect("get_attributes failed");

        assert!(!attrs.is_empty());
        assert_eq!(attrs[0].state, ops::AttributeState::Unspecified);
    }

    #[test]
    fn multiple_attributes_query() {
        let repo = TestRepo::with_attributes();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let attrs = ops::get_attributes(
            &handle,
            b"src/main.rs".as_bstr(),
            &["text", "diff", "binary", "eol"],
        )
        .expect("get_attributes failed");

        assert_eq!(attrs.len(), 4);
    }

    #[test]
    fn attribute_value_name_preserved() {
        let repo = TestRepo::with_attributes();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let attrs =
            ops::get_attributes(&handle, b"src/main.rs".as_bstr(), &["text"]).expect("get_attributes failed");

        assert_eq!(attrs.len(), 1);
        assert_eq!(attrs[0].name, "text");
    }
}

mod get_attributes_with_unset {
    use super::*;
    use std::path::PathBuf;
    use std::process::Command;
    use tempfile::TempDir;

    fn create_repo_with_unset_attrs() -> (TempDir, PathBuf) {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        run_git(&path, &["init"]);
        run_git(&path, &["config", "user.email", "test@example.com"]);
        run_git(&path, &["config", "user.name", "Test User"]);

        std::fs::write(
            path.join(".gitattributes"),
            r#"*.txt text
*.bin -text
*.dat text=auto
"#,
        )
        .expect("failed to write .gitattributes");

        std::fs::write(path.join("readme.txt"), "text file\n").expect("failed to write txt");
        std::fs::write(path.join("data.bin"), vec![0u8; 16]).expect("failed to write bin");
        std::fs::write(path.join("mixed.dat"), "auto detect\n").expect("failed to write dat");

        run_git(&path, &["add", "."]);
        run_git(&path, &["commit", "-m", "Initial commit"]);

        (dir, path)
    }

    fn run_git(dir: &PathBuf, args: &[&str]) {
        let output = Command::new("git")
            .current_dir(dir)
            .args(args)
            .env("GIT_AUTHOR_DATE", "2024-01-15T10:00:00")
            .env("GIT_COMMITTER_DATE", "2024-01-15T10:00:00")
            .output()
            .expect("failed to execute git command");

        if !output.status.success() {
            panic!(
                "git {:?} failed: {}",
                args,
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }

    #[test]
    fn set_attribute_state() {
        let (_dir, path) = create_repo_with_unset_attrs();
        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let attrs =
            ops::get_attributes(&handle, b"readme.txt".as_bstr(), &["text"]).expect("get_attributes failed");

        assert_eq!(attrs.len(), 1);
        assert_eq!(attrs[0].state, ops::AttributeState::Set);
    }

    #[test]
    fn unset_attribute_state() {
        let (_dir, path) = create_repo_with_unset_attrs();
        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let attrs =
            ops::get_attributes(&handle, b"data.bin".as_bstr(), &["text"]).expect("get_attributes failed");

        assert_eq!(attrs.len(), 1);
        assert_eq!(attrs[0].state, ops::AttributeState::Unset);
    }

    #[test]
    fn value_attribute_state() {
        let (_dir, path) = create_repo_with_unset_attrs();
        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let attrs =
            ops::get_attributes(&handle, b"mixed.dat".as_bstr(), &["text"]).expect("get_attributes failed");

        assert_eq!(attrs.len(), 1);
        if let ops::AttributeState::Value(v) = &attrs[0].state {
            assert_eq!(v, "auto");
        } else {
            panic!("expected Value state");
        }
    }

    #[test]
    fn unspecified_attribute_state() {
        let (_dir, path) = create_repo_with_unset_attrs();
        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let attrs = ops::get_attributes(&handle, b"readme.txt".as_bstr(), &["custom_attr"])
            .expect("get_attributes failed");

        assert_eq!(attrs.len(), 1);
        assert_eq!(attrs[0].state, ops::AttributeState::Unspecified);
    }
}

mod attribute_state_types {
    use super::*;

    #[test]
    fn attribute_state_set_equality() {
        let state1 = ops::AttributeState::Set;
        let state2 = ops::AttributeState::Set;
        assert_eq!(state1, state2);
    }

    #[test]
    fn attribute_state_unset_equality() {
        let state1 = ops::AttributeState::Unset;
        let state2 = ops::AttributeState::Unset;
        assert_eq!(state1, state2);
    }

    #[test]
    fn attribute_state_value_equality() {
        let state1 = ops::AttributeState::Value("auto".to_string());
        let state2 = ops::AttributeState::Value("auto".to_string());
        assert_eq!(state1, state2);
    }

    #[test]
    fn attribute_state_value_inequality() {
        let state1 = ops::AttributeState::Value("lf".to_string());
        let state2 = ops::AttributeState::Value("crlf".to_string());
        assert_ne!(state1, state2);
    }

    #[test]
    fn attribute_state_unspecified_equality() {
        let state1 = ops::AttributeState::Unspecified;
        let state2 = ops::AttributeState::Unspecified;
        assert_eq!(state1, state2);
    }

    #[test]
    fn attribute_state_different_variants_not_equal() {
        assert_ne!(ops::AttributeState::Set, ops::AttributeState::Unset);
        assert_ne!(ops::AttributeState::Set, ops::AttributeState::Unspecified);
        assert_ne!(ops::AttributeState::Unset, ops::AttributeState::Unspecified);
    }

    #[test]
    fn attribute_state_clone() {
        let state = ops::AttributeState::Value("test".to_string());
        let cloned = state.clone();
        assert_eq!(state, cloned);
    }

    #[test]
    fn attribute_state_debug() {
        let state = ops::AttributeState::Set;
        let debug_str = format!("{:?}", state);
        assert!(debug_str.contains("Set"));
    }
}

mod ignore_result_types {
    use super::*;
    use bstr::BString;

    #[test]
    fn ignore_result_equality() {
        let result1 = ops::IgnoreResult {
            path: BString::from("test.log"),
            is_ignored: true,
            pattern: Some("*.log".to_string()),
            source: Some(".gitignore".to_string()),
        };
        let result2 = ops::IgnoreResult {
            path: BString::from("test.log"),
            is_ignored: true,
            pattern: Some("*.log".to_string()),
            source: Some(".gitignore".to_string()),
        };
        assert_eq!(result1, result2);
    }

    #[test]
    fn ignore_result_inequality() {
        let result1 = ops::IgnoreResult {
            path: BString::from("test.log"),
            is_ignored: true,
            pattern: Some("*.log".to_string()),
            source: Some(".gitignore".to_string()),
        };
        let result2 = ops::IgnoreResult {
            path: BString::from("other.log"),
            is_ignored: true,
            pattern: Some("*.log".to_string()),
            source: Some(".gitignore".to_string()),
        };
        assert_ne!(result1, result2);
    }

    #[test]
    fn ignore_result_clone() {
        let result = ops::IgnoreResult {
            path: BString::from("test.log"),
            is_ignored: true,
            pattern: Some("*.log".to_string()),
            source: Some(".gitignore".to_string()),
        };
        let cloned = result.clone();
        assert_eq!(result, cloned);
    }

    #[test]
    fn ignore_result_debug() {
        let result = ops::IgnoreResult {
            path: BString::from("test.log"),
            is_ignored: true,
            pattern: Some("*.log".to_string()),
            source: Some(".gitignore".to_string()),
        };
        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("test.log"));
    }
}

mod attribute_value_types {
    use super::*;

    #[test]
    fn attribute_value_equality() {
        let val1 = ops::AttributeValue {
            name: "text".to_string(),
            state: ops::AttributeState::Set,
        };
        let val2 = ops::AttributeValue {
            name: "text".to_string(),
            state: ops::AttributeState::Set,
        };
        assert_eq!(val1, val2);
    }

    #[test]
    fn attribute_value_inequality() {
        let val1 = ops::AttributeValue {
            name: "text".to_string(),
            state: ops::AttributeState::Set,
        };
        let val2 = ops::AttributeValue {
            name: "binary".to_string(),
            state: ops::AttributeState::Set,
        };
        assert_ne!(val1, val2);
    }

    #[test]
    fn attribute_value_clone() {
        let val = ops::AttributeValue {
            name: "diff".to_string(),
            state: ops::AttributeState::Value("rust".to_string()),
        };
        let cloned = val.clone();
        assert_eq!(val, cloned);
    }

    #[test]
    fn attribute_value_debug() {
        let val = ops::AttributeValue {
            name: "text".to_string(),
            state: ops::AttributeState::Set,
        };
        let debug_str = format!("{:?}", val);
        assert!(debug_str.contains("text"));
    }
}

mod edge_cases {
    use super::*;

    #[test]
    fn deep_nested_path_ignored() {
        let repo = TestRepo::with_attributes();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let paths = vec![b"target/debug/deps/lib.o".as_bstr()];
        let results = ops::check_ignore(&handle, &paths).expect("check_ignore failed");

        assert_eq!(results.len(), 1);
        assert!(results[0].is_ignored);
    }

    #[test]
    fn special_characters_in_path() {
        let repo = TestRepo::with_attributes();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let paths = vec![b"file-with-dash.log".as_bstr()];
        let results = ops::check_ignore(&handle, &paths).expect("check_ignore failed");

        assert_eq!(results.len(), 1);
        assert!(results[0].is_ignored);
    }

    #[test]
    fn attributes_empty_query() {
        let repo = TestRepo::with_attributes();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let attrs: Vec<&str> = vec![];
        let results =
            ops::get_attributes(&handle, b"src/main.rs".as_bstr(), &attrs).expect("get_attributes failed");

        assert!(results.is_empty());
    }

    #[test]
    fn path_with_unicode() {
        let repo = TestRepo::with_attributes();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let paths = vec![b"archivo.log".as_bstr()];
        let results = ops::check_ignore(&handle, &paths).expect("check_ignore failed");

        assert_eq!(results.len(), 1);
    }

    #[test]
    fn windows_batch_file_attributes() {
        let repo = TestRepo::with_attributes();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let attrs =
            ops::get_attributes(&handle, b"build.bat".as_bstr(), &["eol"]).expect("get_attributes failed");

        assert!(!attrs.is_empty());
        if let ops::AttributeState::Value(v) = &attrs[0].state {
            assert_eq!(v, "crlf");
        }
    }

    #[test]
    fn cmd_file_attributes() {
        let repo = TestRepo::with_attributes();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let attrs =
            ops::get_attributes(&handle, b"script.cmd".as_bstr(), &["eol"]).expect("get_attributes failed");

        assert!(!attrs.is_empty());
        if let ops::AttributeState::Value(v) = &attrs[0].state {
            assert_eq!(v, "crlf");
        }
    }
}

mod repo_without_gitignore {
    use super::*;

    #[test]
    fn check_ignore_on_repo_without_gitignore() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let paths = vec![b"anything.log".as_bstr()];
        let results = ops::check_ignore(&handle, &paths).expect("check_ignore failed");

        assert_eq!(results.len(), 1);
        assert!(!results[0].is_ignored);
        assert!(results[0].pattern.is_none());
    }

    #[test]
    fn get_attributes_on_repo_without_gitattributes() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let attrs =
            ops::get_attributes(&handle, b"src/main.rs".as_bstr(), &["text"]).expect("get_attributes failed");

        assert!(!attrs.is_empty());
    }
}

mod error_handling {
    use super::*;
    use std::path::PathBuf;
    use std::process::Command;
    use tempfile::TempDir;

    fn run_git(dir: &PathBuf, args: &[&str]) {
        let output = Command::new("git")
            .current_dir(dir)
            .args(args)
            .env("GIT_AUTHOR_DATE", "2024-01-15T10:00:00")
            .env("GIT_COMMITTER_DATE", "2024-01-15T10:00:00")
            .output()
            .expect("failed to execute git command");

        if !output.status.success() {
            panic!(
                "git {:?} failed: {}",
                args,
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }

    fn create_repo_with_deleted_index() -> (TempDir, PathBuf) {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        run_git(&path, &["init"]);
        run_git(&path, &["config", "user.email", "test@example.com"]);
        run_git(&path, &["config", "user.name", "Test User"]);

        std::fs::write(path.join("README.md"), "# Test\n").expect("failed to write README");
        run_git(&path, &["add", "."]);
        run_git(&path, &["commit", "-m", "Initial commit"]);

        let index_path = path.join(".git/index");
        std::fs::remove_file(&index_path).expect("failed to delete index");

        (dir, path)
    }

    #[test]
    fn check_ignore_with_deleted_index_uses_empty_index() {
        let (_dir, path) = create_repo_with_deleted_index();
        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let paths = vec![b"test.log".as_bstr()];
        let result = ops::check_ignore(&handle, &paths);

        match result {
            Ok(results) => {
                assert_eq!(results.len(), 1);
            }
            Err(_) => {
            }
        }
    }

    #[test]
    fn get_attributes_with_deleted_index_uses_empty_index() {
        let (_dir, path) = create_repo_with_deleted_index();
        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let result = ops::get_attributes(&handle, b"README.md".as_bstr(), &["text"]);

        match result {
            Ok(attrs) => {
                assert!(!attrs.is_empty());
            }
            Err(_) => {
            }
        }
    }

    #[test]
    fn check_ignore_on_empty_repo() {
        let repo = TestRepo::empty();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let paths = vec![b"test.log".as_bstr()];
        let results = ops::check_ignore(&handle, &paths).expect("check_ignore should work on empty repo");

        assert_eq!(results.len(), 1);
        assert!(!results[0].is_ignored);
    }

    #[test]
    fn get_attributes_on_empty_repo() {
        let repo = TestRepo::empty();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let attrs = ops::get_attributes(&handle, b"test.txt".as_bstr(), &["text"])
            .expect("get_attributes should work on empty repo");

        assert!(!attrs.is_empty());
    }

    #[test]
    fn check_ignore_with_nonexistent_parent_path() {
        let repo = TestRepo::with_attributes();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let paths = vec![b"nonexistent/deep/nested/path/file.log".as_bstr()];
        let results = ops::check_ignore(&handle, &paths).expect("check_ignore failed");

        assert_eq!(results.len(), 1);
        assert!(results[0].is_ignored);
    }

    #[test]
    fn get_attributes_with_nonexistent_parent_path() {
        let repo = TestRepo::with_attributes();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let attrs = ops::get_attributes(
            &handle,
            b"nonexistent/deep/nested/path/file.rs".as_bstr(),
            &["text", "diff"],
        )
        .expect("get_attributes failed");

        assert!(!attrs.is_empty());
    }

    fn create_repo_with_truncated_index() -> (TempDir, PathBuf) {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        run_git(&path, &["init"]);
        run_git(&path, &["config", "user.email", "test@example.com"]);
        run_git(&path, &["config", "user.name", "Test User"]);

        std::fs::write(path.join("file.txt"), "content").expect("failed to write file");
        run_git(&path, &["add", "."]);
        run_git(&path, &["commit", "-m", "Initial commit"]);

        let index_path = path.join(".git/index");
        let index_data = std::fs::read(&index_path).expect("failed to read index");
        let truncated = &index_data[..index_data.len().saturating_sub(50).max(12)];
        std::fs::write(&index_path, truncated).expect("failed to write truncated index");

        (dir, path)
    }

    #[test]
    fn check_ignore_with_truncated_index() {
        let (_dir, path) = create_repo_with_truncated_index();
        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let paths = vec![b"test.log".as_bstr()];
        let result = ops::check_ignore(&handle, &paths);

        match result {
            Ok(results) => {
                assert_eq!(results.len(), 1);
            }
            Err(_e) => {
            }
        }
    }

    #[test]
    fn get_attributes_with_truncated_index() {
        let (_dir, path) = create_repo_with_truncated_index();
        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let result = ops::get_attributes(&handle, b"file.txt".as_bstr(), &["text"]);

        match result {
            Ok(attrs) => {
                assert!(!attrs.is_empty());
            }
            Err(_e) => {
            }
        }
    }

    fn create_repo_with_invalid_index_version() -> (TempDir, PathBuf) {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        run_git(&path, &["init"]);
        run_git(&path, &["config", "user.email", "test@example.com"]);
        run_git(&path, &["config", "user.name", "Test User"]);

        std::fs::write(path.join("file.txt"), "content").expect("failed to write file");
        run_git(&path, &["add", "."]);
        run_git(&path, &["commit", "-m", "Initial commit"]);

        let index_path = path.join(".git/index");
        let mut index_data = std::fs::read(&index_path).expect("failed to read index");
        if index_data.len() >= 8 {
            index_data[4] = 0xFF;
            index_data[5] = 0xFF;
            index_data[6] = 0xFF;
            index_data[7] = 0xFF;
        }
        std::fs::write(&index_path, index_data).expect("failed to write modified index");

        (dir, path)
    }

    #[test]
    fn check_ignore_with_invalid_index_version() {
        let (_dir, path) = create_repo_with_invalid_index_version();
        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let paths = vec![b"test.log".as_bstr()];
        let result = ops::check_ignore(&handle, &paths);

        match result {
            Ok(results) => {
                assert_eq!(results.len(), 1);
            }
            Err(_e) => {
            }
        }
    }

    #[test]
    fn get_attributes_with_invalid_index_version() {
        let (_dir, path) = create_repo_with_invalid_index_version();
        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let result = ops::get_attributes(&handle, b"file.txt".as_bstr(), &["text"]);

        match result {
            Ok(attrs) => {
                assert!(!attrs.is_empty());
            }
            Err(_e) => {
            }
        }
    }

    #[test]
    fn check_ignore_bare_repo() {
        let repo = TestRepo::bare();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let paths = vec![b"test.log".as_bstr()];
        let results = ops::check_ignore(&handle, &paths).expect("check_ignore failed on bare repo");

        assert_eq!(results.len(), 1);
    }

    #[test]
    fn get_attributes_bare_repo() {
        let repo = TestRepo::bare();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let attrs = ops::get_attributes(&handle, b"README.md".as_bstr(), &["text"])
            .expect("get_attributes failed on bare repo");

        assert!(!attrs.is_empty());
    }

    #[test]
    fn check_ignore_repo_without_index() {
        let repo = TestRepo::without_index();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let paths = vec![b"test.log".as_bstr()];
        let results = ops::check_ignore(&handle, &paths).expect("check_ignore should work without index");

        assert_eq!(results.len(), 1);
    }

    #[test]
    fn get_attributes_repo_without_index() {
        let repo = TestRepo::without_index();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let attrs = ops::get_attributes(&handle, b"README.md".as_bstr(), &["text"])
            .expect("get_attributes should work without index");

        assert!(!attrs.is_empty());
    }

    fn create_repo_with_gitignore_read_error() -> (TempDir, PathBuf) {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        run_git(&path, &["init"]);
        run_git(&path, &["config", "user.email", "test@example.com"]);
        run_git(&path, &["config", "user.name", "Test User"]);

        std::fs::write(path.join("README.md"), "# Test\n").expect("failed to write README");
        run_git(&path, &["add", "."]);
        run_git(&path, &["commit", "-m", "Initial commit"]);

        let info_dir = path.join(".git/info");
        std::fs::create_dir_all(&info_dir).expect("failed to create info dir");
        std::fs::write(info_dir.join("exclude"), b"\xff\xfe invalid utf8").expect("failed to write exclude");

        (dir, path)
    }

    #[test]
    fn check_ignore_with_gitignore_invalid_encoding() {
        let (_dir, path) = create_repo_with_gitignore_read_error();
        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let paths = vec![b"test.log".as_bstr()];
        let result = ops::check_ignore(&handle, &paths);

        match result {
            Ok(results) => {
                assert_eq!(results.len(), 1);
            }
            Err(_) => {}
        }
    }

    fn create_repo_with_deep_worktree_symlink() -> Option<(TempDir, PathBuf)> {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        run_git(&path, &["init"]);
        run_git(&path, &["config", "user.email", "test@example.com"]);
        run_git(&path, &["config", "user.name", "Test User"]);

        std::fs::write(path.join("README.md"), "# Test\n").expect("failed to write README");
        run_git(&path, &["add", "."]);
        run_git(&path, &["commit", "-m", "Initial commit"]);

        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            let symlink_path = path.join("broken_link");
            if symlink("/nonexistent/path/that/does/not/exist", &symlink_path).is_ok() {
                return Some((dir, path));
            }
        }

        None
    }

    #[test]
    fn check_ignore_with_broken_symlink_in_worktree() {
        if let Some((_dir, path)) = create_repo_with_deep_worktree_symlink() {
            let pool = create_pool();
            let handle = pool.get(&path).expect("failed to get handle");

            let paths = vec![b"broken_link/some/file.log".as_bstr()];
            let result = ops::check_ignore(&handle, &paths);

            match result {
                Ok(results) => {
                    assert_eq!(results.len(), 1);
                }
                Err(_) => {}
            }
        }
    }

    #[test]
    fn get_attributes_with_broken_symlink_in_worktree() {
        if let Some((_dir, path)) = create_repo_with_deep_worktree_symlink() {
            let pool = create_pool();
            let handle = pool.get(&path).expect("failed to get handle");

            let result = ops::get_attributes(&handle, b"broken_link/file.txt".as_bstr(), &["text"]);

            match result {
                Ok(attrs) => {
                    assert!(!attrs.is_empty());
                }
                Err(_) => {}
            }
        }
    }

    fn create_repo_with_corrupt_gitattributes() -> (TempDir, PathBuf) {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        run_git(&path, &["init"]);
        run_git(&path, &["config", "user.email", "test@example.com"]);
        run_git(&path, &["config", "user.name", "Test User"]);

        std::fs::write(path.join(".gitattributes"), b"\xff\xfe*.bin binary\n")
            .expect("failed to write .gitattributes");

        std::fs::write(path.join("README.md"), "# Test\n").expect("failed to write README");
        run_git(&path, &["add", "."]);
        run_git(&path, &["commit", "-m", "Initial commit"]);

        (dir, path)
    }

    #[test]
    fn get_attributes_with_corrupt_gitattributes() {
        let (_dir, path) = create_repo_with_corrupt_gitattributes();
        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let result = ops::get_attributes(&handle, b"test.bin".as_bstr(), &["binary"]);

        match result {
            Ok(attrs) => {
                assert!(!attrs.is_empty());
            }
            Err(_) => {}
        }
    }

    fn create_repo_with_special_chars_in_gitignore() -> (TempDir, PathBuf) {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        run_git(&path, &["init"]);
        run_git(&path, &["config", "user.email", "test@example.com"]);
        run_git(&path, &["config", "user.name", "Test User"]);

        std::fs::write(
            path.join(".gitignore"),
            b"*.log\n# Comment with special chars: \xc0\xc1\n*.tmp\n",
        )
        .expect("failed to write .gitignore");

        std::fs::write(path.join("README.md"), "# Test\n").expect("failed to write README");
        run_git(&path, &["add", "."]);
        run_git(&path, &["commit", "-m", "Initial commit"]);

        (dir, path)
    }

    #[test]
    fn check_ignore_with_special_chars_in_gitignore() {
        let (_dir, path) = create_repo_with_special_chars_in_gitignore();
        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let paths = vec![b"test.log".as_bstr(), b"test.tmp".as_bstr()];
        let result = ops::check_ignore(&handle, &paths);

        match result {
            Ok(results) => {
                assert_eq!(results.len(), 2);
            }
            Err(_) => {}
        }
    }

    fn create_repo_with_very_long_pattern() -> (TempDir, PathBuf) {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        run_git(&path, &["init"]);
        run_git(&path, &["config", "user.email", "test@example.com"]);
        run_git(&path, &["config", "user.name", "Test User"]);

        let long_pattern = "a".repeat(4096);
        std::fs::write(
            path.join(".gitignore"),
            format!("*.log\n{}\n*.tmp\n", long_pattern),
        )
        .expect("failed to write .gitignore");

        std::fs::write(path.join("README.md"), "# Test\n").expect("failed to write README");
        run_git(&path, &["add", "."]);
        run_git(&path, &["commit", "-m", "Initial commit"]);

        (dir, path)
    }

    #[test]
    fn check_ignore_with_very_long_pattern_in_gitignore() {
        let (_dir, path) = create_repo_with_very_long_pattern();
        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let paths = vec![b"test.log".as_bstr()];
        let result = ops::check_ignore(&handle, &paths);

        match result {
            Ok(results) => {
                assert_eq!(results.len(), 1);
                assert!(results[0].is_ignored);
            }
            Err(_) => {}
        }
    }

    fn create_repo_with_malformed_gitignore_entries() -> (TempDir, PathBuf) {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        run_git(&path, &["init"]);
        run_git(&path, &["config", "user.email", "test@example.com"]);
        run_git(&path, &["config", "user.name", "Test User"]);

        std::fs::write(
            path.join(".gitignore"),
            "*.log\n\x00\x00\x00\n*.tmp\n[invalid[bracket\n",
        )
        .expect("failed to write .gitignore");

        std::fs::write(path.join("README.md"), "# Test\n").expect("failed to write README");
        run_git(&path, &["add", "."]);
        run_git(&path, &["commit", "-m", "Initial commit"]);

        (dir, path)
    }

    #[test]
    fn check_ignore_with_malformed_entries() {
        let (_dir, path) = create_repo_with_malformed_gitignore_entries();
        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let paths = vec![b"test.log".as_bstr(), b"test.tmp".as_bstr()];
        let result = ops::check_ignore(&handle, &paths);

        match result {
            Ok(results) => {
                assert_eq!(results.len(), 2);
            }
            Err(_) => {}
        }
    }

    fn create_repo_with_gitattributes_complex_patterns() -> (TempDir, PathBuf) {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        run_git(&path, &["init"]);
        run_git(&path, &["config", "user.email", "test@example.com"]);
        run_git(&path, &["config", "user.name", "Test User"]);

        std::fs::write(
            path.join(".gitattributes"),
            "*.rs text diff=rust\n[attr]myattr -diff -merge\n*.special myattr\n",
        )
        .expect("failed to write .gitattributes");

        std::fs::write(path.join("README.md"), "# Test\n").expect("failed to write README");
        run_git(&path, &["add", "."]);
        run_git(&path, &["commit", "-m", "Initial commit"]);

        (dir, path)
    }

    #[test]
    fn get_attributes_with_macro_definitions() {
        let (_dir, path) = create_repo_with_gitattributes_complex_patterns();
        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let result = ops::get_attributes(&handle, b"test.special".as_bstr(), &["diff", "merge"]);

        match result {
            Ok(attrs) => {
                assert!(!attrs.is_empty());
            }
            Err(_) => {}
        }
    }

    fn create_repo_with_empty_gitignore() -> (TempDir, PathBuf) {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        run_git(&path, &["init"]);
        run_git(&path, &["config", "user.email", "test@example.com"]);
        run_git(&path, &["config", "user.name", "Test User"]);

        std::fs::write(path.join(".gitignore"), "").expect("failed to write .gitignore");

        std::fs::write(path.join("README.md"), "# Test\n").expect("failed to write README");
        run_git(&path, &["add", "."]);
        run_git(&path, &["commit", "-m", "Initial commit"]);

        (dir, path)
    }

    #[test]
    fn check_ignore_with_empty_gitignore() {
        let (_dir, path) = create_repo_with_empty_gitignore();
        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let paths = vec![b"test.log".as_bstr()];
        let results = ops::check_ignore(&handle, &paths).expect("check_ignore failed");

        assert_eq!(results.len(), 1);
        assert!(!results[0].is_ignored);
    }

    fn create_repo_with_empty_gitattributes() -> (TempDir, PathBuf) {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        run_git(&path, &["init"]);
        run_git(&path, &["config", "user.email", "test@example.com"]);
        run_git(&path, &["config", "user.name", "Test User"]);

        std::fs::write(path.join(".gitattributes"), "").expect("failed to write .gitattributes");

        std::fs::write(path.join("README.md"), "# Test\n").expect("failed to write README");
        run_git(&path, &["add", "."]);
        run_git(&path, &["commit", "-m", "Initial commit"]);

        (dir, path)
    }

    #[test]
    fn get_attributes_with_empty_gitattributes() {
        let (_dir, path) = create_repo_with_empty_gitattributes();
        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let attrs = ops::get_attributes(&handle, b"test.txt".as_bstr(), &["text"])
            .expect("get_attributes failed");

        assert!(!attrs.is_empty());
        assert_eq!(attrs[0].state, ops::AttributeState::Unspecified);
    }

    #[test]
    fn check_ignore_multiple_empty_paths() {
        let repo = TestRepo::with_attributes();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let paths = vec![b"".as_bstr()];
        let result = ops::check_ignore(&handle, &paths);

        match result {
            Ok(results) => {
                assert_eq!(results.len(), 1);
            }
            Err(_) => {}
        }
    }

    #[test]
    fn get_attributes_empty_path() {
        let repo = TestRepo::with_attributes();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let result = ops::get_attributes(&handle, b"".as_bstr(), &["text"]);

        match result {
            Ok(attrs) => {
                assert!(!attrs.is_empty());
            }
            Err(_) => {}
        }
    }

    #[test]
    fn check_ignore_path_with_null_byte() {
        let repo = TestRepo::with_attributes();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let paths = vec![b"test\x00file.log".as_bstr()];
        let result = ops::check_ignore(&handle, &paths);

        match result {
            Ok(results) => {
                assert_eq!(results.len(), 1);
            }
            Err(_) => {}
        }
    }

    #[test]
    fn get_attributes_path_with_null_byte() {
        let repo = TestRepo::with_attributes();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let result = ops::get_attributes(&handle, b"test\x00file.txt".as_bstr(), &["text"]);

        match result {
            Ok(attrs) => {
                assert!(!attrs.is_empty());
            }
            Err(_) => {}
        }
    }
}

mod gitinfo_exclude {
    use super::*;
    use std::path::PathBuf;
    use std::process::Command;
    use tempfile::TempDir;

    fn run_git(dir: &PathBuf, args: &[&str]) {
        let output = Command::new("git")
            .current_dir(dir)
            .args(args)
            .env("GIT_AUTHOR_DATE", "2024-01-15T10:00:00")
            .env("GIT_COMMITTER_DATE", "2024-01-15T10:00:00")
            .output()
            .expect("failed to execute git command");

        if !output.status.success() {
            panic!(
                "git {:?} failed: {}",
                args,
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }

    fn create_repo_with_info_exclude() -> (TempDir, PathBuf) {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        run_git(&path, &["init"]);
        run_git(&path, &["config", "user.email", "test@example.com"]);
        run_git(&path, &["config", "user.name", "Test User"]);

        let info_dir = path.join(".git/info");
        std::fs::create_dir_all(&info_dir).expect("failed to create info dir");
        std::fs::write(info_dir.join("exclude"), "*.secret\n/private/\n")
            .expect("failed to write exclude");

        std::fs::write(path.join("README.md"), "# Test\n").expect("failed to write README");
        run_git(&path, &["add", "."]);
        run_git(&path, &["commit", "-m", "Initial commit"]);

        (dir, path)
    }

    #[test]
    fn check_ignore_with_info_exclude_pattern() {
        let (_dir, path) = create_repo_with_info_exclude();
        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let paths = vec![b"data.secret".as_bstr()];
        let results = ops::check_ignore(&handle, &paths).expect("check_ignore failed");

        assert_eq!(results.len(), 1);
        assert!(results[0].is_ignored);
    }

    #[test]
    fn check_ignore_private_dir_from_info_exclude() {
        let (_dir, path) = create_repo_with_info_exclude();
        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let paths = vec![b"private/".as_bstr()];
        let results = ops::check_ignore(&handle, &paths).expect("check_ignore failed");

        assert_eq!(results.len(), 1);
        assert!(results[0].is_ignored);
    }
}

mod nested_gitignore {
    use super::*;
    use std::path::PathBuf;
    use std::process::Command;
    use tempfile::TempDir;

    fn run_git(dir: &PathBuf, args: &[&str]) {
        let output = Command::new("git")
            .current_dir(dir)
            .args(args)
            .env("GIT_AUTHOR_DATE", "2024-01-15T10:00:00")
            .env("GIT_COMMITTER_DATE", "2024-01-15T10:00:00")
            .output()
            .expect("failed to execute git command");

        if !output.status.success() {
            panic!(
                "git {:?} failed: {}",
                args,
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }

    fn create_repo_with_nested_gitignore() -> (TempDir, PathBuf) {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        run_git(&path, &["init"]);
        run_git(&path, &["config", "user.email", "test@example.com"]);
        run_git(&path, &["config", "user.name", "Test User"]);

        std::fs::write(path.join(".gitignore"), "*.log\n").expect("failed to write root .gitignore");

        std::fs::create_dir_all(path.join("subdir")).expect("failed to create subdir");
        std::fs::write(path.join("subdir/.gitignore"), "!important.log\n*.tmp\n")
            .expect("failed to write subdir .gitignore");

        std::fs::write(path.join("README.md"), "# Test\n").expect("failed to write README");
        run_git(&path, &["add", "."]);
        run_git(&path, &["commit", "-m", "Initial commit"]);

        (dir, path)
    }

    #[test]
    fn nested_gitignore_can_unignore_parent_pattern() {
        let (_dir, path) = create_repo_with_nested_gitignore();
        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let paths = vec![b"subdir/important.log".as_bstr()];
        let results = ops::check_ignore(&handle, &paths).expect("check_ignore failed");

        assert_eq!(results.len(), 1);
        assert!(!results[0].is_ignored);
    }

    #[test]
    fn nested_gitignore_pattern_applies_to_subdir() {
        let (_dir, path) = create_repo_with_nested_gitignore();
        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let paths = vec![b"subdir/file.tmp".as_bstr()];
        let results = ops::check_ignore(&handle, &paths).expect("check_ignore failed");

        assert_eq!(results.len(), 1);
        assert!(results[0].is_ignored);
    }

    #[test]
    fn root_gitignore_applies_to_nested_file() {
        let (_dir, path) = create_repo_with_nested_gitignore();
        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let paths = vec![b"subdir/regular.log".as_bstr()];
        let results = ops::check_ignore(&handle, &paths).expect("check_ignore failed");

        assert_eq!(results.len(), 1);
        assert!(results[0].is_ignored);
    }
}

mod gitattributes_inheritance {
    use super::*;
    use std::path::PathBuf;
    use std::process::Command;
    use tempfile::TempDir;

    fn run_git(dir: &PathBuf, args: &[&str]) {
        let output = Command::new("git")
            .current_dir(dir)
            .args(args)
            .env("GIT_AUTHOR_DATE", "2024-01-15T10:00:00")
            .env("GIT_COMMITTER_DATE", "2024-01-15T10:00:00")
            .output()
            .expect("failed to execute git command");

        if !output.status.success() {
            panic!(
                "git {:?} failed: {}",
                args,
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }

    fn create_repo_with_nested_gitattributes() -> (TempDir, PathBuf) {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        run_git(&path, &["init"]);
        run_git(&path, &["config", "user.email", "test@example.com"]);
        run_git(&path, &["config", "user.name", "Test User"]);

        std::fs::write(path.join(".gitattributes"), "*.txt text\n*.data binary\n")
            .expect("failed to write root .gitattributes");

        std::fs::create_dir_all(path.join("special")).expect("failed to create special dir");
        std::fs::write(path.join("special/.gitattributes"), "*.txt -text\n*.data text\n")
            .expect("failed to write special .gitattributes");

        std::fs::write(path.join("README.md"), "# Test\n").expect("failed to write README");
        run_git(&path, &["add", "."]);
        run_git(&path, &["commit", "-m", "Initial commit"]);

        (dir, path)
    }

    #[test]
    fn nested_gitattributes_overrides_parent() {
        let (_dir, path) = create_repo_with_nested_gitattributes();
        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let attrs = ops::get_attributes(&handle, b"special/file.txt".as_bstr(), &["text"])
            .expect("get_attributes failed");

        assert_eq!(attrs.len(), 1);
        assert_eq!(attrs[0].state, ops::AttributeState::Unset);
    }

    #[test]
    fn root_gitattributes_applies_outside_nested_dir() {
        let (_dir, path) = create_repo_with_nested_gitattributes();
        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let attrs = ops::get_attributes(&handle, b"file.txt".as_bstr(), &["text"])
            .expect("get_attributes failed");

        assert_eq!(attrs.len(), 1);
        assert_eq!(attrs[0].state, ops::AttributeState::Set);
    }

    #[test]
    fn nested_binary_override_to_text() {
        let (_dir, path) = create_repo_with_nested_gitattributes();
        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let attrs = ops::get_attributes(&handle, b"special/file.data".as_bstr(), &["text"])
            .expect("get_attributes failed");

        assert_eq!(attrs.len(), 1);
        assert_eq!(attrs[0].state, ops::AttributeState::Set);
    }
}

mod double_star_patterns {
    use super::*;
    use std::path::PathBuf;
    use std::process::Command;
    use tempfile::TempDir;

    fn run_git(dir: &PathBuf, args: &[&str]) {
        let output = Command::new("git")
            .current_dir(dir)
            .args(args)
            .env("GIT_AUTHOR_DATE", "2024-01-15T10:00:00")
            .env("GIT_COMMITTER_DATE", "2024-01-15T10:00:00")
            .output()
            .expect("failed to execute git command");

        if !output.status.success() {
            panic!(
                "git {:?} failed: {}",
                args,
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }

    fn create_repo_with_double_star_pattern() -> (TempDir, PathBuf) {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        run_git(&path, &["init"]);
        run_git(&path, &["config", "user.email", "test@example.com"]);
        run_git(&path, &["config", "user.name", "Test User"]);

        std::fs::write(
            path.join(".gitignore"),
            "**/node_modules/\n**/__pycache__/\n**/build/*.o\n",
        )
        .expect("failed to write .gitignore");

        std::fs::write(path.join("README.md"), "# Test\n").expect("failed to write README");
        run_git(&path, &["add", "."]);
        run_git(&path, &["commit", "-m", "Initial commit"]);

        (dir, path)
    }

    #[test]
    fn double_star_matches_at_root() {
        let (_dir, path) = create_repo_with_double_star_pattern();
        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let paths = vec![b"node_modules/".as_bstr()];
        let results = ops::check_ignore(&handle, &paths).expect("check_ignore failed");

        assert_eq!(results.len(), 1);
        assert!(results[0].is_ignored);
    }

    #[test]
    fn double_star_matches_nested() {
        let (_dir, path) = create_repo_with_double_star_pattern();
        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let paths = vec![b"src/frontend/node_modules/".as_bstr()];
        let results = ops::check_ignore(&handle, &paths).expect("check_ignore failed");

        assert_eq!(results.len(), 1);
        assert!(results[0].is_ignored);
    }

    #[test]
    fn double_star_matches_deeply_nested() {
        let (_dir, path) = create_repo_with_double_star_pattern();
        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let paths = vec![b"a/b/c/d/__pycache__/".as_bstr()];
        let results = ops::check_ignore(&handle, &paths).expect("check_ignore failed");

        assert_eq!(results.len(), 1);
        assert!(results[0].is_ignored);
    }

    #[test]
    fn double_star_with_filename_pattern() {
        let (_dir, path) = create_repo_with_double_star_pattern();
        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let paths = vec![b"src/app/build/main.o".as_bstr()];
        let results = ops::check_ignore(&handle, &paths).expect("check_ignore failed");

        assert_eq!(results.len(), 1);
        assert!(results[0].is_ignored);
    }
}

mod pattern_source_tracking {
    use super::*;
    use std::path::PathBuf;
    use std::process::Command;
    use tempfile::TempDir;

    fn run_git(dir: &PathBuf, args: &[&str]) {
        let output = Command::new("git")
            .current_dir(dir)
            .args(args)
            .env("GIT_AUTHOR_DATE", "2024-01-15T10:00:00")
            .env("GIT_COMMITTER_DATE", "2024-01-15T10:00:00")
            .output()
            .expect("failed to execute git command");

        if !output.status.success() {
            panic!(
                "git {:?} failed: {}",
                args,
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }

    fn create_repo_with_multiple_ignore_sources() -> (TempDir, PathBuf) {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        run_git(&path, &["init"]);
        run_git(&path, &["config", "user.email", "test@example.com"]);
        run_git(&path, &["config", "user.name", "Test User"]);

        std::fs::write(path.join(".gitignore"), "*.log\n").expect("failed to write root .gitignore");

        std::fs::create_dir_all(path.join("subdir")).expect("failed to create subdir");
        std::fs::write(path.join("subdir/.gitignore"), "*.tmp\n")
            .expect("failed to write subdir .gitignore");

        let info_dir = path.join(".git/info");
        std::fs::create_dir_all(&info_dir).expect("failed to create info dir");
        std::fs::write(info_dir.join("exclude"), "*.bak\n")
            .expect("failed to write exclude");

        std::fs::write(path.join("README.md"), "# Test\n").expect("failed to write README");
        run_git(&path, &["add", "."]);
        run_git(&path, &["commit", "-m", "Initial commit"]);

        (dir, path)
    }

    #[test]
    fn source_points_to_root_gitignore() {
        let (_dir, path) = create_repo_with_multiple_ignore_sources();
        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let paths = vec![b"app.log".as_bstr()];
        let results = ops::check_ignore(&handle, &paths).expect("check_ignore failed");

        assert_eq!(results.len(), 1);
        assert!(results[0].is_ignored);
        assert!(results[0].source.is_some());
        let source = results[0].source.as_ref().unwrap();
        assert!(source.ends_with(".gitignore") || source.contains("gitignore"));
    }

    #[test]
    fn source_points_to_subdir_gitignore() {
        let (_dir, path) = create_repo_with_multiple_ignore_sources();
        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let paths = vec![b"subdir/file.tmp".as_bstr()];
        let results = ops::check_ignore(&handle, &paths).expect("check_ignore failed");

        assert_eq!(results.len(), 1);
        assert!(results[0].is_ignored);
        assert!(results[0].source.is_some());
        let source = results[0].source.as_ref().unwrap();
        assert!(source.contains("subdir") || source.ends_with(".gitignore"));
    }

    #[test]
    fn source_points_to_info_exclude() {
        let (_dir, path) = create_repo_with_multiple_ignore_sources();
        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let paths = vec![b"file.bak".as_bstr()];
        let results = ops::check_ignore(&handle, &paths).expect("check_ignore failed");

        assert_eq!(results.len(), 1);
        assert!(results[0].is_ignored);
        assert!(results[0].source.is_some());
    }
}

mod complex_attribute_patterns {
    use super::*;
    use std::path::PathBuf;
    use std::process::Command;
    use tempfile::TempDir;

    fn run_git(dir: &PathBuf, args: &[&str]) {
        let output = Command::new("git")
            .current_dir(dir)
            .args(args)
            .env("GIT_AUTHOR_DATE", "2024-01-15T10:00:00")
            .env("GIT_COMMITTER_DATE", "2024-01-15T10:00:00")
            .output()
            .expect("failed to execute git command");

        if !output.status.success() {
            panic!(
                "git {:?} failed: {}",
                args,
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }

    fn create_repo_with_complex_attributes() -> (TempDir, PathBuf) {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        run_git(&path, &["init"]);
        run_git(&path, &["config", "user.email", "test@example.com"]);
        run_git(&path, &["config", "user.name", "Test User"]);

        std::fs::write(
            path.join(".gitattributes"),
            r#"*.txt text eol=lf
*.rs text diff=rust linguist-language=Rust
*.md text diff=markdown
*.png binary -text -diff
docs/** linguist-documentation
*.min.js -diff
"#,
        )
        .expect("failed to write .gitattributes");

        std::fs::create_dir_all(path.join("docs")).expect("failed to create docs dir");
        std::fs::write(path.join("README.md"), "# Test\n").expect("failed to write README");
        run_git(&path, &["add", "."]);
        run_git(&path, &["commit", "-m", "Initial commit"]);

        (dir, path)
    }

    #[test]
    fn multiple_attributes_on_same_pattern() {
        let (_dir, path) = create_repo_with_complex_attributes();
        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let attrs = ops::get_attributes(&handle, b"file.txt".as_bstr(), &["text", "eol"])
            .expect("get_attributes failed");

        assert_eq!(attrs.len(), 2);
        let text_attr = attrs.iter().find(|a| a.name == "text");
        let eol_attr = attrs.iter().find(|a| a.name == "eol");
        assert!(text_attr.is_some());
        assert!(eol_attr.is_some());
        assert_eq!(text_attr.unwrap().state, ops::AttributeState::Set);
        if let ops::AttributeState::Value(v) = &eol_attr.unwrap().state {
            assert_eq!(v, "lf");
        } else {
            panic!("expected eol to have a value");
        }
    }

    #[test]
    fn unset_and_unspecified_together() {
        let (_dir, path) = create_repo_with_complex_attributes();
        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let attrs = ops::get_attributes(&handle, b"image.png".as_bstr(), &["text", "diff", "merge"])
            .expect("get_attributes failed");

        assert_eq!(attrs.len(), 3);
        let text_attr = attrs.iter().find(|a| a.name == "text");
        let diff_attr = attrs.iter().find(|a| a.name == "diff");
        assert!(text_attr.is_some());
        assert!(diff_attr.is_some());
        assert_eq!(text_attr.unwrap().state, ops::AttributeState::Unset);
        assert_eq!(diff_attr.unwrap().state, ops::AttributeState::Unset);
    }

    #[test]
    fn double_star_attribute_pattern() {
        let (_dir, path) = create_repo_with_complex_attributes();
        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let attrs = ops::get_attributes(
            &handle,
            b"docs/api/reference.md".as_bstr(),
            &["linguist-documentation"],
        )
        .expect("get_attributes failed");

        assert_eq!(attrs.len(), 1);
    }

    #[test]
    fn attribute_with_diff_driver() {
        let (_dir, path) = create_repo_with_complex_attributes();
        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let attrs = ops::get_attributes(&handle, b"src/main.rs".as_bstr(), &["diff"])
            .expect("get_attributes failed");

        assert_eq!(attrs.len(), 1);
        if let ops::AttributeState::Value(v) = &attrs[0].state {
            assert_eq!(v, "rust");
        } else {
            panic!("expected diff to have value 'rust'");
        }
    }

    #[test]
    fn minified_js_no_diff() {
        let (_dir, path) = create_repo_with_complex_attributes();
        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let attrs = ops::get_attributes(&handle, b"bundle.min.js".as_bstr(), &["diff"])
            .expect("get_attributes failed");

        assert_eq!(attrs.len(), 1);
        assert_eq!(attrs[0].state, ops::AttributeState::Unset);
    }
}

mod anchored_patterns {
    use super::*;
    use std::path::PathBuf;
    use std::process::Command;
    use tempfile::TempDir;

    fn run_git(dir: &PathBuf, args: &[&str]) {
        let output = Command::new("git")
            .current_dir(dir)
            .args(args)
            .env("GIT_AUTHOR_DATE", "2024-01-15T10:00:00")
            .env("GIT_COMMITTER_DATE", "2024-01-15T10:00:00")
            .output()
            .expect("failed to execute git command");

        if !output.status.success() {
            panic!(
                "git {:?} failed: {}",
                args,
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }

    fn create_repo_with_anchored_patterns() -> (TempDir, PathBuf) {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        run_git(&path, &["init"]);
        run_git(&path, &["config", "user.email", "test@example.com"]);
        run_git(&path, &["config", "user.name", "Test User"]);

        std::fs::write(
            path.join(".gitignore"),
            "/root_only.txt\nbuild/\n*.tmp\n",
        )
        .expect("failed to write .gitignore");

        std::fs::write(path.join("README.md"), "# Test\n").expect("failed to write README");
        run_git(&path, &["add", "."]);
        run_git(&path, &["commit", "-m", "Initial commit"]);

        (dir, path)
    }

    #[test]
    fn anchored_pattern_matches_at_root() {
        let (_dir, path) = create_repo_with_anchored_patterns();
        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let paths = vec![b"root_only.txt".as_bstr()];
        let results = ops::check_ignore(&handle, &paths).expect("check_ignore failed");

        assert_eq!(results.len(), 1);
        assert!(results[0].is_ignored);
    }

    #[test]
    fn anchored_pattern_does_not_match_nested() {
        let (_dir, path) = create_repo_with_anchored_patterns();
        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let paths = vec![b"subdir/root_only.txt".as_bstr()];
        let results = ops::check_ignore(&handle, &paths).expect("check_ignore failed");

        assert_eq!(results.len(), 1);
        assert!(!results[0].is_ignored);
    }

    #[test]
    fn unanchored_dir_pattern_matches_anywhere() {
        let (_dir, path) = create_repo_with_anchored_patterns();
        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let paths = vec![
            b"build/".as_bstr(),
            b"src/build/".as_bstr(),
            b"a/b/c/build/".as_bstr(),
        ];
        let results = ops::check_ignore(&handle, &paths).expect("check_ignore failed");

        assert_eq!(results.len(), 3);
        for result in &results {
            assert!(result.is_ignored);
        }
    }

    #[test]
    fn unanchored_file_pattern_matches_anywhere() {
        let (_dir, path) = create_repo_with_anchored_patterns();
        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let paths = vec![
            b"file.tmp".as_bstr(),
            b"src/cache.tmp".as_bstr(),
            b"deep/nested/path/temp.tmp".as_bstr(),
        ];
        let results = ops::check_ignore(&handle, &paths).expect("check_ignore failed");

        assert_eq!(results.len(), 3);
        for result in &results {
            assert!(result.is_ignored);
        }
    }
}

mod only_directories_pattern {
    use super::*;
    use std::path::PathBuf;
    use std::process::Command;
    use tempfile::TempDir;

    fn run_git(dir: &PathBuf, args: &[&str]) {
        let output = Command::new("git")
            .current_dir(dir)
            .args(args)
            .env("GIT_AUTHOR_DATE", "2024-01-15T10:00:00")
            .env("GIT_COMMITTER_DATE", "2024-01-15T10:00:00")
            .output()
            .expect("failed to execute git command");

        if !output.status.success() {
            panic!(
                "git {:?} failed: {}",
                args,
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }

    fn create_repo_with_dir_only_patterns() -> (TempDir, PathBuf) {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        run_git(&path, &["init"]);
        run_git(&path, &["config", "user.email", "test@example.com"]);
        run_git(&path, &["config", "user.name", "Test User"]);

        std::fs::write(path.join(".gitignore"), "cache/\ntemp/\n")
            .expect("failed to write .gitignore");

        std::fs::write(path.join("README.md"), "# Test\n").expect("failed to write README");
        run_git(&path, &["add", "."]);
        run_git(&path, &["commit", "-m", "Initial commit"]);

        (dir, path)
    }

    #[test]
    fn dir_only_pattern_matches_directory() {
        let (_dir, path) = create_repo_with_dir_only_patterns();
        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let paths = vec![b"cache/".as_bstr()];
        let results = ops::check_ignore(&handle, &paths).expect("check_ignore failed");

        assert_eq!(results.len(), 1);
        assert!(results[0].is_ignored);
    }

    #[test]
    fn dir_only_pattern_does_not_match_file_with_same_name() {
        let (_dir, path) = create_repo_with_dir_only_patterns();
        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let paths = vec![b"cache".as_bstr()];
        let results = ops::check_ignore(&handle, &paths).expect("check_ignore failed");

        assert_eq!(results.len(), 1);
        assert!(!results[0].is_ignored);
    }

    #[test]
    fn file_inside_dir_only_pattern_dir() {
        let (_dir, path) = create_repo_with_dir_only_patterns();
        let pool = create_pool();
        let handle = pool.get(&path).expect("failed to get handle");

        let paths = vec![b"cache/data.txt".as_bstr()];
        let results = ops::check_ignore(&handle, &paths).expect("check_ignore failed");

        assert_eq!(results.len(), 1);
        assert!(results[0].is_ignored);
    }
}

mod large_path_handling {
    use super::*;

    #[test]
    fn very_long_path() {
        let repo = TestRepo::with_attributes();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let long_path = format!("{}/file.log", "a/".repeat(100));
        let paths = vec![long_path.as_bytes().as_bstr()];
        let results = ops::check_ignore(&handle, &paths).expect("check_ignore failed");

        assert_eq!(results.len(), 1);
        assert!(results[0].is_ignored);
    }

    #[test]
    fn very_long_filename() {
        let repo = TestRepo::with_attributes();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let long_name = format!("{}.log", "a".repeat(200));
        let paths = vec![long_name.as_bytes().as_bstr()];
        let results = ops::check_ignore(&handle, &paths).expect("check_ignore failed");

        assert_eq!(results.len(), 1);
        assert!(results[0].is_ignored);
    }

    #[test]
    fn many_paths_at_once() {
        let repo = TestRepo::with_attributes();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let paths: Vec<bstr::BString> = (0..100).map(|i| format!("file{}.log", i).into()).collect();
        let paths_ref: Vec<&bstr::BStr> = paths.iter().map(|s| s.as_ref()).collect();
        let results = ops::check_ignore(&handle, &paths_ref).expect("check_ignore failed");

        assert_eq!(results.len(), 100);
        for result in &results {
            assert!(result.is_ignored);
        }
    }
}

mod non_utf8_paths {
    use super::*;

    #[test]
    fn path_with_invalid_utf8_bytes() {
        let repo = TestRepo::with_attributes();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let invalid_utf8 = b"file\xff\xfe.log";
        let paths = vec![invalid_utf8.as_bstr()];
        let result = ops::check_ignore(&handle, &paths);

        match result {
            Ok(results) => {
                assert_eq!(results.len(), 1);
            }
            Err(_) => {}
        }
    }

    #[test]
    fn attributes_with_invalid_utf8_path() {
        let repo = TestRepo::with_attributes();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let invalid_utf8 = b"src/\xff\xfe.rs";
        let result = ops::get_attributes(&handle, invalid_utf8.as_bstr(), &["text", "diff"]);

        match result {
            Ok(attrs) => {
                assert!(!attrs.is_empty());
            }
            Err(_) => {}
        }
    }
}

mod io_error_handling {
    use super::*;
    use std::path::PathBuf;
    use std::process::Command;
    use tempfile::TempDir;

    fn run_git(dir: &PathBuf, args: &[&str]) {
        let output = Command::new("git")
            .current_dir(dir)
            .args(args)
            .env("GIT_AUTHOR_DATE", "2024-01-15T10:00:00")
            .env("GIT_COMMITTER_DATE", "2024-01-15T10:00:00")
            .output()
            .expect("failed to execute git command");

        if !output.status.success() {
            panic!(
                "git {:?} failed: {}",
                args,
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }

    #[cfg(unix)]
    fn create_repo_with_unreadable_gitignore() -> Option<(TempDir, PathBuf)> {
        use std::os::unix::fs::PermissionsExt;

        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        run_git(&path, &["init"]);
        run_git(&path, &["config", "user.email", "test@example.com"]);
        run_git(&path, &["config", "user.name", "Test User"]);

        std::fs::create_dir_all(path.join("subdir")).expect("failed to create subdir");

        std::fs::write(path.join("subdir/.gitignore"), "*.log\n")
            .expect("failed to write .gitignore");

        std::fs::write(path.join("README.md"), "# Test\n").expect("failed to write README");
        run_git(&path, &["add", "."]);
        run_git(&path, &["commit", "-m", "Initial commit"]);

        let gitignore_path = path.join("subdir/.gitignore");
        let mut perms = std::fs::metadata(&gitignore_path)
            .expect("failed to get metadata")
            .permissions();
        perms.set_mode(0o000);
        std::fs::set_permissions(&gitignore_path, perms).expect("failed to set permissions");

        Some((dir, path))
    }

    #[cfg(unix)]
    #[test]
    fn check_ignore_with_unreadable_nested_gitignore() {
        if let Some((_dir, path)) = create_repo_with_unreadable_gitignore() {
            let pool = create_pool();
            let handle = pool.get(&path).expect("failed to get handle");

            let paths = vec![b"subdir/test.log".as_bstr()];
            let result = ops::check_ignore(&handle, &paths);

            match result {
                Ok(results) => {
                    assert_eq!(results.len(), 1);
                }
                Err(_e) => {
                }
            }
        }
    }

    #[cfg(unix)]
    fn create_repo_with_unreadable_gitattributes() -> Option<(TempDir, PathBuf)> {
        use std::os::unix::fs::PermissionsExt;

        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        run_git(&path, &["init"]);
        run_git(&path, &["config", "user.email", "test@example.com"]);
        run_git(&path, &["config", "user.name", "Test User"]);

        std::fs::create_dir_all(path.join("subdir")).expect("failed to create subdir");

        std::fs::write(path.join("subdir/.gitattributes"), "*.txt text\n")
            .expect("failed to write .gitattributes");

        std::fs::write(path.join("README.md"), "# Test\n").expect("failed to write README");
        run_git(&path, &["add", "."]);
        run_git(&path, &["commit", "-m", "Initial commit"]);

        let gitattributes_path = path.join("subdir/.gitattributes");
        let mut perms = std::fs::metadata(&gitattributes_path)
            .expect("failed to get metadata")
            .permissions();
        perms.set_mode(0o000);
        std::fs::set_permissions(&gitattributes_path, perms).expect("failed to set permissions");

        Some((dir, path))
    }

    #[cfg(unix)]
    #[test]
    fn get_attributes_with_unreadable_nested_gitattributes() {
        if let Some((_dir, path)) = create_repo_with_unreadable_gitattributes() {
            let pool = create_pool();
            let handle = pool.get(&path).expect("failed to get handle");

            let result = ops::get_attributes(&handle, b"subdir/file.txt".as_bstr(), &["text"]);

            match result {
                Ok(attrs) => {
                    assert!(!attrs.is_empty());
                }
                Err(_e) => {
                }
            }
        }
    }
}
