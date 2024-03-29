#![cfg_attr(
all(not(debug_assertions), target_os = "windows"),
windows_subsystem = "windows"
)]

use std::collections::HashMap;
use std::sync::Arc;

use futures_util::future::join_all;
use futures_util::SinkExt;
use parking_lot::Mutex as PLMutex;
use tauri::{CustomMenuItem, Manager, RunEvent, SystemTray, SystemTrayEvent, SystemTrayMenu};
use tokio::sync::RwLock;
use tokio_tungstenite::tungstenite::Message;
use tracing::{error, info};
use tracing_log::LogTracer;
use tracing_subscriber::{filter, Layer};
use tracing_subscriber::layer::SubscriberExt;

use crate::bd::{BdSettings, get_bd_path, install_plugin};
use crate::ds_installer::configure_open_asar;
use crate::license::{check_license, open_ds_invite};
use crate::web::WebServer;
use crate::ws::{DiscordConnection, DiscordStream, DiscordStreams, WebConnections, WebSocketServer};
use crate::ws::message::{CaptureEvent, MessageType};

mod ws;
mod web;
mod bd;
mod license;
mod ds_installer;

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
            bd_path: Some(get_bd_path().get(0).expect("Failed to get BD path").to_string()),
            web_port: DEFAULT_WEB_PORT,
        }
    }
}

impl Config {
    fn load() -> Self {
        info!("Loading config file from: {}", confy::get_configuration_file_path(NAME, None).unwrap().display());
        confy::load(NAME, None).unwrap_or_else(|_| {
            error!("Failed to parse config file, creating new one");
            std::fs::remove_file(confy::get_configuration_file_path(NAME, None).unwrap()).expect("Failed to remove config file");
            Config::default()
        })
    }
    fn save(&self) {
        confy::store(NAME, None, self).expect("Failed to save config");
    }
}

struct State {
    config: PLMutex<Config>,
    bd_settings: PLMutex<BdSettings>,
}

#[derive(serde::Deserialize)]
struct LinkEvent {
    source: Option<String>,
    target: String,
}

pub const DS_APP_ID: discord_sdk::AppId = 1093500259235274763;
pub const DS_INVITE: &str = "https://discord.com/invite/MehYjUJGpA";

