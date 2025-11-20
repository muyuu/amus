use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Area {
    pub name: String,
    pub id: String,
}

impl Area {
    pub fn new(name: String, id: String) -> Self {
        Self { name, id }
    }
}
