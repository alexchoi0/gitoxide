mod fixtures;

use std::io::Cursor;
use bstr::BString;
use fixtures::TestRepo;
use gix_server_sdk::{ops, RepoPool, SdkConfig, SdkError};

fn create_pool() -> RepoPool {
    RepoPool::new(SdkConfig::default())
}

mod archive_format {
    use gix_server_sdk::ops::ArchiveFormat;

    #[test]
    fn tar_variant() {
        let format = ArchiveFormat::Tar;
        assert_eq!(format, ArchiveFormat::Tar);
    }

    #[test]
    fn tar_gz_variant_with_default_compression() {
        let format = ArchiveFormat::TarGz { compression_level: None };
        match format {
            ArchiveFormat::TarGz { compression_level } => {
                assert!(compression_level.is_none());
            }
            _ => panic!("expected TarGz variant"),
        }
    }

    #[test]
    fn tar_gz_variant_with_custom_compression() {
        let format = ArchiveFormat::TarGz { compression_level: Some(6) };
        match format {
            ArchiveFormat::TarGz { compression_level } => {
                assert_eq!(compression_level, Some(6));
            }
            _ => panic!("expected TarGz variant"),
        }
    }

    #[test]
    fn zip_variant_with_default_compression() {
        let format = ArchiveFormat::Zip { compression_level: None };
        match format {
            ArchiveFormat::Zip { compression_level } => {
                assert!(compression_level.is_none());
            }
            _ => panic!("expected Zip variant"),
        }
    }

    #[test]
    fn zip_variant_with_custom_compression() {
        let format = ArchiveFormat::Zip { compression_level: Some(9) };
        match format {
            ArchiveFormat::Zip { compression_level } => {
                assert_eq!(compression_level, Some(9));
            }
            _ => panic!("expected Zip variant"),
        }
    }

    #[test]
    fn format_equality() {
        assert_eq!(ArchiveFormat::Tar, ArchiveFormat::Tar);
        assert_eq!(
            ArchiveFormat::TarGz { compression_level: Some(5) },
            ArchiveFormat::TarGz { compression_level: Some(5) }
        );
        assert_eq!(
            ArchiveFormat::Zip { compression_level: Some(3) },
            ArchiveFormat::Zip { compression_level: Some(3) }
        );
    }

    #[test]
    fn format_inequality() {
        assert_ne!(ArchiveFormat::Tar, ArchiveFormat::TarGz { compression_level: None });
        assert_ne!(
            ArchiveFormat::TarGz { compression_level: Some(5) },
            ArchiveFormat::TarGz { compression_level: Some(6) }
        );
        assert_ne!(
            ArchiveFormat::Zip { compression_level: None },
            ArchiveFormat::TarGz { compression_level: None }
        );
    }

    #[test]
    fn format_debug() {
        let tar = ArchiveFormat::Tar;
        let debug_str = format!("{:?}", tar);
        assert!(debug_str.contains("Tar"));

        let tar_gz = ArchiveFormat::TarGz { compression_level: Some(5) };
        let debug_str = format!("{:?}", tar_gz);
        assert!(debug_str.contains("TarGz"));
        assert!(debug_str.contains("5"));

        let zip = ArchiveFormat::Zip { compression_level: None };
        let debug_str = format!("{:?}", zip);
        assert!(debug_str.contains("Zip"));
    }

    #[test]
    fn format_clone() {
        let format = ArchiveFormat::TarGz { compression_level: Some(7) };
        let cloned = format.clone();
        assert_eq!(format, cloned);
    }

    #[test]
    fn format_copy() {
        let format = ArchiveFormat::Tar;
        let copied = format;
        assert_eq!(format, copied);
    }
}

mod create_archive {
    use super::*;
    use gix_server_sdk::ops::ArchiveFormat;

    #[test]
    fn tar_format_produces_output() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id = gix_hash::ObjectId::from_hex(tree_id_str.as_bytes())
            .expect("failed to parse tree id");

        let mut output: Vec<u8> = Vec::new();
        ops::create_archive(&handle, tree_id, ArchiveFormat::Tar, None, &mut output)
            .expect("failed to create archive");

        assert!(!output.is_empty());
        assert!(output.len() > 512);
    }

    #[test]
    fn tar_gz_format_produces_output() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id = gix_hash::ObjectId::from_hex(tree_id_str.as_bytes())
            .expect("failed to parse tree id");

        let mut output: Vec<u8> = Vec::new();
        ops::create_archive(
            &handle,
            tree_id,
            ArchiveFormat::TarGz { compression_level: None },
            None,
            &mut output,
        )
        .expect("failed to create archive");

        assert!(!output.is_empty());
        assert!(output.starts_with(&[0x1f, 0x8b]));
    }

    #[test]
    fn tar_gz_with_custom_compression() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id = gix_hash::ObjectId::from_hex(tree_id_str.as_bytes())
            .expect("failed to parse tree id");

        let mut output: Vec<u8> = Vec::new();
        ops::create_archive(
            &handle,
            tree_id,
            ArchiveFormat::TarGz { compression_level: Some(9) },
            None,
            &mut output,
        )
        .expect("failed to create archive");

        assert!(!output.is_empty());
        assert!(output.starts_with(&[0x1f, 0x8b]));
    }

    #[test]
    fn zip_format_requires_seekable_writer() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id = gix_hash::ObjectId::from_hex(tree_id_str.as_bytes())
            .expect("failed to parse tree id");

        let mut output: Vec<u8> = Vec::new();
        let result = ops::create_archive(
            &handle,
            tree_id,
            ArchiveFormat::Zip { compression_level: None },
            None,
            &mut output,
        );

        assert!(result.is_err());
        match result.unwrap_err() {
            SdkError::Operation(msg) => {
                assert!(msg.contains("seekable"));
            }
            other => panic!("expected Operation error, got: {:?}", other),
        }
    }

    #[test]
    fn with_prefix() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id = gix_hash::ObjectId::from_hex(tree_id_str.as_bytes())
            .expect("failed to parse tree id");

        let prefix = BString::from("my-project-v1.0/");
        let mut output: Vec<u8> = Vec::new();
        ops::create_archive(&handle, tree_id, ArchiveFormat::Tar, Some(prefix), &mut output)
            .expect("failed to create archive");

        assert!(!output.is_empty());
    }

    #[test]
    fn without_prefix() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id = gix_hash::ObjectId::from_hex(tree_id_str.as_bytes())
            .expect("failed to parse tree id");

        let mut output: Vec<u8> = Vec::new();
        ops::create_archive(&handle, tree_id, ArchiveFormat::Tar, None, &mut output)
            .expect("failed to create archive");

        assert!(!output.is_empty());
    }

    #[test]
    fn invalid_tree_id_returns_error() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let fake_id = gix_hash::ObjectId::from_hex(b"0000000000000000000000000000000000000000")
            .expect("valid hex");

        let mut output: Vec<u8> = Vec::new();
        let result = ops::create_archive(&handle, fake_id, ArchiveFormat::Tar, None, &mut output);

        assert!(result.is_err());
        match result.unwrap_err() {
            SdkError::ObjectNotFound(_) => {}
            other => panic!("expected ObjectNotFound, got: {:?}", other),
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

        let mut output: Vec<u8> = Vec::new();
        let result = ops::create_archive(&handle, commit_id, ArchiveFormat::Tar, None, &mut output);

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
    fn blob_id_returns_invalid_object_type() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let blob_id_str = repo.git_output(&["rev-parse", "HEAD:README.md"]);
        let blob_id = gix_hash::ObjectId::from_hex(blob_id_str.as_bytes())
            .expect("failed to parse blob id");

        let mut output: Vec<u8> = Vec::new();
        let result = ops::create_archive(&handle, blob_id, ArchiveFormat::Tar, None, &mut output);

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
    fn nested_tree_archive() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD:src"]);
        let tree_id = gix_hash::ObjectId::from_hex(tree_id_str.as_bytes())
            .expect("failed to parse tree id");

        let mut output: Vec<u8> = Vec::new();
        ops::create_archive(&handle, tree_id, ArchiveFormat::Tar, None, &mut output)
            .expect("failed to create archive");

        assert!(!output.is_empty());
    }
}

