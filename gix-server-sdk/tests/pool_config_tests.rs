mod fixtures;
use fixtures::TestRepo;
use gix_server_sdk::{RepoPool, SdkConfig};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

const MB: usize = 1024 * 1024;

// ============================================================================
// SdkConfig Tests
// ============================================================================

mod config_tests {
    use super::*;

    mod default_values {
        use super::*;

        #[test]
        fn default_pool_size_is_100() {
            let config = SdkConfig::default();
            assert_eq!(config.pool_size, 100);
        }

        #[test]
        fn default_object_cache_is_16mb() {
            let config = SdkConfig::default();
            assert_eq!(config.object_cache_bytes, 16 * MB);
        }

        #[test]
        fn default_idle_timeout_is_300_seconds() {
            let config = SdkConfig::default();
            assert_eq!(config.idle_timeout, Duration::from_secs(300));
        }

        #[test]
        fn default_max_blob_size_is_100mb() {
            let config = SdkConfig::default();
            assert_eq!(config.max_blob_size, 100 * MB);
        }
    }

    mod builder_pattern {
        use super::*;

        #[test]
        fn builder_creates_default_config() {
            let config = SdkConfig::builder().build();
            let default = SdkConfig::default();

            assert_eq!(config.pool_size, default.pool_size);
            assert_eq!(config.object_cache_bytes, default.object_cache_bytes);
            assert_eq!(config.idle_timeout, default.idle_timeout);
            assert_eq!(config.max_blob_size, default.max_blob_size);
        }

        #[test]
        fn pool_size_can_be_set() {
            let config = SdkConfig::builder().pool_size(50).build();
            assert_eq!(config.pool_size, 50);
        }

        #[test]
        fn pool_size_zero_is_allowed() {
            let config = SdkConfig::builder().pool_size(0).build();
            assert_eq!(config.pool_size, 0);
        }

        #[test]
        fn pool_size_large_value() {
            let config = SdkConfig::builder().pool_size(10_000).build();
            assert_eq!(config.pool_size, 10_000);
        }

        #[test]
        fn object_cache_mb_sets_bytes_correctly() {
            let config = SdkConfig::builder().object_cache_mb(32).build();
            assert_eq!(config.object_cache_bytes, 32 * MB);
        }

        #[test]
        fn object_cache_mb_zero_is_allowed() {
            let config = SdkConfig::builder().object_cache_mb(0).build();
            assert_eq!(config.object_cache_bytes, 0);
        }

        #[test]
        fn object_cache_mb_large_value() {
            let config = SdkConfig::builder().object_cache_mb(1024).build();
            assert_eq!(config.object_cache_bytes, 1024 * MB);
        }

        #[test]
        fn idle_timeout_can_be_set() {
            let timeout = Duration::from_secs(60);
            let config = SdkConfig::builder().idle_timeout(timeout).build();
            assert_eq!(config.idle_timeout, timeout);
        }

        #[test]
        fn idle_timeout_zero_is_allowed() {
            let config = SdkConfig::builder()
                .idle_timeout(Duration::from_secs(0))
                .build();
            assert_eq!(config.idle_timeout, Duration::from_secs(0));
        }

        #[test]
        fn idle_timeout_very_long_duration() {
            let timeout = Duration::from_secs(86400 * 7);
            let config = SdkConfig::builder().idle_timeout(timeout).build();
            assert_eq!(config.idle_timeout, timeout);
        }

        #[test]
        fn idle_timeout_subsecond_precision() {
            let timeout = Duration::from_millis(500);
            let config = SdkConfig::builder().idle_timeout(timeout).build();
            assert_eq!(config.idle_timeout, timeout);
        }

        #[test]
        fn max_blob_size_mb_sets_bytes_correctly() {
            let config = SdkConfig::builder().max_blob_size_mb(200).build();
            assert_eq!(config.max_blob_size, 200 * MB);
        }

        #[test]
        fn max_blob_size_mb_zero_is_allowed() {
            let config = SdkConfig::builder().max_blob_size_mb(0).build();
            assert_eq!(config.max_blob_size, 0);
        }

        #[test]
        fn builder_chain_all_methods() {
            let config = SdkConfig::builder()
                .pool_size(200)
                .object_cache_mb(64)
                .idle_timeout(Duration::from_secs(600))
                .max_blob_size_mb(500)
                .build();

            assert_eq!(config.pool_size, 200);
            assert_eq!(config.object_cache_bytes, 64 * MB);
            assert_eq!(config.idle_timeout, Duration::from_secs(600));
            assert_eq!(config.max_blob_size, 500 * MB);
        }

