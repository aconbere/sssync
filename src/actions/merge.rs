use std::path::Path;

use anyhow::{anyhow, Result};
use rusqlite::Connection;

use crate::db;
use crate::models::commit::{diff_commit_list, Commit, CompareResult};
use crate::models::status::{hash_all, Hashable, Status};
use crate::store;
use crate::tree;

fn get_source_commits(
    connection: &Connection,
    root_path: &Path,
    branch_name: &str,
    maybe_remote_name: &Option<String>,
) -> Result<Vec<Commit>> {
    if let Some(remote_name) = maybe_remote_name {
        let remote_db_path = store::remote_db_path(root_path, &remote_name)?;
        let remote_connection = Connection::open(&remote_db_path)?;

        let branch = db::reference::get(&remote_connection, &branch_name)?;

        db::commit::get_children(&remote_connection, &branch.hash)
    } else {
        let branch = db::reference::get(&connection, &branch_name)?;
        db::commit::get_children(&connection, &branch.hash)
    }
}

pub fn merge(
    connection: &Connection,
    root_path: &Path,
    branch_name: &str,
    maybe_remote_name: &Option<String>,
) -> Result<()> {
    // Check if there are any uncommitted changes
    let status = Status::new(connection, root_path)?;
    if status.has_uncomitted_changes() {
        println!(
            "Aborted: You have uncomitted changes in your working directory."
        );
        println!("{}", status);
    }

    let meta = db::meta::get(connection)?;
    let head = db::commit::get_by_ref_name(connection, &meta.head)?
        .ok_or(anyhow!("No commit"))?;

    let source_commits = get_source_commits(
        connection,
        root_path,
        branch_name,
        maybe_remote_name,
    )?;

    let destination_commits = db::commit::get_children(connection, &head.hash)?;

    /*
     * left is destination commits since the shared parent
     * right is source commits since the shared parent
     *
     * Goal is to place all of the destination commits on top of the source
     * commits. To do this we will start with the source state. Then for
     * each commit in destination we will apply the additions and
     * changes, creating a new commit with the new tree.
     *
     * Possible conflicts:
     * 1. Remote deletes a file, Local changes a file -> Keep the changed
     *    file
     * 2: Remote adds a file, local adds a file with the same name -> Keep
     * the new file
     */
    let CompareResult::Diff {
        left: commits_diff,
        shared_parent,
        ..
    } = diff_commit_list(&destination_commits, &source_commits)
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
    let mut new_commits = Vec::new();
    let mut parent_hash = shared_parent.hash;

    for commit in &commits_diff {
        let diff = tree::diff_parent(connection, &commit)?;
        combined_diff = combined_diff.add(&diff)?;
        let updates = combined_diff.updates();

        let hashable_files = updates
            .into_iter()
            .map(|f| {
                let i: Box<dyn Hashable> = Box::new(f);
                i
            })
            .collect();

        let hash = hash_all(&hashable_files);
        new_commits.push(Commit::new(
            &hash,
            &commit.message,
            &commit.author,
            Some(parent_hash.clone()),
        ));
        parent_hash = hash.to_string();
    }

    store::apply_diff(root_path, &combined_diff)?;

    Ok(())
}
