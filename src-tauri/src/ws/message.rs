use ts_rs::TS;

#[derive(serde::Deserialize, serde::Serialize, Debug, TS)]
#[ts(export)]
#[serde(tag = "type", content = "data")]
pub enum MessageType {
    #[serde(rename = "add")]
    Add(AddStreamEvent),
    #[serde(rename = "remove")]
    Remove(RemoveStreamEvent),
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
}

#[derive(serde::Deserialize, serde::Serialize, Debug, TS)]
#[ts(export)]
pub struct AddStreamEvent {
    #[serde(rename = "streamId")]
    stream_id: u8,
    #[serde(rename = "userId")]
    user_id: String,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, TS)]
#[ts(export)]
pub struct RemoveStreamEvent {
    #[serde(rename = "streamId")]
    stream_id: u8
}

/// stream_id is optional since it's present only if the event is from discord
#[derive(serde::Deserialize, serde::Serialize, Debug, TS)]
#[ts(export)]
pub struct ICEEvent {
    #[serde(rename = "streamId")]
    #[ts(optional)]
    stream_id: Option<u8>,
    candidate: String,
}

/// stream_id is optional since it's present only if the event is from discord
#[derive(serde::Deserialize, serde::Serialize, Debug, TS)]
#[ts(export)]
pub struct AnswerOfferEvent {
    #[serde(rename = "streamId")]
    #[ts(optional)]
    stream_id: Option<u8>,
    sdp: String,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, TS)]
#[ts(export)]
pub struct CaptureEvent {
    #[serde(rename = "streamId")]
    stream_id: u8,
}