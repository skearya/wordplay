use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
#[serde(tag = "type")]
enum ClientMessage {
    General(ClientGeneral),
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum ClientGeneral {}
