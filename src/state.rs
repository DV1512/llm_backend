use kalosm::language::{Chat, Llama, LlamaSource};
use std::sync::Mutex;

pub struct AppState {
    pub chat: Mutex<Chat>,
}

impl AppState {
    pub async fn new() -> Self {
        let model = Llama::builder()
            .with_source(LlamaSource::llama_3_1_8b_chat())
            .build()
            .await
            .unwrap();
        Self {
            chat: Mutex::new(Chat::builder(model).build()),
        }
    }
}
