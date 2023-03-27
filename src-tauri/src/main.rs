#![cfg_attr(
all(not(debug_assertions), target_os = "windows"),
windows_subsystem = "windows"
)]

use std::collections::HashMap;
use std::sync::Arc;

use futures_util::future::join_all;
use parking_lot::Mutex;
use tauri::{CustomMenuItem, Manager, RunEvent, SystemTray, SystemTrayEvent, SystemTrayMenu};
use tokio::sync::RwLock;
use tracing::{error, info};

use crate::bd::BdSettings;
use crate::web::WebServer;
use crate::ws::{DiscordStreams, WebConnections, WebSocketServer};

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

#[derive(serde::Deserialize)]
struct LinkEvent {
    source: Option<u8>,
    target: String,
}

#[tokio::main]
async fn main() {
    // IMPORTANT:
    // Before using the Software,
    // please ensure
    // that you have read and understood the terms and conditions of the License Agreement
    // provided in the LICENSE.md file.
    // In particular,
    // note that the license key check with my servers must be left in place
    // and functioning properly for any distribution of the Software.
    // Therefore,
    // it is essential that you call the function check_license()
    // before proceeding to use the Software.
    // Any attempt to remove this check or redistribute the Software without the license key check may result in a violation of the license agreement.
    check_license().await;

    tracing_subscriber::fmt::init();

    let config = Mutex::new(Config::load());

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
        .manage::<WebConnections>(Arc::new(RwLock::new(HashMap::new())))
        .manage::<DiscordStreams>(Arc::new(RwLock::new(Vec::new())))
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
        .invoke_handler(tauri::generate_handler![bd::get_bd_path, bd::install_plugin, get_config, get_streams, get_targets])
        .setup(|app| {
            let discord_streams: tauri::State<'_, DiscordStreams> = app.state();
            let web_connections: tauri::State<'_, WebConnections> = app.state();

            let discord_streams = Arc::clone(&discord_streams);
            let web_connections = Arc::clone(&web_connections);

            let mut ws_server = WebSocketServer::new(discord_streams, web_connections.clone());
            let mut web_server = WebServer::new();

            let cfg: tauri::State<'_, State> = app.state();

            app.listen_global("link-stream", {
                let web_connections = web_connections.clone();
                move |event| {
                    let data: LinkEvent = serde_json::from_str(event.payload().unwrap()).unwrap();
                    info!("Link stream event: {:?}", event.payload());
                    let web_connections = web_connections.clone();
                    tauri::async_runtime::spawn(async move {
                        //TODO: Actually do the webrtc handshake
                        let _ = web_connections.write().await.get(&data.target).unwrap().linked_stream.write().await.insert(data.source.unwrap());
                    });
                }
            });

            app.listen_global("unlink-stream", move |event| {
                let data: LinkEvent = serde_json::from_str(event.payload().unwrap()).unwrap();
                info!("Unlink stream event: {:?}", event.payload());
                let web_connections = web_connections.clone();
                tauri::async_runtime::spawn(async move {
                    //TODO: Actually unlink the stream
                    let _ = web_connections.write().await.get(&data.target).unwrap().linked_stream.write().await.take();
                });
            });

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

#[tauri::command]
async fn get_targets(web_connections: tauri::State<'_, WebConnections>) -> Result<HashMap<String, Option<u8>>, ()> {
    let web_connections = web_connections.read().await;

    let tasks = web_connections
        .iter()
        .map(|(id, conn)| {
            let id = id.clone();
            let linked_stream = conn.linked_stream.clone();
            tokio::spawn(async move {
                (id, linked_stream.read().await.clone())
            })
        })
        .collect::<Vec<_>>();

    Ok(join_all(tasks).await
        .into_iter()
        .map(|res| res.unwrap())
        .collect::<HashMap<String, Option<u8>>>()
    )
}

#[tauri::command]
async fn get_streams(discord_streams: tauri::State<'_, DiscordStreams>) -> Result<Vec<u8>, ()> {
    let discord_streams = discord_streams.read().await;
    Ok(discord_streams.clone())
}

//TODO: Handle errors sensing the error to the UI and asking the user to change the port
fn bind_servers<R: tauri::Runtime>(mut ws_server: WebSocketServer<R>, mut web_server: WebServer, ws_port: u16, web_port: u16) {
    tauri::async_runtime::spawn(async move {
        ws_server.bind(ws_port).await.expect("Failed to bind WS server");
        ws_server.accept_connections().await;
    });
    tauri::async_runtime::spawn(async move {
        web_server.bind(web_port, ws_port).await.expect("Failed to bind Web server");
        web_server.run().await;
    });
}

// IMPORTANT:
// Before using the Software,
// please ensure
// that you have read and understood the terms and conditions of the License Agreement provided above.
// In particular,
// note that the license key check with my servers must be left in place
// and functioning properly for any distribution of the Software.
// Therefore,
// it is essential that you call the function check_license()
// before proceeding to use the Software.
// Any attempt to remove this check or redistribute the Software without the license key check may result in a violation of the license agreement.
async fn check_license() {
    let hwid = hardware_id::get_id().expect("Failed to get hardware id");

    let license = reqwest::get(&format!("https://discord-source-license.dreamingcodes.workers.dev/{}", hwid)).await.expect("Failed to validate hwid");

    if !license.text().await.expect("Failed to get license text").eq("true") {
        panic!("Invalid license, please contact the developer sending your hardware id: {}", hwid);
    }
}