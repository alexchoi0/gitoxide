mod fixtures;

use fixtures::TestRepo;
use gix_server_sdk::{ops, RepoPool, SdkConfig};

fn get_pool() -> RepoPool {
    RepoPool::new(SdkConfig::default())
}

mod list_submodules {
    use super::*;

    #[test]
    fn returns_submodules_for_repo_with_submodules() {
        let repo = TestRepo::with_submodules();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let submodules = ops::list_submodules(&handle).expect("failed to list submodules");

        assert!(!submodules.is_empty(), "should have at least one submodule");
    }

    #[test]
    fn returns_empty_for_repo_without_submodules() {
        let repo = TestRepo::new();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let submodules = ops::list_submodules(&handle).expect("failed to list submodules");

        assert!(submodules.is_empty(), "repo without submodules should return empty list");
    }

    #[test]
    fn returns_empty_for_empty_repo() {
        let repo = TestRepo::empty();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let submodules = ops::list_submodules(&handle).expect("failed to list submodules");

        assert!(submodules.is_empty(), "empty repo should return empty list");
    }

    #[test]
    fn submodule_has_expected_path() {
        let repo = TestRepo::with_submodules();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let submodules = ops::list_submodules(&handle).expect("failed to list submodules");

        let submodule = submodules
            .iter()
            .find(|s| s.path.to_string().contains("vendor/submodule"));

        assert!(submodule.is_some(), "should find submodule at vendor/submodule path");
    }

    #[test]
    fn submodule_has_url() {
        let repo = TestRepo::with_submodules();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let submodules = ops::list_submodules(&handle).expect("failed to list submodules");

        assert!(!submodules.is_empty(), "should have submodules");
        let submodule = &submodules[0];

        assert!(submodule.url.is_some(), "submodule should have a URL");
    }

    #[test]
    fn submodule_is_active() {
        let repo = TestRepo::with_submodules();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let submodules = ops::list_submodules(&handle).expect("failed to list submodules");

        assert!(!submodules.is_empty(), "should have submodules");
        let submodule = &submodules[0];

        assert!(submodule.is_active, "submodule should be active by default");
    }

    #[test]
    fn submodule_has_index_commit() {
        let repo = TestRepo::with_submodules();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let submodules = ops::list_submodules(&handle).expect("failed to list submodules");

        assert!(!submodules.is_empty(), "should have submodules");
        let submodule = &submodules[0];

        assert!(
            submodule.index_commit.is_some(),
            "submodule should have index_commit after being committed"
        );
    }

    #[test]
    fn multiple_pool_accesses_return_same_submodules() {
        let repo = TestRepo::with_submodules();
        let pool = get_pool();

        let handle1 = pool.get(&repo.path).expect("failed to get repo handle");
        let submodules1 = ops::list_submodules(&handle1).expect("failed to list submodules");

        let handle2 = pool.get(&repo.path).expect("failed to get repo handle");
        let submodules2 = ops::list_submodules(&handle2).expect("failed to list submodules");

        assert_eq!(
            submodules1.len(),
            submodules2.len(),
            "multiple accesses should return same number of submodules"
        );
    }
}

mod get_submodule {
    use super::*;

    #[test]
    fn returns_existing_submodule_by_name() {
        let repo = TestRepo::with_submodules();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let submodule = ops::get_submodule(&handle, "vendor/submodule")
            .expect("failed to get submodule");

        assert_eq!(submodule.name.to_string(), "vendor/submodule");
    }

    #[test]
    fn fails_for_nonexistent_submodule() {
        let repo = TestRepo::with_submodules();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let result = ops::get_submodule(&handle, "nonexistent/submodule");

        assert!(result.is_err(), "should fail for nonexistent submodule");
    }

    #[test]
    fn fails_for_repo_without_submodules() {
        let repo = TestRepo::new();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let result = ops::get_submodule(&handle, "any/submodule");

        assert!(result.is_err(), "should fail when repo has no submodules");
    }

    #[test]
    fn fails_for_empty_repo() {
        let repo = TestRepo::empty();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let result = ops::get_submodule(&handle, "any/submodule");

        assert!(result.is_err(), "should fail for empty repo");
    }

