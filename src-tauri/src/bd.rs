use directories::BaseDirs;
use tokio::fs::read_to_string;
use tokio::io;
use tracing::info;

use crate::DEFAULT_WS_PORT;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct BdSettings {
    #[serde(rename = "wsPort")]
    ws_port: u16
}

impl BdSettings {
    pub fn new(port: u16) -> Self {
        Self {
            ws_port: port
        }
    }

    pub async fn load(path: String) -> serde_json::Result<Self> {
        let settings = read_to_string(path.clone()).await;

        if let Ok(settings) = settings {
            serde_json::from_str(&settings)
        }else {
            Ok(Self::new(DEFAULT_WS_PORT))
        }
    }

    pub async fn save(&self, path: String) -> io::Result<()> {
        let settings = serde_json::to_string(self).unwrap();
        tokio::fs::write(path, settings).await
    }
}

#[tauri::command]
pub fn install_plugin(path: String) -> bool {
    info!("Installing plugin {}", path);
    false
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