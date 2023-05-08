use directories::BaseDirs;
use tokio::fs::read_to_string;
use tokio::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{error, info};

use crate::DEFAULT_WS_PORT;
use crate::ds_installer::kill_discord;

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
pub async fn install_plugin(path: String) -> bool {
    info!("Installing plugin {}", path);

    let file = tokio::fs::read_to_string(path.clone()).await.unwrap_or_default();

    let installed_version_md5 = md5::compute(file);
    let latest_version_md5 = md5::compute(PLUGIN);

    if installed_version_md5 == latest_version_md5 {
        info!("Plugin is up to date");
        return true;
    }

    info!("Plugin is out of date, updating, old md5: {:?}, new md5: {:?}", installed_version_md5, latest_version_md5);

    if let Err(e) = tokio::fs::write(path, PLUGIN).await {
        error!("Failed to write plugin file: {}", e);
        return false;
    }

    kill_discord();

    info!("Plugin updated");

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