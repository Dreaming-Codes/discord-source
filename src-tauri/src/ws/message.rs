use ts_rs::TS;

#[derive(serde::Deserialize, serde::Serialize, Debug, TS)]
#[ts(export)]
#[serde(tag = "type", content = "detail")]
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
    pub stream_id: u8,
    #[serde(rename = "userId")]
    pub user_id: String,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, TS)]
#[ts(export)]
pub struct RemoveStreamEvent {
    #[serde(rename = "streamId")]
    pub stream_id: u8
}

/// stream_id is optional since it's present only if the event is from discord
#[derive(serde::Deserialize, serde::Serialize, Debug, TS)]
#[ts(export)]
pub struct ICEEvent {
    #[serde(rename = "streamId")]
    #[ts(optional)]
    pub stream_id: Option<u8>,
    pub candidate: String,
}

/// stream_id is optional since it's present only if the event is from discord
#[derive(serde::Deserialize, serde::Serialize, Debug, TS)]
#[ts(export)]
pub struct AnswerOfferEvent {
    #[serde(rename = "streamId")]
    #[ts(optional)]
    pub stream_id: Option<u8>,
    pub sdp: String,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, TS)]
#[ts(export)]
pub struct CaptureEvent {
    #[serde(rename = "streamId")]
    pub stream_id: u8,
}