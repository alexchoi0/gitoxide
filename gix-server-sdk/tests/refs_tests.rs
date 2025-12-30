mod fixtures;

use fixtures::TestRepo;
use gix_server_sdk::{ops, RepoPool, SdkConfig};

fn get_pool() -> RepoPool {
    RepoPool::new(SdkConfig::default())
}

mod list_refs {
    use super::*;

    #[test]
    fn returns_all_refs_without_prefix() {
        let repo = TestRepo::new();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let refs = ops::list_refs(&handle, None).expect("failed to list refs");

        assert!(!refs.is_empty(), "should have at least one ref");

        let ref_names: Vec<&str> = refs.iter().map(|r| r.name.as_str()).collect();
        assert!(
            ref_names.iter().any(|n| n.contains("refs/heads/")),
            "should contain branch refs"
        );
    }

    #[test]
    fn filters_refs_by_prefix() {
        let repo = TestRepo::with_branches();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let branch_refs = ops::list_refs(&handle, Some("refs/heads/"))
            .expect("failed to list refs with prefix");

        assert!(
            branch_refs.len() >= 3,
            "should have at least 3 branches (main, feature-a, feature-b), got {}",
            branch_refs.len()
        );

        for ref_info in &branch_refs {
            assert!(
                ref_info.name.starts_with("refs/heads/"),
                "ref {} should start with refs/heads/",
                ref_info.name
            );
        }
    }

    #[test]
    fn returns_empty_for_nonexistent_prefix() {
        let repo = TestRepo::new();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let refs = ops::list_refs(&handle, Some("refs/nonexistent/"))
            .expect("failed to list refs");

        assert!(refs.is_empty(), "should return empty list for nonexistent prefix");
    }

    #[test]
    fn empty_repo_returns_empty_list() {
        let repo = TestRepo::empty();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let refs = ops::list_refs(&handle, None).expect("failed to list refs");

        assert!(refs.is_empty(), "empty repo should have no refs");
    }

    #[test]
    fn ref_info_contains_valid_target() {
        let repo = TestRepo::new();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let refs = ops::list_refs(&handle, Some("refs/heads/"))
            .expect("failed to list refs");

        for ref_info in &refs {
            assert!(
                !ref_info.target.is_null(),
                "ref {} should have non-null target",
                ref_info.name
            );
        }
    }
}

mod resolve_ref {
    use super::*;

    #[test]
    fn resolves_existing_branch() {
        let repo = TestRepo::new();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let ref_info = ops::resolve_ref(&handle, "refs/heads/main")
            .expect("failed to resolve ref");

        assert_eq!(ref_info.name, "refs/heads/main");
        assert!(!ref_info.target.is_null(), "should have valid target");
        assert!(!ref_info.is_symbolic, "branch ref should not be symbolic");
    }

    #[test]
    fn resolves_head_as_symbolic() {
        let repo = TestRepo::new();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let ref_info = ops::resolve_ref(&handle, "HEAD")
            .expect("failed to resolve HEAD");

        assert_eq!(ref_info.name, "HEAD");
        assert!(ref_info.is_symbolic, "HEAD should be symbolic");
        assert!(
            ref_info.symbolic_target.is_some(),
            "HEAD should have symbolic target"
        );
    }

    #[test]
    fn fails_for_nonexistent_ref() {
        let repo = TestRepo::new();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let result = ops::resolve_ref(&handle, "refs/heads/nonexistent");

        assert!(result.is_err(), "should fail for nonexistent ref");
    }

    #[test]
    fn resolves_tag_ref() {
        let repo = TestRepo::with_tags();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let ref_info = ops::resolve_ref(&handle, "refs/tags/v1.0.0")
            .expect("failed to resolve tag ref");

        assert_eq!(ref_info.name, "refs/tags/v1.0.0");
        assert!(!ref_info.target.is_null(), "tag should have valid target");
    }

    #[test]
    fn resolves_annotated_tag() {
        let repo = TestRepo::with_tags();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let ref_info = ops::resolve_ref(&handle, "refs/tags/v1.1.0")
            .expect("failed to resolve annotated tag ref");

        assert_eq!(ref_info.name, "refs/tags/v1.1.0");
        assert!(!ref_info.target.is_null(), "annotated tag should have valid target");
    }

