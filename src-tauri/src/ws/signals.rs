#[derive(serde::Deserialize, serde::Serialize, Debug)]
#[serde(tag = "type", content = "data")]
pub enum SignalType {
    #[serde(rename = "add")]
    Add,
    #[serde(rename = "remove")]
    Remove,
    #[serde(rename = "ice")]
    ICE,
    #[serde(rename = "answer")]
    Answer,
    #[serde(rename = "offer")]
    Offer,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct AddStreamEvent {
    #[serde(rename = "streamId")]
    stream_id: String,
    #[serde(rename = "userId")]
    user_id: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct RemoveStreamEvent {
    #[serde(rename = "streamId")]
    stream_id: String
}

/// stream_id is optional since it's present only if the event is from discord
#[derive(serde::Deserialize, serde::Serialize)]
pub struct ICEEvent {
    #[serde(rename = "streamId")]
    stream_id: Option<String>,
    candidate: String,
}

/// stream_id is optional since it's present only if the event is from discord
#[derive(serde::Deserialize, serde::Serialize)]
pub struct AnswerOfferEvent {
    #[serde(rename = "streamId")]
    stream_id: Option<String>,
    sdp: String,
}