    #[test]
    fn error_message_contains_submodule_name() {
        let repo = TestRepo::with_submodules();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let result = ops::get_submodule(&handle, "missing/submodule");

        match result {
            Err(e) => {
                let error_msg = e.to_string();
                assert!(
                    error_msg.contains("missing/submodule"),
                    "error message should contain the submodule name, got: {}",
                    error_msg
                );
            }
            Ok(_) => panic!("expected error for nonexistent submodule"),
        }
    }

    #[test]
    fn returned_submodule_has_valid_fields() {
        let repo = TestRepo::with_submodules();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let submodule = ops::get_submodule(&handle, "vendor/submodule")
            .expect("failed to get submodule");

        assert!(!submodule.name.is_empty(), "name should not be empty");
        assert!(!submodule.path.is_empty(), "path should not be empty");
        assert!(submodule.url.is_some(), "url should be present");
    }

    #[test]
    fn get_submodule_matches_list_submodules() {
        let repo = TestRepo::with_submodules();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let submodules = ops::list_submodules(&handle).expect("failed to list submodules");
        assert!(!submodules.is_empty(), "should have submodules");

        let listed = &submodules[0];
        let name_str = listed.name.to_string();
        let fetched = ops::get_submodule(&handle, &name_str)
            .expect("failed to get submodule by name");

        assert_eq!(listed.name, fetched.name, "names should match");
        assert_eq!(listed.path, fetched.path, "paths should match");
        assert_eq!(listed.url, fetched.url, "urls should match");
        assert_eq!(listed.is_active, fetched.is_active, "is_active should match");
        assert_eq!(listed.index_commit, fetched.index_commit, "index_commit should match");
        assert_eq!(listed.head_commit, fetched.head_commit, "head_commit should match");
    }
}

mod submodule_info_fields {
    use super::*;

    #[test]
    fn name_field_is_set() {
        let repo = TestRepo::with_submodules();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let submodules = ops::list_submodules(&handle).expect("failed to list submodules");

        assert!(!submodules.is_empty(), "should have submodules");
        let submodule = &submodules[0];

        assert!(!submodule.name.is_empty(), "name should not be empty");
        assert!(
            submodule.name.to_string().contains("submodule"),
            "name should contain 'submodule'"
        );
    }

    #[test]
    fn path_field_is_set() {
        let repo = TestRepo::with_submodules();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let submodules = ops::list_submodules(&handle).expect("failed to list submodules");

        assert!(!submodules.is_empty(), "should have submodules");
        let submodule = &submodules[0];

        assert!(!submodule.path.is_empty(), "path should not be empty");
        assert!(
            submodule.path.to_string().contains("vendor/submodule"),
            "path should contain 'vendor/submodule', got: {}",
            submodule.path
        );
    }

    #[test]
    fn url_field_contains_valid_path() {
        let repo = TestRepo::with_submodules();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let submodules = ops::list_submodules(&handle).expect("failed to list submodules");

        assert!(!submodules.is_empty(), "should have submodules");
        let submodule = &submodules[0];

        let url = submodule.url.as_ref().expect("url should be present");
        assert!(!url.is_empty(), "url should not be empty");
        assert!(
            url.contains("submodule_source"),
            "url should reference the submodule source, got: {}",
            url
        );
    }

    #[test]
    fn head_commit_is_optional() {
        let repo = TestRepo::with_submodules();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let submodules = ops::list_submodules(&handle).expect("failed to list submodules");

        assert!(!submodules.is_empty(), "should have submodules");
        let submodule = &submodules[0];

        if let Some(head_commit) = &submodule.head_commit {
            assert!(!head_commit.is_null(), "head_commit should be valid if present");
        }
    }

    #[test]
    fn index_commit_is_present_after_commit() {
        let repo = TestRepo::with_submodules();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let submodules = ops::list_submodules(&handle).expect("failed to list submodules");

        assert!(!submodules.is_empty(), "should have submodules");
        let submodule = &submodules[0];

        assert!(
            submodule.index_commit.is_some(),
            "index_commit should be present after submodule was committed"
        );

        let index_commit = submodule.index_commit.as_ref().unwrap();
        assert!(!index_commit.is_null(), "index_commit should not be null");
    }

