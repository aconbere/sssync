# Remote
## Attach a file validity header to the put request
- https://docs.aws.amazon.com/AmazonS3/latest/userguide/checking-object-integrity.html

## Fetch remote objects
I have the remote db being fetched down and I can use that to diff a subsequent push, but there isn't the fetch down of the remote data to sync up.

## Join databases
Right now when we complete a migration up to a remote, we upload a copy of our local database. This means two clients can only operate on the same history. Actually what should happen is that we should probablly join these databases into a new one and then when pushing add into that remotebd? (Probably something to look at what happens with git.)

Thoughts:
- I don't really have pushing "branches" worked out. Pushing a branch is a matter of diffing between the local state and remote state and pushing the additional objects. Along with all of the new commits.
- So when I'm pushing master the same applies. We should add the commits and their tree to the remote db before pushing it back up.

# Files

Currently there exists an StagedFile, TreeFile, and IntermediateTree that all in some ways represent a "file". StagedFile and TreeFile differ only in the kinds of metadata they can track. In staging we have a last_modified time available since it's derived from the users local disk, while a TreeFile does not. Conversly a TreeFile which is derived from the object store has the parent commit hash, but the StagedFile does not. This is all just ugly and should get unified in some way.


# Migrations
## Restart
## Clear after complete
