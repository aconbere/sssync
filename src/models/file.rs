use std::path::PathBuf;

pub trait File {
    fn path_str(&self) -> &str;
    fn path(&self) -> PathBuf;
    fn file_hash(&self) -> &str;
}
