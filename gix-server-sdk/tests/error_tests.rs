use gix_server_sdk::SdkError;
use std::path::PathBuf;

mod display_tests {
    use super::*;

    #[test]
    fn repo_not_found_display() {
        let path = PathBuf::from("/some/repo/path");
        let err = SdkError::RepoNotFound(path.clone());
        let msg = err.to_string();
        assert!(msg.contains("Repository not found"));
        assert!(msg.contains("/some/repo/path"));
    }

    #[test]
    fn object_not_found_display() {
        let id = gix_hash::ObjectId::empty_blob(gix_hash::Kind::Sha1);
        let err = SdkError::ObjectNotFound(id);
        let msg = err.to_string();
        assert!(msg.contains("Object not found"));
        assert!(msg.contains(&id.to_string()));
    }

    #[test]
    fn ref_not_found_display() {
        let err = SdkError::RefNotFound("refs/heads/main".to_string());
        let msg = err.to_string();
        assert!(msg.contains("Reference not found"));
        assert!(msg.contains("refs/heads/main"));
    }

    #[test]
    fn tree_entry_not_found_display() {
        let err = SdkError::TreeEntryNotFound("src/lib.rs".to_string());
        let msg = err.to_string();
        assert!(msg.contains("Tree entry not found"));
        assert!(msg.contains("src/lib.rs"));
    }

    #[test]
    fn invalid_object_type_display() {
        let err = SdkError::InvalidObjectType {
            expected: "blob".to_string(),
            actual: "tree".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("Invalid object type"));
        assert!(msg.contains("expected blob"));
        assert!(msg.contains("got tree"));
    }

    #[test]
    fn invalid_revision_display() {
        let err = SdkError::InvalidRevision("HEAD^^^".to_string());
        let msg = err.to_string();
        assert!(msg.contains("Invalid revision spec"));
        assert!(msg.contains("HEAD^^^"));
    }

    #[test]
    fn operation_display() {
        let err = SdkError::Operation("something went wrong".to_string());
        let msg = err.to_string();
        assert!(msg.contains("Operation failed"));
        assert!(msg.contains("something went wrong"));
    }

    #[test]
    fn io_error_display() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = SdkError::Io(io_err);
        let msg = err.to_string();
        assert!(msg.contains("IO error"));
        assert!(msg.contains("file not found"));
    }

    #[test]
    fn git_error_display() {
        let boxed_err: Box<dyn std::error::Error + Send + Sync> =
            Box::new(std::io::Error::new(std::io::ErrorKind::Other, "git error"));
        let err = SdkError::Git(boxed_err);
        let msg = err.to_string();
        assert!(msg.contains("git error"));
    }
}

mod from_conversion_tests {
    use super::*;