mod create_archive_seekable {
    use super::*;
    use gix_server_sdk::ops::ArchiveFormat;

    #[test]
    fn tar_format_produces_output() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id = gix_hash::ObjectId::from_hex(tree_id_str.as_bytes())
            .expect("failed to parse tree id");

        let mut output = Cursor::new(Vec::new());
        ops::create_archive_seekable(&handle, tree_id, ArchiveFormat::Tar, None, &mut output)
            .expect("failed to create archive");

        let data = output.into_inner();
        assert!(!data.is_empty());
        assert!(data.len() > 512);
    }

    #[test]
    fn tar_gz_format_produces_output() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id = gix_hash::ObjectId::from_hex(tree_id_str.as_bytes())
            .expect("failed to parse tree id");

        let mut output = Cursor::new(Vec::new());
        ops::create_archive_seekable(
            &handle,
            tree_id,
            ArchiveFormat::TarGz { compression_level: None },
            None,
            &mut output,
        )
        .expect("failed to create archive");

        let data = output.into_inner();
        assert!(!data.is_empty());
        assert!(data.starts_with(&[0x1f, 0x8b]));
    }

    #[test]
    fn zip_format_produces_output() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id = gix_hash::ObjectId::from_hex(tree_id_str.as_bytes())
            .expect("failed to parse tree id");

        let mut output = Cursor::new(Vec::new());
        ops::create_archive_seekable(
            &handle,
            tree_id,
            ArchiveFormat::Zip { compression_level: None },
            None,
            &mut output,
        )
        .expect("failed to create archive");

        let data = output.into_inner();
        assert!(!data.is_empty());
        assert!(data.starts_with(&[0x50, 0x4b]));
    }

    #[test]
    fn zip_with_custom_compression() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id = gix_hash::ObjectId::from_hex(tree_id_str.as_bytes())
            .expect("failed to parse tree id");

        let mut output = Cursor::new(Vec::new());
        ops::create_archive_seekable(
            &handle,
            tree_id,
            ArchiveFormat::Zip { compression_level: Some(9) },
            None,
            &mut output,
        )
        .expect("failed to create archive");

        let data = output.into_inner();
        assert!(!data.is_empty());
        assert!(data.starts_with(&[0x50, 0x4b]));
    }

    #[test]
    fn tar_gz_with_custom_compression() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id = gix_hash::ObjectId::from_hex(tree_id_str.as_bytes())
            .expect("failed to parse tree id");

        let mut output = Cursor::new(Vec::new());
        ops::create_archive_seekable(
            &handle,
            tree_id,
            ArchiveFormat::TarGz { compression_level: Some(9) },
            None,
            &mut output,
        )
        .expect("failed to create archive");

        let data = output.into_inner();
        assert!(!data.is_empty());
        assert!(data.starts_with(&[0x1f, 0x8b]));
    }

    #[test]
    fn with_prefix() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id = gix_hash::ObjectId::from_hex(tree_id_str.as_bytes())
            .expect("failed to parse tree id");

        let prefix = BString::from("release-v2.0/");
        let mut output = Cursor::new(Vec::new());
        ops::create_archive_seekable(
            &handle,
            tree_id,
            ArchiveFormat::Zip { compression_level: None },
            Some(prefix),
            &mut output,
        )
        .expect("failed to create archive");

        let data = output.into_inner();
        assert!(!data.is_empty());
    }

    #[test]
    fn invalid_tree_id_returns_error() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let fake_id = gix_hash::ObjectId::from_hex(b"0000000000000000000000000000000000000000")
            .expect("valid hex");

        let mut output = Cursor::new(Vec::new());
        let result = ops::create_archive_seekable(
            &handle,
            fake_id,
            ArchiveFormat::Zip { compression_level: None },
            None,
            &mut output,
        );

        assert!(result.is_err());
        match result.unwrap_err() {
            SdkError::ObjectNotFound(_) => {}
            other => panic!("expected ObjectNotFound, got: {:?}", other),
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

        let mut output = Cursor::new(Vec::new());
        let result = ops::create_archive_seekable(
            &handle,
            commit_id,
            ArchiveFormat::Zip { compression_level: None },
            None,
            &mut output,
        );

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
    fn blob_id_returns_invalid_object_type() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let blob_id_str = repo.git_output(&["rev-parse", "HEAD:README.md"]);
        let blob_id = gix_hash::ObjectId::from_hex(blob_id_str.as_bytes())
            .expect("failed to parse blob id");

        let mut output = Cursor::new(Vec::new());
        let result = ops::create_archive_seekable(
            &handle,
            blob_id,
            ArchiveFormat::Zip { compression_level: None },
            None,
            &mut output,
        );

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
    fn different_formats_produce_different_output() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id = gix_hash::ObjectId::from_hex(tree_id_str.as_bytes())
            .expect("failed to parse tree id");

        let mut tar_output = Cursor::new(Vec::new());
        ops::create_archive_seekable(&handle, tree_id, ArchiveFormat::Tar, None, &mut tar_output)
            .expect("failed to create tar archive");

        let mut tar_gz_output = Cursor::new(Vec::new());
        ops::create_archive_seekable(
            &handle,
            tree_id,
            ArchiveFormat::TarGz { compression_level: None },
            None,
            &mut tar_gz_output,
        )
        .expect("failed to create tar.gz archive");

        let mut zip_output = Cursor::new(Vec::new());
        ops::create_archive_seekable(
            &handle,
            tree_id,
            ArchiveFormat::Zip { compression_level: None },
            None,
            &mut zip_output,
        )
        .expect("failed to create zip archive");

        let tar_data = tar_output.into_inner();
        let tar_gz_data = tar_gz_output.into_inner();
        let zip_data = zip_output.into_inner();

        assert_ne!(tar_data, tar_gz_data);
        assert_ne!(tar_data, zip_data);
        assert_ne!(tar_gz_data, zip_data);
    }
}

