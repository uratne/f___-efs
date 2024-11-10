use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    row: String,
    application: String,
    replace_last_row: bool
}

impl Message {
    pub fn new(row: String, application: String, replace_last_row: bool) -> Self {
        Self { row, application, replace_last_row }
    }

    pub fn row(&self) -> &str {
        &self.row
    }
}