use std::collections::HashMap;

use futures_util::StreamExt;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::handshake::server::{ErrorResponse, Request, Response};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;
use tracing::{error, info, warn};
use crate::ws::message::MessageType;

mod message;

pub struct WebSocketServer {
    listener: Option<TcpListener>,
    web_connections: HashMap<String, WebSocketStream<TcpStream>>,
    discord_connection: Option<WebSocketStream<TcpStream>>,
}

enum Status {
    Ok(MessageType),
    Unhandled(Message),
    Closed,
}

impl WebSocketServer {
    pub fn new() -> Self {
        Self {
            listener: None,
            discord_connection: None,
            web_connections: HashMap::new(),
        }
    }

    pub async fn bind(&mut self, port: u16) -> Result<(), Box<dyn std::error::Error>> {
        info!("WS server listening on: {}", port);
        let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;

        self.listener = Some(listener);
        Ok(())
    }

    pub async fn accept_connections(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            let (raw_tcp_stream, _) = self.listener.as_ref().unwrap().accept().await?;

            let mut uri = String::new();

            let mut ws_stream = tokio_tungstenite::accept_hdr_async(raw_tcp_stream, |request: &Request, response: Response| -> Result<Response, ErrorResponse> {
                info!("WS connection request: {:?}", request.uri());
                uri = request.uri().to_string();

                Ok(response)
            }).await?;

            if uri == "/discord" {
                self.discord_connection = Some(ws_stream);
                info!("Discord connection established");

                loop {
                    let msg = self.discord_connection.as_mut().unwrap().next().await;
                    let status = handle_message(msg.unwrap().unwrap());
                    match status {
                        Status::Ok(event) => {
                            match event {
                                MessageType::Add(_) => {}
                                MessageType::Remove(_) => {}
                                MessageType::ICE(_) => {}
                                MessageType::Answer(_) => {}
                                MessageType::Offer(_) => {}
                                _ => {}
                            }
                        }
                        Status::Unhandled(msg) => {
                            warn!("Unhandled message from web: {:?}", msg);
                        }
                        Status::Closed => {
                            info!("Discord connection closed");
                            self.discord_connection = None;
                            break;
                        }
                    }
                }
            } else {
                let id = uri.split("/").last().unwrap();
                if id.len() < 1 {
                    warn!("Invalid web connection request: {}", uri);
                    ws_stream.close(None).await?;
                    continue;
                }
                if self.web_connections.contains_key(id) {
                    warn!("Web connection already exists: {}", id);
                    ws_stream.close(None).await?;
                    continue;
                }

                info!("Web connection established: {}", id);
                self.web_connections.insert(id.to_string(), ws_stream);
                let connection = self.web_connections.get_mut(id).unwrap();

                loop {
                    let msg = connection.next().await;
                    let status = handle_message(msg.unwrap().unwrap());
                    match status {
                        Status::Ok(event) => {
                            match event {
                                MessageType::ICE(_) => {}
                                MessageType::Answer(_) => {}
                                MessageType::Offer(_) => {}
                                _ => {
                                    error!("Invalid signal from web: {:?}", event);
                                }
                            }
                        }
                        Status::Unhandled(msg) => {
                            warn!("Unhandled message from web: {:?}", msg);
                        }
                        Status::Closed => {
                            info!("Web connection closed: {}", id);
                            self.web_connections.remove(id);
                            break;
                        }
                    }
                }
            }
        }
    }
}


fn handle_message(message: Message) -> Status {
    if message.is_close() {
        return Status::Closed;
    } else if let Ok(text) = message.to_text() {
        if let Ok(event) = serde_json::from_str::<MessageType>(text) {
            return Status::Ok(event);
        }
    }

    return Status::Unhandled(message);
}