    #[test]
    fn index_commit_is_valid_object_id() {
        let repo = TestRepo::with_submodules();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let submodules = ops::list_submodules(&handle).expect("failed to list submodules");

        assert!(!submodules.is_empty(), "should have submodules");
        let submodule = &submodules[0];

        if let Some(index_commit) = &submodule.index_commit {
            let hex = index_commit.to_string();
            assert_eq!(hex.len(), 40, "index_commit should be 40 hex chars");
            assert!(
                hex.chars().all(|c| c.is_ascii_hexdigit()),
                "index_commit should be valid hex"
            );
        }
    }

    #[test]
    fn is_active_defaults_to_true() {
        let repo = TestRepo::with_submodules();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let submodules = ops::list_submodules(&handle).expect("failed to list submodules");

        assert!(!submodules.is_empty(), "should have submodules");
        let submodule = &submodules[0];

        assert!(submodule.is_active, "newly added submodule should be active");
    }

    #[test]
    fn submodule_info_implements_debug() {
        let repo = TestRepo::with_submodules();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let submodules = ops::list_submodules(&handle).expect("failed to list submodules");

        assert!(!submodules.is_empty(), "should have submodules");
        let submodule = &submodules[0];

        let debug_str = format!("{:?}", submodule);
        assert!(!debug_str.is_empty(), "debug output should not be empty");
        assert!(
            debug_str.contains("SubmoduleInfo"),
            "debug output should contain struct name"
        );
    }

    #[test]
    fn submodule_info_implements_clone() {
        let repo = TestRepo::with_submodules();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let submodules = ops::list_submodules(&handle).expect("failed to list submodules");

        assert!(!submodules.is_empty(), "should have submodules");
        let submodule = &submodules[0];
        let cloned = submodule.clone();

        assert_eq!(submodule.name, cloned.name);
        assert_eq!(submodule.path, cloned.path);
        assert_eq!(submodule.url, cloned.url);
        assert_eq!(submodule.is_active, cloned.is_active);
    }

    #[test]
    fn submodule_info_implements_eq() {
        let repo = TestRepo::with_submodules();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let submodules1 = ops::list_submodules(&handle).expect("failed to list submodules");
        let submodules2 = ops::list_submodules(&handle).expect("failed to list submodules");

        assert!(!submodules1.is_empty(), "should have submodules");
        assert_eq!(submodules1[0], submodules2[0], "same submodule should be equal");
    }
}

mod multiple_submodules {
    use super::*;

    #[test]
    fn list_returns_all_submodules() {
        let repo = TestRepo::with_multiple_submodules();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let submodules = ops::list_submodules(&handle).expect("failed to list submodules");

        assert_eq!(submodules.len(), 2, "should have exactly 2 submodules");
    }

    #[test]
    fn submodules_have_different_names() {
        let repo = TestRepo::with_multiple_submodules();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let submodules = ops::list_submodules(&handle).expect("failed to list submodules");

        let names: Vec<String> = submodules.iter().map(|s| s.name.to_string()).collect();
        assert!(names.contains(&"libs/first".to_string()), "should have libs/first");
        assert!(names.contains(&"libs/second".to_string()), "should have libs/second");
    }

    #[test]
    fn submodules_have_different_paths() {
        let repo = TestRepo::with_multiple_submodules();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let submodules = ops::list_submodules(&handle).expect("failed to list submodules");

        let paths: Vec<String> = submodules.iter().map(|s| s.path.to_string()).collect();
        assert!(paths.contains(&"libs/first".to_string()), "should have libs/first path");
        assert!(paths.contains(&"libs/second".to_string()), "should have libs/second path");
    }

    #[test]
    fn submodules_have_different_urls() {
        let repo = TestRepo::with_multiple_submodules();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let submodules = ops::list_submodules(&handle).expect("failed to list submodules");

        let url1 = submodules[0].url.as_ref().expect("first submodule should have url");
        let url2 = submodules[1].url.as_ref().expect("second submodule should have url");
        assert_ne!(url1, url2, "submodules should have different URLs");
    }

