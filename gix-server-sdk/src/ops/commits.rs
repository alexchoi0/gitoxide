use bstr::BString;
use gix_hash::ObjectId;
use gix_object::FindExt;

use crate::error::{Result, SdkError};
use crate::pool::RepoHandle;
use crate::types::{CommitInfo, Signature};

pub fn get_commit(repo: &RepoHandle, id: ObjectId) -> Result<CommitInfo> {
    let local = repo.to_local();
    let mut buf = Vec::new();

    let commit = local
        .objects
        .find_commit(&id, &mut buf)
        .map_err(|e| SdkError::Git(Box::new(e)))?;

    let tree_id = commit.tree();
    let parent_ids: Vec<ObjectId> = commit.parents().collect();
    let author_sig = commit.author().map_err(|e| SdkError::Git(Box::new(e)))?;
    let committer_sig = commit.committer().map_err(|e| SdkError::Git(Box::new(e)))?;

    Ok(CommitInfo {
        id,
        tree_id,
        parent_ids,
        author: Signature {
            name: author_sig.name.into(),
            email: author_sig.email.into(),
            time: author_sig.time().map(|t| t.seconds).unwrap_or(0),
        },
        committer: Signature {
            name: committer_sig.name.into(),
            email: committer_sig.email.into(),
            time: committer_sig.time().map(|t| t.seconds).unwrap_or(0),
        },
        message: BString::from(commit.message),
    })
}

pub fn log(repo: &RepoHandle, start: ObjectId, limit: Option<usize>) -> Result<Vec<CommitInfo>> {
    let local = repo.to_local();

    let walk = gix_traverse::commit::Simple::new([start], &local.objects)
        .sorting(gix_traverse::commit::simple::Sorting::ByCommitTime(
            gix_traverse::commit::simple::CommitTimeOrder::NewestFirst,
        ))
        .map_err(|e| SdkError::Git(Box::new(e)))?;

    let mut commits = Vec::new();
    let mut buf = Vec::new();

    for info in walk {
        let info = info.map_err(|e| SdkError::Git(Box::new(e)))?;

        let commit = local
            .objects
            .find_commit(&info.id, &mut buf)
            .map_err(|e| SdkError::Git(Box::new(e)))?;

        let tree_id = commit.tree();
        let parent_ids: Vec<ObjectId> = commit.parents().collect();
        let author_sig = commit.author().map_err(|e| SdkError::Git(Box::new(e)))?;
        let committer_sig = commit.committer().map_err(|e| SdkError::Git(Box::new(e)))?;

        commits.push(CommitInfo {
            id: info.id,
            tree_id,
            parent_ids,
            author: Signature {
                name: author_sig.name.into(),
                email: author_sig.email.into(),
                time: author_sig.time().map(|t| t.seconds).unwrap_or(0),
            },
            committer: Signature {
                name: committer_sig.name.into(),
                email: committer_sig.email.into(),
                time: committer_sig.time().map(|t| t.seconds).unwrap_or(0),
            },
            message: BString::from(commit.message),
        });

        if let Some(max) = limit {
            if commits.len() >= max {
                break;
            }
        }
    }

    Ok(commits)
}

pub fn log_with_path(
    repo: &RepoHandle,
    start: ObjectId,
    path: &str,
    limit: Option<usize>,
) -> Result<Vec<CommitInfo>> {
    let local = repo.to_local();
    let path_bytes = path.as_bytes();

    let walk = gix_traverse::commit::Simple::new([start], &local.objects)
        .sorting(gix_traverse::commit::simple::Sorting::ByCommitTime(
            gix_traverse::commit::simple::CommitTimeOrder::NewestFirst,
        ))
        .map_err(|e| SdkError::Git(Box::new(e)))?;

    let mut commits = Vec::new();
    let mut buf = Vec::new();
    let mut diff_state = gix_diff::tree::State::default();

    for info in walk {
        let info = info.map_err(|e| SdkError::Git(Box::new(e)))?;

        let commit = local
            .objects
            .find_commit(&info.id, &mut buf)
            .map_err(|e| SdkError::Git(Box::new(e)))?;

        let tree_id = commit.tree();

        let modified_path = if info.parent_ids.is_empty() {
            tree_contains_path(&local.objects, tree_id, path_bytes)?
        } else {
            let mut found = false;
            for parent_id in &info.parent_ids {
                let mut parent_buf = Vec::new();
                let parent_commit = local
                    .objects
                    .find_commit(parent_id, &mut parent_buf)
                    .map_err(|e| SdkError::Git(Box::new(e)))?;
                let parent_tree_id = parent_commit.tree();

                if path_changed_between_trees(
                    &local.objects,
                    parent_tree_id,
                    tree_id,
                    path_bytes,
                    &mut diff_state,
                )
                .unwrap_or(false)
                {
                    found = true;
                    break;
                }
            }
            found
        };

        if modified_path {
            let author_sig = commit.author().map_err(|e| SdkError::Git(Box::new(e)))?;
            let committer_sig = commit.committer().map_err(|e| SdkError::Git(Box::new(e)))?;

            commits.push(CommitInfo {
                id: info.id,
                tree_id,
                parent_ids: info.parent_ids.to_vec(),
                author: Signature {
                    name: author_sig.name.into(),
                    email: author_sig.email.into(),
                    time: author_sig.time().map(|t| t.seconds).unwrap_or(0),
                },
                committer: Signature {
                    name: committer_sig.name.into(),
                    email: committer_sig.email.into(),
                    time: committer_sig.time().map(|t| t.seconds).unwrap_or(0),
                },
                message: BString::from(commit.message),
            });

            if let Some(max) = limit {
                if commits.len() >= max {
                    break;
                }
            }
        }
    }

    Ok(commits)
}