mod create_archive_from_commit {
    use super::*;
    use gix_server_sdk::ops::ArchiveFormat;

    #[test]
    fn tar_format_produces_output() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let mut output: Vec<u8> = Vec::new();
        ops::create_archive_from_commit(&handle, commit_id, ArchiveFormat::Tar, None, &mut output)
            .expect("failed to create archive");

        assert!(!output.is_empty());
        assert!(output.len() > 512);
    }

    #[test]
    fn tar_gz_format_produces_output() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let mut output: Vec<u8> = Vec::new();
        ops::create_archive_from_commit(
            &handle,
            commit_id,
            ArchiveFormat::TarGz { compression_level: None },
            None,
            &mut output,
        )
        .expect("failed to create archive");

        assert!(!output.is_empty());
        assert!(output.starts_with(&[0x1f, 0x8b]));
    }

    #[test]
    fn zip_format_requires_seekable_writer() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let mut output: Vec<u8> = Vec::new();
        let result = ops::create_archive_from_commit(
            &handle,
            commit_id,
            ArchiveFormat::Zip { compression_level: None },
            None,
            &mut output,
        );

        assert!(result.is_err());
        match result.unwrap_err() {
            SdkError::Operation(msg) => {
                assert!(msg.contains("seekable"));
            }
            other => panic!("expected Operation error, got: {:?}", other),
        }
    }

    #[test]
    fn with_prefix() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let prefix = BString::from("project-snapshot/");
        let mut output: Vec<u8> = Vec::new();
        ops::create_archive_from_commit(&handle, commit_id, ArchiveFormat::Tar, Some(prefix), &mut output)
            .expect("failed to create archive");

        assert!(!output.is_empty());
    }

    #[test]
    fn invalid_commit_id_returns_error() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let fake_id = gix_hash::ObjectId::from_hex(b"0000000000000000000000000000000000000000")
            .expect("valid hex");

        let mut output: Vec<u8> = Vec::new();
        let result = ops::create_archive_from_commit(&handle, fake_id, ArchiveFormat::Tar, None, &mut output);

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
        let tree_id = gix_hash::ObjectId::from_hex(tree_id_str.as_bytes())
            .expect("failed to parse tree id");

        let mut output: Vec<u8> = Vec::new();
        let result = ops::create_archive_from_commit(&handle, tree_id, ArchiveFormat::Tar, None, &mut output);

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
        let blob_id = gix_hash::ObjectId::from_hex(blob_id_str.as_bytes())
            .expect("failed to parse blob id");

        let mut output: Vec<u8> = Vec::new();
        let result = ops::create_archive_from_commit(&handle, blob_id, ArchiveFormat::Tar, None, &mut output);

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
    fn older_commit_archive() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD~2"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let mut output: Vec<u8> = Vec::new();
        ops::create_archive_from_commit(&handle, commit_id, ArchiveFormat::Tar, None, &mut output)
            .expect("failed to create archive");

        assert!(!output.is_empty());
    }

    #[test]
    fn head_archive_matches_tree_archive() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id = gix_hash::ObjectId::from_hex(tree_id_str.as_bytes())
            .expect("failed to parse tree id");

        let mut commit_output: Vec<u8> = Vec::new();
        ops::create_archive_from_commit(&handle, commit_id, ArchiveFormat::Tar, None, &mut commit_output)
            .expect("failed to create archive from commit");

        let mut tree_output: Vec<u8> = Vec::new();
        ops::create_archive(&handle, tree_id, ArchiveFormat::Tar, None, &mut tree_output)
            .expect("failed to create archive from tree");

        assert_eq!(commit_output.len(), tree_output.len());
    }
}

mod create_archive_from_commit_seekable {
    use super::*;
    use gix_server_sdk::ops::ArchiveFormat;

    #[test]
    fn tar_format_produces_output() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let mut output = Cursor::new(Vec::new());
        ops::create_archive_from_commit_seekable(&handle, commit_id, ArchiveFormat::Tar, None, &mut output)
            .expect("failed to create archive");