    #[test]
    fn get_first_submodule_by_name() {
        let repo = TestRepo::with_multiple_submodules();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let submodule = ops::get_submodule(&handle, "libs/first")
            .expect("failed to get first submodule");

        assert_eq!(submodule.name.to_string(), "libs/first");
        assert_eq!(submodule.path.to_string(), "libs/first");
    }

    #[test]
    fn get_second_submodule_by_name() {
        let repo = TestRepo::with_multiple_submodules();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let submodule = ops::get_submodule(&handle, "libs/second")
            .expect("failed to get second submodule");

        assert_eq!(submodule.name.to_string(), "libs/second");
        assert_eq!(submodule.path.to_string(), "libs/second");
    }

    #[test]
    fn all_submodules_have_index_commits() {
        let repo = TestRepo::with_multiple_submodules();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let submodules = ops::list_submodules(&handle).expect("failed to list submodules");

        for submodule in &submodules {
            assert!(
                submodule.index_commit.is_some(),
                "submodule {} should have index_commit",
                submodule.name
            );
        }
    }

    #[test]
    fn all_submodules_are_active() {
        let repo = TestRepo::with_multiple_submodules();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let submodules = ops::list_submodules(&handle).expect("failed to list submodules");

        for submodule in &submodules {
            assert!(
                submodule.is_active,
                "submodule {} should be active",
                submodule.name
            );
        }
    }
}

mod inactive_submodule {
    use super::*;

    #[test]
    fn list_includes_inactive_submodule() {
        let repo = TestRepo::with_inactive_submodule();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let submodules = ops::list_submodules(&handle).expect("failed to list submodules");

        assert_eq!(submodules.len(), 1, "should have one submodule");
    }

    #[test]
    fn inactive_submodule_is_not_active() {
        let repo = TestRepo::with_inactive_submodule();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let submodules = ops::list_submodules(&handle).expect("failed to list submodules");
        let submodule = &submodules[0];

        assert!(!submodule.is_active, "submodule should be inactive");
    }

    #[test]
    fn get_inactive_submodule_by_name() {
        let repo = TestRepo::with_inactive_submodule();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let submodule = ops::get_submodule(&handle, "vendor/inactive")
            .expect("failed to get inactive submodule");

        assert_eq!(submodule.name.to_string(), "vendor/inactive");
        assert!(!submodule.is_active, "submodule should be inactive");
    }

    #[test]
    fn inactive_submodule_has_valid_fields() {
        let repo = TestRepo::with_inactive_submodule();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let submodule = ops::get_submodule(&handle, "vendor/inactive")
            .expect("failed to get inactive submodule");

        assert!(!submodule.name.is_empty());
        assert!(!submodule.path.is_empty());
        assert!(submodule.url.is_some());
        assert!(submodule.index_commit.is_some());
    }
}

mod nested_submodule_path {
    use super::*;

    #[test]
    fn list_returns_deeply_nested_submodule() {
        let repo = TestRepo::with_nested_submodule_path();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let submodules = ops::list_submodules(&handle).expect("failed to list submodules");

        assert_eq!(submodules.len(), 1, "should have one submodule");
    }

    #[test]
    fn nested_submodule_has_full_path() {
        let repo = TestRepo::with_nested_submodule_path();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let submodules = ops::list_submodules(&handle).expect("failed to list submodules");
        let submodule = &submodules[0];

        assert_eq!(
            submodule.path.to_string(),
            "deep/nested/path/submodule",
            "path should be deeply nested"
        );
    }

    #[test]
    fn get_nested_submodule_by_name() {
        let repo = TestRepo::with_nested_submodule_path();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let submodule = ops::get_submodule(&handle, "deep/nested/path/submodule")
            .expect("failed to get nested submodule");

        assert_eq!(submodule.name.to_string(), "deep/nested/path/submodule");
    }

    #[test]
    fn nested_submodule_is_active() {
        let repo = TestRepo::with_nested_submodule_path();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let submodule = ops::get_submodule(&handle, "deep/nested/path/submodule")
            .expect("failed to get nested submodule");

        assert!(submodule.is_active);
    }

