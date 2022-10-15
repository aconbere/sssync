# sssync

Sssync is a version control system oriented around addressing two limitations of Git; The ability to work with large files, and the ability to easily store those files into online blob storage.

It works a bit like git, where every file is hashed and stored and tracked in commits, that themselves are hashed and tracked and stored. However, it uses a xxhash instead of SHA1 in order to improve the performance of hashing large files, and it has the ability to use an S3 bucket as a remote storage backend.

Sssync doesn't solve any of the binary file duplication challanges around using git, so it's best suited for a collection of files that while large, aren't expected to change that frequently.

## How to use it

### Initialize a new sssync repository

Inside of whatever directory you want to start managing with sssync, run:

```bash
> sssync init .
```

This will generate a new directory `.sssync` and set up the repository. Once that's done you can run `sssync add` to stage files for addition, and `sssync commit` to add the staged changes to the repository.

### Setting up a remote

Sssync has the ability to use S3 as a remote backend. To set up an S3 remote run the following:

```bash
# sssync remote add <remote-name> <s3-url>
> sssync remote add new-remote s3://example.com/path/to/bucket
```

Then initialize the remote:

```bash
# sssync remote init <remote-name>
> sssync remote init new-remote
```

After this you can push changes to the remote:

```bash
# sssync remote push <remote-name>
> sssync remote push new-remote
```

# How it works

Sssync init creates a directory .sssnyc in the given directory. This directory contains the sssync.db file as well as two other directories: `objects` and `remotes`.

### Objects

An object in sssync is a file in the objects directory stored by the hash of its contents.

## sssnc.db

`sssync.db` is a Sqlite3 database that contains all the running information about the sssync project: commits, refs, remotes, the index, trees, and uploads.

### Commits

A commit in ssync is a hash constructed of the combined object hashes of all the objects in the repository at that time, a link the commit directlty before it, as well as some meta information about the commit (comment, author, created timestamp)

```sql
CREATE TABLE commits (
    hash TEXT PRIMARY KEY,
    comment TEXT NOT NULL,
    author TEXT NOT NULL,
    created_unix_timestamp INTEGER NOT NULL,
    parent_hash TEXT
);
```

### Trees

A tree in sssync is a representation of the filepath of the repository at the time that the commit was created. When you're switching the repository to a different HEAD the tree is what allows the system to place the objects in the commit into their correct space in the filesystem.

```sql
CREATE TABLE trees (
    path TEXT NOT NULL,
    file_hash TEXT NOT NULL,
    size_bytes INTEGER NOT NULL,
    commit_hash TEXT NOT NULL
);
```

### Staging

Staging is a storage area for all the changes in the current repository that haven't yet been commited (but we want them to be).

```sql
CREATE TABLE staging (
    file_hash TEXT PRIMARY KEY,
    path TEXT NOT NULL,
    size_bytes INTEGER NOT NULL,
    modified_time_seconds INTEGER NOT NULL
);
```

# How to use

```
> sssync init
> sssync add
> sssync commit
> sssync remote add example --kind s3 --location s3://test.bucket
> sssync remote init example
> sssync remote push example
```
