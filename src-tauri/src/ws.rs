use std::collections::HashMap;
use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use futures_util::lock::Mutex;
use futures_util::stream::{SplitSink, SplitStream};
use parking_lot::RwLock as PLRwLock;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;
use tokio_tungstenite::tungstenite::handshake::server::{ErrorResponse, Request, Response};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;
use tracing::{error, info, warn};

use crate::ws::message::MessageType;

pub mod message;

pub struct WebConnection {
    pub ws_sink: Arc<Mutex<SplitSink<WebSocketStream<TcpStream>, Message>>>,
    pub ws_stream: Arc<Mutex<SplitStream<WebSocketStream<TcpStream>>>>,
    pub linked_stream: Arc<PLRwLock<Option<u8>>>,
}

pub struct DiscordSplittedConnection {
    pub ws_sink: Arc<Mutex<SplitSink<WebSocketStream<TcpStream>, Message>>>,
    pub ws_stream: Arc<Mutex<SplitStream<WebSocketStream<TcpStream>>>>,
}


pub type WebConnections = Arc<RwLock<HashMap<String, WebConnection>>>;
pub type DiscordStreams = Arc<RwLock<Vec<u8>>>;
pub type DiscordConnection = Arc<RwLock<Option<DiscordSplittedConnection>>>;


pub struct WebSocketServer<R: tauri::Runtime> {
    listener: Option<TcpListener>,
    web_connections: WebConnections,
    discord_streams: DiscordStreams,
    discord_connection: DiscordConnection,
    window: Option<tauri::Window<R>>,
}

enum Status {
    Ok(MessageType),
    Unhandled(Message),
    Closed,
}

impl<R: tauri::Runtime> WebSocketServer<R> {
    pub fn new(discord_streams: DiscordStreams, web_connections: WebConnections, discord_connection: DiscordConnection) -> Self {
        Self {
            listener: None,
            discord_connection,
            discord_streams,
            web_connections,
            window: None,
        }
    }

    pub async fn bind(&mut self, port: u16) -> Result<(), Box<dyn std::error::Error>> {
        info!("WS server listening on: {}", port);
        let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;

        self.listener = Some(listener);
        Ok(())
    }

    pub fn set_window(&mut self, window: tauri::Window<R>) {
        self.window = Some(window);
    }

    pub async fn accept_connections(&mut self) {
        loop {
            let listener = self.listener.as_ref().unwrap().accept().await;

            let Ok((raw_tcp_stream, _)) = listener else {
                continue;
            };

            let mut uri = String::new();

            let ws_stream = tokio_tungstenite::accept_hdr_async(raw_tcp_stream, |request: &Request, response: Response| -> Result<Response, ErrorResponse> {
                info!("WS connection request: {:?}", request.uri());
                uri = request.uri().to_string();

                Ok(response)
            }).await;

            let Ok(mut ws_stream) = ws_stream else {
                continue;
            };

            if uri == "/discord" {
                let discord_connection = self.discord_connection.clone();
                {
                    let mut discord_connection = discord_connection.write().await;
                    if discord_connection.is_some() {
                        discord_connection.as_mut().unwrap().ws_sink.lock().await.close().await.unwrap();
                    }
                    let ws_stream_split = ws_stream.split();
                    *discord_connection = Some(DiscordSplittedConnection {
                        ws_sink: Arc::new(Mutex::new(ws_stream_split.0)),
                        ws_stream: Arc::new(Mutex::new(ws_stream_split.1)),
                    });
                }
                let window = self.window.clone().unwrap();
                let discord_streams = self.discord_streams.clone();
                let web_connections = self.web_connections.clone();
                tauri::async_runtime::spawn(async move {
                    loop {
                        let discord_connection = discord_connection.clone();
                        let msg = discord_connection.read().await.as_ref().unwrap().ws_stream.lock().await.next().await;
                        let status = handle_message(msg.unwrap().unwrap());
                        match status {
                            Status::Ok(event) => {
                                match event {
                                    MessageType::Add(stream) => {
                                        info!("Added stream: {:?}", stream);
                                        discord_streams.write().await.push(stream.stream_id);
                                        window.emit("stream-added", stream.stream_id).unwrap();
                                    }
                                    MessageType::Remove(stream) => {
                                        info!("Removed stream: {:?}", stream);
                                        window.emit("stream-removed", stream.stream_id).unwrap();
                                        discord_streams.write().await.retain(|id| *id != stream.stream_id);
                                    }
                                    MessageType::ICE(_) => {}
                                    MessageType::Offer(offer) => {
                                        info!("Offer: {:?}", offer);

                                        let web_connections = web_connections.read().await;

                                        let connection = web_connections.values().find(|connection| {
                                            connection.linked_stream.read().is_some() && connection.linked_stream.read().unwrap() == offer.stream_id.expect("Offer stream id is none on offer from discord")
                                        }).expect("No web connection found for offer from discord");

                                        connection.ws_sink.lock().await.send(Message::Text(serde_json::to_string(&offer).unwrap())).await.unwrap();
                                    }
                                    _ => {
                                        error!("Invalid signal from discord: {:?}", event);
                                    }
                                }
                            }
                            Status::Unhandled(msg) => {
                                warn!("Unhandled message from discord: {:?}", msg);
                            }
                            Status::Closed => {
                                info!("Discord connection closed");
                                discord_connection.write().await.take();
                                break;
                            }
                        }
                    };
                });
            } else {
                let id = uri.split('/').last().unwrap_or_default();
                if id.is_empty() {
                    warn!("Invalid web connection request: {:?}", uri);
                    let _ = ws_stream.close(None).await;
                    continue;
                }
                if self.web_connections.read().await.contains_key(id) {
                    warn!("Web connection already exists: {}", id);
                    let _ = ws_stream.close(None).await;
                    continue;
                }

                info!("Web connection established: {}", id);
                let (ws_sink, ws_stream) = ws_stream.split();

                self.web_connections.write().await.insert(id.to_string(), WebConnection {
                    ws_sink: Arc::new(Mutex::new(ws_sink)),
                    ws_stream: Arc::new(Mutex::new(ws_stream)),
                    linked_stream: Arc::new(PLRwLock::new(None)),
                });
                let connection = self.web_connections.read().await.get(id).unwrap().ws_stream.clone();
                let window = self.window.clone().unwrap();
                window.emit("web-added", id).unwrap();
                let web_connections = self.web_connections.clone();
                let discord_connection = self.discord_connection.clone();
                tauri::async_runtime::spawn({
                    let id = id.to_string();
                    async move {
                        loop {
                            let mut connection = connection.lock().await;
                            let msg = connection.next().await;
                            let status = handle_message(msg.unwrap().unwrap_or_else(|e| {
                                error!("Error reading message: {}", e);
                                Message::Close(None)
                            }));
                            match status {
                                Status::Ok(event) => {
                                    match event {
                                        MessageType::Answer(mut answer) => {
                                            info!("Answer: {:?}", answer);

                                            let target_stream_id = web_connections.read().await.get(&id).unwrap().linked_stream.read().unwrap();

                                            let _ = answer.stream_id.insert(target_stream_id);

                                            discord_connection.read().await.as_ref().unwrap().ws_sink.lock().await.send(Message::Text(serde_json::to_string(&answer).unwrap())).await.unwrap();
                                        }
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
                                    web_connections.write().await.remove(&id);
                                    window.emit("web-removed", id).unwrap();
                                    break;
                                }
                            }
                        }
                    }
                });
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

    Status::Unhandled(message)
}