use uuid::Uuid;

pub enum MigrationState {
    Waiting,
    Running,
    Complete,
    Canceled,
    Failed,
}

pub struct Migration {
    pub id: Uuid,
    pub state: MigrationState,
}

impl Migration {
    pub fn new() -> Self {
        Self {
            state: MigrationState::Waiting,
            id: Uuid::new_v4(),
        }
    }
}
