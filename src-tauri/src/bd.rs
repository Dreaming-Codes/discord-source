use directories::BaseDirs;
use tokio::fs::read_to_string;
use tokio::io;
use tokio::io::AsyncWriteExt;
use tracing::{error, info};

use crate::DEFAULT_WS_PORT;

const PLUGIN: &str = include_str!("../../dist-bd/DiscordSourcePlugin.plugin.js");

#[derive(serde::Deserialize, serde::Serialize)]
pub struct BdSettings {
    #[serde(rename = "wsPort")]
    pub ws_port: u16,
}

impl BdSettings {
    pub fn new(port: u16) -> Self {
        Self {
            ws_port: port
        }
    }

    pub async fn load(path: String) -> anyhow::Result<Self> {
        let settings = read_to_string(path.clone()).await;

        if let Ok(settings) = settings {
            Ok(serde_json::from_str(&settings)?)
        } else {
            info!("Failed to load BetterDiscord settings, creating new settings file at {}", path);
            let settings = Self::new(DEFAULT_WS_PORT);
            settings.save(path).await?;
            Ok(settings)
        }
    }

    pub async fn save(&self, path: String) -> io::Result<()> {
        let settings = serde_json::to_string(self).unwrap();
        tokio::fs::write(path, settings).await
    }
}

#[tauri::command]
pub async fn restart_plugin(path: String) -> bool {
    info!("Installing plugin {}", path);

    //Removing the plugin file to force a reload (if it exists)
    if let Ok(..) = tokio::fs::remove_file(path.clone()).await {
        info!("Removed plugin {}", path);
        //Sleep for 5 seconds to allow BetterDiscord to unload the plugin
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }

    if let Ok(mut file) = tokio::fs::OpenOptions::new().write(true).create(true).open(path).await {
        if let Err(e) = file.write_all(PLUGIN.as_bytes()).await {
            error!("Failed to write plugin: {}", e);
            return false;
        }
    }

    true
}

#[tauri::command]
pub fn get_bd_path() -> Vec<String> {
    let mut paths = vec![];
    if let Some(base_dirs) = BaseDirs::new() {
        let path = base_dirs.config_dir().join("BetterDiscord");
        if path.exists() {
            paths.push(path.to_str().unwrap().to_string());
        }
    }

    paths
}