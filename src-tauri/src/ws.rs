use std::collections::HashMap;

use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::handshake::server::{ErrorResponse, Request, Response};
use tracing::info;

pub struct WebSocketServer {
    listener: Option<TcpListener>,
    web_connections: HashMap<u16, WebSocketStream<TcpStream>>,
    discord_connection: Option<WebSocketStream<TcpStream>>,
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
            } else {
                let id = uri.split("/").last().unwrap().parse::<u16>();
                let id = match id {
                    Ok(id) => id,
                    Err(_) => {
                        info!("Received invalid ws connection: {}", uri);
                        ws_stream.close(None).await?;
                        continue;
                    }
                };
                info!("Web connection established: {}", id);
                self.web_connections.insert(id, ws_stream);
            }
        }
    }
}

