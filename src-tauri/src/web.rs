use std::collections::HashMap;
use tokio::fs;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tracing::info;

pub struct WebServer {
    listener: Option<TcpListener>,
    env: HashMap<String, String>,
    html: Option<String>
}

impl WebServer {
    pub fn new(env: HashMap<String, String>) -> Self {
        Self {
            listener: None,
            env,
            html: None
        }
    }

    pub async fn build_html(&mut self) {
        let env = self.env.iter().map(|(k, v)| format!("{}:{},", k, v)).collect::<Vec<String>>().join("");

        println!("env: {}", env);

        let env_definitions = format!("<script>window.env = {{{env}}};</script>\n");

        self.html = Some(format!("{}{}", env_definitions, fs::read_to_string("web/index.html").await.unwrap()));
    }

    pub async fn bind(&mut self, addr: String) -> Result<(), Box<dyn std::error::Error>> {
        info!("Webserver server listening on: {}", addr);
        let listener = TcpListener::bind(addr).await?;

        self.listener = Some(listener);
        Ok(())
    }

    pub async fn run(&self) {
        if let Some(listener) = &self.listener {
            while let Ok((stream, _)) = listener.accept().await {
                tauri::async_runtime::spawn(handle_connection(stream, self.html.clone().unwrap()));
            }
        }
    }
}

async fn handle_connection(mut stream: TcpStream, contents: String) {
    let buf_reader = BufReader::new(&mut stream);
    let http_request = buf_reader.lines().next_line().await.unwrap().unwrap();

    let status_line = "HTTP/1.1 200 OK";

    let length = contents.len();

    let response =
        format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).await.unwrap();
}