        let data = output.into_inner();
        assert!(!data.is_empty());
        assert!(data.len() > 512);
    }

    #[test]
    fn tar_gz_format_produces_output() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let mut output = Cursor::new(Vec::new());
        ops::create_archive_from_commit_seekable(
            &handle,
            commit_id,
            ArchiveFormat::TarGz { compression_level: None },
            None,
            &mut output,
        )
        .expect("failed to create archive");

        let data = output.into_inner();
        assert!(!data.is_empty());
        assert!(data.starts_with(&[0x1f, 0x8b]));
    }

    #[test]
    fn zip_format_produces_output() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let mut output = Cursor::new(Vec::new());
        ops::create_archive_from_commit_seekable(
            &handle,
            commit_id,
            ArchiveFormat::Zip { compression_level: None },
            None,
            &mut output,
        )
        .expect("failed to create archive");

        let data = output.into_inner();
        assert!(!data.is_empty());
        assert!(data.starts_with(&[0x50, 0x4b]));
    }

    #[test]
    fn zip_with_custom_compression() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let mut output = Cursor::new(Vec::new());
        ops::create_archive_from_commit_seekable(
            &handle,
            commit_id,
            ArchiveFormat::Zip { compression_level: Some(6) },
            None,
            &mut output,
        )
        .expect("failed to create archive");

        let data = output.into_inner();
        assert!(!data.is_empty());
        assert!(data.starts_with(&[0x50, 0x4b]));
    }

    #[test]
    fn tar_gz_with_custom_compression() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let mut output = Cursor::new(Vec::new());
        ops::create_archive_from_commit_seekable(
            &handle,
            commit_id,
            ArchiveFormat::TarGz { compression_level: Some(6) },
            None,
            &mut output,
        )
        .expect("failed to create archive");

        let data = output.into_inner();
        assert!(!data.is_empty());
        assert!(data.starts_with(&[0x1f, 0x8b]));
    }

    #[test]
    fn with_prefix() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let prefix = BString::from("archive-prefix/");
        let mut output = Cursor::new(Vec::new());
        ops::create_archive_from_commit_seekable(
            &handle,
            commit_id,
            ArchiveFormat::Zip { compression_level: None },
            Some(prefix),
            &mut output,
        )
        .expect("failed to create archive");

        let data = output.into_inner();
        assert!(!data.is_empty());
    }

    #[test]
    fn invalid_commit_id_returns_error() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let fake_id = gix_hash::ObjectId::from_hex(b"0000000000000000000000000000000000000000")
            .expect("valid hex");

        let mut output = Cursor::new(Vec::new());
        let result = ops::create_archive_from_commit_seekable(
            &handle,
            fake_id,
            ArchiveFormat::Zip { compression_level: None },
            None,
            &mut output,
        );

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
        let tree_id = gix_hash::ObjectId::from_hex(tree_id_str.as_bytes())
            .expect("failed to parse tree id");

        let mut output = Cursor::new(Vec::new());
        let result = ops::create_archive_from_commit_seekable(
            &handle,
            tree_id,
            ArchiveFormat::Zip { compression_level: None },
            None,
            &mut output,
        );

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
        let blob_id = gix_hash::ObjectId::from_hex(blob_id_str.as_bytes())
            .expect("failed to parse blob id");

        let mut output = Cursor::new(Vec::new());
        let result = ops::create_archive_from_commit_seekable(
            &handle,
            blob_id,
            ArchiveFormat::Zip { compression_level: None },
            None,
            &mut output,
        );

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
    fn older_commit_archive() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD~3"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let mut output = Cursor::new(Vec::new());
        ops::create_archive_from_commit_seekable(
            &handle,
            commit_id,
            ArchiveFormat::Zip { compression_level: None },
            None,
            &mut output,
        )
        .expect("failed to create archive");

        let data = output.into_inner();
        assert!(!data.is_empty());
    }

    #[test]
    fn different_formats_produce_different_output() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let mut tar_output = Cursor::new(Vec::new());
        ops::create_archive_from_commit_seekable(
            &handle,
            commit_id,
            ArchiveFormat::Tar,
            None,
            &mut tar_output,
        )
        .expect("failed to create tar archive");

        let mut tar_gz_output = Cursor::new(Vec::new());
        ops::create_archive_from_commit_seekable(
            &handle,
            commit_id,
            ArchiveFormat::TarGz { compression_level: None },
            None,
            &mut tar_gz_output,
        )
        .expect("failed to create tar.gz archive");

        let mut zip_output = Cursor::new(Vec::new());
        ops::create_archive_from_commit_seekable(
            &handle,
            commit_id,
            ArchiveFormat::Zip { compression_level: None },
            None,
            &mut zip_output,
        )
        .expect("failed to create zip archive");

        let tar_data = tar_output.into_inner();
        let tar_gz_data = tar_gz_output.into_inner();
        let zip_data = zip_output.into_inner();

        assert_ne!(tar_data, tar_gz_data);
        assert_ne!(tar_data, zip_data);
        assert_ne!(tar_gz_data, zip_data);
    }

    #[test]
    fn seekable_matches_non_seekable_for_tar() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let mut non_seekable_output: Vec<u8> = Vec::new();
        ops::create_archive_from_commit(&handle, commit_id, ArchiveFormat::Tar, None, &mut non_seekable_output)
            .expect("failed to create non-seekable archive");

        let mut seekable_output = Cursor::new(Vec::new());
        ops::create_archive_from_commit_seekable(&handle, commit_id, ArchiveFormat::Tar, None, &mut seekable_output)
            .expect("failed to create seekable archive");

        let seekable_data = seekable_output.into_inner();
        assert_eq!(non_seekable_output.len(), seekable_data.len());
    }
}

mod archive_with_various_repos {
    use super::*;
    use gix_server_sdk::ops::ArchiveFormat;

    #[test]
    fn repo_with_history() {
        let repo = TestRepo::with_history();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id = gix_hash::ObjectId::from_hex(tree_id_str.as_bytes())
            .expect("failed to parse tree id");

        let mut output: Vec<u8> = Vec::new();
        ops::create_archive(&handle, tree_id, ArchiveFormat::Tar, None, &mut output)
            .expect("failed to create archive");

        assert!(!output.is_empty());
    }

    #[test]
    fn repo_with_attributes() {
        let repo = TestRepo::with_attributes();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id = gix_hash::ObjectId::from_hex(tree_id_str.as_bytes())
            .expect("failed to parse tree id");

        let mut output: Vec<u8> = Vec::new();
        ops::create_archive(&handle, tree_id, ArchiveFormat::Tar, None, &mut output)
            .expect("failed to create archive");

        assert!(!output.is_empty());
    }

