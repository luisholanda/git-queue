//! # Queue State Log
//!
//! Each queue have a log attached to it (in the ref `refs/queuelogs/<queue>`). The
//! purpose of this log is to track each operation executed in the queue.
//!
//! Each entry in the log ensures it have the proper references to objects it needs
//! to make it safe against GC.
//!
//! As the tool may evolve, the log format can change, see the documentation for each
//! version struct to know the specifics of each one.
//!
//! ## Log Entry Version 1
//!
//! ### Commit message
//!
//! Each entry contains a message describing what command was executed in the
//! queue. This is most for human consumption, but later will be used to provide
//! undo and redo operations.
//!
//! ### Tree
//!
//! The tree in the entry contains one extra blob `meta`, which contains the metadata
//! of the entry, and is used to recover the stack state at the specific time when this
//! entry was created. It is a JSON encoded file containing the following fields:
//!
//!   * `version: 1`
//!   * `previous: <sha1 or missing>`: the OID of the previous log entry or
//!      nothing if it is the first one.
//!   * `head: <sha1>`: the queue head at the time this entry was created.
//!   * `base_name`: the name of the base branch of this queue.
//!   * `base: <sha1>`: The OID of the last commit before the applied patches.
//!   * `applied`: a list of the applied patches.
//!   * `unapplied`: same of `applied`, but for unapplied patches.
//!   * `patches`: a map of each patch name to its commit OID when the entry was created.
//!
//! ### Parents
//!
//! Each entry commit have the following parents, in order:
//!
//! * The previous entry commit.
//! * The branch head commit when the entry was created.
//! * All applied or unapplied patches commits when the entry was created.

use crate::error::Error;
use git2::{Oid, Repository, Tree};
use std::collections::HashMap;

/// The queue state at a specific point in time.
pub struct QueueState {
    oid: Option<Oid>,
    gitref_name: String,
    entry: LogEntryV1,
}

impl QueueState {
    /// Get the current state for the given queue.
    ///
    /// # Errors
    ///
    /// If the queue does not exist, this function will return a [`git2::Error`]
    /// instance with code [`git2::ErrorCode::NotFound`].
    pub fn current_for_queue(repo: &Repository, queue: &str) -> Result<Self, Error> {
        let gitref_name = Self::gitref_name(queue);
        let gitref = repo.find_reference(&gitref_name)?;
        let mut commit = None;
        let mut maybe_inconsistent = || {
            let c = gitref.peel_to_commit()?;
            let tree = c.tree()?;
            commit = Some(c);

            let meta_obj = tree.get_path("meta".as_ref())?.to_object(repo)?;
            let meta_blob = meta_obj
                .as_blob()
                .ok_or_else(|| invalid_meta("expected meta object was a blob, but it wasn't"))?;

            let entry: LogEntryV1 = serde_json::from_slice(meta_blob.content())
                .map_err(|_| invalid_meta("expected meta content to be a JSON"))?;

            Ok(entry)
        };

        let entry = maybe_inconsistent()
            .map_err(|_: git2::Error| Error::Inconsistency("queuelog reference"))?;

        Ok(Self {
            oid: commit.map(|c| c.id()),
            gitref_name,
            entry,
        })
    }

    /// Create a new stack state in the given branch.
    pub fn new(repo: &Repository, queue: &str, base: &git2::Branch<'_>) -> Result<Self, Error> {
        let gitref_name = Self::gitref_name(queue);
        if repo.find_reference(&gitref_name).is_ok() {
            return Err(Error::AlreadyExists("queuelog"));
        }

        let base_commit = base.get().peel_to_commit()?;
        let base_oid = base_commit.id();
        let base_name = base.name()?.ok_or(Error::NonUtf8)?.to_string();

        let message = "initialise stack log".to_string();
        let entry = LogEntryV1 {
            message,
            previous: None,
            head: LogOid(base_oid),
            base_name,
            base: LogOid(base_oid),
            applied: vec![],
            unapplied: vec![],
            patches: HashMap::new(),
        };

        let tree = entry.build_tree(repo, &base_commit.tree()?)?;

        let user = repo.signature()?;
        let commit = repo.commit(
            Some(&gitref_name),
            &user,
            &user,
            &entry.message,
            &tree,
            // There is no previous entry, nor patches to add.
            &[&base_commit],
        )?;

        Ok(Self {
            oid: Some(commit),
            gitref_name,
            entry,
        })
    }

    pub fn base_name(&self) -> &str {
        &self.entry.base_name
    }

    /// The HEAD commit of thi state.
    pub fn head(&self) -> Oid {
        self.entry.head.0
    }

    pub fn name(&self) -> &str {
        self.gitref().split_at("refs/queuelogs/".len()).1
    }

    pub fn gitref(&self) -> &str {
        &self.gitref_name
    }

    pub fn patches_num(&self) -> usize {
        self.entry.patches.len()
    }