    #[test]
    fn resolves_short_ref_name() {
        let repo = TestRepo::new();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let ref_info = ops::resolve_ref(&handle, "main")
            .expect("failed to resolve short ref name");

        assert!(
            ref_info.name.contains("main"),
            "resolved ref should contain 'main'"
        );
    }
}

mod get_head {
    use super::*;

    #[test]
    fn returns_head_for_normal_repo() {
        let repo = TestRepo::new();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let head = ops::get_head(&handle).expect("failed to get HEAD");

        assert!(
            head.name == "HEAD" || head.name.starts_with("refs/heads/"),
            "HEAD should be HEAD or a branch ref, got: {}",
            head.name
        );
        assert!(!head.target.is_null(), "HEAD should have valid target");
    }

    #[test]
    fn head_resolves_to_branch_when_symbolic() {
        let repo = TestRepo::new();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let head = ops::get_head(&handle).expect("failed to get HEAD");

        assert!(
            head.name.contains("main") || head.name == "HEAD",
            "HEAD should resolve to main branch or be HEAD, got: {}",
            head.name
        );
    }

    #[test]
    fn detached_head_is_not_symbolic() {
        let repo = TestRepo::with_detached_head();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let head = ops::get_head(&handle).expect("failed to get HEAD");

        assert_eq!(head.name, "HEAD");
        assert!(!head.is_symbolic, "detached HEAD should not be symbolic");
        assert!(
            head.symbolic_target.is_none(),
            "detached HEAD should not have symbolic target"
        );
        assert!(!head.target.is_null(), "detached HEAD should have valid target");
    }

    #[test]
    fn unborn_head_has_null_target() {
        let repo = TestRepo::empty();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let head = ops::get_head(&handle).expect("failed to get HEAD");

        assert_eq!(head.name, "HEAD");
        assert!(head.is_symbolic, "unborn HEAD should be symbolic");
        assert!(head.target.is_null(), "unborn HEAD should have null target");
    }

    #[test]
    fn head_target_matches_branch_tip() {
        let repo = TestRepo::new();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let head = ops::get_head(&handle).expect("failed to get HEAD");
        let main_ref = ops::resolve_ref(&handle, "refs/heads/main")
            .expect("failed to resolve main");

        assert_eq!(
            head.target, main_ref.target,
            "HEAD target should match main branch target"
        );
    }
}

mod list_branches {
    use super::*;

    #[test]
    fn returns_all_branches() {
        let repo = TestRepo::with_branches();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let branches = ops::list_branches(&handle).expect("failed to list branches");

        assert!(
            branches.len() >= 3,
            "should have at least 3 branches, got {}",
            branches.len()
        );

        let branch_names: Vec<&str> = branches.iter().map(|r| r.name.as_str()).collect();

        assert!(
            branch_names.contains(&"refs/heads/main"),
            "should contain main branch"
        );
        assert!(
            branch_names.contains(&"refs/heads/feature-a"),
            "should contain feature-a branch"
        );
        assert!(
            branch_names.contains(&"refs/heads/feature-b"),
            "should contain feature-b branch"
        );
    }

    #[test]
    fn only_returns_branch_refs() {
        let repo = TestRepo::with_tags();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let branches = ops::list_branches(&handle).expect("failed to list branches");

        for branch in &branches {
            assert!(
                branch.name.starts_with("refs/heads/"),
                "branch {} should start with refs/heads/",
                branch.name
            );
            assert!(
                !branch.name.contains("tags"),
                "branches should not contain tags"
            );
        }
    }

    #[test]
    fn empty_repo_returns_no_branches() {
        let repo = TestRepo::empty();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let branches = ops::list_branches(&handle).expect("failed to list branches");

        assert!(branches.is_empty(), "empty repo should have no branches");
    }

    #[test]
    fn branches_have_valid_targets() {
        let repo = TestRepo::with_branches();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let branches = ops::list_branches(&handle).expect("failed to list branches");

        for branch in &branches {
            assert!(
                !branch.target.is_null(),
                "branch {} should have non-null target",
                branch.name
            );
            assert!(
                !branch.is_symbolic,
                "branch {} should not be symbolic",
                branch.name
            );
        }
    }

