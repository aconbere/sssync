# Remote
- Attach a file validity header to the put request [object-integrity](https://docs.aws.amazon.com/AmazonS3/latest/userguide/checking-object-integrity.html)

# Files
- Currently there exists an StagedFile, TreeFile, and IntermediateTree that all in some ways represent a "file". StagedFile and TreeFile differ only in the kinds of metadata they can track. In staging we have a last_modified time available since it's derived from the users local disk, while a TreeFile does not. Conversly a TreeFile which is derived from the object store has the parent commit hash, but the StagedFile does not. This is all just ugly and should get unified in some way.

# Migrations
- Restart
- Clear after complete

# Remote
- Change permissions?
- Publish a file to a shared directory?

# Bugs
- Remote push isn't narrowing the set of files correctly. Always tries to upload all files in the most recent commit. This isn't terrible because we check head first. But it shouldn't be happening
