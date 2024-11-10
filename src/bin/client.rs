use futures_util::{SinkExt, StreamExt};
use tungstenite::handshake::client::{generate_key, Request};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

#[tokio::main]
async fn main() {
    let host = "localhost:8080";
    let path = "/ws";
    let uri = format!("ws://{}{}", host, path);

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
    println!("WebSocket connected");

    // Split the WebSocket stream
    let (mut write, mut read) = ws_stream.split();

    // Spawn a task to handle incoming messages
    let receive_task = tokio::spawn(async move {
        while let Some(message) = read.next().await {
            match message {
                Ok(msg) => {
                    match msg {
                        Message::Text(text) => println!("Received: {}", text),
                        Message::Binary(data) => println!("Received binary data: {:?}", data),
                        Message::Ping(_) => println!("Received ping"),
                        Message::Pong(_) => println!("Received pong"),
                        Message::Close(_) => {
                            println!("Server closed connection");
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

    // Send messages
    let send_task = tokio::spawn(async move {
        // Send a text message
        let message = lib::message::Message::new("Hello from client!".to_string(), "test client".to_string());
        let message = serde_json::to_string(&message).unwrap();
        if let Err(e) = write.send(Message::Text(message.clone())).await {
            eprintln!("Error sending message: {}", e);
        }

        // Keep the connection alive
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            if let Err(e) = write.send(Message::Ping(vec![])).await {
                eprintln!("Error sending ping: {}", e);
                break;
            }
        }
    });

    // Wait for tasks to complete
    let _ = tokio::join!(receive_task, send_task);
}