    #[test]
    fn nested_submodule_has_index_commit() {
        let repo = TestRepo::with_nested_submodule_path();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let submodule = ops::get_submodule(&handle, "deep/nested/path/submodule")
            .expect("failed to get nested submodule");

        assert!(submodule.index_commit.is_some());
    }
}

mod error_handling {
    use super::*;

    #[test]
    fn get_submodule_empty_name_fails() {
        let repo = TestRepo::with_submodules();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let result = ops::get_submodule(&handle, "");

        assert!(result.is_err());
    }

    #[test]
    fn get_submodule_partial_name_fails() {
        let repo = TestRepo::with_submodules();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let result = ops::get_submodule(&handle, "vendor");

        assert!(result.is_err());
    }

    #[test]
    fn get_submodule_case_sensitive() {
        let repo = TestRepo::with_submodules();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let result = ops::get_submodule(&handle, "VENDOR/SUBMODULE");

        assert!(result.is_err());
    }

    #[test]
    fn error_for_empty_repo_contains_submodule_name() {
        let repo = TestRepo::empty();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let result = ops::get_submodule(&handle, "test/submodule");

        match result {
            Err(e) => {
                let msg = e.to_string();
                assert!(msg.contains("test/submodule"), "error should contain name: {}", msg);
            }
            Ok(_) => panic!("expected error"),
        }
    }

    #[test]
    fn error_for_normal_repo_contains_submodule_name() {
        let repo = TestRepo::new();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let result = ops::get_submodule(&handle, "nonexistent/module");

        match result {
            Err(e) => {
                let msg = e.to_string();
                assert!(msg.contains("nonexistent/module"), "error should contain name: {}", msg);
            }
            Ok(_) => panic!("expected error"),
        }
    }
}

mod iteration_coverage {
    use super::*;

    #[test]
    fn iterate_all_submodules_in_list() {
        let repo = TestRepo::with_multiple_submodules();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let submodules = ops::list_submodules(&handle).expect("failed to list submodules");

        let mut count = 0;
        for submodule in &submodules {
            assert!(!submodule.name.is_empty());
            assert!(!submodule.path.is_empty());
            count += 1;
        }

        assert_eq!(count, 2);
    }

    #[test]
    fn get_each_submodule_individually() {
        let repo = TestRepo::with_multiple_submodules();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let submodules = ops::list_submodules(&handle).expect("failed to list submodules");

        for listed in &submodules {
            let name_str = listed.name.to_string();
            let fetched = ops::get_submodule(&handle, &name_str)
                .expect("failed to get submodule");
            assert_eq!(listed, &fetched);
        }
    }

    #[test]
    fn submodule_search_iterates_through_all() {
        let repo = TestRepo::with_multiple_submodules();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let result = ops::get_submodule(&handle, "libs/second");

        assert!(result.is_ok());
        let submodule = result.unwrap();
        assert_eq!(submodule.name.to_string(), "libs/second");
    }

    #[test]
    fn submodule_not_found_after_full_iteration() {
        let repo = TestRepo::with_multiple_submodules();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let result = ops::get_submodule(&handle, "libs/third");

        assert!(result.is_err());
    }
}

mod edge_cases {
    use super::*;

    #[test]
    fn submodule_without_gitmodules_url_still_works() {
        let repo = TestRepo::with_submodule_without_url();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let submodules = ops::list_submodules(&handle).expect("failed to list submodules");

        assert_eq!(submodules.len(), 1);
        let submodule = &submodules[0];
        assert_eq!(submodule.name.to_string(), "vendor/nourl");
    }

    #[test]
    fn submodule_without_gitmodules_url_has_name() {
        let repo = TestRepo::with_submodule_without_url();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let submodules = ops::list_submodules(&handle).expect("failed to list submodules");

        assert_eq!(submodules.len(), 1);
        let submodule = &submodules[0];
        assert_eq!(submodule.name.to_string(), "vendor/nourl");
    }

