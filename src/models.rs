use std::ops::{Deref, DerefMut};
use ulid::Ulid;
use serde_json::Value;
use serde::{Deserialize, Serialize};
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MitreMitigation {
    #[serde(rename = "ID")]
    pub(crate) id: String,
    name: String,
    description: String,
    url: String
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MitreMitigations(pub Vec<MitreMitigation>);

impl Deref for MitreMitigations {
    type Target = Vec<MitreMitigation>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for MitreMitigations {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl MitreMitigations {
    pub fn format_mitigations(&self) -> String {
        self
            .iter()
            .map(|mitigation| {
                format!(
                    "-name: {}, description: {}\n  url: {}",
                    mitigation.name.as_str(),
                    mitigation.description.as_str(),
                    mitigation.url.as_str()
                )
            }).collect::<Vec<_>>().join("\n")
    }
}