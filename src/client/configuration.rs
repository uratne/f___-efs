use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LogConfiguration {
    #[serde(rename = "app_name")]
    application: String,
    log_file_dir: String,
    log_file_name_regex: String,
    server_host: String,
    server_port: i16,
    server_path: String,
    channel_buffer: usize
}

impl LogConfiguration {
    pub fn get_application(&self) -> String {
        self.application.clone()
    }

    pub fn get_log_file_dir(&self) -> String {
        self.log_file_dir.clone()
    }

    pub fn get_log_file_name_regex(&self) -> String {
        self.log_file_name_regex.clone()
    }

    pub fn get_server_host(&self) -> String {
        self.server_host.clone()
    }

    pub fn get_server_port(&self) -> i16 {
        self.server_port
    }

    pub fn get_server_path(&self) -> String {
        self.server_path.clone()
    }

    pub fn get_channel_buffer(&self) -> usize {
        self.channel_buffer
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientConfiguration {
    #[serde(rename = "configs")]
    configurations: Vec<LogConfiguration>
}

impl ClientConfiguration {
    pub fn read_from_file() -> Self {
        let config = std::fs::read_to_string("fefs_config.json").unwrap();
        serde_json::from_str(&config).unwrap()
    }

    pub fn get_configurations(self) -> Vec<LogConfiguration> {
        self.configurations
    }
}