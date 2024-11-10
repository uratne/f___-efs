use std::{path::Path, time::Duration};

use log::info;
use tokio::{fs::File, io::{AsyncBufReadExt, AsyncSeekExt, BufReader}, time::sleep};

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


    pub async fn tail(&mut self, n: i64) {
        self.reader.seek(std::io::SeekFrom::End(n)).await.unwrap();

        info!("Tailing file: {}", self.path);
        info!("Last {} lines:", n);

        let mut last_line = String::new();
        let mut end_by_new_line = false;
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
                handle_line(line, &mut last_line, &mut end_by_new_line);
            }
        }
        info!("Tailing file {} ended", self.path);
    }
}

fn handle_line(mut line: String, last_line: &mut String, end_by_new_line: &mut bool) {
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
        if *end_by_new_line {
            *end_by_new_line = false;
        } else {
            last_line.push_str(line);
            line = &last_line;
        }
        info!("{}", line);
        *last_line = line.to_string();
    }
}