    #[test]
    fn repo_with_branches() {
        let repo = TestRepo::with_branches();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "feature-a^{tree}"]);
        let tree_id = gix_hash::ObjectId::from_hex(tree_id_str.as_bytes())
            .expect("failed to parse tree id");

        let mut output: Vec<u8> = Vec::new();
        ops::create_archive(&handle, tree_id, ArchiveFormat::Tar, None, &mut output)
            .expect("failed to create archive");

        assert!(!output.is_empty());
    }

    #[test]
    fn repo_with_tags() {
        let repo = TestRepo::with_tags();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "v1.0.0^{tree}"]);
        let tree_id = gix_hash::ObjectId::from_hex(tree_id_str.as_bytes())
            .expect("failed to parse tree id");

        let mut output: Vec<u8> = Vec::new();
        ops::create_archive(&handle, tree_id, ArchiveFormat::Tar, None, &mut output)
            .expect("failed to create archive");

        assert!(!output.is_empty());
    }

    #[test]
    fn repo_with_submodules() {
        let repo = TestRepo::with_submodules();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id = gix_hash::ObjectId::from_hex(tree_id_str.as_bytes())
            .expect("failed to parse tree id");

        let mut output: Vec<u8> = Vec::new();
        ops::create_archive(&handle, tree_id, ArchiveFormat::Tar, None, &mut output)
            .expect("failed to create archive");

        assert!(!output.is_empty());
    }

    #[test]
    fn repo_with_detached_head() {
        let repo = TestRepo::with_detached_head();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let mut output: Vec<u8> = Vec::new();
        ops::create_archive_from_commit(&handle, commit_id, ArchiveFormat::Tar, None, &mut output)
            .expect("failed to create archive");

        assert!(!output.is_empty());
    }
}

mod archive_error_handling {
    use super::*;
    use gix_server_sdk::ops::ArchiveFormat;
    use std::io::{self, Cursor, Write, Seek, SeekFrom};

    struct FailingWriter {
        bytes_written: usize,
        fail_after: usize,
    }

    impl FailingWriter {
        fn new(fail_after: usize) -> Self {
            Self {
                bytes_written: 0,
                fail_after,
            }
        }
    }

    impl Write for FailingWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            if self.bytes_written >= self.fail_after {
                return Err(io::Error::new(io::ErrorKind::Other, "simulated write failure"));
            }
            self.bytes_written += buf.len();
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            if self.bytes_written >= self.fail_after {
                return Err(io::Error::new(io::ErrorKind::Other, "simulated flush failure"));
            }
            Ok(())
        }
    }

    struct FailingSeekableWriter {
        inner: Cursor<Vec<u8>>,
        bytes_written: usize,
        fail_after: usize,
    }

    impl FailingSeekableWriter {
        fn new(fail_after: usize) -> Self {
            Self {
                inner: Cursor::new(Vec::new()),
                bytes_written: 0,
                fail_after,
            }
        }
    }

    impl Write for FailingSeekableWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            if self.bytes_written >= self.fail_after {
                return Err(io::Error::new(io::ErrorKind::Other, "simulated write failure"));
            }
            let written = self.inner.write(buf)?;
            self.bytes_written += written;
            Ok(written)
        }

        fn flush(&mut self) -> io::Result<()> {
            if self.bytes_written >= self.fail_after {
                return Err(io::Error::new(io::ErrorKind::Other, "simulated flush failure"));
            }
            self.inner.flush()
        }
    }

    impl Seek for FailingSeekableWriter {
        fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
            self.inner.seek(pos)
        }
    }

    #[test]
    fn create_archive_with_failing_writer_returns_git_error() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id = gix_hash::ObjectId::from_hex(tree_id_str.as_bytes())
            .expect("failed to parse tree id");

        let mut writer = FailingWriter::new(100);
        let result = ops::create_archive(&handle, tree_id, ArchiveFormat::Tar, None, &mut writer);

        assert!(result.is_err());
        match result.unwrap_err() {
            SdkError::Git(_) => {}
            other => panic!("expected Git error, got: {:?}", other),
        }
    }

    #[test]
    fn create_archive_tar_gz_with_failing_writer_returns_git_error() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id = gix_hash::ObjectId::from_hex(tree_id_str.as_bytes())
            .expect("failed to parse tree id");

        let mut writer = FailingWriter::new(50);
        let result = ops::create_archive(
            &handle,
            tree_id,
            ArchiveFormat::TarGz { compression_level: None },
            None,
            &mut writer,
        );

        assert!(result.is_err());
        match result.unwrap_err() {
            SdkError::Git(_) => {}
            other => panic!("expected Git error, got: {:?}", other),
        }
    }

    #[test]
    fn create_archive_seekable_with_failing_writer_returns_git_error() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id = gix_hash::ObjectId::from_hex(tree_id_str.as_bytes())
            .expect("failed to parse tree id");

        let mut writer = FailingSeekableWriter::new(100);
        let result = ops::create_archive_seekable(
            &handle,
            tree_id,
            ArchiveFormat::Zip { compression_level: None },
            None,
            &mut writer,
        );

        assert!(result.is_err());
        match result.unwrap_err() {
            SdkError::Git(_) => {}
            other => panic!("expected Git error, got: {:?}", other),
        }
    }

    #[test]
    fn create_archive_seekable_tar_with_failing_writer_returns_git_error() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id = gix_hash::ObjectId::from_hex(tree_id_str.as_bytes())
            .expect("failed to parse tree id");

        let mut writer = FailingSeekableWriter::new(100);
        let result = ops::create_archive_seekable(
            &handle,
            tree_id,
            ArchiveFormat::Tar,
            None,
            &mut writer,
        );

        assert!(result.is_err());
        match result.unwrap_err() {
            SdkError::Git(_) => {}
            other => panic!("expected Git error, got: {:?}", other),
        }
    }

    #[test]
    fn create_archive_from_commit_with_failing_writer_returns_git_error() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let mut writer = FailingWriter::new(100);
        let result = ops::create_archive_from_commit(
            &handle,
            commit_id,
            ArchiveFormat::Tar,
            None,
            &mut writer,
        );

        assert!(result.is_err());
        match result.unwrap_err() {
            SdkError::Git(_) => {}
            other => panic!("expected Git error, got: {:?}", other),
        }
    }

    #[test]
    fn create_archive_from_commit_seekable_with_failing_writer_returns_git_error() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let mut writer = FailingSeekableWriter::new(100);
        let result = ops::create_archive_from_commit_seekable(
            &handle,
            commit_id,
            ArchiveFormat::Zip { compression_level: None },
            None,
            &mut writer,
        );

        assert!(result.is_err());
        match result.unwrap_err() {
            SdkError::Git(_) => {}
            other => panic!("expected Git error, got: {:?}", other),
        }
    }
}

