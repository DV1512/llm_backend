use ulid::Ulid;
use serde_json::Value;
use serde::Serialize;
use serde::de::DeserializeOwned;

#[derive(Serialize)]
#[allow(dead_code)]
pub enum ChatRole {
    System,
    User,
    Assistant,
}

#[derive(Serialize)]
pub struct ChatMessageChunk {
    id: Ulid,
    timestamp: u64,
    role: ChatRole,
    contents: Value,
}

impl ChatMessageChunk {
    pub fn new(id: Ulid, timestamp: u64, role: ChatRole, contents: Value) -> Self {
        Self {
            id,
            timestamp,
            role,
            contents,
        }
    }

    pub fn new_serialized(
        id: Ulid,
        timestamp: u64,
        role: ChatRole,
        contents: impl Serialize + DeserializeOwned,
    ) -> Self {
        let content_str =
            serde_json::to_string(&contents).expect("Unable to serialize the content");
        let value = serde_json::from_str(&content_str).expect("Unable to Deserialize the string");

        Self::new(id, timestamp, role, value)
    }
}