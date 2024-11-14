use std::{sync::Arc, time::Duration};

use actix_web::{body::MessageBody, get, rt, web, Error, HttpRequest, HttpResponse, Responder, Result};
use actix_ws::{AggregatedMessage, ProtocolError, Session};
use broadcaster::Broadcasters;
use log::{error, info, trace};
use tokio::{sync::broadcast::{self, Sender}, time::sleep};
use futures::{future, stream::StreamExt};
use tokio_stream::wrappers::BroadcastStream;

use crate::{message::{Message, SystemMessage, SystemMessages}, Applicatiton};

pub mod broadcaster;

#[get("/sse")]
pub async fn data_outbound(req: HttpRequest, broadcasters: web::Data<Arc<Broadcasters>>) -> impl Responder {
    let application = req.headers().get("Application").unwrap().to_str().unwrap();
    let application: Applicatiton = serde_json::from_str(application).unwrap();
    let broadcasters = broadcasters.lock().await;
    let rx = match broadcasters.get(&application) {
        Some(tx) => tx.subscribe(),
        None => {
            error!("No broadcaster found for application: {}", application.name());
            return HttpResponse::BadRequest().finish();
        },
    };
    drop(broadcasters);

    let stream = BroadcastStream::new(rx)
    .take_while(|msg| future::ready(
        match msg {
            Ok(Message::ClientDisconnect) => false,
            Err(_) => false,
            _ => true,
        }
    ))
    .map(|msg| {
        match msg {
            Ok(msg) => match msg {
                Message::Data(data) => {
                    info!("Sending data message: {:#?}", data);
                    let msg = serde_json::to_string(&data).unwrap();
                    format!("data: {}\n\n", msg).try_into_bytes()
                }
                Message::System(sys) => {
                    info!("Sending system message: {:#?}", sys);
                    let msg = serde_json::to_string(&sys).unwrap();
                    format!("data: {}\n\n", msg).try_into_bytes()
                }
                Message::ClientDisconnect => {
                    info!("Client disconnected");
                    format!("data: Client disconnected\n\n").try_into_bytes()
                }
            },
            Err(err) => {
                error!("Error sending message: {:?}", err);
                format!("data: Error: {}\n\n", err).try_into_bytes()
            },
        }
    });

    HttpResponse::Ok()
        .append_header(("content-type", "text/event-stream"))
        .append_header(("cache-control", "no-cache"))
        .append_header(("connection", "keep-alive"))
        .streaming(stream)
}

#[actix_web::get("/ws")]
pub async fn data_inbound(req: HttpRequest, stream: web::Payload, broadcasters: web::Data<Arc<Broadcasters>>) -> Result<HttpResponse, Error> {
    info!("WebSocket connection request from {}", req.peer_addr().unwrap());
    let (res, mut session, stream) = actix_ws::handle(&req, stream)?;
    let application = req.headers().get("Application").unwrap().to_str().unwrap().to_string();
    let application: Applicatiton = serde_json::from_str(&application).unwrap();
    
    let mut stream = stream
    .aggregate_continuations()
    // aggregate continuation frames up to 1MiB
    .max_continuation_size(2_usize.pow(20));
    
    info!("WebSocket connection established for application: {}", application.name());

    let (tx, _) = broadcast::channel(100);

    let mut locked_broadcasters = broadcasters.lock().await;
    locked_broadcasters.insert(application.clone(), tx.clone());
    drop(locked_broadcasters);

    let start_message = Message::System(SystemMessage::new(application.clone(), SystemMessages::Start));
    session.text(serde_json::to_string(&start_message).unwrap()).await.unwrap();

    let app = application.clone();

    let mut ping_session = session.clone();
    let handle = rt::spawn(async move {
        while let Some(msg) = stream.recv().await {
            if tx.receiver_count() > 0 {
                match handle_message(msg, &mut session, &tx).await {
                    false => break,
                    _ => {
                        continue;
                    }
                }
            }

            let pause_message = Message::System(SystemMessage::new(application.clone(), SystemMessages::Pause));
            session.text(serde_json::to_string(&pause_message).unwrap()).await.unwrap();
            loop {
                if tx.receiver_count() > 0 {
                    // Consume any pending messages in the stream buffer
                    while let Ok(msg) = tokio::time::timeout(
                        Duration::from_millis(50), 
                        stream.recv()
                    ).await {
                        info!("Consuming pending message: {:?}", msg);
                        continue;
                    }

                    let resume_message = Message::System(SystemMessage::new(application.clone(), SystemMessages::Resume));
                    session.text(serde_json::to_string(&resume_message).unwrap()).await.unwrap();
                    break;
                }
                sleep(Duration::from_secs(1)).await;
            }
        }
        info!("webSocket connection closed");
    });

    rt::spawn(async move {
        let ping_interval = Duration::from_secs(1);
        
        while let Ok(()) = ping_session.ping(b"ping").await {
            sleep(ping_interval).await;
        }
        
        info!("Ping failed, aborting message handler");
        let mut locked_broadcasters = broadcasters.lock().await;
        let rx = locked_broadcasters.remove(&app).unwrap();
        drop(locked_broadcasters);
        let _ = rx.send(Message::ClientDisconnect);
        handle.abort();
        info!("WebSocket connection terminated by ping monitor");
    });
    
    Ok(res)
}

async fn handle_message(msg: Result<AggregatedMessage, ProtocolError>, session: &mut Session, tx: &Sender<Message>) -> bool {
    match msg {
        Ok(AggregatedMessage::Text(text)) => {
            // echo text message
            let message: Result<Message, serde_json::Error> = serde_json::from_str(&text);
            match message {
                Ok(message) => {
                    info!("Received message: {:#?}", message);
                    match tx.send(message) {
                        Ok(n) => trace!("message broadcasted to {} subscribers", n),
                        Err(err) => error!("Error broadcasting message: {:?}", err),
                    }
                }
                Err(e) => {
                    error!("Failed to parse message: {:?}", e);
                }
            }
        }
        
        Ok(AggregatedMessage::Binary(bin)) => {
            // echo binary message
            session.binary(bin).await.unwrap();
        }
        
        Ok(AggregatedMessage::Ping(msg)) => {
            // respond to PING frame with PONG frame
            session.pong(&msg).await.unwrap();
        }
        
        Ok(AggregatedMessage::Close(reason)) => {
            // close the session
            if reason.is_some() {
                info!("Closing session with reason code: {:?} and description: {:?}", reason.clone().unwrap().code, reason.unwrap().description);
            } else {
                error!("Closing session without reason");
            }

            return false;
        }

        _ => {}
    }

    true
}