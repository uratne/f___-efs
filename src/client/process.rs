use log::{debug, error, info, warn};
use tokio::{sync::mpsc::Sender, time};
use tokio_tungstenite::connect_async;
use futures_util::{SinkExt, StreamExt};
use tungstenite::{handshake::client::generate_key, http::Request, Message, Error};

use crate::{client::FileTailer, message};

use super::configuration::LogConfiguration;

pub async fn file(config: LogConfiguration) {
    loop {
        process_until_error(config.clone()).await;
        time::sleep(time::Duration::from_secs(20)).await;
    }
}

//TODO fix the unwraps with actual errors
async fn process_until_error(config: LogConfiguration) {
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
        .header("Application", config.get_application())
        .body(())
        .unwrap();
    
    let (ws_stream, _) = match connect_async(request).await {
        Ok(data) => data,
        Err(err) => {
            error!("Error connecting to WebSocket server: {}", err);
            return;
        },
    };

    info!("webSocket connected");
    
    // Split the WebSocket stream
    let (mut write, mut read) = ws_stream.split();
    let (tx, mut rx) = tokio::sync::mpsc::channel(config.get_channel_buffer());
    let (tx_client_abort, mut rx_client_abort) = tokio::sync::mpsc::channel::<()>(1);
    let (tx_server_abort, mut rx_server_abort) = tokio::sync::mpsc::channel::<()>(1);
    
    // Spawn a task to handle incoming messages
    let tx_clone = tx.clone();
    let receive_task = tokio::spawn(async move {
        let mut abort_send_task = true;
        loop {
            tokio::select! {
                _ = rx_client_abort.recv() => {
                    info!("client receive task aborted");
                    abort_send_task = false;
                    break;
                },
                message = read.next() => {
                    if message.is_some() && !process_message(message.unwrap(), &tx_clone).await {
                        break;
                    }
                }
            }
        }
        
        if abort_send_task {
            tx_server_abort.send(()).await.unwrap();
        }
        info!("client receive task stopped");
    });

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
        let mut send = false;
        let mut abort_receive_task= true;
        loop {
            let msg = tokio::select! {
                _ = rx_server_abort.recv() => {
                    info!("client send task aborted");
                    abort_receive_task = false;
                    break;
                },
                msg = rx.recv() => {
                    match msg {
                        Some(msg) => msg,
                        None => break,
                    }
                }
            };

            if msg.system().is_some() {
                match msg.system().unwrap().message() {
                    message::SystemMessages::Stop => {
                        info!("stopped sending messages");
                        break;
                    }, 
                    message::SystemMessages::Start => {
                        info!("starting to send messages");
                        send = true;
                    },
                    message::SystemMessages::Pause => {
                        info!("paused sending messages");
                        send = false;
                    },
                    message::SystemMessages::Resume => {
                        info!("resumed sending messages");
                        send = true;
                    },
                    _ => {}
                }
            }
            let msg = serde_json::to_string(&msg).unwrap();

            if send {
                if let Err(e) = write.send(Message::Text(msg)).await {
                    error!("Error sending message: {}", e);
                    break;
                }
            }
        }

        if abort_receive_task {
            tx_client_abort.send(()).await.unwrap();
        }
        info!("client send task stopped");
    });

    let _ = tokio::join!(receive_task, send_task);
    info!("client stopped");
}

async fn process_message(message: Result<Message, Error>, tx_clone: &Sender<crate::message::Message>) -> bool {
    match message {
        Ok(msg) => {
            match msg {
                Message::Text(text) => {
                    info!("Received: {}", text);
                    let message: crate::message::Message = serde_json::from_str(&text).unwrap();
                    tx_clone.send(message).await.unwrap();
                },
                Message::Binary(data) => info!("Received binary data: {:?}", data),
                Message::Ping(_) => debug!("Received ping"),
                Message::Pong(_) => debug!("Received pong"),
                Message::Close(_) => {
                    info!("Server closed connection");
                    return false
                },
                Message::Frame(_) => warn!("Received raw frame"),
            }
        }
        Err(e) => {
            error!("Error receiving message: {}", e);
            return false
        }
    }

    true
}