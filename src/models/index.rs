use crate::models::staged_file::StagedFile;

pub enum Index {
    DeletedFile(StagedFile),
    AddedFile(StagedFile),
    ChangedFile(StagedFile),
}
