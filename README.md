# sssync

Sssync is a version control system oriented around addressing two limitations of Git; The ability to work with large files, and the ability to easily store those files into online blob storage.

It works a bit like git, where every file is hashed and stored and tracked in commits, that themselves are hashed and tracked and stored. However, it uses a xxhash instead of SHA1 in order to improve the performance of hashing large files, and it has the ability to use an S3 bucket as a remote storage backend.

Sssync doesn't solve any of the binary file duplication challanges around using git, so it's best suited for a collection of files that while large, aren't expected to change that frequently.

Sssyncc also doesn't intend to deal with diffing and merging file states. When merging two branches with commits that both alter the same file, the authors job is to simply pick the final revision desired. If both changes need to be kept, the author must make a copy.

## How to get it

Sssync is available on [crates.io](https://crates.io/crates/sssync). You can install it with:

```bash
> cargo install sssync
```

## How to use it

### Initialize a new sssync repository

Inside of whatever directory you want to start managing with sssync, run:

```bash
# sssync init <directory-path>
> sssync init my-repository
```

This will generate a new directory `my-repository` and set up the sssnyc state. Once that's done you can run `sssync add` to stage files for addition, and `sssync commit` to add the staged changes to the repository.

### Setting up a remote

Sssync has the ability to use S3 as a remote backend. To set up an S3 remote run the following:

```bash
# sssync remote add <remote-name> <remote-url>
> sssync remote add origin s3://example.com/path/to/bucket
```

Then initialize the remote:

```bash
# sssync remote init <remote-name>
> sssync remote init origin
```

You can push changes to the remote:

```bash
# sssync remote push <remote-name>
> sssync remote push origin
```

Once pushed a ssync repository somewhere else can clone the repository (note that clone will automatically name the remote "origin").

```bash
# sssync remote clone <remote-url>
> sssync remote clone s3://example.com/path/to/bucket
```

And fetch new changes

```bash
# sssync remote fetch <remote-name>
> sssync remote fetch origin
```

You can merge the remote changes into your current branch. A note, in sssync merges always act like git rebases. Your sequence of commits after the shared parent from the remote are all placed on top of the sequences of commits from the remote. Files that conflict are noted and need to be resolved by the author by picking which they want to keep.

```bash
# sssync merge <remote-name>:<branch-name>
> sssync merge origin:main
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
> sssync remote add example --location s3://test.bucket
> sssync remote init example
> sssync remote push example
> sssync remote clone example
> sssync remote fetch example
> sssync merge example:main
```
