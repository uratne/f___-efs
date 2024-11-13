use std::env;

use futures_util::{SinkExt, StreamExt};
use log::{debug, error, info};
use tungstenite::{handshake::client::{generate_key, Request}, Message};
use tokio_tungstenite::connect_async;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let host = env::var("HOST").unwrap_or_else(|_| "localhost".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let path = "ws";
    let host = format!("{}:{}", host, port);
    let uri = format!("ws://{}/{}", host, path);

    info!("connecting to {}", uri);
    // Connect to WebSocket server
    let mut request = Request::builder()
        .uri(uri)
        .header("Host", host)
        .header("Sec-WebSocket-Key", generate_key())
        .header("Sec-WebSocket-Version", "13")
        .header("Upgrade", "websocket")
        .header("Connection", "Upgrade")
        .body(())
        .unwrap();

    request.headers_mut().insert("api-key", "42".parse().unwrap());
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

    let (tx, mut rx) = tokio::sync::mpsc::channel(10);
    let file_tailer = lib::client::FileTailer::new(r#"^log.*\.txt$"#.to_string(), ".".to_string()).await;

    match file_tailer {
        Some(mut file_tailer) => {
            tokio::spawn(async move {
                file_tailer.tail(tx).await;
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
            if let Err(e) = write.send(Message::Ping(vec![])).await {
                error!("Error sending ping: {}", e);
                break;
            }
        }
    });



    // Wait for tasks to complete
    let _ = tokio::join!(receive_task, send_task);
}