    #[test]
    fn from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let sdk_err: SdkError = io_err.into();
        match sdk_err {
            SdkError::Io(e) => {
                assert_eq!(e.kind(), std::io::ErrorKind::PermissionDenied);
            }
            _ => panic!("expected SdkError::Io"),
        }
    }

    #[test]
    fn from_boxed_error() {
        let boxed_err: Box<dyn std::error::Error + Send + Sync> =
            Box::new(std::io::Error::new(std::io::ErrorKind::Other, "boxed error"));
        let sdk_err: SdkError = boxed_err.into();
        match sdk_err {
            SdkError::Git(_) => {}
            _ => panic!("expected SdkError::Git"),
        }
    }

    #[test]
    fn from_gix_hash_decode_error() {
        let invalid_hex = "not_valid_hex";
        let result: Result<gix_hash::ObjectId, _> = invalid_hex.parse();
        if let Err(decode_err) = result {
            let sdk_err: SdkError = decode_err.into();
            match sdk_err {
                SdkError::Git(_) => {}
                _ => panic!("expected SdkError::Git"),
            }
        }
    }

    #[test]
    fn from_gix_object_decode_error() {
        // Create invalid commit data to trigger decode error
        let invalid_commit_data = b"invalid commit data";
        let result = gix_object::CommitRef::from_bytes(invalid_commit_data);
        if let Err(decode_err) = result {
            let sdk_err: SdkError = decode_err.into();
            match sdk_err {
                SdkError::Git(_) => {}
                _ => panic!("expected SdkError::Git"),
            }
        }
    }

    #[test]
    fn from_gix_ref_file_find_error() {
        use gix_ref::file::find::Error;
        // Create a find error using ReadFileContents variant
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let find_err = Error::ReadFileContents {
            source: io_err,
            path: std::path::PathBuf::from("/some/ref/path"),
        };
        let sdk_err: SdkError = find_err.into();
        match sdk_err {
            SdkError::Git(_) => {}
            _ => panic!("expected SdkError::Git"),
        }
    }

    #[test]
    fn from_gix_ref_file_find_existing_error() {
        use gix_ref::file::find::existing::Error;
        let find_err = Error::NotFound {
            name: std::path::PathBuf::from("refs/heads/nonexistent"),
        };
        let sdk_err: SdkError = find_err.into();
        match sdk_err {
            SdkError::Git(_) => {}
            _ => panic!("expected SdkError::Git"),
        }
    }

    #[test]
    fn from_gix_odb_store_find_error() {
        use gix_odb::store::find::Error;
        // Create error using LoadPack variant which wraps std::io::Error
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "pack not found");
        let find_err = Error::LoadPack(io_err);
        let sdk_err: SdkError = find_err.into();
        match sdk_err {
            SdkError::Git(_) => {}
            _ => panic!("expected SdkError::Git"),
        }
    }

    #[test]
    fn from_gix_traverse_commit_simple_error() {
        use gix_traverse::commit::simple::Error;
        // Create error using ObjectDecode variant
        let invalid_data = b"invalid";
        let decode_err = gix_object::CommitRef::from_bytes(invalid_data).unwrap_err();
        let traverse_err = Error::ObjectDecode(decode_err);
        let sdk_err: SdkError = traverse_err.into();
        match sdk_err {
            SdkError::Git(_) => {}
            _ => panic!("expected SdkError::Git"),
        }
    }

    #[test]
    fn from_gix_diff_tree_error() {
        use gix_diff::tree::Error;
        // Create error using EntriesDecode variant
        let invalid_data = b"invalid tree";
        let decode_err = gix_object::TreeRef::from_bytes(invalid_data).unwrap_err();
        let diff_err = Error::EntriesDecode(decode_err);
        let sdk_err: SdkError = diff_err.into();
        match sdk_err {
            SdkError::Git(_) => {}
            _ => panic!("expected SdkError::Git"),
        }
    }

    #[test]
    fn from_gix_diff_tree_error_cancelled() {
        use gix_diff::tree::Error;
        // Create the Cancelled variant
        let diff_err = Error::Cancelled;
        let sdk_err: SdkError = diff_err.into();
        match sdk_err {
            SdkError::Git(_) => {}
            _ => panic!("expected SdkError::Git"),
        }
    }
}

mod error_trait_tests {
    use super::*;

    #[test]
    fn sdk_error_implements_error_trait() {
        let err = SdkError::Operation("test".to_string());
        let _: &dyn std::error::Error = &err;
    }

    #[test]
    fn sdk_error_implements_send() {
        fn assert_send<T: Send>() {}
        assert_send::<SdkError>();
    }

    #[test]
    fn sdk_error_implements_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<SdkError>();
    }

    #[test]
    fn sdk_error_source_for_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "source error");
        let err = SdkError::Io(io_err);
        let source = std::error::Error::source(&err);
        assert!(source.is_some());
    }

    #[test]
    fn sdk_error_source_for_git_transparent() {
        let boxed_err: Box<dyn std::error::Error + Send + Sync> =
            Box::new(std::io::Error::new(std::io::ErrorKind::Other, "inner"));
        let err = SdkError::Git(boxed_err);
        let msg = err.to_string();
        assert!(msg.contains("inner"));
    }

    #[test]
    fn sdk_error_source_for_simple_variants() {
        let err = SdkError::Operation("op".to_string());
        let source = std::error::Error::source(&err);
        assert!(source.is_none());
    }
}

mod debug_tests {
    use super::*;

    #[test]
    fn sdk_error_implements_debug() {
        let err = SdkError::Operation("debug test".to_string());
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("Operation"));
    }

    #[test]
    fn all_variants_debug() {
        let path = PathBuf::from("/path");
        let id = gix_hash::ObjectId::empty_blob(gix_hash::Kind::Sha1);

        let variants: Vec<SdkError> = vec![
            SdkError::RepoNotFound(path),
            SdkError::ObjectNotFound(id),
            SdkError::RefNotFound("ref".to_string()),
            SdkError::TreeEntryNotFound("entry".to_string()),
            SdkError::InvalidObjectType {
                expected: "a".to_string(),
                actual: "b".to_string(),
            },
            SdkError::InvalidRevision("rev".to_string()),
            SdkError::Operation("op".to_string()),
            SdkError::Io(std::io::Error::new(std::io::ErrorKind::Other, "io")),
            SdkError::Git(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "git",
            ))),
        ];

        for err in variants {
            let debug_str = format!("{:?}", err);
            assert!(!debug_str.is_empty());
        }
    }
}
