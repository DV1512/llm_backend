use serde::{Deserialize, Serialize};

use crate::models::{Embedding, Entry, EntryType};

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Request {
    Structured { data: String },
    Chat { prompt: String },
}

#[derive(Serialize, Deserialize)]
pub struct EmbeddingsRequest {
    #[serde(rename = "type")]
    pub entry_type: EntryType,

    pub entries: Vec<Entry>,
}

#[derive(Serialize, Deserialize)]
pub struct EmbeddingsResponse {
    #[serde(rename = "type")]
    pub entry_type: EntryType,
    pub entries: Vec<Embedding>,
}

#[derive(Serialize, Deserialize)]
pub struct EmbeddingQuery {
    #[serde(rename = "type")]
    pub entry_type: EntryType,

    pub query: String,
    pub num_neighbors: u32,
}

#[derive(Serialize, Deserialize)]
pub struct SearchEmbeddingsRequest {
    #[serde(rename = "type")]
    pub entry_type: EntryType,

    pub embedding: Vec<f32>,
    pub num_neighbors: u32,
}
