use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum SystemMessages {
    FileFound,
    FileRemoved,
    NewFileFound,
    TailingStarted,
    Start,
    Stop,
    Pause,
    Resume
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

    pub fn message(&self) -> &SystemMessages {
        &self.message
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    Data(DataMessage),
    System(SystemMessage)
}

impl Message {
    pub fn data(&self) -> Option<&DataMessage> {
        match self {
            Message::Data(data) => Some(data),
            _ => None
        }
    }

    pub fn system(&self) -> Option<&SystemMessage> {
        match self {
            Message::System(system) => Some(system),
            _ => None
        }
    }
}