    /// The list of applied patches and their specific commits.
    pub fn applied(&self) -> impl Iterator<Item = (&str, Oid)> + '_ {
        self.entry.applied.iter().map(move |pn| {
            let oid = self.entry.patches[pn].0;
            (pn.as_str(), oid)
        })
    }

    /// The list of unapplied patches and their specific commits.
    pub fn unapplied(&self) -> impl Iterator<Item = (&str, Oid)> + '_ {
        self.entry.unapplied.iter().map(move |pn| {
            let oid = self.entry.patches[pn].0;
            (pn.as_str(), oid)
        })
    }

    /// Does this state have the given patch?
    pub fn has_patch(&self, name: &str) -> bool {
        self.entry.patches.contains_key(name)
    }

    /// Pop a patch from the stack.
    ///
    /// The received function is used to resolve the _parent_ of a given commit
    /// when we're popping the last applied commit.
    pub fn pop(&mut self, get_parent: impl FnOnce(Oid) -> Result<Oid, Error>) -> Result<(), Error> {
        if let Some(patch) = self.entry.applied.pop() {
            let patch_oid = self.entry.patches[&patch].0;
            self.entry.unapplied.push(patch);

            if let Some(patch) = self.entry.applied.last() {
                self.entry.head = self.entry.patches[patch];
            } else {
                self.entry.head = LogOid(get_parent(patch_oid)?);
            }
        }

        Ok(())
    }

    /// Pushes a patch to the stack.
    ///
    /// Does nothing if there is no patch to push.
    pub fn push(&mut self) {
        if let Some(patch) = self.entry.unapplied.pop() {
            self.entry.head = self.entry.patches[&patch];
            self.entry.applied.push(patch);
        }
    }

    /// Renames a patch.
    ///
    /// # Panics
    ///
    /// Will panic if the there is no patch with the old name in this state.
    /// Use [`Self::has_patch`] before calling this function.
    pub fn rename_patch(&mut self, old_name: &str, new_name: String) {
        assert!(
            self.has_patch(old_name),
            "patch {} not found in state",
            new_name
        );

        let patch_oid = self.entry.patches.remove(old_name).unwrap();
        self.entry.patches.insert(new_name.clone(), patch_oid);

        if let Some(idx) = self.entry.applied.iter().position(|pn| pn == old_name) {
            self.entry.applied[idx] = new_name;
        } else if let Some(idx) = self.entry.unapplied.iter().position(|pn| pn == old_name) {
            self.entry.unapplied[idx] = new_name;
        }
    }

    /// Update or create a patch OID in the state.
    pub fn upsert_patch(&mut self, patch: String, commit: Oid) {
        if !self.has_patch(&patch) {
            self.entry.applied.push(patch.clone());
        }

        self.entry.patches.insert(patch, LogOid(commit));
    }

    /// Creates a new state in the log entry after modifying with the given
    /// function.
    pub fn create_next<T, F>(
        &self,
        repo: &Repository,
        message: String,
        func: F,
    ) -> Result<(Self, T), Error>
    where
        F: FnOnce(&mut Self) -> Result<T, Error>,
    {
        let mut next = self.next(message);

        let res = func(&mut next)?;

        next.commit(repo)?;

        Ok((next, res))
    }

    fn next(&self, message: String) -> Self {
        assert!(
            self.oid.is_some(),
            "tried to get next state from an uncommited one"
        );
        Self {
            oid: None,
            gitref_name: self.gitref_name.clone(),
            entry: LogEntryV1 {
                message,
                head: LogOid(self.head()),
                base: self.entry.base,
                base_name: self.entry.base_name.clone(),
                previous: self.oid.map(LogOid),
                applied: self.entry.applied.clone(),
                unapplied: self.entry.unapplied.clone(),
                patches: self.entry.patches.clone(),
            },
        }
    }

    fn commit(&mut self, repo: &Repository) -> Result<(), Error> {
        assert!(self.oid.is_none(), "tried to commit already commited entry");
        let prev_oid = self.entry.previous.expect("tried to commit root state").0;
        let prev = repo.find_commit(prev_oid)?;
        let user = repo.signature()?;

        let tree = self.entry.build_tree(repo, &prev.tree()?)?;

        let mut parents = vec![prev, repo.head()?.peel_to_commit()?];

        for &LogOid(patch) in self.entry.patches.values() {
            parents.push(repo.find_commit(patch)?);
        }

        let parent_refs: Vec<_> = parents.iter().collect();

        let oid = repo.commit(
            Some(&self.gitref_name),
            &user,
            &user,
            &self.entry.message,
            &tree,
            &parent_refs,
        )?;
        self.oid = Some(oid);

        Ok(())
    }

    fn gitref_name(queue: &str) -> String {
        format!("refs/queuelogs/{}", queue)
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct LogEntryV1 {
    message: String,
    previous: Option<LogOid>,
    head: LogOid,
    base: LogOid,
    base_name: String,
    applied: Vec<String>,
    unapplied: Vec<String>,
    patches: HashMap<String, LogOid>,
}

impl LogEntryV1 {
    fn build_tree<'r>(
        &self,
        repo: &'r Repository,
        prev: &Tree<'r>,
    ) -> Result<Tree<'r>, git2::Error> {
        let mut builder = repo.treebuilder(Some(prev))?;

        let meta_oid = {
            let mut writer = repo.blob_writer(Some("meta".as_ref()))?;
            serde_json::to_writer_pretty(&mut writer, self).map_err(|e| {
                git2::Error::new(
                    git2::ErrorCode::GenericError,
                    git2::ErrorClass::Os,
                    &e.to_string(),
                )
            })?;

            writer.commit()?
        };

        builder.insert("meta", meta_oid, 0o100644)?;

        let tree_oid = builder.write()?;
        let tree = repo.find_tree(tree_oid)?;
        Ok(tree)
    }
}

#[derive(Clone, Copy)]
struct LogOid(Oid);

impl serde::Serialize for LogOid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(&format_args!("{}", self.0))
    }
}

impl<'de> serde::Deserialize<'de> for LogOid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let str: &str = <_>::deserialize(deserializer)?;

        let oid =
            Oid::from_str(str).map_err(|e| <D::Error as serde::de::Error>::custom(e.message()))?;

        Ok(Self(oid))
    }
}

fn invalid_meta(message: &str) -> git2::Error {
    git2::Error::new(git2::ErrorCode::Modified, git2::ErrorClass::Object, message)
}