        #[test]
        fn builder_methods_can_be_called_in_any_order() {
            let config1 = SdkConfig::builder()
                .pool_size(10)
                .object_cache_mb(8)
                .build();

            let config2 = SdkConfig::builder()
                .object_cache_mb(8)
                .pool_size(10)
                .build();

            assert_eq!(config1.pool_size, config2.pool_size);
            assert_eq!(config1.object_cache_bytes, config2.object_cache_bytes);
        }

        #[test]
        fn builder_last_value_wins() {
            let config = SdkConfig::builder()
                .pool_size(10)
                .pool_size(20)
                .pool_size(30)
                .build();

            assert_eq!(config.pool_size, 30);
        }
    }

    mod config_traits {
        use super::*;

        #[test]
        fn config_is_clone() {
            let config1 = SdkConfig::builder().pool_size(42).build();
            let config2 = config1.clone();
            assert_eq!(config1.pool_size, config2.pool_size);
        }

        #[test]
        fn config_is_debug() {
            let config = SdkConfig::default();
            let debug_str = format!("{:?}", config);
            assert!(debug_str.contains("SdkConfig"));
        }

        #[test]
        fn builder_is_clone() {
            let builder1 = SdkConfig::builder().pool_size(42);
            let builder2 = builder1.clone();
            let config1 = builder1.build();
            let config2 = builder2.build();
            assert_eq!(config1.pool_size, config2.pool_size);
        }

        #[test]
        fn builder_is_debug() {
            let builder = SdkConfig::builder();
            let debug_str = format!("{:?}", builder);
            assert!(debug_str.contains("SdkConfigBuilder"));
        }
    }
}

// ============================================================================
// RepoPool Tests
// ============================================================================

mod pool_tests {
    use super::*;

    mod creation {
        use super::*;

        #[test]
        fn new_pool_with_default_config() {
            let config = SdkConfig::default();
            let pool = RepoPool::new(config);
            let stats = pool.stats();
            assert_eq!(stats.cached_count, 0);
            assert_eq!(stats.open_count, 0);
            assert_eq!(stats.hit_count, 0);
            assert_eq!(stats.hit_rate, 0.0);
        }

        #[test]
        fn new_pool_with_custom_config() {
            let config = SdkConfig::builder()
                .pool_size(50)
                .object_cache_mb(8)
                .idle_timeout(Duration::from_secs(60))
                .build();

            let pool = RepoPool::new(config);
            let stats = pool.stats();
            assert_eq!(stats.cached_count, 0);
        }
    }

    mod get_repository {
        use super::*;

        #[test]
        fn get_valid_repository() {
            let test_repo = TestRepo::new();
            let config = SdkConfig::default();
            let pool = RepoPool::new(config);

            let result = pool.get(&test_repo.path);
            assert!(result.is_ok());
        }

        #[test]
        fn get_returns_repo_handle_that_can_be_converted_to_local() {
            let test_repo = TestRepo::new();
            let config = SdkConfig::default();
            let pool = RepoPool::new(config);

            let handle = pool.get(&test_repo.path).expect("should get repo");
            let _local = handle.to_local();
        }

        #[test]
        fn get_nonexistent_repository_returns_error() {
            let config = SdkConfig::default();
            let pool = RepoPool::new(config);

            let result = pool.get("/nonexistent/path/to/repo");
            assert!(result.is_err());
        }

        #[test]
        fn get_empty_path_returns_error() {
            let config = SdkConfig::default();
            let pool = RepoPool::new(config);

            let result = pool.get("");
            assert!(result.is_err());
        }

        #[test]
        fn get_same_repo_twice_returns_cached() {
            let test_repo = TestRepo::new();
            let config = SdkConfig::default();
            let pool = RepoPool::new(config);

            let _ = pool.get(&test_repo.path).expect("first get");
            let _ = pool.get(&test_repo.path).expect("second get");

            let stats = pool.stats();
            assert_eq!(stats.open_count, 1);
            assert_eq!(stats.hit_count, 1);
        }

