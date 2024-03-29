use ts_rs::TS;

#[derive(serde::Deserialize, serde::Serialize, Debug, TS, Clone)]
#[ts(export)]
#[serde(tag = "type", content = "detail")]
pub enum MessageType {
    #[serde(rename = "remove")]
    Remove(Vec<RemoveStreamEvent>),
    #[serde(rename = "ice")]
    ICE(ICEEvent),
    #[serde(rename = "answer")]
    Answer(AnswerOfferEvent),
    #[serde(rename = "offer")]
    Offer(AnswerOfferEvent),
    #[serde(rename = "capture")]
    Capture(CaptureEvent),
    #[serde(rename = "endCapture")]
    EndCapture(CaptureEvent),
    #[serde(rename = "unlink")]
    Unlink,
    #[serde(rename = "updateUserInfo")]
    UpdateUserInfo(Vec<UpdateUserInfoEvent>)
}

#[derive(serde::Deserialize, serde::Serialize, Debug, TS, Clone)]
#[ts(export)]
pub struct UserInfo {
    pub nickname: String,
    /// base64 encoded image
    #[serde(rename = "streamPreview")]
    pub stream_preview: String
}

#[derive(serde::Deserialize, serde::Serialize, Debug, TS, Clone)]
#[ts(export)]
pub struct UpdateUserInfoEvent {
    #[serde(rename = "streamId")]
    pub stream_id: String,
    #[serde(rename = "userId")]
    pub user_id: String,
    #[serde(rename = "info")]
    pub info: UserInfo
}

#[derive(serde::Deserialize, serde::Serialize, Debug, TS, Clone)]
#[ts(export)]
pub struct RemoveStreamEvent {
    #[serde(rename = "streamId")]
    pub stream_id: String
}

/// stream_id is optional since it's present only if the event is from discord
#[derive(serde::Deserialize, serde::Serialize, Debug, TS, Clone)]
#[ts(export)]
pub struct ICEEvent {
    #[serde(rename = "streamId")]
    #[ts(optional)]
    pub stream_id: Option<String>,
    pub candidate: String,
}

/// stream_id is optional since it's present only if the event is from discord
#[derive(serde::Deserialize, serde::Serialize, Debug, TS, Clone)]
#[ts(export)]
pub struct AnswerOfferEvent {
    #[serde(rename = "streamId")]
    #[ts(optional)]
    pub stream_id: Option<String>,
    pub sdp: String,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, TS, Clone)]
#[ts(export)]
pub struct CaptureEvent {
    #[serde(rename = "streamId")]
    pub stream_id: String,
}