use std::{path::Path, time::Duration};

use log::info;
use tokio::{fs::File, io::{AsyncBufReadExt, AsyncSeekExt, BufReader}, sync::mpsc::Sender, time::sleep};

use crate::message::Message;

pub struct FileTailer {
    reader: BufReader<File>,
    path: String,
}

impl FileTailer {
    pub async fn new(path: String) -> Self {
        let file = File::open(&path).await.unwrap();
        let reader = BufReader::new(file);
        Self { reader, path }
    }


    pub async fn tail(&mut self, tx: Sender<Message>) {
        //TODO find about END(value), what value is
        self.reader.seek(std::io::SeekFrom::End(0)).await.unwrap();

        info!("Tailing file: {}", self.path);

        let mut last_line = String::new();
        let mut end_by_new_line = true;
        loop {
            let mut line = String::new();
            let bytes_read = self.reader.read_line(&mut line).await.unwrap();

            if bytes_read == 0 {
                sleep(Duration::from_millis(100)).await;
                let path = Path::new(self.path.as_str());
                let file = Path::exists(path);
                if !file {
                    info!("File not found: {}", self.path);
                    break;
                }
            } else {
                handle_line(line, &mut last_line, &mut end_by_new_line, &tx).await;
            }
        }
        info!("Tailing file {} ended", self.path);
    }
}

async fn handle_line(mut line: String, last_line: &mut String, end_by_new_line: &mut bool, tx: &Sender<Message>) {
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
        let message = Message::new(line.to_string(), "application".to_string(), append);
        match tx.send(message).await{
            Ok(_) => {}
            Err(e) => {
                info!("Error sending message: {}", e);
                break;
            }
        }
        info!("{}", line);
        *last_line = line.to_string();
    }
}