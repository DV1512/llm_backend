use kalosm::language::{Chat, Llama, LlamaSource};
use serde::{Deserialize, Serialize};
use std::fs;
use std::sync::Mutex;

#[derive(Serialize, Deserialize, Debug)]
pub struct ThreatActor {
    pub ID: String,
    pub name: String,
    pub description: String,
    pub url: String,
    pub associated_names: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Prompt {
    pub name: String,
    pub prompt_template: String,
}

pub struct AppState {
    pub chat: Mutex<Chat>,
    pub threat_groups: Vec<ThreatActor>,
    pub prompts: Vec<Prompt>,
}

fn load_prompts() -> Vec<Prompt> {
    let file_data = fs::read_to_string("prompt.json").expect("unable to read prompt.json");
    serde_json::from_str(&file_data).expect("unable to parse prompt.json")
}

impl AppState {
    pub async fn new() -> Self {
        let model = Llama::builder()
            .with_source(LlamaSource::llama_3_1_8b_chat())
            .build()
            .await
            .unwrap();

        let json_data = fs::read_to_string("data.json").expect("Failed to read");
        let threat_groups: Vec<ThreatActor> =
            serde_json::from_str(&json_data).expect("JSON failed");

        let prompts = load_prompts();

        Self {
            chat: Mutex::new(Chat::builder(model).build()),
            threat_groups,
            prompts,
        }
    }
}