        #[test]
        fn get_different_repos_opens_both() {
            let test_repo1 = TestRepo::new();
            let test_repo2 = TestRepo::new();
            let config = SdkConfig::default();
            let pool = RepoPool::new(config);

            let _ = pool.get(&test_repo1.path).expect("first repo");
            let _ = pool.get(&test_repo2.path).expect("second repo");

            let stats = pool.stats();
            assert_eq!(stats.cached_count, 2);
            assert_eq!(stats.open_count, 2);
            assert_eq!(stats.hit_count, 0);
        }

        #[test]
        fn get_accepts_path_ref() {
            let test_repo = TestRepo::new();
            let config = SdkConfig::default();
            let pool = RepoPool::new(config);

            let path = &test_repo.path;
            let result = pool.get(path);
            assert!(result.is_ok());
        }

        #[test]
        fn get_accepts_pathbuf() {
            let test_repo = TestRepo::new();
            let config = SdkConfig::default();
            let pool = RepoPool::new(config);

            let path = test_repo.path.clone();
            let result = pool.get(path);
            assert!(result.is_ok());
        }

        #[test]
        fn get_accepts_string() {
            let test_repo = TestRepo::new();
            let config = SdkConfig::default();
            let pool = RepoPool::new(config);

            let path = test_repo.path.to_string_lossy().to_string();
            let result = pool.get(&path);
            assert!(result.is_ok());
        }
    }

    mod statistics {
        use super::*;

        #[test]
        fn initial_stats_are_zero() {
            let pool = RepoPool::new(SdkConfig::default());
            let stats = pool.stats();

            assert_eq!(stats.cached_count, 0);
            assert_eq!(stats.open_count, 0);
            assert_eq!(stats.hit_count, 0);
            assert_eq!(stats.hit_rate, 0.0);
        }

        #[test]
        fn open_count_increments_on_new_repo() {
            let test_repo = TestRepo::new();
            let pool = RepoPool::new(SdkConfig::default());

            let _ = pool.get(&test_repo.path);
            assert_eq!(pool.stats().open_count, 1);
        }

        #[test]
        fn hit_count_increments_on_cache_hit() {
            let test_repo = TestRepo::new();
            let pool = RepoPool::new(SdkConfig::default());

            let _ = pool.get(&test_repo.path);
            let _ = pool.get(&test_repo.path);
            let _ = pool.get(&test_repo.path);

            let stats = pool.stats();
            assert_eq!(stats.open_count, 1);
            assert_eq!(stats.hit_count, 2);
        }

        #[test]
        fn cached_count_reflects_current_pool_size() {
            let test_repo1 = TestRepo::new();
            let test_repo2 = TestRepo::new();
            let pool = RepoPool::new(SdkConfig::default());

            assert_eq!(pool.stats().cached_count, 0);

            let _ = pool.get(&test_repo1.path);
            assert_eq!(pool.stats().cached_count, 1);

            let _ = pool.get(&test_repo2.path);
            assert_eq!(pool.stats().cached_count, 2);
        }

        #[test]
        fn hit_rate_calculation_no_requests() {
            let pool = RepoPool::new(SdkConfig::default());
            assert_eq!(pool.stats().hit_rate, 0.0);
        }

        #[test]
        fn hit_rate_calculation_all_misses() {
            let test_repo1 = TestRepo::new();
            let test_repo2 = TestRepo::new();
            let pool = RepoPool::new(SdkConfig::default());

            let _ = pool.get(&test_repo1.path);
            let _ = pool.get(&test_repo2.path);

            let stats = pool.stats();
            assert_eq!(stats.hit_rate, 0.0);
        }

        #[test]
        fn hit_rate_calculation_mixed() {
            let test_repo = TestRepo::new();
            let pool = RepoPool::new(SdkConfig::default());

            let _ = pool.get(&test_repo.path);
            let _ = pool.get(&test_repo.path);

            let stats = pool.stats();
            assert!((stats.hit_rate - 0.5).abs() < 0.001);
        }

        #[test]
        fn hit_rate_calculation_mostly_hits() {
            let test_repo = TestRepo::new();
            let pool = RepoPool::new(SdkConfig::default());

            let _ = pool.get(&test_repo.path);
            for _ in 0..9 {
                let _ = pool.get(&test_repo.path);
            }

            let stats = pool.stats();
            assert!((stats.hit_rate - 0.9).abs() < 0.001);
        }
    }

    mod evict_idle {
        use super::*;

