use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum SystemMessages {
    FileFound,
    FileRemoved,
    NewFileFound,
    TailingStarted
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DataMessage {
    row: String,
    application: String,
    replace_last_row: bool,
    timestamp: NaiveDateTime
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemMessage {
    application: String,
    message: SystemMessages,
    timestamp: NaiveDateTime
}

impl DataMessage {
    pub fn new(row: String, application: String, replace_last_row: bool) -> Self {
        Self { row, application, replace_last_row, timestamp: chrono::Utc::now().naive_utc() }
    }

    pub fn row(&self) -> &str {
        &self.row
    }
}

impl SystemMessage {
    pub fn new(application: String, message: SystemMessages) -> Self {
        Self { application, message, timestamp: chrono::Utc::now().naive_utc() }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    Data(DataMessage),
    System(SystemMessage)
}