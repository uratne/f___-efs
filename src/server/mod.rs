use actix_web::{rt, web, Error, HttpRequest, HttpResponse, Result};
use actix_ws::AggregatedMessage;
use log::{error, info};

use crate::message::Message;

#[actix_web::get("/ws")]
pub async fn listen(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    info!("WebSocket connection request from {}", req.peer_addr().unwrap());
    let (res, mut session, stream) = actix_ws::handle(&req, stream)?;

    let mut stream = stream
        .aggregate_continuations()
        // aggregate continuation frames up to 1MiB
        .max_continuation_size(2_usize.pow(20));

    info!("WebSocket connection established");
    // start task but don't wait for it
    rt::spawn(async move {
        // receive messages from websocket
        while let Some(msg) = stream.recv().await{
            match msg {
                Ok(AggregatedMessage::Text(text)) => {
                    // echo text message
                    let message: Result<Message, serde_json::Error> = serde_json::from_str(&text);
                    match message {
                        Ok(message) => {
                            info!("Received message: {:#?}", message);
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
                        info!("Closing session with reason code: {:?} and description: {:?}", reason.clone().unwrap().code, reason.clone().unwrap().description);
                    } else {
                        error!("Closing session without reason");
                    }
   
                    session.close(reason).await.unwrap();
                    break;
                }

                _ => {}
            }
        }
    });

    Ok(res)
}