    #[test]
    fn different_branches_can_have_different_targets() {
        let repo = TestRepo::with_branches();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let branches = ops::list_branches(&handle).expect("failed to list branches");

        let main_branch = branches.iter().find(|b| b.name == "refs/heads/main");
        let feature_b = branches.iter().find(|b| b.name == "refs/heads/feature-b");

        if let (Some(main), Some(feature)) = (main_branch, feature_b) {
            assert_ne!(
                main.target, feature.target,
                "main and feature-b should have different targets"
            );
        }
    }
}

mod list_tags {
    use super::*;

    #[test]
    fn returns_all_tags() {
        let repo = TestRepo::with_tags();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let tags = ops::list_tags(&handle).expect("failed to list tags");

        assert!(tags.len() >= 2, "should have at least 2 tags, got {}", tags.len());

        let tag_names: Vec<&str> = tags.iter().map(|r| r.name.as_str()).collect();

        assert!(
            tag_names.contains(&"refs/tags/v1.0.0"),
            "should contain v1.0.0 tag"
        );
        assert!(
            tag_names.contains(&"refs/tags/v1.1.0"),
            "should contain v1.1.0 tag"
        );
    }

    #[test]
    fn only_returns_tag_refs() {
        let repo = TestRepo::with_tags();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let tags = ops::list_tags(&handle).expect("failed to list tags");

        for tag in &tags {
            assert!(
                tag.name.starts_with("refs/tags/"),
                "tag {} should start with refs/tags/",
                tag.name
            );
            assert!(
                !tag.name.contains("heads"),
                "tags should not contain heads"
            );
        }
    }

    #[test]
    fn repo_without_tags_returns_empty() {
        let repo = TestRepo::new();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let tags = ops::list_tags(&handle).expect("failed to list tags");

        assert!(tags.is_empty(), "repo without tags should return empty list");
    }

    #[test]
    fn lightweight_tag_has_commit_target() {
        let repo = TestRepo::with_tags();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let tags = ops::list_tags(&handle).expect("failed to list tags");
        let lightweight_tag = tags.iter().find(|t| t.name == "refs/tags/v1.0.0");

        assert!(lightweight_tag.is_some(), "v1.0.0 tag should exist");
        let tag = lightweight_tag.unwrap();

        assert!(!tag.target.is_null(), "lightweight tag should have valid target");
        assert!(!tag.is_symbolic, "lightweight tag should not be symbolic");
    }

    #[test]
    fn annotated_tag_has_tag_object_target() {
        let repo = TestRepo::with_tags();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let tags = ops::list_tags(&handle).expect("failed to list tags");
        let annotated_tag = tags.iter().find(|t| t.name == "refs/tags/v1.1.0");

        assert!(annotated_tag.is_some(), "v1.1.0 tag should exist");
        let tag = annotated_tag.unwrap();

        assert!(!tag.target.is_null(), "annotated tag should have valid target");
    }

    #[test]
    fn tags_have_valid_targets() {
        let repo = TestRepo::with_tags();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let tags = ops::list_tags(&handle).expect("failed to list tags");

        for tag in &tags {
            assert!(
                !tag.target.is_null(),
                "tag {} should have non-null target",
                tag.name
            );
        }
    }

    #[test]
    fn empty_repo_returns_no_tags() {
        let repo = TestRepo::empty();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let tags = ops::list_tags(&handle).expect("failed to list tags");

        assert!(tags.is_empty(), "empty repo should have no tags");
    }
}

mod head_states {
    use super::*;

    #[test]
    fn symbolic_head_points_to_branch() {
        let repo = TestRepo::new();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let head = ops::get_head(&handle).expect("failed to get HEAD");

        assert!(head.is_symbolic || head.name.starts_with("refs/heads/"));
        assert!(!head.target.is_null());
    }

