use crate::error::ServerError;
use kalosm::language::*;
use std::sync::Arc;

pub struct AppState {
    pub model: Arc<Llama>,
}

impl AppState {
    #[tracing::instrument]
    pub async fn new() -> Result<Self, ServerError> {
        let model = Llama::builder()
            .with_source(LlamaSource::llama_3_2_3b_chat())
            .build()
            .await?;

        Ok(Self {
            model: Arc::new(model),
        })
    }
}
