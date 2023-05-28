use tracing::{error, info};

use crate::{DS_APP_ID, DS_INVITE};

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
#[tauri::command]
pub async fn check_license() -> Result<bool, ()> {
    let (wheel, handler) = discord_sdk::wheel::Wheel::new(Box::new(|err| {
        error!(error = ?err, "encountered an error");
    }));

    let mut user = wheel.user();

    let Ok(discord) = discord_sdk::Discord::new(discord_sdk::DiscordApp::PlainId(DS_APP_ID), discord_sdk::Subscriptions::USER, Box::new(handler)) else {
        error!("Unable to create discord client for license check");
        return Err(());
    };

    info!("waiting for handshake with discord client...");
    user.0.changed().await.unwrap();
    info!("handshake with discord client completed");

    let user_id = match &*user.0.borrow() {
        discord_sdk::wheel::UserState::Connected(user) => user.clone().id.0,
        discord_sdk::wheel::UserState::Disconnected(err) => {
            error!("failed to connect to Discord: {}", err);
            return Err(());
        },
    };

    let Ok(license) = reqwest::get(&format!("https://discord-source-license.dreamingcodes.workers.dev/v2/{}", user_id)).await else {
        error!("Failed to validate user_id");
        return Err(());
    };
    let Ok(license_text) = license.text().await else {
        error!("Failed to get license text");
        return Err(());
    };

    if license_text.eq("not_a_member") {
        //Timeout to prevent the task from hanging if the user doesn't accept the invite
        let _ = discord.open_guild_invite(DS_INVITE).await;
        info!("Opened invite");
    }

    discord.disconnect().await;
    info!("Disconnected from discord sdk");

    if license_text.eq("licensed") {
        info!("User is licensed");
        Ok(true)
    } else {
        info!("User is not licensed");
        Ok(false)
    }
}

#[tauri::command]
pub fn open_ds_invite() {
    let _ = open::that(DS_INVITE);
}