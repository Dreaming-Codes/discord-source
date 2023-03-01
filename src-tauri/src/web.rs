use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tracing::info;

const HTML: &str = include_str!("../dist/web/index.html");

pub struct WebServer {
    listener: Option<TcpListener>,
    html: Option<String>
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

            format!("{}{}", env_definitions, HTML)
        });

        let (listener, html) = tokio::join!(listener_task, html_task);

        self.html = Some(html.unwrap());

        self.listener = Some(listener?);
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

async fn handle_connection(mut stream: TcpStream, content: String) {
    //let buf_reader = BufReader::new(&mut stream);
    //let http_request = buf_reader.lines().next_line().await.unwrap().unwrap();

    const STATUS_LINE: &str = "HTTP/1.1 200 OK";

    let response = format!("{}\r\nContent-Length: {}\r\n\r\n{}", STATUS_LINE, content.len(), content);

    stream.write_all(response.as_bytes()).await.unwrap();
}