        #[test]
        fn evict_idle_removes_old_repos() {
            let test_repo = TestRepo::new();
            let config = SdkConfig::builder()
                .idle_timeout(Duration::from_millis(50))
                .build();
            let pool = RepoPool::new(config);

            let _ = pool.get(&test_repo.path);
            assert_eq!(pool.stats().cached_count, 1);

            thread::sleep(Duration::from_millis(100));
            pool.evict_idle();

            assert_eq!(pool.stats().cached_count, 0);
        }

        #[test]
        fn evict_idle_keeps_recent_repos() {
            let test_repo = TestRepo::new();
            let config = SdkConfig::builder()
                .idle_timeout(Duration::from_secs(60))
                .build();
            let pool = RepoPool::new(config);

            let _ = pool.get(&test_repo.path);
            pool.evict_idle();

            assert_eq!(pool.stats().cached_count, 1);
        }

        #[test]
        fn evict_idle_on_empty_pool_is_noop() {
            let pool = RepoPool::new(SdkConfig::default());
            pool.evict_idle();
            assert_eq!(pool.stats().cached_count, 0);
        }

        #[test]
        fn evict_idle_partial_eviction() {
            let test_repo1 = TestRepo::new();
            let test_repo2 = TestRepo::new();
            let config = SdkConfig::builder()
                .idle_timeout(Duration::from_millis(100))
                .build();
            let pool = RepoPool::new(config);

            let _ = pool.get(&test_repo1.path);
            thread::sleep(Duration::from_millis(150));
            let _ = pool.get(&test_repo2.path);

            pool.evict_idle();

            assert_eq!(pool.stats().cached_count, 1);
        }

        #[test]
        fn accessing_repo_refreshes_last_accessed() {
            let test_repo = TestRepo::new();
            let config = SdkConfig::builder()
                .idle_timeout(Duration::from_millis(100))
                .build();
            let pool = RepoPool::new(config);

            let _ = pool.get(&test_repo.path);
            thread::sleep(Duration::from_millis(60));
            let _ = pool.get(&test_repo.path);
            thread::sleep(Duration::from_millis(60));

            pool.evict_idle();
            assert_eq!(pool.stats().cached_count, 1);
        }

        #[test]
        fn evict_with_zero_timeout_removes_all() {
            let test_repo = TestRepo::new();
            let config = SdkConfig::builder()
                .idle_timeout(Duration::from_secs(0))
                .build();
            let pool = RepoPool::new(config);

            let _ = pool.get(&test_repo.path);
            thread::sleep(Duration::from_millis(1));
            pool.evict_idle();

            assert_eq!(pool.stats().cached_count, 0);
        }
    }

    mod concurrent_access {
        use super::*;

        #[test]
        fn concurrent_get_same_repo() {
            let test_repo = TestRepo::new();
            let config = SdkConfig::default();
            let pool = Arc::new(RepoPool::new(config));
            let path = test_repo.path.clone();

            let handles: Vec<_> = (0..10)
                .map(|_| {
                    let pool = Arc::clone(&pool);
                    let path = path.clone();
                    thread::spawn(move || {
                        for _ in 0..10 {
                            let result = pool.get(&path);
                            assert!(result.is_ok());
                        }
                    })
                })
                .collect();

            for handle in handles {
                handle.join().expect("thread panicked");
            }

            let stats = pool.stats();
            assert!(stats.cached_count >= 1);
            let total_requests = stats.open_count + stats.hit_count;
            assert_eq!(total_requests, 100);
        }

        #[test]
        fn concurrent_get_different_repos() {
            let test_repos: Vec<_> = (0..5).map(|_| TestRepo::new()).collect();
            let config = SdkConfig::default();
            let pool = Arc::new(RepoPool::new(config));

            let handles: Vec<_> = test_repos
                .iter()
                .map(|repo| {
                    let pool = Arc::clone(&pool);
                    let path = repo.path.clone();
                    thread::spawn(move || {
                        for _ in 0..5 {
                            let result = pool.get(&path);
                            assert!(result.is_ok());
                        }
                    })
                })
                .collect();

            for handle in handles {
                handle.join().expect("thread panicked");
            }

            let stats = pool.stats();
            assert!(stats.cached_count >= 1 && stats.cached_count <= 5);
        }

