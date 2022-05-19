use uuid::Uuid;

pub enum UploadState {
    Waiting,
    Uploading,
    Failed,
    Complete,
}

pub struct Upload {
    pub migration_id: Uuid,
    pub object_hash: String,
    pub state: UploadState,
}

impl Upload {
    pub fn new(migration_id: Uuid, object_hash: &str) -> Self {
        Self {
            migration_id: migration_id,
            object_hash: object_hash.to_string(),
            state: UploadState::Waiting,
        }
    }
}
