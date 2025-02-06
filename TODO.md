# Remote
- Attach a file validity header to the put request [object-integrity](https://docs.aws.amazon.com/AmazonS3/latest/userguide/checking-object-integrity.html)
- Merge a remote back into local main
    - Maybe just always rebase?

# Checkout
- Get a file from one ref / origin:ref into the current branch

# Files
- Currently there exists an StagedFile, TreeFile, and IntermediateTree that all in some ways represent a "file". StagedFile and TreeFile differ only in the kinds of metadata they can track. In staging we have a last_modified time available since it's derived from the users local disk, while a TreeFile does not. Conversly a TreeFile which is derived from the object store has the parent commit hash, but the StagedFile does not. This is all just ugly and should get unified in some way.

# Migrations
- Restart
- Clear after complete

# Bugs
- using i64 for size_bytes should be u64 (what's a negative byte?!)
- need to add, message, author, and parent_hash into the hash for a commit