        #[test]
        fn concurrent_get_and_evict() {
            let test_repo = TestRepo::new();
            let config = SdkConfig::builder()
                .idle_timeout(Duration::from_millis(50))
                .build();
            let pool = Arc::new(RepoPool::new(config));
            let path = test_repo.path.clone();

            let pool_getter = Arc::clone(&pool);
            let path_getter = path.clone();
            let getter_handle = thread::spawn(move || {
                for _ in 0..50 {
                    let _ = pool_getter.get(&path_getter);
                    thread::sleep(Duration::from_millis(10));
                }
            });

            let pool_evicter = Arc::clone(&pool);
            let evicter_handle = thread::spawn(move || {
                for _ in 0..20 {
                    pool_evicter.evict_idle();
                    thread::sleep(Duration::from_millis(25));
                }
            });

            getter_handle.join().expect("getter panicked");
            evicter_handle.join().expect("evicter panicked");
        }

        #[test]
        fn concurrent_stats_access() {
            let test_repo = TestRepo::new();
            let config = SdkConfig::default();
            let pool = Arc::new(RepoPool::new(config));
            let path = test_repo.path.clone();

            let pool_getter = Arc::clone(&pool);
            let path_getter = path.clone();
            let getter_handle = thread::spawn(move || {
                for _ in 0..100 {
                    let _ = pool_getter.get(&path_getter);
                }
            });

            let pool_stats = Arc::clone(&pool);
            let stats_handle = thread::spawn(move || {
                for _ in 0..100 {
                    let _ = pool_stats.stats();
                }
            });

            getter_handle.join().expect("getter panicked");
            stats_handle.join().expect("stats panicked");

            let final_stats = pool.stats();
            assert_eq!(final_stats.open_count + final_stats.hit_count, 100);
        }
    }

    mod repo_handle {
        use super::*;

        #[test]
        fn handle_to_local_works() {
            let test_repo = TestRepo::new();
            let pool = RepoPool::new(SdkConfig::default());

            let handle = pool.get(&test_repo.path).expect("should get repo");
            let local = handle.to_local();

            assert!(local.head_name().is_ok());
        }

        #[test]
        fn multiple_handles_to_same_repo() {
            let test_repo = TestRepo::new();
            let pool = RepoPool::new(SdkConfig::default());

            let handle1 = pool.get(&test_repo.path).expect("first handle");
            let handle2 = pool.get(&test_repo.path).expect("second handle");

            let local1 = handle1.to_local();
            let local2 = handle2.to_local();

            let head1 = local1.head_name();
            let head2 = local2.head_name();
            assert!(head1.is_ok());
            assert!(head2.is_ok());
        }

        #[test]
        fn handle_can_be_used_after_pool_eviction() {
            let test_repo = TestRepo::new();
            let config = SdkConfig::builder()
                .idle_timeout(Duration::from_millis(10))
                .build();
            let pool = RepoPool::new(config);

            let handle = pool.get(&test_repo.path).expect("get handle");

            thread::sleep(Duration::from_millis(50));
            pool.evict_idle();

            let local = handle.to_local();
            assert!(local.head_name().is_ok());
        }
    }

    mod error_handling {
        use super::*;

        #[test]
        fn error_on_invalid_git_repo() {
            let temp_dir = tempfile::TempDir::new().expect("create temp dir");
            let pool = RepoPool::new(SdkConfig::default());

            let result = pool.get(temp_dir.path());
            assert!(result.is_err());
        }

        #[test]
        fn error_on_file_not_directory() {
            let temp_dir = tempfile::TempDir::new().expect("create temp dir");
            let file_path = temp_dir.path().join("not_a_repo.txt");
            std::fs::write(&file_path, "content").expect("write file");

            let pool = RepoPool::new(SdkConfig::default());
            let result = pool.get(&file_path);
            assert!(result.is_err());
        }

        #[test]
        fn multiple_failed_opens_dont_crash() {
            let pool = RepoPool::new(SdkConfig::default());

            for i in 0..10 {
                let result = pool.get(format!("/nonexistent/path/{}", i));
                assert!(result.is_err());
            }

            let stats = pool.stats();
            assert_eq!(stats.cached_count, 0);
            assert_eq!(stats.open_count, 0);
        }
    }

    mod integration {
        use super::*;

        #[test]
        fn full_workflow() {
            let test_repo1 = TestRepo::with_history();
            let test_repo2 = TestRepo::new();

            let config = SdkConfig::builder()
                .pool_size(10)
                .object_cache_mb(8)
                .idle_timeout(Duration::from_secs(60))
                .build();

            let pool = RepoPool::new(config);

            let handle1 = pool.get(&test_repo1.path).expect("get repo1");
            let local1 = handle1.to_local();
            assert!(local1.head_name().is_ok());

            let handle2 = pool.get(&test_repo2.path).expect("get repo2");
            let local2 = handle2.to_local();
            assert!(local2.head_name().is_ok());

            let _ = pool.get(&test_repo1.path).expect("get repo1 again");

            let stats = pool.stats();
            assert_eq!(stats.cached_count, 2);
            assert_eq!(stats.open_count, 2);
            assert_eq!(stats.hit_count, 1);

            pool.evict_idle();
            assert_eq!(pool.stats().cached_count, 2);
        }

