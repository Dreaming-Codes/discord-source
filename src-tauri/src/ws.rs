use std::future;
use tokio::net::{TcpListener, TcpStream};
use tracing::info;
use futures_util::stream::StreamExt;
use futures_util::TryStreamExt;

pub struct WebSocketServer {
    listener: Option<TcpListener>,
}

impl WebSocketServer {
    pub fn new() -> Self {
        Self {
            listener: None,
        }
    }

    pub async fn bind(&mut self, addr: String) -> Result<(), Box<dyn std::error::Error>> {
        info!("WS server listening on: {}", addr);
        let listener = TcpListener::bind(addr).await?;

        self.listener = Some(listener);
        Ok(())
    }

    pub async fn run(&self) {
        if let Some(listener) = &self.listener {
            while let Ok((stream, _)) = listener.accept().await {
                tauri::async_runtime::spawn(accept_connection(stream));
            }
        }
    }
}

async fn accept_connection(stream: TcpStream) {
    let addr = stream.peer_addr().expect("connected streams should have a peer address");
    info!("Peer address: {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Error during the websocket handshake occurred");

    info!("New WebSocket connection: {}", addr);

    let (write, read) = ws_stream.split();
    // We should not forward messages other than text or binary.
    read.try_filter(|msg| future::ready(msg.is_text() || msg.is_binary()))
        .forward(write)
        .await
        .expect("Failed to forward messages")
}