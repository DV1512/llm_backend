use crate::error::ServerError;
use kalosm::language::*;
use std::collections::HashMap;
use std::sync::Arc;

pub struct AppState {
    pub model: Arc<Llama>,
    pub data: HashMap<String, String>,
}

impl AppState {
    #[tracing::instrument]
    pub async fn new() -> Result<Self, ServerError> {
        let model = Llama::builder()
            .with_source(LlamaSource::llama_3_1_8b_chat())
            .build()
            .await?;

        let mut data = HashMap::new();
        let data_files = [(
            "mitre_mitigations",
            include_str!("../filter_mitigations.json"),
        )];

        for (key, file_data) in data_files {
            data.insert(key.to_string(), file_data.to_string());
        }

        let app_state = AppState {
            model: Arc::new(model),
            data,
        };
        Ok(app_state)
    }

    pub fn get_data(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }
}