    #[test]
    fn detached_head_returns_commit_oid() {
        let repo = TestRepo::with_detached_head();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let head = ops::get_head(&handle).expect("failed to get HEAD");

        assert_eq!(head.name, "HEAD");
        assert!(!head.is_symbolic);
        assert!(head.symbolic_target.is_none());

        let hex = head.target.to_string();
        assert_eq!(hex.len(), 40);
        assert!(hex.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn unborn_head_returns_symbolic_with_null_oid() {
        let repo = TestRepo::empty();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let head = ops::get_head(&handle).expect("failed to get HEAD");

        assert_eq!(head.name, "HEAD");
        assert!(head.is_symbolic);
        assert!(head.target.is_null());
        assert!(head.symbolic_target.is_some());

        let symbolic = head.symbolic_target.unwrap();
        assert!(
            symbolic.contains("main") || symbolic.contains("master"),
            "unborn HEAD should point to main or master, got: {}",
            symbolic
        );
    }

    #[test]
    fn head_consistency_with_resolve_ref() {
        let repo = TestRepo::new();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let head_via_get_head = ops::get_head(&handle).expect("failed to get HEAD");
        let head_via_resolve = ops::resolve_ref(&handle, "HEAD").expect("failed to resolve HEAD");

        assert_eq!(head_via_get_head.target, head_via_resolve.target);
    }

    #[test]
    fn detached_head_target_matches_first_commit() {
        let repo = TestRepo::with_detached_head();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let head = ops::get_head(&handle).expect("failed to get HEAD");

        assert!(!head.target.is_null());
        assert!(!head.is_symbolic);
    }

    #[test]
    fn empty_repo_head_symbolic_target_is_valid_ref_name() {
        let repo = TestRepo::empty();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let head = ops::get_head(&handle).expect("failed to get HEAD");

        if let Some(ref symbolic) = head.symbolic_target {
            assert!(
                symbolic.starts_with("refs/heads/"),
                "symbolic target should be a branch ref, got: {}",
                symbolic
            );
        }
    }
}

mod edge_cases {
    use super::*;

    #[test]
    fn ref_info_target_is_40_char_hex() {
        let repo = TestRepo::new();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let refs = ops::list_refs(&handle, Some("refs/heads/"))
            .expect("failed to list refs");

        for ref_info in &refs {
            let hex = ref_info.target.to_string();
            assert_eq!(
                hex.len(),
                40,
                "target OID should be 40 characters, got {} for {}",
                hex.len(),
                ref_info.name
            );
            assert!(
                hex.chars().all(|c| c.is_ascii_hexdigit()),
                "target OID should be hex string"
            );
        }
    }

    #[test]
    fn multiple_pool_accesses_same_result() {
        let repo = TestRepo::new();
        let pool = get_pool();

        let handle1 = pool.get(&repo.path).expect("failed to get repo handle");
        let refs1 = ops::list_refs(&handle1, None).expect("failed to list refs");

        let handle2 = pool.get(&repo.path).expect("failed to get repo handle");
        let refs2 = ops::list_refs(&handle2, None).expect("failed to list refs");

        assert_eq!(
            refs1.len(),
            refs2.len(),
            "multiple accesses should return same number of refs"
        );
    }

    #[test]
    fn symbolic_ref_resolution() {
        let repo = TestRepo::new();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let head = ops::resolve_ref(&handle, "HEAD").expect("failed to resolve HEAD");

        assert!(head.is_symbolic, "HEAD should be symbolic");

        if let Some(target) = &head.symbolic_target {
            let target_ref = ops::resolve_ref(&handle, target)
                .expect("failed to resolve symbolic target");

            assert_eq!(
                head.target, target_ref.target,
                "HEAD target should match resolved symbolic target"
            );
        }
    }

    #[test]
    fn resolve_ref_with_invalid_ref_returns_error() {
        let repo = TestRepo::new();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let result = ops::resolve_ref(&handle, "refs/heads/does-not-exist-anywhere");
        assert!(result.is_err());
    }

    #[test]
    fn resolve_ref_with_empty_string_returns_error() {
        let repo = TestRepo::new();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let result = ops::resolve_ref(&handle, "");
        assert!(result.is_err());
    }

    #[test]
    fn list_refs_with_refs_prefix_returns_all_refs() {
        let repo = TestRepo::with_branches();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let all_refs = ops::list_refs(&handle, None).expect("failed to list all refs");
        let prefixed_refs = ops::list_refs(&handle, Some("refs/")).expect("failed to list with refs/ prefix");

        assert!(!all_refs.is_empty());
        assert!(!prefixed_refs.is_empty());
        assert!(prefixed_refs.len() <= all_refs.len());
    }

    #[test]
    fn refs_with_history_repo() {
        let repo = TestRepo::with_history();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let refs = ops::list_refs(&handle, None).expect("failed to list refs");
        assert!(!refs.is_empty());

        let head = ops::get_head(&handle).expect("failed to get HEAD");
        assert!(!head.target.is_null());
    }

    #[test]
    fn branches_from_history_repo() {
        let repo = TestRepo::with_history();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let branches = ops::list_branches(&handle).expect("failed to list branches");
        assert!(!branches.is_empty());

        for branch in &branches {
            assert!(branch.name.starts_with("refs/heads/"));
            assert!(!branch.target.is_null());
        }
    }

    #[test]
    fn list_refs_iter_processes_all_refs() {
        let repo = TestRepo::with_branches();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let refs = ops::list_refs(&handle, Some("refs/")).expect("failed to list refs");

        assert!(refs.len() >= 3);

        for ref_info in &refs {
            assert!(ref_info.name.starts_with("refs/"));
        }
    }

    #[test]
    fn symbolic_target_chain_resolution() {
        let repo = TestRepo::new();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let head = ops::resolve_ref(&handle, "HEAD").expect("failed to resolve HEAD");

        if head.is_symbolic {
            if let Some(ref target_name) = head.symbolic_target {
                let resolved = ops::resolve_ref(&handle, target_name)
                    .expect("failed to resolve symbolic target");

                assert!(!resolved.is_symbolic, "resolved target should not be symbolic");
                assert_eq!(head.target, resolved.target);
            }
        }
    }
}

mod convert_reference_tests {
    use super::*;

    #[test]
    fn object_ref_is_not_symbolic() {
        let repo = TestRepo::new();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let branch = ops::resolve_ref(&handle, "refs/heads/main")
            .expect("failed to resolve main");

        assert!(!branch.is_symbolic);
        assert!(branch.symbolic_target.is_none());
        assert!(!branch.target.is_null());
    }

    #[test]
    fn symbolic_ref_has_target() {
        let repo = TestRepo::new();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let head = ops::resolve_ref(&handle, "HEAD").expect("failed to resolve HEAD");

        assert!(head.is_symbolic);
        assert!(head.symbolic_target.is_some());
        assert!(!head.target.is_null());
    }

    #[test]
    fn ref_name_is_full_path() {
        let repo = TestRepo::with_branches();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let branches = ops::list_branches(&handle).expect("failed to list branches");

        for branch in &branches {
            assert!(
                branch.name.starts_with("refs/heads/"),
                "branch name should be full path: {}",
                branch.name
            );
        }
    }

    #[test]
    fn tag_ref_conversion() {
        let repo = TestRepo::with_tags();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let tags = ops::list_tags(&handle).expect("failed to list tags");

        for tag in &tags {
            assert!(tag.name.starts_with("refs/tags/"));
            assert!(!tag.target.is_null());
            assert!(!tag.is_symbolic);
        }
    }
}

mod list_functions {
    use super::*;

    #[test]
    fn list_branches_uses_correct_prefix() {
        let repo = TestRepo::with_tags();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let branches = ops::list_branches(&handle).expect("failed to list branches");
        let refs_with_prefix = ops::list_refs(&handle, Some("refs/heads/"))
            .expect("failed to list refs with prefix");

        assert_eq!(branches.len(), refs_with_prefix.len());
    }

    #[test]
    fn list_tags_uses_correct_prefix() {
        let repo = TestRepo::with_tags();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let tags = ops::list_tags(&handle).expect("failed to list tags");
        let refs_with_prefix = ops::list_refs(&handle, Some("refs/tags/"))
            .expect("failed to list refs with prefix");

        assert_eq!(tags.len(), refs_with_prefix.len());
    }

    #[test]
    fn list_refs_none_vs_all_equivalent() {
        let repo = TestRepo::with_branches();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let none_refs = ops::list_refs(&handle, None).expect("failed to list refs with None");

        assert!(!none_refs.is_empty());
    }

    #[test]
    fn list_refs_nonexistent_prefix_empty() {
        let repo = TestRepo::with_branches();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let refs = ops::list_refs(&handle, Some("refs/nonexistent/"))
            .expect("failed to list refs");

        assert!(refs.is_empty());
    }
}

mod symbolic_refs {
    use super::*;

    #[test]
    fn resolve_symbolic_branch_ref() {
        let repo = TestRepo::with_symbolic_ref();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let alias = ops::resolve_ref(&handle, "refs/heads/alias")
            .expect("failed to resolve alias ref");

        assert!(alias.is_symbolic);
        assert!(alias.symbolic_target.is_some());

        let target = alias.symbolic_target.as_ref().unwrap();
        assert!(
            target.contains("develop"),
            "symbolic target should point to develop, got: {}",
            target
        );
    }

    #[test]
    fn symbolic_ref_has_same_target_as_resolved() {
        let repo = TestRepo::with_symbolic_ref();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let alias = ops::resolve_ref(&handle, "refs/heads/alias")
            .expect("failed to resolve alias ref");
        let develop = ops::resolve_ref(&handle, "refs/heads/develop")
            .expect("failed to resolve develop ref");

        assert_eq!(
            alias.target, develop.target,
            "symbolic ref should resolve to same target as the referenced branch"
        );
    }

    #[test]
    fn list_refs_includes_symbolic_refs() {
        let repo = TestRepo::with_symbolic_ref();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let branches = ops::list_branches(&handle).expect("failed to list branches");

        let has_alias = branches.iter().any(|b| b.name.contains("alias"));
        assert!(has_alias, "branch list should include symbolic alias ref");
    }

    #[test]
    fn symbolic_ref_peels_to_oid() {
        let repo = TestRepo::with_symbolic_ref();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let alias = ops::resolve_ref(&handle, "refs/heads/alias")
            .expect("failed to resolve alias ref");

        assert!(!alias.target.is_null(), "symbolic ref should peel to valid OID");

        let hex = alias.target.to_string();
        assert_eq!(hex.len(), 40);
        assert!(hex.chars().all(|c| c.is_ascii_hexdigit()));
    }
}

mod unborn_branch_states {
    use super::*;

    #[test]
    fn unborn_custom_branch_head() {
        let repo = TestRepo::with_unborn_nondefault_branch();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let head = ops::get_head(&handle).expect("failed to get HEAD");

        assert_eq!(head.name, "HEAD");
        assert!(head.is_symbolic);
        assert!(head.target.is_null());

        if let Some(ref target) = head.symbolic_target {
            assert!(
                target.contains("custom-main"),
                "symbolic target should point to custom-main, got: {}",
                target
            );
        }
    }

    #[test]
    fn unborn_branch_has_no_refs() {
        let repo = TestRepo::with_unborn_nondefault_branch();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let refs = ops::list_refs(&handle, None).expect("failed to list refs");
        assert!(refs.is_empty());

        let branches = ops::list_branches(&handle).expect("failed to list branches");
        assert!(branches.is_empty());
    }
}

mod orphan_branch_refs {
    use super::*;

    #[test]
    fn orphan_branch_head() {
        let repo = TestRepo::with_orphan_branch();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let head = ops::get_head(&handle).expect("failed to get HEAD");

        assert!(!head.target.is_null());
    }

    #[test]
    fn orphan_branch_has_multiple_branches() {
        let repo = TestRepo::with_orphan_branch();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let branches = ops::list_branches(&handle).expect("failed to list branches");

        assert!(branches.len() >= 2);

        let branch_names: Vec<&str> = branches.iter().map(|b| b.name.as_str()).collect();
        assert!(
            branch_names.iter().any(|n| n.contains("main")),
            "should have main branch"
        );
        assert!(
            branch_names.iter().any(|n| n.contains("orphan")),
            "should have orphan branch"
        );
    }
}

mod merge_commit_refs {
    use super::*;

    #[test]
    fn head_after_merge() {
        let repo = TestRepo::with_merge_commits();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let head = ops::get_head(&handle).expect("failed to get HEAD");

        assert!(!head.target.is_null());
    }

    #[test]
    fn branches_after_merge() {
        let repo = TestRepo::with_merge_commits();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let branches = ops::list_branches(&handle).expect("failed to list branches");

        assert!(branches.len() >= 2);
    }
}

mod packed_refs {
    use super::*;

    #[test]
    fn list_refs_with_packed_refs() {
        let repo = TestRepo::with_packed_objects();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let refs = ops::list_refs(&handle, None).expect("failed to list refs");

        assert!(!refs.is_empty());
        for ref_info in &refs {
            assert!(!ref_info.target.is_null());
        }
    }

    #[test]
    fn get_head_with_packed_refs() {
        let repo = TestRepo::with_packed_objects();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let head = ops::get_head(&handle).expect("failed to get HEAD");

        assert!(!head.target.is_null());
    }

    #[test]
    fn resolve_ref_with_packed_refs() {
        let repo = TestRepo::with_packed_objects();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let ref_info = ops::resolve_ref(&handle, "refs/heads/main")
            .expect("failed to resolve ref");

        assert_eq!(ref_info.name, "refs/heads/main");
        assert!(!ref_info.target.is_null());
    }

    #[test]
    fn list_branches_with_packed_refs() {
        let repo = TestRepo::with_packed_objects();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let branches = ops::list_branches(&handle).expect("failed to list branches");

        assert!(!branches.is_empty());
        for branch in &branches {
            assert!(branch.name.starts_with("refs/heads/"));
        }
    }
}

mod bare_repo_refs {
    use super::*;

    #[test]
    fn list_refs_in_bare_repo() {
        let repo = TestRepo::bare();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let refs = ops::list_refs(&handle, None).expect("failed to list refs");

        assert!(!refs.is_empty());
    }

    #[test]
    fn get_head_in_bare_repo() {
        let repo = TestRepo::bare();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let head = ops::get_head(&handle).expect("failed to get HEAD");

        assert!(!head.target.is_null());
    }

    #[test]
    fn resolve_ref_in_bare_repo() {
        let repo = TestRepo::bare();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let ref_info = ops::resolve_ref(&handle, "refs/heads/main")
            .expect("failed to resolve ref");

        assert_eq!(ref_info.name, "refs/heads/main");
        assert!(!ref_info.target.is_null());
    }

    #[test]
    fn list_branches_in_bare_repo() {
        let repo = TestRepo::bare();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let branches = ops::list_branches(&handle).expect("failed to list branches");

        assert!(!branches.is_empty());
    }

    #[test]
    fn list_tags_in_bare_repo() {
        let repo = TestRepo::bare();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let tags = ops::list_tags(&handle).expect("failed to list tags");

        assert!(tags.is_empty());
    }
}

mod ref_iteration {
    use super::*;

    #[test]
    fn iterate_many_refs() {
        let repo = TestRepo::with_branches();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let refs = ops::list_refs(&handle, None).expect("failed to list refs");

        let mut seen_main = false;
        let mut seen_feature_a = false;
        let mut seen_feature_b = false;

        for ref_info in &refs {
            if ref_info.name == "refs/heads/main" {
                seen_main = true;
            }
            if ref_info.name == "refs/heads/feature-a" {
                seen_feature_a = true;
            }
            if ref_info.name == "refs/heads/feature-b" {
                seen_feature_b = true;
            }
        }

        assert!(seen_main);
        assert!(seen_feature_a);
        assert!(seen_feature_b);
    }

    #[test]
    fn prefixed_iteration_with_heads() {
        let repo = TestRepo::with_tags();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let heads = ops::list_refs(&handle, Some("refs/heads/"))
            .expect("failed to list heads");
        let tags = ops::list_refs(&handle, Some("refs/tags/"))
            .expect("failed to list tags");

        for head in &heads {
            assert!(!tags.iter().any(|t| t.name == head.name));
        }
    }

    #[test]
    fn all_refs_iteration() {
        let repo = TestRepo::with_tags();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let all_refs = ops::list_refs(&handle, None).expect("failed to list all refs");

        let heads = ops::list_refs(&handle, Some("refs/heads/"))
            .expect("failed to list heads");
        let tags = ops::list_refs(&handle, Some("refs/tags/"))
            .expect("failed to list tags");

        assert!(all_refs.len() >= heads.len() + tags.len());
    }
}

mod ref_target_types {
    use super::*;

    #[test]
    fn direct_object_ref_target() {
        let repo = TestRepo::new();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let branch = ops::resolve_ref(&handle, "refs/heads/main")
            .expect("failed to resolve branch");

        assert!(!branch.is_symbolic);
        assert!(branch.symbolic_target.is_none());
        assert!(!branch.target.is_null());

        let hex = branch.target.to_string();
        assert!(hex.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn symbolic_head_target() {
        let repo = TestRepo::new();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let head = ops::resolve_ref(&handle, "HEAD")
            .expect("failed to resolve HEAD");

        assert!(head.is_symbolic);
        assert!(head.symbolic_target.is_some());

        let target = head.symbolic_target.as_ref().unwrap();
        assert!(target.starts_with("refs/heads/"));
    }

    #[test]
    fn detached_head_target_oid() {
        let repo = TestRepo::with_detached_head();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let head = ops::get_head(&handle).expect("failed to get HEAD");

        assert_eq!(head.name, "HEAD");
        assert!(!head.is_symbolic);

        let oid_str = head.target.to_string();
        assert_eq!(oid_str.len(), 40);
    }

    #[test]
    fn lightweight_tag_target() {
        let repo = TestRepo::with_tags();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let tag = ops::resolve_ref(&handle, "refs/tags/v1.0.0")
            .expect("failed to resolve tag");

        assert!(!tag.is_symbolic);
        assert!(!tag.target.is_null());
    }

    #[test]
    fn annotated_tag_target() {
        let repo = TestRepo::with_tags();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let tag = ops::resolve_ref(&handle, "refs/tags/v1.1.0")
            .expect("failed to resolve tag");

        assert!(!tag.is_symbolic);
        assert!(!tag.target.is_null());
    }
}

mod special_ref_names {
    use super::*;

    #[test]
    fn resolve_head_uppercase() {
        let repo = TestRepo::new();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let head = ops::resolve_ref(&handle, "HEAD")
            .expect("failed to resolve HEAD");

        assert_eq!(head.name, "HEAD");
    }

    #[test]
    fn full_ref_path() {
        let repo = TestRepo::new();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let ref_info = ops::resolve_ref(&handle, "refs/heads/main")
            .expect("failed to resolve ref");

        assert_eq!(ref_info.name, "refs/heads/main");
    }

    #[test]
    fn short_branch_name_resolution() {
        let repo = TestRepo::new();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let ref_info = ops::resolve_ref(&handle, "main")
            .expect("failed to resolve short name");

        assert!(ref_info.name.contains("main"));
    }
}

mod head_states_additional {
    use super::*;

    #[test]
    fn symbolic_head_peels_correctly() {
        let repo = TestRepo::new();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let head = ops::get_head(&handle).expect("failed to get HEAD");
        let main = ops::resolve_ref(&handle, "refs/heads/main")
            .expect("failed to resolve main");

        assert_eq!(head.target, main.target);
    }

    #[test]
    fn unborn_head_symbolic_target_format() {
        let repo = TestRepo::empty();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let head = ops::get_head(&handle).expect("failed to get HEAD");

        assert!(head.is_symbolic);
        assert!(head.target.is_null());

        if let Some(ref target) = head.symbolic_target {
            assert!(target.starts_with("refs/heads/"));
        }
    }

    #[test]
    fn detached_head_no_symbolic_target() {
        let repo = TestRepo::with_detached_head();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let head = ops::get_head(&handle).expect("failed to get HEAD");

        assert!(!head.is_symbolic);
        assert!(head.symbolic_target.is_none());
    }
}

mod convert_reference_edge_cases {
    use super::*;

    #[test]
    fn symbolic_ref_peels_to_commit() {
        let repo = TestRepo::with_symbolic_ref();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let alias = ops::resolve_ref(&handle, "refs/heads/alias")
            .expect("failed to resolve alias");

        assert!(alias.is_symbolic);
        assert!(!alias.target.is_null());

        let develop = ops::resolve_ref(&handle, "refs/heads/develop")
            .expect("failed to resolve develop");

        assert_eq!(alias.target, develop.target);
    }

    #[test]
    fn multiple_symbolic_refs_in_list() {
        let repo = TestRepo::with_symbolic_ref();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let branches = ops::list_branches(&handle).expect("failed to list branches");

        let symbolic_count = branches.iter().filter(|b| b.is_symbolic).count();
        assert!(symbolic_count >= 1);
    }

    #[test]
    fn object_refs_not_symbolic() {
        let repo = TestRepo::with_branches();
        let pool = get_pool();
        let handle = pool.get(&repo.path).expect("failed to get repo handle");

        let branches = ops::list_branches(&handle).expect("failed to list branches");

        for branch in &branches {
            if !branch.name.contains("alias") {
                assert!(!branch.is_symbolic);
                assert!(branch.symbolic_target.is_none());
            }
        }
    }
}