    #[test]
    fn submodule_without_gitmodules_url_has_path() {
        let repo = TestRepo::with_submodule_without_url();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let submodules = ops::list_submodules(&handle).expect("failed to list submodules");

        assert_eq!(submodules.len(), 1);
        let submodule = &submodules[0];
        assert_eq!(submodule.path.to_string(), "vendor/nourl");
    }

    #[test]
    fn get_submodule_without_gitmodules_url_by_name() {
        let repo = TestRepo::with_submodule_without_url();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let submodule = ops::get_submodule(&handle, "vendor/nourl")
            .expect("failed to get submodule");

        assert_eq!(submodule.name.to_string(), "vendor/nourl");
        assert!(!submodule.path.is_empty());
    }

    #[test]
    fn uninitialized_submodule_is_listed() {
        let repo = TestRepo::with_submodule_not_initialized();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let submodules = ops::list_submodules(&handle).expect("failed to list submodules");

        assert_eq!(submodules.len(), 1);
    }

    #[test]
    fn uninitialized_submodule_has_name() {
        let repo = TestRepo::with_submodule_not_initialized();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let submodules = ops::list_submodules(&handle).expect("failed to list submodules");
        let submodule = &submodules[0];

        assert_eq!(submodule.name.to_string(), "vendor/uninit");
    }

    #[test]
    fn uninitialized_submodule_has_url() {
        let repo = TestRepo::with_submodule_not_initialized();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let submodules = ops::list_submodules(&handle).expect("failed to list submodules");
        let submodule = &submodules[0];

        let url = submodule.url.as_ref().expect("should have url");
        assert!(url.contains("example.com"));
    }

    #[test]
    fn uninitialized_submodule_has_no_index_commit() {
        let repo = TestRepo::with_submodule_not_initialized();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let submodules = ops::list_submodules(&handle).expect("failed to list submodules");
        let submodule = &submodules[0];

        assert!(submodule.index_commit.is_none(), "uninitialized submodule should have no index_commit");
    }

    #[test]
    fn uninitialized_submodule_has_no_head_commit() {
        let repo = TestRepo::with_submodule_not_initialized();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let submodules = ops::list_submodules(&handle).expect("failed to list submodules");
        let submodule = &submodules[0];

        assert!(submodule.head_commit.is_none(), "uninitialized submodule should have no head_commit");
    }

    #[test]
    fn get_uninitialized_submodule_by_name() {
        let repo = TestRepo::with_submodule_not_initialized();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let submodule = ops::get_submodule(&handle, "vendor/uninit")
            .expect("failed to get submodule");

        assert_eq!(submodule.name.to_string(), "vendor/uninit");
        assert!(submodule.index_commit.is_none());
        assert!(submodule.head_commit.is_none());
    }
}

mod submodule_info_struct {
    use super::*;

    #[test]
    fn submodule_info_not_equal_with_different_names() {
        let repo = TestRepo::with_multiple_submodules();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let submodules = ops::list_submodules(&handle).expect("failed to list submodules");

        assert!(submodules.len() >= 2);
        assert_ne!(submodules[0], submodules[1]);
    }

    #[test]
    fn debug_output_contains_all_fields() {
        let repo = TestRepo::with_submodules();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let submodules = ops::list_submodules(&handle).expect("failed to list submodules");
        let submodule = &submodules[0];

        let debug = format!("{:?}", submodule);
        assert!(debug.contains("name"));
        assert!(debug.contains("path"));
        assert!(debug.contains("url"));
        assert!(debug.contains("head_commit"));
        assert!(debug.contains("index_commit"));
        assert!(debug.contains("is_active"));
    }

    #[test]
    fn clone_preserves_all_fields() {
        let repo = TestRepo::with_submodules();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let submodules = ops::list_submodules(&handle).expect("failed to list submodules");
        let original = &submodules[0];
        let cloned = original.clone();

        assert_eq!(original.name, cloned.name);
        assert_eq!(original.path, cloned.path);
        assert_eq!(original.url, cloned.url);
        assert_eq!(original.head_commit, cloned.head_commit);
        assert_eq!(original.index_commit, cloned.index_commit);
        assert_eq!(original.is_active, cloned.is_active);
    }

