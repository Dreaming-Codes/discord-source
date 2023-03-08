#![cfg_attr(
all(not(debug_assertions), target_os = "windows"),
windows_subsystem = "windows"
)]

use parking_lot::Mutex;
use tauri::{CustomMenuItem, Manager, RunEvent, SystemTray, SystemTrayEvent, SystemTrayMenu};
use tracing::error;

use crate::bd::BdSettings;
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
    web_port: u16,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            bd_path: None,
            web_port: DEFAULT_WEB_PORT,
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

struct State {
    config: Mutex<Config>,
    bd_settings: Mutex<Option<BdSettings>>,
}

#[tokio::main]
async fn main() {
    let hwid = hardware_id::get_id().expect("Failed to get hardware id");

    let license = reqwest::get(&format!("https://discord-source-license.dreamingcodes.workers.dev/{}", hwid)).await.expect("Failed to validate hwid");

    if !license.text().await.expect("Failed to get license text").eq("true") {
        panic!("Invalid license, please contact the developer sending your hardware id: {}", hwid);
    }

    tracing_subscriber::fmt::init();

    let config = Mutex::new(Config::load());

    let mut ws_server = WebSocketServer::new();
    let mut web_server = WebServer::new();

    tauri::async_runtime::set(tokio::runtime::Handle::current());

    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let show = CustomMenuItem::new("show".to_string(), "Show");
    let tray_menu = SystemTrayMenu::new()
        .add_item(show)
        .add_item(quit);

    #[allow(clippy::single_match)]
    tauri::Builder::default()
        .manage(State {
            config,
            bd_settings: Mutex::new(None),
        })
        .system_tray(SystemTray::new().with_menu(tray_menu))
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::MenuItemClick { id, .. } => {
                match id.as_str() {
                    "quit" => app.exit(0),
                    "show" => {
                        app.windows().get("main").unwrap().show().expect("Failed to show window");
                        app.windows().get("main").unwrap().set_focus().expect("Failed to focus window");

                        let window = app.get_window("main").unwrap();

                        window.show().expect("Failed to show window");
                        window.set_focus().expect("Failed to focus window");
                    }
                    _ => {}
                }
            }
            SystemTrayEvent::LeftClick { .. } => {
                let window = app.get_window("main").unwrap();

                window.show().expect("Failed to show window");
                window.set_focus().expect("Failed to focus window");
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![bd::get_bd_path, bd::install_plugin, get_config])
        .setup(|app| {
            let cfg: tauri::State<'_, State> = app.state();

            ws_server.set_window(app.get_window("main").unwrap());

            bind_servers(ws_server, web_server, DEFAULT_WS_PORT, cfg.config.lock().web_port);
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app_handle, event| match event {
            RunEvent::WindowEvent { event, label, .. } => {
                if label == "main" {
                    //TODO: Add a way to reinitialize the window instead of just hiding it
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        _app_handle.windows().get("main").unwrap().hide().expect("Failed to hide window");
                    }
                }
            }
            _ => {}
        });
}

#[tauri::command]
async fn get_config(state: tauri::State<'_, State>) -> Result<Config, ()> {
    Ok(state.config.lock().clone())
}

#[tauri::command]
async fn set_bd_path(state: tauri::State<'_, State>, path: String) -> Result<(), ()> {
    let mut cfg = state.config.lock();
    cfg.bd_path = Some(path);
    cfg.save();
    Ok(())
}

#[tauri::command]
async fn set_web_port(state: tauri::State<'_, State>, port: u16) -> Result<(), ()> {
    let mut cfg = state.config.lock();
    cfg.web_port = port;
    cfg.save();
    Ok(())
}

//TODO: Handle errors sensing the error to the UI and asking the user to change the port
fn bind_servers<R: tauri::Runtime>(mut ws_server: WebSocketServer<R>, mut web_server: WebServer, ws_port: u16, web_port: u16) {
    tauri::async_runtime::spawn(async move {
        ws_server.bind(ws_port.clone()).await.expect("Failed to bind WS server");
        ws_server.accept_connections().await.expect("Failed to accept WS connections");
    });
    tauri::async_runtime::spawn(async move {
        web_server.bind(web_port, ws_port).await.expect("Failed to bind Web server");
        web_server.run().await;
    });
}