fn tree_contains_path(
    objects: &impl gix_object::Find,
    tree_id: ObjectId,
    path: &[u8],
) -> Result<bool> {
    let mut buf = Vec::new();
    let mut current_tree_id = tree_id;

    for component in path.split(|&b| b == b'/') {
        if component.is_empty() {
            continue;
        }

        let tree = objects
            .find_tree_iter(&current_tree_id, &mut buf)
            .map_err(|e| SdkError::Git(Box::new(e)))?;

        let mut found = None;
        for entry in tree {
            let entry = entry.map_err(|e| SdkError::Git(Box::new(e)))?;
            if entry.filename == component {
                found = Some((entry.oid.to_owned(), entry.mode.is_tree()));
                break;
            }
        }

        match found {
            Some((oid, is_tree)) => {
                if is_tree {
                    current_tree_id = oid;
                } else {
                    return Ok(true);
                }
            }
            None => return Ok(false),
        }
    }

    Ok(true)
}

fn path_changed_between_trees(
    objects: &impl gix_object::Find,
    lhs_tree: ObjectId,
    rhs_tree: ObjectId,
    path: &[u8],
    state: &mut gix_diff::tree::State,
) -> Result<bool> {
    if lhs_tree == rhs_tree {
        return Ok(false);
    }

    let mut lhs_buf = Vec::new();
    let mut rhs_buf = Vec::new();

    let lhs_iter = objects
        .find_tree_iter(&lhs_tree, &mut lhs_buf)
        .map_err(|e| SdkError::Git(Box::new(e)))?;
    let rhs_iter = objects
        .find_tree_iter(&rhs_tree, &mut rhs_buf)
        .map_err(|e| SdkError::Git(Box::new(e)))?;

    let mut recorder = PathChangeRecorder {
        target_path: path.to_vec(),
        current_path: Vec::new(),
        changed: false,
    };

    match gix_diff::tree(lhs_iter, rhs_iter, state, objects, &mut recorder) {
        Ok(()) => Ok(recorder.changed),
        Err(gix_diff::tree::Error::Cancelled) => Ok(recorder.changed),
        Err(e) => Err(SdkError::Git(Box::new(e))),
    }
}

struct PathChangeRecorder {
    target_path: Vec<u8>,
    current_path: Vec<u8>,
    changed: bool,
}

impl gix_diff::tree::Visit for PathChangeRecorder {
    fn pop_front_tracked_path_and_set_current(&mut self) {}

    fn push_back_tracked_path_component(&mut self, component: &bstr::BStr) {
        if !self.current_path.is_empty() {
            self.current_path.push(b'/');
        }
        self.current_path.extend_from_slice(component);
    }

    fn push_path_component(&mut self, component: &bstr::BStr) {
        if !self.current_path.is_empty() {
            self.current_path.push(b'/');
        }
        self.current_path.extend_from_slice(component);
    }

    fn pop_path_component(&mut self) {
        if let Some(pos) = self.current_path.iter().rposition(|&b| b == b'/') {
            self.current_path.truncate(pos);
        } else {
            self.current_path.clear();
        }
    }

    fn visit(&mut self, _change: gix_diff::tree::visit::Change) -> gix_diff::tree::visit::Action {
        if self.current_path == self.target_path
            || self.target_path.starts_with(&self.current_path)
            || self.current_path.starts_with(&self.target_path)
        {
            self.changed = true;
            return gix_diff::tree::visit::Action::Cancel;
        }
        gix_diff::tree::visit::Action::Continue
    }
}

pub fn merge_base(
    repo: &RepoHandle,
    commit1: ObjectId,
    commit2: ObjectId,
) -> Result<ObjectId> {
    let local = repo.to_local();
    let cache = local.commit_graph_if_enabled().ok().flatten();
    let mut graph = gix_revwalk::Graph::new(&local.objects, cache.as_ref());

    let bases = gix_revision::merge_base(commit1, &[commit2], &mut graph)
        .map_err(|e| SdkError::Git(Box::new(e)))?;

    match bases {
        Some(ids) if !ids.is_empty() => Ok(ids[0]),
        _ => Err(SdkError::Operation(format!(
            "no merge base found between {} and {}",
            commit1, commit2
        ))),
    }
}

