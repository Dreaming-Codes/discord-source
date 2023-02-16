#![cfg_attr(
all(not(debug_assertions), target_os = "windows"),
windows_subsystem = "windows"
)]

use std::collections::HashMap;

use tracing::info;
use tracing_subscriber;

mod ws;
mod web;

// Learn more about Tauri commands at https://tauri.app/v1/cguides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn main() {
    const WS_PORT: u16 = 1111;
    const WEB_PORT: u16 = 8080;

    tracing_subscriber::fmt::init();

    let mut webENV = HashMap::new();
    webENV.insert("wsPort".to_string(), WS_PORT.to_string());

    let mut ws_server = ws::WebSocketServer::new();
    let mut web_server = web::WebServer::new(webENV);

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet])
        .setup(|app| {
            tauri::async_runtime::spawn(async move {
                ws_server.bind(format!("0.0.0.0:{}", WS_PORT)).await.expect("Failed to bind WS server");
                ws_server.run().await;
            });
            tauri::async_runtime::spawn(async move {
                web_server.build_html().await;
                web_server.bind(format!("0.0.0.0:{}", WEB_PORT)).await.expect("Failed to bind Web server");
                web_server.run().await;
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
