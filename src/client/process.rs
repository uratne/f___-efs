use log::{debug, error, info};
use tokio_tungstenite::connect_async;
use futures_util::{SinkExt, StreamExt};
use tungstenite::{handshake::client::generate_key, http::Request, Message};

use crate::client::FileTailer;

use super::configuration::LogConfiguration;

pub async fn file(config: LogConfiguration) {
    let host = config.get_server_host();
    let port = config.get_server_port();
    let path = config.get_server_path();
    let host = format!("{}:{}", host, port);
    let uri = format!("ws://{}/{}", host, path);

    info!("connecting to {}", uri);
    // Connect to WebSocket server
    let request = Request::builder()
        .uri(uri)
        .header("Host", host)
        .header("Sec-WebSocket-Key", generate_key())
        .header("Sec-WebSocket-Version", "13")
        .header("Upgrade", "websocket")
        .header("Connection", "Upgrade")
        .body(())
        .unwrap();

    let (ws_stream, _) = connect_async(request).await.expect("Failed to connect");
    info!("webSocket connected");

    // Split the WebSocket stream
    let (mut write, mut read) = ws_stream.split();

    // Spawn a task to handle incoming messages
    let receive_task = tokio::spawn(async move {
        while let Some(message) = read.next().await {
            match message {
                Ok(msg) => {
                    match msg {
                        Message::Text(text) => info!("Received: {}", text),
                        Message::Binary(data) => info!("Received binary data: {:?}", data),
                        Message::Ping(_) => debug!("Received ping"),
                        Message::Pong(_) => debug!("Received pong"),
                        Message::Close(_) => {
                            info!("Server closed connection");
                            break;
                        },
                        Message::Frame(_) => println!("Received raw frame"),
                    }
                }
                Err(e) => {
                    eprintln!("Error receiving message: {}", e);
                    break;
                }
            }
        }
    });

    let (tx, mut rx) = tokio::sync::mpsc::channel(config.get_channel_buffer());
    let file_tailer = FileTailer::new(config.get_log_file_name_regex(), config.get_log_file_dir()).await;

    match file_tailer {
        Some(mut file_tailer) => {
            tokio::spawn(async move {
                file_tailer.tail(tx, config).await;
            });
        }
        None => {
            error!("No file found");
        }
    }
    
    
    // Send messages
    let send_task = tokio::spawn(async move {
        // Keep the connection alive
        loop {
            let msg = match rx.recv().await {
                Some(msg) => msg,
                None => break,
            };
            let msg = serde_json::to_string(&msg).unwrap();
            if let Err(e) = write.send(Message::Text(msg)).await {
                error!("Error sending message: {}", e);
                break;
            }

            //TODO move to a separate task as this is useless
            if let Err(e) = write.send(Message::Ping(vec![])).await {
                error!("Error sending ping: {}", e);
                break;
            }
        }
    });



    // Wait for tasks to complete
    let _ = tokio::join!(receive_task, send_task);
}