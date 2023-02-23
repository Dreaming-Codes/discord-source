use std::collections::HashMap;

use futures_util::StreamExt;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::handshake::server::{ErrorResponse, Request, Response};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;
use tracing::{info, warn};

mod signals;

pub struct WebSocketServer {
    listener: Option<TcpListener>,
    web_connections: HashMap<String, WebSocketStream<TcpStream>>,
    discord_connection: Option<WebSocketStream<TcpStream>>,
}

enum Status {
    Ok,
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
                        Status::Ok => {}
                        Status::Unhandled(msg) => {
                            warn!("Unhandled message from web: {:?}", msg);
                        }
                        Status::Closed => {}
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
                let mut connection = self.web_connections.insert(id.to_string(), ws_stream).unwrap();

                loop {
                    let msg = connection.next().await;
                    let status = handle_message(msg.unwrap().unwrap());
                    match status {
                        Status::Ok => {}
                        Status::Unhandled(msg) => {
                            warn!("Unhandled message from web: {:?}", msg);
                        }
                        Status::Closed => {}
                    }
                }
            }
        }
    }
}


fn handle_message(message: Message) -> Status {
    if message.is_close() {
        return Status::Closed;
    } else if message.is_text() {
        let text = message.to_text().unwrap();
        //TODO: Handle messages
    }

    return Status::Unhandled(message);
}