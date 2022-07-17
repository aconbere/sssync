#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Meta {
    pub head: String,
}

impl Meta {
    pub fn new(head: &str) -> Self {
        Self {
            head: String::from(head),
        }
    }
}