pub fn is_ancestor(repo: &RepoHandle, ancestor: ObjectId, descendant: ObjectId) -> Result<bool> {
    if ancestor == descendant {
        return Ok(true);
    }

    let local = repo.to_local();

    let walk = gix_traverse::commit::Simple::new([descendant], &local.objects)
        .sorting(gix_traverse::commit::simple::Sorting::ByCommitTime(
            gix_traverse::commit::simple::CommitTimeOrder::NewestFirst,
        ))
        .map_err(|e| SdkError::Git(Box::new(e)))?;

    for info in walk {
        let info = info.map_err(|e| SdkError::Git(Box::new(e)))?;
        if info.id == ancestor {
            return Ok(true);
        }
    }

    Ok(false)
}

pub fn count_commits(
    repo: &RepoHandle,
    start: ObjectId,
    stop: Option<ObjectId>,
) -> Result<usize> {
    let local = repo.to_local();

    let walk = gix_traverse::commit::Simple::new([start], &local.objects)
        .sorting(gix_traverse::commit::simple::Sorting::ByCommitTime(
            gix_traverse::commit::simple::CommitTimeOrder::NewestFirst,
        ))
        .map_err(|e| SdkError::Git(Box::new(e)))?;

    let mut count = 0;

    for info in walk {
        let info = info.map_err(|e| SdkError::Git(Box::new(e)))?;

        if let Some(stop_id) = stop {
            if info.id == stop_id {
                break;
            }
        }

        count += 1;
    }

    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gix_diff::tree::Visit;
    use gix_object::tree::EntryKind;

    #[test]
    fn path_change_recorder_push_path_component_empty_path() {
        let mut recorder = PathChangeRecorder {
            target_path: b"src/main.rs".to_vec(),
            current_path: Vec::new(),
            changed: false,
        };

        recorder.push_path_component(b"src".into());
        assert_eq!(recorder.current_path, b"src");
    }

    #[test]
    fn path_change_recorder_push_path_component_adds_separator() {
        let mut recorder = PathChangeRecorder {
            target_path: b"src/main.rs".to_vec(),
            current_path: b"src".to_vec(),
            changed: false,
        };

        recorder.push_path_component(b"main.rs".into());
        assert_eq!(recorder.current_path, b"src/main.rs");
    }

    #[test]
    fn path_change_recorder_push_back_tracked_path_component_empty() {
        let mut recorder = PathChangeRecorder {
            target_path: b"a/b/c".to_vec(),
            current_path: Vec::new(),
            changed: false,
        };

        recorder.push_back_tracked_path_component(b"a".into());
        assert_eq!(recorder.current_path, b"a");
    }

    #[test]
    fn path_change_recorder_push_back_tracked_path_component_with_separator() {
        let mut recorder = PathChangeRecorder {
            target_path: b"a/b/c".to_vec(),
            current_path: b"a".to_vec(),
            changed: false,
        };

        recorder.push_back_tracked_path_component(b"b".into());
        assert_eq!(recorder.current_path, b"a/b");
    }

    #[test]
    fn path_change_recorder_pop_path_component_single() {
        let mut recorder = PathChangeRecorder {
            target_path: b"src".to_vec(),
            current_path: b"src".to_vec(),
            changed: false,
        };

        recorder.pop_path_component();
        assert!(recorder.current_path.is_empty());
    }

    #[test]
    fn path_change_recorder_pop_path_component_nested() {
        let mut recorder = PathChangeRecorder {
            target_path: b"src/main.rs".to_vec(),
            current_path: b"src/main.rs".to_vec(),
            changed: false,
        };

        recorder.pop_path_component();
        assert_eq!(recorder.current_path, b"src");
    }

    #[test]
    fn path_change_recorder_pop_path_component_deeply_nested() {
        let mut recorder = PathChangeRecorder {
            target_path: b"a/b/c/d".to_vec(),
            current_path: b"a/b/c/d".to_vec(),
            changed: false,
        };

        recorder.pop_path_component();
        assert_eq!(recorder.current_path, b"a/b/c");

        recorder.pop_path_component();
        assert_eq!(recorder.current_path, b"a/b");

        recorder.pop_path_component();
        assert_eq!(recorder.current_path, b"a");

        recorder.pop_path_component();
        assert!(recorder.current_path.is_empty());
    }

    #[test]
    fn path_change_recorder_pop_front_tracked_path_and_set_current_is_noop() {
        let mut recorder = PathChangeRecorder {
            target_path: b"test".to_vec(),
            current_path: b"something".to_vec(),
            changed: false,
        };

        recorder.pop_front_tracked_path_and_set_current();
        assert_eq!(recorder.current_path, b"something");
    }

    #[test]
    fn path_change_recorder_visit_exact_match_cancels() {
        let mut recorder = PathChangeRecorder {
            target_path: b"src/lib.rs".to_vec(),
            current_path: b"src/lib.rs".to_vec(),
            changed: false,
        };

        let change = gix_diff::tree::visit::Change::Modification {
            previous_entry_mode: EntryKind::Blob.into(),
            previous_oid: gix_hash::ObjectId::null(gix_hash::Kind::Sha1),
            entry_mode: EntryKind::Blob.into(),
            oid: gix_hash::ObjectId::null(gix_hash::Kind::Sha1),
        };

        let action = recorder.visit(change);
        assert!(recorder.changed);
        assert!(action.cancelled());
    }

    #[test]
    fn path_change_recorder_visit_target_starts_with_current() {
        let mut recorder = PathChangeRecorder {
            target_path: b"src/main.rs".to_vec(),
            current_path: b"src".to_vec(),
            changed: false,
        };

        let change = gix_diff::tree::visit::Change::Addition {
            entry_mode: EntryKind::Tree.into(),
            oid: gix_hash::ObjectId::null(gix_hash::Kind::Sha1),
            relation: None,
        };

        let action = recorder.visit(change);
        assert!(recorder.changed);
        assert!(action.cancelled());
    }

    #[test]
    fn path_change_recorder_visit_current_starts_with_target() {
        let mut recorder = PathChangeRecorder {
            target_path: b"src".to_vec(),
            current_path: b"src/lib.rs".to_vec(),
            changed: false,
        };

        let change = gix_diff::tree::visit::Change::Deletion {
            entry_mode: EntryKind::Blob.into(),
            oid: gix_hash::ObjectId::null(gix_hash::Kind::Sha1),
            relation: None,
        };

        let action = recorder.visit(change);
        assert!(recorder.changed);
        assert!(action.cancelled());
    }

    #[test]
    fn path_change_recorder_visit_no_match_continues() {
        let mut recorder = PathChangeRecorder {
            target_path: b"src/lib.rs".to_vec(),
            current_path: b"tests/mod.rs".to_vec(),
            changed: false,
        };

        let change = gix_diff::tree::visit::Change::Modification {
            previous_entry_mode: EntryKind::Blob.into(),
            previous_oid: gix_hash::ObjectId::null(gix_hash::Kind::Sha1),
            entry_mode: EntryKind::Blob.into(),
            oid: gix_hash::ObjectId::null(gix_hash::Kind::Sha1),
        };

        let action = recorder.visit(change);
        assert!(!recorder.changed);
        assert!(!action.cancelled());
    }

    #[test]
    fn path_change_recorder_full_traversal_simulation() {
        let mut recorder = PathChangeRecorder {
            target_path: b"a/b/c.txt".to_vec(),
            current_path: Vec::new(),
            changed: false,
        };

        recorder.push_path_component(b"a".into());
        assert_eq!(recorder.current_path, b"a");

        recorder.push_path_component(b"b".into());
        assert_eq!(recorder.current_path, b"a/b");

        recorder.push_path_component(b"c.txt".into());
        assert_eq!(recorder.current_path, b"a/b/c.txt");

        let change = gix_diff::tree::visit::Change::Modification {
            previous_entry_mode: EntryKind::Blob.into(),
            previous_oid: gix_hash::ObjectId::null(gix_hash::Kind::Sha1),
            entry_mode: EntryKind::Blob.into(),
            oid: gix_hash::ObjectId::null(gix_hash::Kind::Sha1),
        };

        let action = recorder.visit(change);
        assert!(recorder.changed);
        assert!(action.cancelled());

        recorder.pop_path_component();
        assert_eq!(recorder.current_path, b"a/b");

        recorder.pop_path_component();
        assert_eq!(recorder.current_path, b"a");

        recorder.pop_path_component();
        assert!(recorder.current_path.is_empty());
    }

    #[test]
    fn path_change_recorder_push_back_tracked_multiple() {
        let mut recorder = PathChangeRecorder {
            target_path: b"x/y/z".to_vec(),
            current_path: Vec::new(),
            changed: false,
        };

        recorder.push_back_tracked_path_component(b"x".into());
        recorder.push_back_tracked_path_component(b"y".into());
        recorder.push_back_tracked_path_component(b"z".into());

        assert_eq!(recorder.current_path, b"x/y/z");
    }

    #[test]
    fn path_change_recorder_pop_empty_path() {
        let mut recorder = PathChangeRecorder {
            target_path: b"test".to_vec(),
            current_path: Vec::new(),
            changed: false,
        };

        recorder.pop_path_component();
        assert!(recorder.current_path.is_empty());
    }

    mod tree_contains_path_tests {
        use gix_object::{tree::Entry, WriteTo};
        use gix_hash::ObjectId;
        use gix_object::tree::EntryKind;
        use std::collections::HashMap;

        struct MockObjects {
            trees: HashMap<ObjectId, Vec<u8>>,
        }

        #[coverage(off)]
        impl gix_object::Find for MockObjects {
            fn try_find<'a>(
                &self,
                id: &gix_hash::oid,
                buffer: &'a mut Vec<u8>,
            ) -> Result<Option<gix_object::Data<'a>>, gix_object::find::Error> {
                if let Some(data) = self.trees.get(id) {
                    buffer.clear();
                    buffer.extend_from_slice(data);
                    Ok(Some(gix_object::Data {
                        kind: gix_object::Kind::Tree,
                        data: buffer.as_slice(),
                    }))
                } else {
                    Ok(None)
                }
            }
        }

        fn create_tree_data(entries: &[(&[u8], EntryKind, ObjectId)]) -> Vec<u8> {
            let mut tree = gix_object::Tree::empty();
            for (name, kind, oid) in entries {
                tree.entries.push(Entry {
                    mode: (*kind).into(),
                    filename: (*name).into(),
                    oid: *oid,
                });
            }
            let mut buf = Vec::new();
            tree.write_to(&mut buf).expect("failed to write tree");
            buf
        }

        fn id_from_byte(b: u8) -> ObjectId {
            let mut bytes = [0u8; 20];
            bytes[0] = b;
            ObjectId::from(bytes)
        }

        #[test]
        fn path_with_empty_components_skipped() {
            let tree_id = id_from_byte(1);
            let blob_id = id_from_byte(2);

            let tree_data = create_tree_data(&[
                (b"file.txt", EntryKind::Blob, blob_id),
            ]);

            let mut objects = MockObjects {
                trees: HashMap::new(),
            };
            objects.trees.insert(tree_id, tree_data);

            let result = super::super::tree_contains_path(&objects, tree_id, b"//file.txt");
            assert!(result.is_ok());
            assert!(result.unwrap());

            let result = super::super::tree_contains_path(&objects, tree_id, b"file.txt//");
            assert!(result.is_ok());
        }

        #[test]
        fn path_not_found_returns_false() {
            let tree_id = id_from_byte(1);
            let blob_id = id_from_byte(2);

            let tree_data = create_tree_data(&[
                (b"existing.txt", EntryKind::Blob, blob_id),
            ]);

            let mut objects = MockObjects {
                trees: HashMap::new(),
            };
            objects.trees.insert(tree_id, tree_data);

            let result = super::super::tree_contains_path(&objects, tree_id, b"nonexistent.txt");
            assert!(result.is_ok());
            assert!(!result.unwrap());
        }

        #[test]
        fn file_in_nested_directory() {
            let root_tree_id = id_from_byte(1);
            let src_tree_id = id_from_byte(2);
            let blob_id = id_from_byte(3);

            let src_tree_data = create_tree_data(&[
                (b"main.rs", EntryKind::Blob, blob_id),
            ]);

            let root_tree_data = create_tree_data(&[
                (b"src", EntryKind::Tree, src_tree_id),
            ]);

            let mut objects = MockObjects {
                trees: HashMap::new(),
            };
            objects.trees.insert(root_tree_id, root_tree_data);
            objects.trees.insert(src_tree_id, src_tree_data);

            let result = super::super::tree_contains_path(&objects, root_tree_id, b"src/main.rs");
            assert!(result.is_ok());
            assert!(result.unwrap());
        }

        #[test]
        fn directory_path_returns_true() {
            let root_tree_id = id_from_byte(1);
            let src_tree_id = id_from_byte(2);
            let blob_id = id_from_byte(3);

            let src_tree_data = create_tree_data(&[
                (b"main.rs", EntryKind::Blob, blob_id),
            ]);

            let root_tree_data = create_tree_data(&[
                (b"src", EntryKind::Tree, src_tree_id),
            ]);

            let mut objects = MockObjects {
                trees: HashMap::new(),
            };
            objects.trees.insert(root_tree_id, root_tree_data);
            objects.trees.insert(src_tree_id, src_tree_data);

            let result = super::super::tree_contains_path(&objects, root_tree_id, b"src");
            assert!(result.is_ok());
            assert!(result.unwrap());
        }

        #[test]
        fn deeply_nested_file() {
            let root_id = id_from_byte(1);
            let a_id = id_from_byte(2);
            let b_id = id_from_byte(3);
            let c_id = id_from_byte(4);
            let blob_id = id_from_byte(5);

            let c_tree = create_tree_data(&[
                (b"deep.txt", EntryKind::Blob, blob_id),
            ]);
            let b_tree = create_tree_data(&[
                (b"c", EntryKind::Tree, c_id),
            ]);
            let a_tree = create_tree_data(&[
                (b"b", EntryKind::Tree, b_id),
            ]);
            let root_tree = create_tree_data(&[
                (b"a", EntryKind::Tree, a_id),
            ]);

            let mut objects = MockObjects {
                trees: HashMap::new(),
            };
            objects.trees.insert(root_id, root_tree);
            objects.trees.insert(a_id, a_tree);
            objects.trees.insert(b_id, b_tree);
            objects.trees.insert(c_id, c_tree);

            let result = super::super::tree_contains_path(&objects, root_id, b"a/b/c/deep.txt");
            assert!(result.is_ok());
            assert!(result.unwrap());
        }

        #[test]
        fn partial_path_not_found_in_nested_tree() {
            let root_id = id_from_byte(1);
            let src_id = id_from_byte(2);
            let blob_id = id_from_byte(3);

            let src_tree = create_tree_data(&[
                (b"lib.rs", EntryKind::Blob, blob_id),
            ]);
            let root_tree = create_tree_data(&[
                (b"src", EntryKind::Tree, src_id),
            ]);

            let mut objects = MockObjects {
                trees: HashMap::new(),
            };
            objects.trees.insert(root_id, root_tree);
            objects.trees.insert(src_id, src_tree);

            let result = super::super::tree_contains_path(&objects, root_id, b"src/nonexistent.rs");
            assert!(result.is_ok());
            assert!(!result.unwrap());
        }

        #[test]
        fn file_treated_as_directory_returns_true() {
            let root_id = id_from_byte(1);
            let blob_id = id_from_byte(2);

            let root_tree = create_tree_data(&[
                (b"file.txt", EntryKind::Blob, blob_id),
            ]);

            let mut objects = MockObjects {
                trees: HashMap::new(),
            };
            objects.trees.insert(root_id, root_tree);

            let result = super::super::tree_contains_path(&objects, root_id, b"file.txt/nested");
            assert!(result.is_ok());
            assert!(result.unwrap());
        }

        #[test]
        fn empty_path_returns_true() {
            let root_id = id_from_byte(1);
            let blob_id = id_from_byte(2);

            let root_tree = create_tree_data(&[
                (b"file.txt", EntryKind::Blob, blob_id),
            ]);

            let mut objects = MockObjects {
                trees: HashMap::new(),
            };
            objects.trees.insert(root_id, root_tree);

            let result = super::super::tree_contains_path(&objects, root_id, b"");
            assert!(result.is_ok());
            assert!(result.unwrap());
        }

        #[test]
        fn path_only_slashes_returns_true() {
            let root_id = id_from_byte(1);
            let blob_id = id_from_byte(2);

            let root_tree = create_tree_data(&[
                (b"file.txt", EntryKind::Blob, blob_id),
            ]);

            let mut objects = MockObjects {
                trees: HashMap::new(),
            };
            objects.trees.insert(root_id, root_tree);

            let result = super::super::tree_contains_path(&objects, root_id, b"///");
            assert!(result.is_ok());
            assert!(result.unwrap());
        }

        #[test]
        fn intermediate_directory_not_found() {
            let root_id = id_from_byte(1);
            let src_id = id_from_byte(2);
            let blob_id = id_from_byte(3);

            let src_tree = create_tree_data(&[
                (b"main.rs", EntryKind::Blob, blob_id),
            ]);
            let root_tree = create_tree_data(&[
                (b"src", EntryKind::Tree, src_id),
            ]);

            let mut objects = MockObjects {
                trees: HashMap::new(),
            };
            objects.trees.insert(root_id, root_tree);
            objects.trees.insert(src_id, src_tree);

            let result = super::super::tree_contains_path(&objects, root_id, b"lib/main.rs");
            assert!(result.is_ok());
            assert!(!result.unwrap());
        }

        #[test]
        fn multiple_entries_in_tree() {
            let root_id = id_from_byte(1);
            let blob1_id = id_from_byte(2);
            let blob2_id = id_from_byte(3);
            let blob3_id = id_from_byte(4);

            let root_tree = create_tree_data(&[
                (b"aaa.txt", EntryKind::Blob, blob1_id),
                (b"bbb.txt", EntryKind::Blob, blob2_id),
                (b"ccc.txt", EntryKind::Blob, blob3_id),
            ]);

            let mut objects = MockObjects {
                trees: HashMap::new(),
            };
            objects.trees.insert(root_id, root_tree);

            let result = super::super::tree_contains_path(&objects, root_id, b"bbb.txt");
            assert!(result.is_ok());
            assert!(result.unwrap());

            let result = super::super::tree_contains_path(&objects, root_id, b"ddd.txt");
            assert!(result.is_ok());
            assert!(!result.unwrap());
        }
    }

    mod path_changed_between_trees_tests {
        use gix_hash::ObjectId;
        use gix_object::tree::EntryKind;
        use gix_object::{tree::Entry, WriteTo};
        use std::collections::HashMap;

        struct MockObjects {
            trees: HashMap<ObjectId, Vec<u8>>,
        }

        #[coverage(off)]
        impl gix_object::Find for MockObjects {
            fn try_find<'a>(
                &self,
                id: &gix_hash::oid,
                buffer: &'a mut Vec<u8>,
            ) -> Result<Option<gix_object::Data<'a>>, gix_object::find::Error> {
                if let Some(data) = self.trees.get(id) {
                    buffer.clear();
                    buffer.extend_from_slice(data);
                    Ok(Some(gix_object::Data {
                        kind: gix_object::Kind::Tree,
                        data: buffer.as_slice(),
                    }))
                } else {
                    Ok(None)
                }
            }
        }

        struct MalformedTreeFind;

        #[coverage(off)]
        impl gix_object::Find for MalformedTreeFind {
            fn try_find<'a>(
                &self,
                _id: &gix_hash::oid,
                buffer: &'a mut Vec<u8>,
            ) -> Result<Option<gix_object::Data<'a>>, gix_object::find::Error> {
                buffer.clear();
                buffer.extend_from_slice(b"invalid tree data without null terminator");
                Ok(Some(gix_object::Data {
                    kind: gix_object::Kind::Tree,
                    data: buffer.as_slice(),
                }))
            }
        }

        struct NotFoundFind;

        #[coverage(off)]
        impl gix_object::Find for NotFoundFind {
            fn try_find<'a>(
                &self,
                _id: &gix_hash::oid,
                _buffer: &'a mut Vec<u8>,
            ) -> Result<Option<gix_object::Data<'a>>, gix_object::find::Error> {
                Ok(None)
            }
        }

        fn create_tree_data(entries: &[(&[u8], EntryKind, ObjectId)]) -> Vec<u8> {
            let mut tree = gix_object::Tree::empty();
            for (name, kind, oid) in entries {
                tree.entries.push(Entry {
                    mode: (*kind).into(),
                    filename: (*name).into(),
                    oid: *oid,
                });
            }
            let mut buf = Vec::new();
            tree.write_to(&mut buf).expect("failed to write tree");
            buf
        }

        fn id_from_byte(b: u8) -> ObjectId {
            let mut bytes = [0u8; 20];
            bytes[0] = b;
            ObjectId::from(bytes)
        }

        #[test]
        #[coverage(off)]
        fn path_changed_with_malformed_tree_data_returns_error() {
            let mut state = gix_diff::tree::State::default();
            let result = super::super::path_changed_between_trees(
                &MalformedTreeFind,
                id_from_byte(1),
                id_from_byte(2),
                b"some/path",
                &mut state,
            );
            assert!(result.is_err());
            let err = result.unwrap_err();
            match err {
                crate::error::SdkError::Git(_) => {}
                _ => panic!("Expected SdkError::Git, got {:?}", err),
            }
        }

        #[test]
        #[coverage(off)]
        fn path_changed_with_not_found_tree_returns_error() {
            let mut state = gix_diff::tree::State::default();
            let result = super::super::path_changed_between_trees(
                &NotFoundFind,
                id_from_byte(1),
                id_from_byte(2),
                b"some/path",
                &mut state,
            );
            assert!(result.is_err());
            let err = result.unwrap_err();
            match err {
                crate::error::SdkError::Git(_) => {}
                _ => panic!("Expected SdkError::Git, got {:?}", err),
            }
        }

        #[test]
        fn path_changed_with_same_trees_returns_false() {
            let tree_id = id_from_byte(1);
            let blob_id = id_from_byte(2);

            let tree_data = create_tree_data(&[(b"file.txt", EntryKind::Blob, blob_id)]);

            let mut objects = MockObjects {
                trees: HashMap::new(),
            };
            objects.trees.insert(tree_id, tree_data);

            let mut state = gix_diff::tree::State::default();
            let result = super::super::path_changed_between_trees(
                &objects,
                tree_id,
                tree_id,
                b"file.txt",
                &mut state,
            );
            assert!(result.is_ok());
            assert!(!result.unwrap());
        }

        #[test]
        fn path_changed_with_different_trees_detects_change() {
            let lhs_tree_id = id_from_byte(1);
            let rhs_tree_id = id_from_byte(2);
            let blob1_id = id_from_byte(3);
            let blob2_id = id_from_byte(4);

            let lhs_tree_data =
                create_tree_data(&[(b"file.txt", EntryKind::Blob, blob1_id)]);
            let rhs_tree_data =
                create_tree_data(&[(b"file.txt", EntryKind::Blob, blob2_id)]);

            let mut objects = MockObjects {
                trees: HashMap::new(),
            };
            objects.trees.insert(lhs_tree_id, lhs_tree_data);
            objects.trees.insert(rhs_tree_id, rhs_tree_data);

            let mut state = gix_diff::tree::State::default();
            let result = super::super::path_changed_between_trees(
                &objects,
                lhs_tree_id,
                rhs_tree_id,
                b"file.txt",
                &mut state,
            );
            assert!(result.is_ok());
            assert!(result.unwrap());
        }

        #[test]
        fn path_changed_with_different_trees_no_change_at_path() {
            let lhs_tree_id = id_from_byte(1);
            let rhs_tree_id = id_from_byte(2);
            let blob1_id = id_from_byte(3);
            let blob2_id = id_from_byte(4);

            let lhs_tree_data = create_tree_data(&[
                (b"changed.txt", EntryKind::Blob, blob1_id),
                (b"unchanged.txt", EntryKind::Blob, blob1_id),
            ]);
            let rhs_tree_data = create_tree_data(&[
                (b"changed.txt", EntryKind::Blob, blob2_id),
                (b"unchanged.txt", EntryKind::Blob, blob1_id),
            ]);

            let mut objects = MockObjects {
                trees: HashMap::new(),
            };
            objects.trees.insert(lhs_tree_id, lhs_tree_data);
            objects.trees.insert(rhs_tree_id, rhs_tree_data);

            let mut state = gix_diff::tree::State::default();
            let result = super::super::path_changed_between_trees(
                &objects,
                lhs_tree_id,
                rhs_tree_id,
                b"unchanged.txt",
                &mut state,
            );
            assert!(result.is_ok());
            assert!(!result.unwrap());
        }

        #[test]
        fn path_changed_detects_file_addition() {
            let lhs_tree_id = id_from_byte(1);
            let rhs_tree_id = id_from_byte(2);
            let blob_id = id_from_byte(3);

            let lhs_tree_data = create_tree_data(&[]);
            let rhs_tree_data = create_tree_data(&[(b"new_file.txt", EntryKind::Blob, blob_id)]);

            let mut objects = MockObjects {
                trees: HashMap::new(),
            };
            objects.trees.insert(lhs_tree_id, lhs_tree_data);
            objects.trees.insert(rhs_tree_id, rhs_tree_data);

            let mut state = gix_diff::tree::State::default();
            let result = super::super::path_changed_between_trees(
                &objects,
                lhs_tree_id,
                rhs_tree_id,
                b"new_file.txt",
                &mut state,
            );
            assert!(result.is_ok());
            assert!(result.unwrap());
        }

        #[test]
        fn path_changed_detects_file_deletion() {
            let lhs_tree_id = id_from_byte(1);
            let rhs_tree_id = id_from_byte(2);
            let blob_id = id_from_byte(3);

            let lhs_tree_data = create_tree_data(&[(b"deleted_file.txt", EntryKind::Blob, blob_id)]);
            let rhs_tree_data = create_tree_data(&[]);

            let mut objects = MockObjects {
                trees: HashMap::new(),
            };
            objects.trees.insert(lhs_tree_id, lhs_tree_data);
            objects.trees.insert(rhs_tree_id, rhs_tree_data);

            let mut state = gix_diff::tree::State::default();
            let result = super::super::path_changed_between_trees(
                &objects,
                lhs_tree_id,
                rhs_tree_id,
                b"deleted_file.txt",
                &mut state,
            );
            assert!(result.is_ok());
            assert!(result.unwrap());
        }

        #[test]
        fn path_changed_in_nested_directory() {
            let lhs_root_id = id_from_byte(1);
            let rhs_root_id = id_from_byte(2);
            let lhs_src_id = id_from_byte(3);
            let rhs_src_id = id_from_byte(4);
            let blob1_id = id_from_byte(5);
            let blob2_id = id_from_byte(6);

            let lhs_src_data = create_tree_data(&[(b"main.rs", EntryKind::Blob, blob1_id)]);
            let rhs_src_data = create_tree_data(&[(b"main.rs", EntryKind::Blob, blob2_id)]);

            let lhs_root_data = create_tree_data(&[(b"src", EntryKind::Tree, lhs_src_id)]);
            let rhs_root_data = create_tree_data(&[(b"src", EntryKind::Tree, rhs_src_id)]);

            let mut objects = MockObjects {
                trees: HashMap::new(),
            };
            objects.trees.insert(lhs_root_id, lhs_root_data);
            objects.trees.insert(rhs_root_id, rhs_root_data);
            objects.trees.insert(lhs_src_id, lhs_src_data);
            objects.trees.insert(rhs_src_id, rhs_src_data);

            let mut state = gix_diff::tree::State::default();
            let result = super::super::path_changed_between_trees(
                &objects,
                lhs_root_id,
                rhs_root_id,
                b"src/main.rs",
                &mut state,
            );
            assert!(result.is_ok());
            assert!(result.unwrap());
        }

        #[test]
        #[coverage(off)]
        fn path_changed_with_missing_subtree_returns_error() {
            let lhs_root_id = id_from_byte(1);
            let rhs_root_id = id_from_byte(2);
            let lhs_src_id = id_from_byte(3);
            let rhs_src_id = id_from_byte(4);
            let blob_id = id_from_byte(5);

            let lhs_src_data = create_tree_data(&[(b"main.rs", EntryKind::Blob, blob_id)]);

            let lhs_root_data = create_tree_data(&[(b"src", EntryKind::Tree, lhs_src_id)]);
            let rhs_root_data = create_tree_data(&[(b"src", EntryKind::Tree, rhs_src_id)]);

            let mut objects = MockObjects {
                trees: HashMap::new(),
            };
            objects.trees.insert(lhs_root_id, lhs_root_data);
            objects.trees.insert(rhs_root_id, rhs_root_data);
            objects.trees.insert(lhs_src_id, lhs_src_data);

            let mut state = gix_diff::tree::State::default();
            let result = super::super::path_changed_between_trees(
                &objects,
                lhs_root_id,
                rhs_root_id,
                b"other/file.txt",
                &mut state,
            );
            assert!(result.is_err());
            let err = result.unwrap_err();
            match err {
                crate::error::SdkError::Git(_) => {}
                _ => panic!("Expected SdkError::Git, got {:?}", err),
            }
        }
    }
}