#[tokio::main]
async fn main() {
    init_logging();
    info!("Configuring Open ASAR...");
    configure_open_asar().await;
    info!("Configured Open ASAR");

    let config = Config::load();

    let bd_settings = PLMutex::new(BdSettings::load(format!("{}/plugins/DiscordSourcePlugin.config.json", config.bd_path.as_ref().expect("bd_path isn't defined").clone())).await.expect("Failed to load BD settings"));

    let config = PLMutex::new(config);

    tauri::async_runtime::set(tokio::runtime::Handle::current());

    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let show = CustomMenuItem::new("show".to_string(), "Show");
    let tray_menu = SystemTrayMenu::new()
        .add_item(show)
        .add_item(quit);

    #[allow(clippy::single_match)]
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, argv, cwd| {
        }))
        .manage(State {
            config,
            bd_settings
        })
        .manage::<WebConnections>(Arc::new(RwLock::new(HashMap::new())))
        .manage::<DiscordStreams>(Arc::new(RwLock::new(HashMap::new())))
        .manage::<DiscordConnection>(Arc::new(RwLock::new(None)))
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
        .invoke_handler(tauri::generate_handler![bd::get_bd_path, bd::install_plugin, get_config, get_streams, get_targets, open_ds_invite, check_license])
        .setup(|app| {
            let discord_streams: tauri::State<'_, DiscordStreams> = app.state();
            let web_connections: tauri::State<'_, WebConnections> = app.state();
            let discord_connection: tauri::State<'_, DiscordConnection> = app.state();

            let discord_streams = Arc::clone(&discord_streams);
            let web_connections = Arc::clone(&web_connections);
            let discord_connection = Arc::clone(&discord_connection);

            let mut ws_server = WebSocketServer::new(discord_streams, web_connections.clone(), discord_connection.clone());
            let web_server = WebServer::new();

            let cfg: tauri::State<'_, State> = app.state();

            app.listen_global("link-stream", {
                let discord_connection = discord_connection.clone();
                let web_connections = web_connections.clone();
                move |event| {
                    let data: LinkEvent = serde_json::from_str(event.payload().unwrap()).unwrap();
                    info!("Link stream event: {:?}", event.payload());
                    let web_connections = web_connections.clone();
                    let discord_connection = discord_connection.clone();
                    tauri::async_runtime::spawn(async move {
                        let source = data.source.unwrap();
                        let _ = web_connections.write().await.get(&data.target).unwrap().linked_stream.write().insert(source.clone());
                        let discord_connection = discord_connection.read().await;
                        let discord_connection = discord_connection.as_ref().unwrap();
                        let _ = discord_connection.ws_sink.lock().await.send(Message::Text(serde_json::to_string(
                            &MessageType::Capture(CaptureEvent {
                                stream_id: source,
                            })
                        ).unwrap())).await;
                        info!("Sent capture event");
                    });
                }
            });

            app.listen_global("unlink-stream", move |event| {
                let data: LinkEvent = serde_json::from_str(event.payload().unwrap()).unwrap();
                info!("Unlink stream event: {:?}", event.payload());
                let web_connections = web_connections.clone();
                let discord_connection = discord_connection.clone();
                tauri::async_runtime::spawn(async move {
                    let web_connections = web_connections.write().await;
                    let Some(web_connection) = web_connections.get(&data.target) else {
                        return;
                    };
                    let _ = web_connection.ws_sink.lock().await.send(Message::Text(serde_json::to_string(
                        &MessageType::Unlink
                    ).unwrap())).await;
                    let Some(stream_id) = web_connection.linked_stream.write().take() else {
                        return;
                    };
                    let discord_connection = discord_connection.read().await;
                    let discord_connection = discord_connection.as_ref().unwrap();
                    let _ =  discord_connection.ws_sink.lock().await.send(Message::Text(serde_json::to_string(
                        &MessageType::EndCapture(CaptureEvent {
                            stream_id,
                        })
                    ).unwrap())).await;
                    info!("Sent end capture event");
                });
            });

            ws_server.set_window(app.get_window("main").unwrap());

            bind_servers(ws_server, web_server, cfg.bd_settings.lock().ws_port, cfg.config.lock().web_port);

            let path = cfg.config.lock().bd_path.as_ref().expect("bd_path isn't defined").clone();
            tauri::async_runtime::spawn(async move {
                install_plugin(format!("{}/plugins/DiscordSourcePlugin.plugin.js", path)).await;
            });

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app_handle, event| match event {
            RunEvent::WindowEvent { event, label, .. } => {
                if label == "main" {
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
async fn get_targets(web_connections: tauri::State<'_, WebConnections>) -> Result<HashMap<String, Option<String>>, ()> {
    let web_connections = web_connections.read().await;

    let tasks = web_connections
        .iter()
        .map(|(id, conn)| {
            let id = id.clone();
            let linked_stream = conn.linked_stream.clone();
            tokio::spawn(async move {
                (id, linked_stream.read().clone())
            })
        })
        .collect::<Vec<_>>();

    Ok(join_all(tasks).await
        .into_iter()
        .map(|res| res.unwrap())
        .collect::<HashMap<String, Option<String>>>()
    )
}

#[tauri::command]
async fn get_streams(discord_streams: tauri::State<'_, DiscordStreams>) -> Result<HashMap<String, DiscordStream>, ()> {
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

fn init_logging(){
    let log_dir = directories::BaseDirs::new().expect("Failed to get base dirs").config_local_dir().join(NAME);

    if !log_dir.exists() {
        std::fs::create_dir_all(&log_dir).expect("Failed to create log directory");
    }

    let file = std::fs::File::create(log_dir.join("log.txt")).unwrap();

    let file_log = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .with_writer(file)
        .with_filter(filter::LevelFilter::INFO);

    let stdout_log = tracing_subscriber::fmt::layer()
        .with_filter(filter::LevelFilter::INFO);

    let subscriber = tracing_subscriber::registry()
        .with(file_log)
        .with(stdout_log);

    info!("Logging initialized, logging to {} and stdout", log_dir.display());

    match tracing::subscriber::set_global_default(subscriber) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Failed to set global logging default subscriber: {}", e);
        }
    }

    LogTracer::init().unwrap();
}