mod archive_format_conversion {
    use gix_server_sdk::ops::ArchiveFormat;

    #[test]
    fn tar_format_converts_to_gix_archive_format() {
        let format = ArchiveFormat::Tar;
        let gix_format: gix_archive::Format = format.into();
        assert!(matches!(gix_format, gix_archive::Format::Tar));
    }

    #[test]
    fn tar_gz_format_converts_to_gix_archive_format() {
        let format = ArchiveFormat::TarGz { compression_level: None };
        let gix_format: gix_archive::Format = format.into();
        assert!(matches!(gix_format, gix_archive::Format::TarGz { compression_level: None }));
    }

    #[test]
    fn tar_gz_with_compression_converts_to_gix_archive_format() {
        let format = ArchiveFormat::TarGz { compression_level: Some(9) };
        let gix_format: gix_archive::Format = format.into();
        assert!(matches!(gix_format, gix_archive::Format::TarGz { compression_level: Some(9) }));
    }

    #[test]
    fn zip_format_converts_to_gix_archive_format() {
        let format = ArchiveFormat::Zip { compression_level: None };
        let gix_format: gix_archive::Format = format.into();
        assert!(matches!(gix_format, gix_archive::Format::Zip { compression_level: None }));
    }

    #[test]
    fn zip_with_compression_converts_to_gix_archive_format() {
        let format = ArchiveFormat::Zip { compression_level: Some(6) };
        let gix_format: gix_archive::Format = format.into();
        assert!(matches!(gix_format, gix_archive::Format::Zip { compression_level: Some(6) }));
    }
}

mod archive_empty_tree {
    use super::*;
    use gix_server_sdk::ops::ArchiveFormat;

    #[test]
    fn empty_tree_creates_valid_tar() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let empty_tree_hash = repo.git_output(&["hash-object", "-t", "tree", "--stdin", "-w"]);
        let tree_id = gix_hash::ObjectId::from_hex(empty_tree_hash.as_bytes())
            .expect("failed to parse tree id");

        let mut output: Vec<u8> = Vec::new();
        ops::create_archive(&handle, tree_id, ArchiveFormat::Tar, None, &mut output)
            .expect("failed to create archive");

        assert!(!output.is_empty());
    }

    #[test]
    fn empty_tree_creates_valid_tar_gz() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let empty_tree_hash = repo.git_output(&["hash-object", "-t", "tree", "--stdin", "-w"]);
        let tree_id = gix_hash::ObjectId::from_hex(empty_tree_hash.as_bytes())
            .expect("failed to parse tree id");

        let mut output: Vec<u8> = Vec::new();
        ops::create_archive(
            &handle,
            tree_id,
            ArchiveFormat::TarGz { compression_level: None },
            None,
            &mut output,
        )
        .expect("failed to create archive");

        assert!(!output.is_empty());
        assert!(output.starts_with(&[0x1f, 0x8b]));
    }

    #[test]
    fn empty_tree_seekable_creates_valid_zip() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let empty_tree_hash = repo.git_output(&["hash-object", "-t", "tree", "--stdin", "-w"]);
        let tree_id = gix_hash::ObjectId::from_hex(empty_tree_hash.as_bytes())
            .expect("failed to parse tree id");

        let mut output = Cursor::new(Vec::new());
        ops::create_archive_seekable(
            &handle,
            tree_id,
            ArchiveFormat::Zip { compression_level: None },
            None,
            &mut output,
        )
        .expect("failed to create archive");

        let data = output.into_inner();
        assert!(!data.is_empty());
    }

    #[test]
    fn empty_tree_with_prefix() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let empty_tree_hash = repo.git_output(&["hash-object", "-t", "tree", "--stdin", "-w"]);
        let tree_id = gix_hash::ObjectId::from_hex(empty_tree_hash.as_bytes())
            .expect("failed to parse tree id");

        let prefix = BString::from("empty-archive/");
        let mut output: Vec<u8> = Vec::new();
        ops::create_archive(&handle, tree_id, ArchiveFormat::Tar, Some(prefix), &mut output)
            .expect("failed to create archive");

        assert!(!output.is_empty());
    }
}

mod archive_compression_levels {
    use super::*;
    use gix_server_sdk::ops::ArchiveFormat;

    #[test]
    fn tar_gz_min_compression() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id = gix_hash::ObjectId::from_hex(tree_id_str.as_bytes())
            .expect("failed to parse tree id");

        let mut output: Vec<u8> = Vec::new();
        ops::create_archive(
            &handle,
            tree_id,
            ArchiveFormat::TarGz { compression_level: Some(1) },
            None,
            &mut output,
        )
        .expect("failed to create archive");

        assert!(!output.is_empty());
        assert!(output.starts_with(&[0x1f, 0x8b]));
    }

    #[test]
    fn zip_min_compression() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id = gix_hash::ObjectId::from_hex(tree_id_str.as_bytes())
            .expect("failed to parse tree id");

        let mut output = Cursor::new(Vec::new());
        ops::create_archive_seekable(
            &handle,
            tree_id,
            ArchiveFormat::Zip { compression_level: Some(1) },
            None,
            &mut output,
        )
        .expect("failed to create archive");

        let data = output.into_inner();
        assert!(!data.is_empty());
        assert!(data.starts_with(&[0x50, 0x4b]));
    }

    #[test]
    fn tar_gz_compression_affects_size() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id = gix_hash::ObjectId::from_hex(tree_id_str.as_bytes())
            .expect("failed to parse tree id");

        let mut output_low: Vec<u8> = Vec::new();
        ops::create_archive(
            &handle,
            tree_id,
            ArchiveFormat::TarGz { compression_level: Some(1) },
            None,
            &mut output_low,
        )
        .expect("failed to create low compression archive");

        let mut output_high: Vec<u8> = Vec::new();
        ops::create_archive(
            &handle,
            tree_id,
            ArchiveFormat::TarGz { compression_level: Some(9) },
            None,
            &mut output_high,
        )
        .expect("failed to create high compression archive");

        assert!(output_low.len() >= output_high.len());
    }

    #[test]
    fn zip_compression_produces_valid_output() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id = gix_hash::ObjectId::from_hex(tree_id_str.as_bytes())
            .expect("failed to parse tree id");

        for level in [1, 5, 9] {
            let mut output = Cursor::new(Vec::new());
            ops::create_archive_seekable(
                &handle,
                tree_id,
                ArchiveFormat::Zip { compression_level: Some(level) },
                None,
                &mut output,
            )
            .expect(&format!("failed to create archive with level {}", level));

            let data = output.into_inner();
            assert!(!data.is_empty());
            assert!(data.starts_with(&[0x50, 0x4b]));
        }
    }
}

