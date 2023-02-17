#![cfg_attr(
all(not(debug_assertions), target_os = "windows"),
windows_subsystem = "windows"
)]

use tauri::Manager;
use crate::web::WebServer;
use crate::ws::WebSocketServer;

mod ws;
mod web;

// Learn more about Tauri commands at https://tauri.app/v1/cguides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn main() {
    tracing_subscriber::fmt::init();

    let mut ws_server = WebSocketServer::new();
    let mut web_server = WebServer::new();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet])
        .setup(|app| {
            let id = app.listen_global("event-name", |event| {
                println!("got event-name with payload {:?}", event.payload());
            });
            bind_servers(ws_server, web_server, 1111, 8080);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn bind_servers(mut ws_server: WebSocketServer, mut web_server: WebServer, ws_port: u16, web_port: u16) {
    tauri::async_runtime::spawn(async move {
        ws_server.bind(ws_port.clone()).await.expect("Failed to bind WS server");
        ws_server.run().await;
    });
    tauri::async_runtime::spawn(async move {
        web_server.bind(web_port, ws_port).await.expect("Failed to bind Web server");
        web_server.run().await;
    });
}