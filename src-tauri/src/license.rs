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
pub async fn check_license() {
    let (wheel, handler) = discord_sdk::wheel::Wheel::new(Box::new(|err| {
        error!(error = ?err, "encountered an error");
    }));

    let mut user = wheel.user();

    let discord = discord_sdk::Discord::new(discord_sdk::DiscordApp::PlainId(DS_APP_ID), discord_sdk::Subscriptions::USER, Box::new(handler))
        .expect("Unable to create discord client for license check");

    info!("waiting for handshake with discord client...");
    user.0.changed().await.unwrap();
    info!("handshake with discord client completed");

    discord.open_guild_invite(DS_INVITE).await.expect("Failed to open guild invite");

    let user_id = match &*user.0.borrow() {
        discord_sdk::wheel::UserState::Connected(user) => user.clone().id.0,
        discord_sdk::wheel::UserState::Disconnected(err) => panic!("failed to connect to Discord: {}", err),
    };

    let license = reqwest::get(&format!("https://discord-source-license.dreamingcodes.workers.dev/{}", user_id)).await.expect("Failed to validate user_id");

    if !license.text().await.expect("Failed to get license text").eq("true") {
        panic!("Invalid license, please contact the developer sending your user_id or login with the correct discord account: {}", user_id);
    }
}