        #[test]
        fn pool_can_reopen_evicted_repo() {
            let test_repo = TestRepo::new();
            let config = SdkConfig::builder()
                .idle_timeout(Duration::from_millis(10))
                .build();
            let pool = RepoPool::new(config);

            let _ = pool.get(&test_repo.path).expect("first open");
            assert_eq!(pool.stats().open_count, 1);

            thread::sleep(Duration::from_millis(50));
            pool.evict_idle();
            assert_eq!(pool.stats().cached_count, 0);

            let _ = pool.get(&test_repo.path).expect("second open");
            assert_eq!(pool.stats().open_count, 2);
            assert_eq!(pool.stats().cached_count, 1);
        }

        #[test]
        fn pool_works_with_repo_with_history() {
            let test_repo = TestRepo::with_history();
            let pool = RepoPool::new(SdkConfig::default());

            let handle = pool.get(&test_repo.path).expect("get repo with history");
            let local = handle.to_local();
            assert!(local.head_name().is_ok());
        }

        #[test]
        fn pool_works_with_repo_with_branches() {
            let test_repo = TestRepo::with_branches();
            let pool = RepoPool::new(SdkConfig::default());

            let handle = pool.get(&test_repo.path).expect("get repo with branches");
            let local = handle.to_local();
            assert!(local.head_name().is_ok());
        }

        #[test]
        fn pool_works_with_repo_with_tags() {
            let test_repo = TestRepo::with_tags();
            let pool = RepoPool::new(SdkConfig::default());

            let handle = pool.get(&test_repo.path).expect("get repo with tags");
            let local = handle.to_local();
            assert!(local.head_name().is_ok());
        }

        #[test]
        fn pool_works_with_repo_with_submodules() {
            let test_repo = TestRepo::with_submodules();
            let pool = RepoPool::new(SdkConfig::default());

            let handle = pool.get(&test_repo.path).expect("get repo with submodules");
            let local = handle.to_local();
            assert!(local.head_name().is_ok());
        }

        #[test]
        fn pool_works_with_repo_with_attributes() {
            let test_repo = TestRepo::with_attributes();
            let pool = RepoPool::new(SdkConfig::default());

            let handle = pool.get(&test_repo.path).expect("get repo with attributes");
            let local = handle.to_local();
            assert!(local.head_name().is_ok());
        }

        #[test]
        fn pool_works_with_detached_head_repo() {
            let test_repo = TestRepo::with_detached_head();
            let pool = RepoPool::new(SdkConfig::default());

            let handle = pool.get(&test_repo.path).expect("get repo with detached head");
            let local = handle.to_local();
            let head = local.head().expect("should get head");
            assert!(head.is_detached());
        }

        #[test]
        fn pool_handles_many_repos() {
            let test_repos: Vec<_> = (0..20).map(|_| TestRepo::new()).collect();
            let pool = RepoPool::new(SdkConfig::default());

            for repo in &test_repos {
                let result = pool.get(&repo.path);
                assert!(result.is_ok());
            }

            let stats = pool.stats();
            assert_eq!(stats.cached_count, 20);
            assert_eq!(stats.open_count, 20);
            assert_eq!(stats.hit_count, 0);
        }

        #[test]
        fn pool_stats_persist_across_evictions() {
            let test_repo1 = TestRepo::new();
            let test_repo2 = TestRepo::new();
            let config = SdkConfig::builder()
                .idle_timeout(Duration::from_millis(10))
                .build();
            let pool = RepoPool::new(config);

            let _ = pool.get(&test_repo1.path);
            let _ = pool.get(&test_repo2.path);
            let _ = pool.get(&test_repo1.path);
            let _ = pool.get(&test_repo2.path);

            thread::sleep(Duration::from_millis(50));
            pool.evict_idle();

            let stats = pool.stats();
            assert_eq!(stats.cached_count, 0);
            assert_eq!(stats.open_count, 2);
            assert_eq!(stats.hit_count, 2);
        }
    }
}
