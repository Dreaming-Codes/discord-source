#![cfg_attr(
all(not(debug_assertions), target_os = "windows"),
windows_subsystem = "windows"
)]

use parking_lot::Mutex;
use tauri::{Manager, State};
use tracing::{error};
use crate::web::WebServer;
use crate::ws::WebSocketServer;

mod ws;
mod web;
mod bd;

const NAME: &str = env!("CARGO_CRATE_NAME");
const DEFAULT_WS_PORT: u16 = 8214;
const DEFAULT_WEB_PORT: u16 = 4651;

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
struct Config {
    bd_path: Option<String>,
    web_port: u16
}

impl Default for Config {
    fn default() -> Self {
        Self {
            bd_path: None,
            web_port: DEFAULT_WEB_PORT
        }
    }
}

impl Config {
    fn load() -> Self {
        confy::load(NAME, None).unwrap_or_else(|_| {
            error!("Failed to parse config file, creating new one");
            std::fs::remove_file(confy::get_configuration_file_path(NAME, None).unwrap()).expect("Failed to remove config file");
            Config::default()
        })
    }
    fn save(&self) {
        confy::store(NAME, None, &(*self)).expect("Failed to save config");
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let cfg = Mutex::new(Config::load());

    let mut ws_server = WebSocketServer::new();
    let mut web_server = WebServer::new();

    tauri::async_runtime::set(tokio::runtime::Handle::current());
    tauri::Builder::default()
        .manage(cfg)
        .invoke_handler(tauri::generate_handler![bd::get_bd_path, bd::install_plugin, get_config])
        .setup(|app| {
            let cfg: State<'_, Mutex<Config>> = app.state();
            bind_servers(ws_server, web_server, DEFAULT_WS_PORT, cfg.lock().web_port);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
async fn get_config(state: State<'_, Mutex<Config>>) -> Result<Config,()>{
    Ok(state.lock().clone())
}

#[tauri::command]
async fn set_bd_path(state: State<'_, Mutex<Config>>, path: String) -> Result<(),()>{
    let mut cfg = state.lock();
    cfg.bd_path = Some(path);
    cfg.save();
    Ok(())
}

#[tauri::command]
async fn set_web_port(state: State<'_, Mutex<Config>>, port: u16) -> Result<(),()>{
    let mut cfg = state.lock();
    cfg.web_port = port;
    cfg.save();
    Ok(())
}

//TODO: Handle errors sensing the error to the UI and asking the user to change the port
fn bind_servers(mut ws_server: WebSocketServer, mut web_server: WebServer, ws_port: u16, web_port: u16) {
    tauri::async_runtime::spawn(async move {
        ws_server.bind(ws_port.clone()).await.expect("Failed to bind WS server");
        ws_server.accept_connections().await.expect("Failed to accept WS connections");
    });
    tauri::async_runtime::spawn(async move {
        web_server.bind(web_port, ws_port).await.expect("Failed to bind Web server");
        web_server.run().await;
    });
}