use crate::error::ServerError;
use kalosm::language::*;
use std::collections::HashMap;
use std::sync::Arc;
//
use std::fs::File;
use std::io::{self, Read};

pub struct AppState {
    pub model: Arc<Llama>,
    pub data: HashMap<String, String>,
}

fn json_to_string(path: &str) -> Result<String, io::Error> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

impl AppState {
    #[tracing::instrument]
    pub async fn new() -> Result<Self, ServerError> {
        let model = Llama::builder()
            .with_source(LlamaSource::llama_3_1_8b_chat())
            .build()
            .await?;

        // Load JSON data during initialization
        let mut data = HashMap::new();
        let data_files = [("mitre_mitigations", "filter_mitigations.json")];

        for (key, path) in data_files {
            let file_data = json_to_string(path).map_err(|e| {
                anyhow::Error::new(e).context(format!("Failed to load JSON file: {}", path))
            })?;
            data.insert(key.to_string(), file_data);
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
