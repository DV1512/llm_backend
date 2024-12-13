use crate::error::ServerError;
use kalosm::language::*;
use std::sync::Arc;

pub const MITRE_MITIGATIONS: &str = include_str!("../filter_mitigations.json");

pub struct AppState {
    pub model: Arc<Llama>,
}

impl AppState {
    #[tracing::instrument]
    pub async fn new() -> Result<Self, ServerError> {
        let model = Llama::builder()
            .with_source(LlamaSource::llama_3_1_8b_chat())
            .build()
            .await?;

        let app_state = AppState {
            model: Arc::new(model),
        };
        Ok(app_state)
    }
}
