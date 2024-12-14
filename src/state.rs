use crate::error::ServerError;
use kalosm::language::*;
use std::sync::Arc;

pub struct AppState {
    pub model: Arc<Llama>,
    pub embedding_model: Arc<Bert>,
}

impl AppState {
    #[tracing::instrument]
    pub async fn new() -> Result<Self, ServerError> {
        let model = Llama::builder()
            .with_source(LlamaSource::llama_3_2_3b_chat())
            .build()
            .await?;

        let embedding_model_source = BertSource::mini_lm_l6_v2();
        let embedding_model = Bert::builder()
            .with_source(embedding_model_source)
            .build()
            .await?;

        Ok(Self {
            model: Arc::new(model),
            embedding_model: Arc::new(embedding_model),
        })
    }
}
