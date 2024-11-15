use std::{fs, path::Path, time::{Duration, SystemTime}};

use configuration::LogConfiguration;
use log::{error, info};
use tokio::{fs::File, io::{AsyncBufReadExt, AsyncSeekExt, BufReader}, sync::mpsc::Sender, time::{self, sleep}};

use crate::message::{DataMessage, Message, SystemMessage, SystemMessages};

pub mod process;
pub mod configuration;

pub struct FileTailer {
    reader: BufReader<File>,
    path: String,
    regex: String,
    dir: String,
    created_date_time: SystemTime
}

impl FileTailer {
    pub async fn new(regex: String, dir: String) -> Option<Self> {
        let mut files = fs::read_dir(dir.clone()).unwrap();
        loop {
            let file = files.next();
            match file {
                Some(file) => {
                    let file = file.unwrap();
                    let file_name = file.file_name().into_string().unwrap();
                    if match_file_name(&file_name, &regex) {
                        info!("Found file: {}", file_name);
                        let file = File::open(&file_name).await.unwrap();
                        let created_date_time = file.metadata().await.unwrap().created().unwrap();
                        let reader = BufReader::new(file);
                        return Some(Self { reader, path: file_name, regex, dir, created_date_time })
                    }
                }
                None => {
                    break;
                }
            }
        }
        None
    }


    pub async fn tail(&mut self, tx: Sender<Message>, config: LogConfiguration) {

        self.reader.seek(std::io::SeekFrom::End(0)).await.unwrap();

        let sys_message = Message::System(SystemMessage::new(config.get_application(), SystemMessages::TailingStarted));
        tx.send(sys_message).await.unwrap();

        info!("Tailing file: {}", self.path);

        let mut last_line = String::new();
        let mut end_by_new_line = true;

        //TODO break on SIGTERM
        'OUTER: loop {
            loop {
                if tx.is_closed() {
                    break 'OUTER;
                }

                if !self.read_line(&tx, &mut last_line, &mut end_by_new_line, &config).await {
                    last_line.clear();
                    end_by_new_line = true;
                    let sys_message = Message::System(SystemMessage::new(config.get_application(), SystemMessages::FileRemoved));
                    if tx.is_closed() {
                        break 'OUTER;
                    }   
                    tx.send(sys_message).await.unwrap();
                    break;
                }
            }

            loop {
                if tx.is_closed() {
                    break 'OUTER;
                }

                if self.find_next_file().await {
                    let sys_message = Message::System(SystemMessage::new(config.get_application(), SystemMessages::NewFileFound));
                    tx.send(sys_message).await.unwrap();
                    break;
                }
                time::sleep(Duration::from_millis(100)).await;
            }
        }

        info!("Tailing stopped");
    }

    async fn read_line(&mut self, tx: &Sender<Message>, last_line: &mut String, end_by_new_line: &mut bool, config: &LogConfiguration) -> bool {
        let mut line = String::new();
        let bytes_read = self.reader.read_line(&mut line).await.unwrap();
    
        if bytes_read == 0 {
            sleep(Duration::from_millis(100)).await;
            let path = Path::new(self.path.as_str());
            let exists = Path::exists(path);
            if !exists {
                info!("File removed: {}", self.path);
                return false
            }
            let file = File::open(&self.path).await.unwrap();
            let same_file = file.metadata().await.unwrap().created().unwrap() == self.created_date_time;
            if !same_file {
                info!("File replaced: {}", self.path);
                return false
            }
        } else {
            process_line(line, &mut *last_line, &mut *end_by_new_line, &tx, &config).await;
        }    
        true
    }

    async fn find_next_file(&mut self) -> bool {
        let mut files = fs::read_dir(self.dir.clone()).unwrap();
        error!("Regex: {}", self.regex);
        loop {
            let file = files.next();
            match file {
                Some(file) => {
                    let file = file.unwrap();
                    let file_name = file.file_name().into_string().unwrap();
                    info!("Checking file: {}", file_name);
                    if match_file_name(&file_name, &self.regex) {
                        info!("Found file: {}", file_name);
                        let file = File::open(&file_name).await.unwrap();
                        let created_date_time = file.metadata().await.unwrap().created().unwrap();
                        self.created_date_time = created_date_time;
                        self.reader = BufReader::new(file);
                        self.path = file_name;
                        return true
                    }
                }
                None => {
                    break;
                }
            }
        }
        false
    }
}

async fn process_line(mut line: String, last_line: &mut String, end_by_new_line: &mut bool, tx: &Sender<Message>, config: &LogConfiguration) {
    let replacent: &str = "👻🛸👻";
    line = line.replace('\n', replacent);
    let lines = line.split('👻');
    for mut line in lines {
        if line.eq("") {
            continue;
        }
        if line.eq("🛸") {
            *end_by_new_line = true;
            continue;
        }
        let append = if *end_by_new_line {
            *end_by_new_line = false;
            false
        } else {
            last_line.push_str(line);
            line = &last_line;
            true
        };
        let message = DataMessage::new(line.to_string(), config.get_application(), append);
        let message = Message::Data(message);
        if tx.is_closed() {
            break;
        }
        match tx.send(message).await{
            Ok(_) => {}
            Err(e) => {
                error!("Error sending message: {}", e);
                break;
            }
        }
        info!("{}", line);
        *last_line = line.to_string();
    }
}

fn match_file_name(file_name: &str, regex: &str) -> bool {
    let re = regex::Regex::new(regex).unwrap();
    re.is_match(file_name)
}