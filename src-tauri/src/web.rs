use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tracing::info;

pub struct WebServer {
    listener: Option<TcpListener>,
    html: Option<String>,
}

impl WebServer {
    pub fn new() -> Self {
        Self {
            listener: None,
            html: None,
        }
    }


    pub async fn bind(&mut self, port: u16, ws_port: u16) -> Result<(), Box<dyn std::error::Error>> {
        info!("Webserver server listening on: {}", port);
        let listener_task = TcpListener::bind(format!("0.0.0.0:{}", port));
        let html_task = tokio::spawn(async move {
            let env_definitions = format!("<script>window.ws_port = {ws_port};</script>\n");

            format!("{}{}", env_definitions, fs::read_to_string("web/index.html").await.unwrap())
        });

        let (listener, html) = tokio::join!(listener_task, html_task);

        self.listener = Some(listener?);
        self.html = Some(html.unwrap());
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
    //let buf_reader = BufReader::new(&mut stream);
    //let http_request = buf_reader.lines().next_line().await.unwrap().unwrap();

    let status_line = "HTTP/1.1 200 OK";

    let length = contents.len();

    let response =
        format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).await.unwrap();
}