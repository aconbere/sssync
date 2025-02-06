use std::path::Path;

use anyhow::{anyhow, Result};
use rusqlite::Connection;

use crate::db;
use crate::models::commit::{diff_commit_list, Commit, CompareResult};
use crate::models::reference;
use crate::models::status::{hash_all, Hashable, Status};
use crate::store;
use crate::tree;

/* Take the differing commits in the local branch and recreate them
 * on top of the referenced destination.
 *
 * local: a -> e
 * remote: a -> b -> c -> d
 *
 * a rebase from remote onto local will end up
 *
 * a -> b -> c -> d -> e
 *
 */
pub fn rebase(
    connection: &Connection,
    root_path: &Path,
    branch_name: &str,
    maybe_remote_name: &Option<String>,
) -> Result<()> {
    let resolver =
        ConnectionResolver::new(connection, root_path, maybe_remote_name)?;

    // Check if there are any uncommitted changes
    let status = Status::new(resolver.destination(), root_path)?;
    if status.has_uncomitted_changes() {
        println!(
            "Aborted: You have uncomitted changes in your working directory."
        );
        println!("{}", status);
    }

    let meta = db::meta::get(resolver.destination())?;
    let head = db::commit::get_by_ref_name(resolver.destination(), &meta.head)?
        .ok_or(anyhow!("No commit"))?;

    let commits = db::commit::get_children(resolver.destination(), &head.hash)?;

    let source_head =
        db::commit::get_by_ref_name(resolver.source(), branch_name)?
            .ok_or(anyhow!("Invalid branch, no commit hash found"))?;
    let source_commits =
        db::commit::get_children(resolver.source(), &source_head.hash)?;

    // commits_diff is the destination commits that need to get rebased
    let CompareResult::Diff {
        left: commits_diff, ..
    } = diff_commit_list(&commits, &source_commits)
    else {
        return Err(anyhow!("no shared parent"));
    };

    // Note: Need to understand when both sides have differences, it /should/
    // be as simple as

    // For each commit in the set of deetination commits that diverge from
    // the shared parent, rebuild on top of the source commits.
    //
    // 1. get the commit
    // 2. diff the commit and its parent
    // 3. apply the diff to the new parent
    // 4. create a new commit copying the contents of the old commit with that
    //    tree
    let mut combined_diff = tree::TreeDiff::empty();
    let mut parent_hash = source_head.hash;

    let mut commits_to_rebase = commits_diff.iter().rev().peekable();

    while let Some(commit) = commits_to_rebase.next() {
        println!("rebasing {} onto {}", commit.hash, parent_hash);
        // get the changes in the current commit
        let diff = tree::diff_parent(connection, &commit)?;

        // sum them into a bigger diff that will be applied to
        // the filesytem at the end
        combined_diff = combined_diff.add(&diff)?;

        // find all the addition and changes
        let updates = combined_diff.updates();

        // make a new hash with the new file state from the changes
        let hashable_files = updates
            .into_iter()
            .map(|f| {
                let i: Box<dyn Hashable> = Box::new(f);
                i
            })
            .collect();

        let hash = hash_all(&hashable_files);

        let new_commit = Commit::new(
            &hash,
            &commit.message,
            &commit.author,
            Some(parent_hash.clone()),
        )?;

        // create a new commit with that hash
        db::commit::insert(&resolver.destination(), &new_commit)?;

        // update the parent hash, it should now be
        // the hash of the commit we just added
        parent_hash = new_commit.hash.clone();

        // we are at the last commit, update the ref
        if commits_to_rebase.peek().is_none() {
            println!("updating head to {}", new_commit.hash);
            db::reference::update(
                resolver.destination(),
                &meta.head,
                reference::Kind::Branch,
                &new_commit.hash,
            )?;
        }
    }

    store::apply_diff(root_path, &combined_diff)?;

    Ok(())
}

/* You will merge branches into main
 *
 * This will take the commits from the branch and put them on top of main. It
 * will fail if there are commits on main that are not found in the branch.
 *
 * Therefor in order to merge a branch into main when there has been changes
 * in main you will need to go to the branch and rebase it on main first.
 *
 *
 * local: a -> b -> c -> d
 * remote: a -> e
 *
 * a merge from remote to local will end with
 * local: a -> b -> c -> d -> e
 */
