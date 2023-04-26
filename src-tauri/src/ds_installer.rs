// Write a rust code to get the last release of https://github.com/GooseMod/OpenAsar and download the file app.asar

use std::collections::HashMap;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use lazy_static::lazy_static;
use sysinfo::{NetworkExt, NetworksExt, ProcessExt, System, SystemExt};
use tauri::async_runtime::set;
use tokio::fs;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::info;

#[cfg(target_os = "windows")]
const DISCORD_EXE_NAMES: [&str; 3] = [
    "discord.exe",
    "discordcanary.exe",
    "discordptb.exe"
];

#[cfg(not(target_os = "windows"))]
const DISCORD_EXE_NAMES: [&str; 3] = [
    "discord",
    "discord-canary",
    "discord-ptb"
];

#[cfg(target_os = "windows")]
const DISCORD_ASAR_PATH: [&str; 3] = [
    "%localappdata%\\Discord\\*\\resources\\app.asar",
    "%localappdata%\\DiscordPTB\\*\\resources\\app.asar",
    "%localappdata%\\DiscordCanary\\*\\resources\\app.asar"
];

#[cfg(target_os = "macos")]
const DISCORD_ASAR_PATH: [&str; 3] = [
    "/Applications/Discord.app/Contents/Resources/app.asar",
    "/Applications/Discord Canary.app/Contents/Resources/app.asar",
    "/Applications/Discord PTB.app/Contents/Resources/app.asar"
];

#[cfg(target_os = "linux")]
const DISCORD_ASAR_PATH: [&str; 18] = [
    //Discord stable
    "/opt/discord/resources/app.asar",
    "/usr/lib/discord/resources/app.asar",
    "/usr/lib64/discord/resources/app.asar",
    "/usr/share/discord/resources/app.asar",
    "/var/lib/flatpak/app/com.discordapp.Discord/current/active/files/discord/resources/app.asar",
    "~/.local/share/flatpak/app/com.discordapp.Discord/current/active/files/discord/resources/app.asar",

    //Discord PTB
    "/opt/discord-ptb/resources/app.asar",
    "/usr/lib/discord-ptb/resources/app.asar",
    "/usr/lib64/discord-ptb/resources/app.asar",
    "/usr/share/discord-ptb/resources/app.asar",
    "/var/lib/flatpak/app/com.discordapp.DiscordPtb/current/active/files/discord-ptb/resources/app.asar",
    "~/.local/share/flatpak/app/com.discordapp.DiscordPtb/current/active/files/discordPtb/resources/app.asar",

    //Discord Canary
    "/opt/discord-canary/resources/app.asar",
    "/usr/lib/discord-canary/resources/app.asar",
    "/usr/lib64/discord-canary/resources/app.asar",
    "/usr/share/discord-canary/resources/app.asar",
    "/var/lib/flatpak/app/com.discordapp.DiscordCanary/current/active/files/discord-canary/resources/app.asar",
    "~/.local/share/flatpak/app/com.discordapp.DiscordCanary/current/active/files/discordCanary/resources/app.asar"
];

//TODO: Check if those are correct paths
#[cfg(target_os = "windows")]
const DISCORD_SETTINGS_PATH: [&str; 3] = [
    "%localappdata%\\Discord\\settings.json",
    "%localappdata%\\DiscordPTB\\settings.json",
    "%localappdata%\\DiscordCanary\\settings.json"
];

//TODO: Check if those are correct paths
#[cfg(target_os = "macos")]
const DISCORD_SETTINGS_PATH: [&str; 3] = [
    "~/Library/Application Support/discord/settings.json",
    "~/Library/Application Support/discordptb/settings.json",
    "~/Library/Application Support/discordcanary/settings.json"
];

//TODO: Add the missing paths
#[cfg(target_os = "linux")]
const DISCORD_SETTINGS_PATH: [&str; 3] = [
    "~/.config/discord/settings.json",
    "~/.config/discordptb/settings.json",
    "~/.config/discordcanary/settings.json",
];

const DISCORD_OPEN_ASAR_URL: &str = "https://github.com/GooseMod/OpenAsar/releases/download/nightly/app.asar";

lazy_static!(
    static ref SYS: System = System::new_all();
);

fn is_discord(cmdline: &str) -> bool {
    DISCORD_EXE_NAMES.iter().any(|exe_name| !cmdline.to_lowercase().contains("discord-source") && cmdline.to_lowercase().contains(exe_name))
}

fn kill_discord() {
    info!("Killing Discord");

    let processes = SYS.processes();

    for process in processes.iter() {
        let Some(cmd) = process.1.cmd().get(0) else {
            continue;
        };
        if is_discord(cmd) {
            process.1.kill();
        }
    }
}