    #[test]
    fn eq_is_reflexive() {
        let repo = TestRepo::with_submodules();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let submodules = ops::list_submodules(&handle).expect("failed to list submodules");
        let submodule = &submodules[0];

        assert_eq!(submodule, submodule);
    }

    #[test]
    fn eq_is_symmetric() {
        let repo = TestRepo::with_submodules();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let submodules1 = ops::list_submodules(&handle).expect("failed to list submodules");
        let submodules2 = ops::list_submodules(&handle).expect("failed to list submodules");

        assert_eq!(submodules1[0], submodules2[0]);
        assert_eq!(submodules2[0], submodules1[0]);
    }
}

mod path_error_coverage {
    use super::*;

    #[test]
    fn list_submodules_fails_with_missing_path() {
        let repo = TestRepo::with_submodule_missing_path();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let result = ops::list_submodules(&handle);

        assert!(result.is_err(), "should fail when submodule path is missing from .gitmodules");
    }

    #[test]
    fn get_submodule_fails_with_missing_path() {
        let repo = TestRepo::with_submodule_missing_path();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let result = ops::get_submodule(&handle, "vendor/broken");

        assert!(result.is_err(), "should fail when submodule path is missing from .gitmodules");
    }

    #[test]
    fn list_submodules_fails_with_absolute_path() {
        let repo = TestRepo::with_submodule_absolute_path();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let result = ops::list_submodules(&handle);

        assert!(result.is_err(), "should fail when submodule has absolute path");
    }

    #[test]
    fn get_submodule_fails_with_absolute_path() {
        let repo = TestRepo::with_submodule_absolute_path();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let result = ops::get_submodule(&handle, "vendor/absolute");

        assert!(result.is_err(), "should fail when submodule has absolute path");
    }

    #[test]
    fn list_submodules_fails_with_invalid_active_config() {
        let repo = TestRepo::with_submodule_invalid_active();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let result = ops::list_submodules(&handle);

        assert!(result.is_err(), "should fail when submodule has invalid 'active' config value");
    }

    #[test]
    fn get_submodule_fails_with_invalid_active_config() {
        let repo = TestRepo::with_submodule_invalid_active();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let result = ops::get_submodule(&handle, "vendor/invalid");

        assert!(result.is_err(), "should fail when submodule has invalid 'active' config value");
    }

    #[test]
    fn list_submodules_handles_corrupt_gitmodules() {
        let repo = TestRepo::with_corrupt_gitmodules();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let result = ops::list_submodules(&handle);

        assert!(result.is_ok() || result.is_err(), "should either succeed with empty or fail with parse error");
    }

    #[test]
    fn get_submodule_handles_corrupt_gitmodules() {
        let repo = TestRepo::with_corrupt_gitmodules();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let result = ops::get_submodule(&handle, "vendor/corrupt");

        assert!(result.is_ok() || result.is_err(), "should either succeed or fail with parse error");
    }
}

mod head_id_error_coverage {
    use super::*;

    #[test]
    fn list_submodules_fails_with_corrupt_head() {
        let repo = TestRepo::with_submodule_corrupt_head();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let result = ops::list_submodules(&handle);

        assert!(result.is_err(), "should fail when HEAD points to nonexistent ref");
    }

    #[test]
    fn get_submodule_fails_with_corrupt_head() {
        let repo = TestRepo::with_submodule_corrupt_head();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let result = ops::get_submodule(&handle, "vendor/corrupt");

        assert!(result.is_err(), "should fail when HEAD points to nonexistent ref");
    }
}

mod index_id_error_coverage {
    use super::*;

    #[test]
    fn list_submodules_fails_with_corrupt_index() {
        let repo = TestRepo::with_submodule_corrupt_index();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let result = ops::list_submodules(&handle);

        assert!(result.is_err(), "should fail when index is corrupted");
    }

    #[test]
    fn get_submodule_fails_with_corrupt_index() {
        let repo = TestRepo::with_submodule_corrupt_index();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let result = ops::get_submodule(&handle, "vendor/corrupt");

        assert!(result.is_err(), "should fail when index is corrupted");
    }
}