pub fn merge(
    connection: &Connection,
    root_path: &Path,
    branch_name: &str,
    maybe_remote_name: &Option<String>,
) -> Result<()> {
    let resolver =
        ConnectionResolver::new(connection, root_path, maybe_remote_name)?;

    // Destination stuff
    //
    // Check if there are any uncommitted changes
    let status = Status::new(resolver.destination(), root_path)?;
    if status.has_uncomitted_changes() {
        println!(
            "Aborted: You have uncomitted changes in your working directory."
        );
        println!("{}", status);
    }
    let meta = db::meta::get(resolver.destination())?;
    let head = db::commit::get_by_ref_name(resolver.destination(), &meta.head)?
        .ok_or(anyhow!("No commit"))?;
    let branch_head =
        db::commit::get_by_ref_name(resolver.destination(), branch_name)?
            .ok_or(anyhow!("Invalid branch, no commit hash found"))?;
    let commits = db::commit::get_children(resolver.destination(), &head.hash)?;

    // Source stuff
    let source_branch = db::reference::get(resolver.source(), &branch_name)?;
    let source_commits =
        db::commit::get_children(resolver.source(), &source_branch.hash)?;

    /*
     * When merging we expect the head of the destination branch to be
     * a child of the source branch. That is, all commits that need merging
     * are part of a linear history on top of the destination commit list.
     *
     * As a result of this invariant a "merge" should be as easy as
     * re-assinging the hash of the destination branches ref to that of
     * the source branch and then applying any necessary changes to the
     * filesystem.
     */
    let CompareResult::Diff {
        left: destination_commits_diff,
        right: source_commits_diff,
        ..
    } = diff_commit_list(&commits, &source_commits)
    else {
        return Err(anyhow!("no shared parent"));
    };

    // We force merges to be simple "on top of" relationships
    // if that's not the case exit early
    if !destination_commits_diff.is_empty() {
        return Err(anyhow!(
            "{} has more recent changes. Rebase {}  onto {}",
            meta.head,
            branch_name,
            meta.head
        ));
    }

    // if we're working on a remote clone the trees from the remote
    if resolver.is_remote() {
        for commit in &source_commits_diff {
            let tree = db::tree::get(resolver.source(), &commit.hash)?;
            for t in tree {
                // Skip insertion errors since we'll expect duplicates
                _ = db::tree::insert(resolver.destination(), &t);
            }
        }
    }

    // Update the current filesystem to match the latest tree
    let current_tree = db::tree::get(resolver.destination(), &head.hash)?;
    let future_tree = db::tree::get(resolver.source(), &branch_head.hash)?;
    let diff = tree::TreeDiff::new(&current_tree, &future_tree);
    store::apply_diff(root_path, &diff)?;

    db::reference::update(
        resolver.destination(),
        &meta.head,
        reference::Kind::Branch,
        &branch_head.hash,
    )?;

    Ok(())
}

// Sets up a structure to hold onto a local and remote database connection
//
// Often times we have situations where a reference may map either to a local
// or remote. It's useful in these cases to write a single set of functions
// but have this object pick wether to return the local or remote connection
// depending.
//
// In order to pass out the reference to the remote db (needed to map to the
// reference type we get for the local connection) we need a struct to attach
// the lifetime to.
struct ConnectionResolver<'a> {
    base_connection: &'a Connection,
    maybe_remote_connection: Option<Connection>,
}

impl<'a> ConnectionResolver<'a> {
    pub fn new(
        base_connection: &'a Connection,
        root_path: &Path,
        maybe_remote_name: &Option<String>,
    ) -> Result<Self> {
        if let Some(remote_name) = maybe_remote_name {
            let remote_db_path =
                store::remote_db_path(root_path, &remote_name)?;
            let remote_connection = Connection::open(&remote_db_path)?;
            Ok(ConnectionResolver {
                base_connection,
                maybe_remote_connection: Some(remote_connection),
            })
        } else {
            Ok(ConnectionResolver {
                base_connection,
                maybe_remote_connection: None,
            })
        }
    }

    pub fn source(&self) -> &Connection {
        if let Some(remote_connection) = &self.maybe_remote_connection {
            &remote_connection
        } else {
            self.base_connection
        }
    }

    pub fn destination(&self) -> &Connection {
        self.base_connection
    }

    pub fn is_remote(&self) -> bool {
        self.maybe_remote_connection.is_some()
    }
}