//Steps:
//1. Kill Discord
//2. Find existing Discord installation using DISCORD_ASAR_PATH (it is important to use a crate to resolve ~ and %localappdata% and handle * for windows)
//3. Download OpenAsar from DISCORD_OPEN_ASAR_URL to all found Discord installations
//4. Start Discord
async fn install_open_asar() {
    kill_discord();

    let mut asar_paths = Vec::new();

    for path in DISCORD_ASAR_PATH.iter() {
        let inner_paths = path.split('*').collect::<Vec<&str>>();

        if inner_paths.len() == 1 {
            let Ok(path) = shellexpand::full(path) else {
                continue;
            };

            if !Path::new(&path.to_string()).exists() {
                continue;
            }

            asar_paths.push(path.to_string());
        } else {
            //Resolve the first part of the path then for every folder in the resolved path, append the second part of the path and if it exists, add it to the list
            let Ok(first_part) = shellexpand::full(inner_paths[0]) else {
                continue;
            };
            let second_part = inner_paths[1];

            let path = format!("{}/{}", first_part, second_part);

            if Path::new(&path).exists() {
                asar_paths.push(path);
            }
        }
    }

    let response = reqwest::get(DISCORD_OPEN_ASAR_URL).await.expect("Error downloading OpenAsar");
    let bytes = response.bytes().await.expect("Error reading response");

    //Discord resourced folder perms need
    // to be changed to 777 on Linux and Mac for OpenAsar to be able to auto-update
    //TODO: Find a way to get root privileges on Mac as well
    #[cfg(target_os = "linux")]
    {
        info!("Changing Discord resources folder permissions");
        tokio::process::Command::new("pkexec")
            .arg("chmod")
            .arg("-R")
            .arg("777")
            .args(asar_paths.iter().map(|path| path.replace("app.asar", "")))
            .spawn().expect("Error changing Discord resources folder permissions")
            .wait().await.expect("Error changing Discord resources folder permissions");
        info!("Changed Discord resources folder permissions");
    }

    for path in asar_paths.iter() {
        info!("Installing OpenAsar to {}", path);
        let mut file = File::create(path).await.unwrap_or_else(|error| panic!("Error creating file {} with error {}", path, error));
        file.write_all(bytes.as_ref()).await.unwrap_or_else(|error| panic!("Error writing to file {} with error {}", path, error));
    }
}

#[derive(serde::Serialize, serde::Deserialize, PartialEq)]
enum OpenasarPerformanceMode {
    #[serde(rename = "perf")]
    Performance,
    #[serde(rename = "balanced")]
    Balanced,
    #[serde(rename = "battery")]
    Battery,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Openasar {
    #[serde(rename = "customFlags")]
    pub custom_flags: Option<String>,
    #[serde(rename = "cmdPreset")]
    cmd_preset: Option<OpenasarPerformanceMode>,
    #[serde(flatten)]
    other: serde_json::Value,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct DiscordSettings {
    #[serde(rename = "DANGEROUS_ENABLE_DEVTOOLS_ONLY_ENABLE_IF_YOU_KNOW_WHAT_YOURE_DOING")]
    pub dangerous_enable_devtools_only_enable_if_you_know_what_youre_doing: Option<bool>,
    pub openasar: Option<Openasar>,
    #[serde(flatten)]
    other: serde_json::Value,
}

pub async fn configure_open_asar() {
    let mut installation_run = false;
    let mut should_kill_discord = false;

    for path in DISCORD_SETTINGS_PATH.iter() {
        let Ok(path) = shellexpand::full(path) else {
            continue;
        };

        let Ok(settings) = fs::read_to_string(Path::new(&path.to_string())).await else {
            continue;
        };

        let Ok(mut settings) = serde_json::from_str::<DiscordSettings>(&settings) else {
            continue;
        };

        info!("Configuring OpenAsar for {}", path);

        settings.dangerous_enable_devtools_only_enable_if_you_know_what_youre_doing = Some(true);

        let openasar = {
            if let Some(openasar) = settings.openasar.as_mut() {
                openasar
            } else {
                if !installation_run {
                    installation_run = true;
                    install_open_asar().await;
                }
                settings.openasar = Some(Openasar {
                    custom_flags: None,
                    cmd_preset: None,
                    other: serde_json::Value::Null,
                });
                settings.openasar.as_mut().unwrap()
            }
        };

        let custom_flags = {
            if let Some(custom_flags) = openasar.custom_flags.as_mut() {
                custom_flags
            } else {
                openasar.custom_flags = Some(String::new());
                openasar.custom_flags.as_mut().unwrap()
            }
        };

        if !custom_flags.contains("--use-gl=desktop") {
            info!("Setting OpenAsar to use desktop OpenGL for {}", path);
            custom_flags.push_str(" --use-gl=desktop");
            should_kill_discord = true;
        }

        if openasar.cmd_preset != Some(OpenasarPerformanceMode::Performance) {
            info!("Setting OpenAsar to performance mode for {}", path);
            openasar.cmd_preset = Some(OpenasarPerformanceMode::Performance);
            should_kill_discord = true;
        }

        let settings = serde_json::to_string_pretty(&settings).expect("Error serializing settings");

        fs::write(Path::new(&path.to_string()), settings).await.expect("Error writing settings");

        if should_kill_discord {
            kill_discord();
        }
    }
}