mod archive_nested_tree_seekable {
    use super::*;
    use gix_server_sdk::ops::ArchiveFormat;

    #[test]
    fn nested_tree_tar_seekable() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD:src"]);
        let tree_id = gix_hash::ObjectId::from_hex(tree_id_str.as_bytes())
            .expect("failed to parse tree id");

        let mut output = Cursor::new(Vec::new());
        ops::create_archive_seekable(&handle, tree_id, ArchiveFormat::Tar, None, &mut output)
            .expect("failed to create archive");

        let data = output.into_inner();
        assert!(!data.is_empty());
    }

    #[test]
    fn nested_tree_tar_gz_seekable() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD:src"]);
        let tree_id = gix_hash::ObjectId::from_hex(tree_id_str.as_bytes())
            .expect("failed to parse tree id");

        let mut output = Cursor::new(Vec::new());
        ops::create_archive_seekable(
            &handle,
            tree_id,
            ArchiveFormat::TarGz { compression_level: None },
            None,
            &mut output,
        )
        .expect("failed to create archive");

        let data = output.into_inner();
        assert!(!data.is_empty());
        assert!(data.starts_with(&[0x1f, 0x8b]));
    }

    #[test]
    fn nested_tree_zip_seekable() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD:src"]);
        let tree_id = gix_hash::ObjectId::from_hex(tree_id_str.as_bytes())
            .expect("failed to parse tree id");

        let mut output = Cursor::new(Vec::new());
        ops::create_archive_seekable(
            &handle,
            tree_id,
            ArchiveFormat::Zip { compression_level: None },
            None,
            &mut output,
        )
        .expect("failed to create archive");

        let data = output.into_inner();
        assert!(!data.is_empty());
        assert!(data.starts_with(&[0x50, 0x4b]));
    }

    #[test]
    fn nested_tree_with_prefix() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD:src"]);
        let tree_id = gix_hash::ObjectId::from_hex(tree_id_str.as_bytes())
            .expect("failed to parse tree id");

        let prefix = BString::from("src-backup/");
        let mut output = Cursor::new(Vec::new());
        ops::create_archive_seekable(
            &handle,
            tree_id,
            ArchiveFormat::Zip { compression_level: None },
            Some(prefix),
            &mut output,
        )
        .expect("failed to create archive");

        let data = output.into_inner();
        assert!(!data.is_empty());
    }
}

mod archive_prefix_variations {
    use super::*;
    use gix_server_sdk::ops::ArchiveFormat;

    #[test]
    fn prefix_without_trailing_slash() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id = gix_hash::ObjectId::from_hex(tree_id_str.as_bytes())
            .expect("failed to parse tree id");

        let prefix = BString::from("no-trailing-slash");
        let mut output: Vec<u8> = Vec::new();
        ops::create_archive(&handle, tree_id, ArchiveFormat::Tar, Some(prefix), &mut output)
            .expect("failed to create archive");

        assert!(!output.is_empty());
    }

    #[test]
    fn prefix_with_nested_path() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id = gix_hash::ObjectId::from_hex(tree_id_str.as_bytes())
            .expect("failed to parse tree id");

        let prefix = BString::from("releases/v1.0.0/");
        let mut output: Vec<u8> = Vec::new();
        ops::create_archive(&handle, tree_id, ArchiveFormat::Tar, Some(prefix), &mut output)
            .expect("failed to create archive");

        assert!(!output.is_empty());
    }

    #[test]
    fn prefix_with_special_chars() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id = gix_hash::ObjectId::from_hex(tree_id_str.as_bytes())
            .expect("failed to parse tree id");

        let prefix = BString::from("project-name_v1.2.3-beta/");
        let mut output: Vec<u8> = Vec::new();
        ops::create_archive(&handle, tree_id, ArchiveFormat::Tar, Some(prefix), &mut output)
            .expect("failed to create archive");

        assert!(!output.is_empty());
    }

    #[test]
    fn empty_prefix_string() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id = gix_hash::ObjectId::from_hex(tree_id_str.as_bytes())
            .expect("failed to parse tree id");

        let prefix = BString::from("");
        let mut output: Vec<u8> = Vec::new();
        ops::create_archive(&handle, tree_id, ArchiveFormat::Tar, Some(prefix), &mut output)
            .expect("failed to create archive");

        assert!(!output.is_empty());
    }

    #[test]
    fn prefix_seekable_zip() {
        let repo = TestRepo::new();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id = gix_hash::ObjectId::from_hex(tree_id_str.as_bytes())
            .expect("failed to parse tree id");

        let prefix = BString::from("archive-dir/sub/");
        let mut output = Cursor::new(Vec::new());
        ops::create_archive_seekable(
            &handle,
            tree_id,
            ArchiveFormat::Zip { compression_level: Some(5) },
            Some(prefix),
            &mut output,
        )
        .expect("failed to create archive");

        let data = output.into_inner();
        assert!(!data.is_empty());
    }
}

mod archive_with_deep_nesting_repo {
    use super::*;
    use gix_server_sdk::ops::ArchiveFormat;

    #[test]
    fn deep_nesting_tar() {
        let repo = TestRepo::with_deep_nesting();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id = gix_hash::ObjectId::from_hex(tree_id_str.as_bytes())
            .expect("failed to parse tree id");

        let mut output: Vec<u8> = Vec::new();
        ops::create_archive(&handle, tree_id, ArchiveFormat::Tar, None, &mut output)
            .expect("failed to create archive");

        assert!(!output.is_empty());
    }

