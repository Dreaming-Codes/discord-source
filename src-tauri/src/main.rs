#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use tracing::info;
use tracing_subscriber;

mod ws;

// Learn more about Tauri commands at https://tauri.app/v1/cguides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn main() {
    tracing_subscriber::fmt::init();

    let mut server = ws::WebSocketServer::new();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet])
        .setup(|app| {
            tauri::async_runtime::spawn(async move{
                server.bind("0.0.0.0:1111").await.expect("Failed to bind WS server");
                server.run().await;
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
