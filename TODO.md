# Remote
## Attach a file validity header to the put request
- https://docs.aws.amazon.com/AmazonS3/latest/userguide/checking-object-integrity.html

## Fetch remote objects
I have the remote db being fetched down and I can use that to diff a subsequent
push, but there isn't the fetch down of the remote data to sync up.

# Files

Currently there exists an StagedFile, TreeFile, and IntermediateTree that all in some ways represent a "file". StagedFile and TreeFile differ only in the kinds of metadata they can track. In staging we have a last_modified time available since it's derived from the users local disk, while a TreeFile does not. Conversly a TreeFile which is derived from the object store has the parent commit hash, but the StagedFile does not. This is all just ugly and should get unified in some way.

# Migrations
## Restart
