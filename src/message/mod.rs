use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::Applicatiton;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataMessage {
    #[serde(rename = "type")]
    message_type: String,
    row: String,
    application: Applicatiton,
    replace_last_row: bool,
    timestamp: NaiveDateTime
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SystemMessage {
    #[serde(rename = "type")]
    message_type: String,
    application: Applicatiton,
    message: SystemMessages,
    timestamp: NaiveDateTime
}

impl DataMessage {
    pub fn new(row: String, application: Applicatiton, replace_last_row: bool) -> Self {
        Self { message_type: "Data".to_string(), row, application, replace_last_row, timestamp: chrono::Utc::now().naive_utc() }
    }

    pub fn row(&self) -> &str {
        &self.row
    }
}

impl SystemMessage {
    pub fn new(application: Applicatiton, message: SystemMessages) -> Self {
        Self { message_type: "System".to_string(), application, message, timestamp: chrono::Utc::now().naive_utc() }
    }

    pub fn message(&self) -> &SystemMessages {
        &self.message
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Message {
    Data(DataMessage),
    System(SystemMessage),
    ClientDisconnect
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