    #[test]
    fn deep_nesting_zip() {
        let repo = TestRepo::with_deep_nesting();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id = gix_hash::ObjectId::from_hex(tree_id_str.as_bytes())
            .expect("failed to parse tree id");

        let mut output = Cursor::new(Vec::new());
        ops::create_archive_seekable(
            &handle,
            tree_id,
            ArchiveFormat::Zip { compression_level: None },
            None,
            &mut output,
        )
        .expect("failed to create archive");

        let data = output.into_inner();
        assert!(!data.is_empty());
    }

    #[test]
    fn deep_nesting_subtree() {
        let repo = TestRepo::with_deep_nesting();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD:a/b"]);
        let tree_id = gix_hash::ObjectId::from_hex(tree_id_str.as_bytes())
            .expect("failed to parse tree id");

        let mut output: Vec<u8> = Vec::new();
        ops::create_archive(&handle, tree_id, ArchiveFormat::Tar, None, &mut output)
            .expect("failed to create archive");

        assert!(!output.is_empty());
    }
}

mod archive_with_single_file_repo {
    use super::*;
    use gix_server_sdk::ops::ArchiveFormat;

    #[test]
    fn single_file_tar() {
        let repo = TestRepo::single_file();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id = gix_hash::ObjectId::from_hex(tree_id_str.as_bytes())
            .expect("failed to parse tree id");

        let mut output: Vec<u8> = Vec::new();
        ops::create_archive(&handle, tree_id, ArchiveFormat::Tar, None, &mut output)
            .expect("failed to create archive");

        assert!(!output.is_empty());
    }

    #[test]
    fn single_file_tar_gz() {
        let repo = TestRepo::single_file();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id = gix_hash::ObjectId::from_hex(tree_id_str.as_bytes())
            .expect("failed to parse tree id");

        let mut output: Vec<u8> = Vec::new();
        ops::create_archive(
            &handle,
            tree_id,
            ArchiveFormat::TarGz { compression_level: None },
            None,
            &mut output,
        )
        .expect("failed to create archive");

        assert!(!output.is_empty());
        assert!(output.starts_with(&[0x1f, 0x8b]));
    }

    #[test]
    fn single_file_zip() {
        let repo = TestRepo::single_file();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tree_id_str = repo.git_output(&["rev-parse", "HEAD^{tree}"]);
        let tree_id = gix_hash::ObjectId::from_hex(tree_id_str.as_bytes())
            .expect("failed to parse tree id");

        let mut output = Cursor::new(Vec::new());
        ops::create_archive_seekable(
            &handle,
            tree_id,
            ArchiveFormat::Zip { compression_level: None },
            None,
            &mut output,
        )
        .expect("failed to create archive");

        let data = output.into_inner();
        assert!(!data.is_empty());
        assert!(data.starts_with(&[0x50, 0x4b]));
    }

    #[test]
    fn single_file_from_commit() {
        let repo = TestRepo::single_file();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let mut output: Vec<u8> = Vec::new();
        ops::create_archive_from_commit(&handle, commit_id, ArchiveFormat::Tar, None, &mut output)
            .expect("failed to create archive");

        assert!(!output.is_empty());
    }
}

mod archive_corrupted_objects {
    use super::*;
    use gix_server_sdk::ops::ArchiveFormat;

    #[test]
    fn corrupted_commit_returns_error() {
        let repo = TestRepo::with_corrupted_loose_object();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let mut output: Vec<u8> = Vec::new();
        let result = ops::create_archive_from_commit(
            &handle,
            commit_id,
            ArchiveFormat::Tar,
            None,
            &mut output,
        );

        assert!(result.is_err());
    }

    #[test]
    fn corrupted_commit_seekable_returns_error() {
        let repo = TestRepo::with_corrupted_loose_object();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let mut output = Cursor::new(Vec::new());
        let result = ops::create_archive_from_commit_seekable(
            &handle,
            commit_id,
            ArchiveFormat::Zip { compression_level: None },
            None,
            &mut output,
        );

        assert!(result.is_err());
    }

    #[test]
    fn missing_tree_in_commit_returns_error() {
        let repo = TestRepo::with_corrupt_tree_reference();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let commit_id_str = repo.git_output(&["rev-parse", "HEAD"]);
        let commit_id = gix_hash::ObjectId::from_hex(commit_id_str.as_bytes())
            .expect("failed to parse commit id");

        let mut output: Vec<u8> = Vec::new();
        let result = ops::create_archive_from_commit(
            &handle,
            commit_id,
            ArchiveFormat::Tar,
            None,
            &mut output,
        );

        assert!(result.is_err());
    }
}

mod archive_with_tag_objects {
    use super::*;
    use gix_server_sdk::ops::ArchiveFormat;

    #[test]
    fn annotated_tag_id_returns_invalid_object_type() {
        let repo = TestRepo::with_tags();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tag_id_str = repo.git_output(&["rev-parse", "v1.1.0"]);
        let tag_id = gix_hash::ObjectId::from_hex(tag_id_str.as_bytes())
            .expect("failed to parse tag id");

        let mut output: Vec<u8> = Vec::new();
        let result = ops::create_archive_from_commit(&handle, tag_id, ArchiveFormat::Tar, None, &mut output);

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
    fn annotated_tag_id_seekable_returns_invalid_object_type() {
        let repo = TestRepo::with_tags();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tag_id_str = repo.git_output(&["rev-parse", "v1.1.0"]);
        let tag_id = gix_hash::ObjectId::from_hex(tag_id_str.as_bytes())
            .expect("failed to parse tag id");

        let mut output = Cursor::new(Vec::new());
        let result = ops::create_archive_from_commit_seekable(
            &handle,
            tag_id,
            ArchiveFormat::Zip { compression_level: None },
            None,
            &mut output,
        );

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
    fn lightweight_tag_works_as_commit() {
        let repo = TestRepo::with_tags();
        let pool = create_pool();
        let handle = pool.get(&repo.path).expect("failed to get handle");

        let tag_id_str = repo.git_output(&["rev-parse", "v1.0.0"]);
        let commit_id = gix_hash::ObjectId::from_hex(tag_id_str.as_bytes())
            .expect("failed to parse tag id");

        let mut output: Vec<u8> = Vec::new();
        let result = ops::create_archive_from_commit(&handle, commit_id, ArchiveFormat::Tar, None, &mut output);

        assert!(result.is_ok());
        assert!(!output.is_empty());
    }
}
