use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    row: String,
    application: String,
}

impl Message {
    pub fn new(row: String, application: String) -> Self {
        Self { row, application }
    }

    pub fn row(&self) -> &str {
        &self.row
    }
}