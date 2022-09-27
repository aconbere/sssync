# sssync
s3 file sync tool

# .sssync

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
