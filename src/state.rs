use crate::error::ServerError;
use crate::models::{MitreMitigation, MitreMitigations};
use kalosm::language::*;
use once_cell::sync::Lazy;
use std::sync::Arc;

pub const MITRE_MITIGATIONS_JSON: &str = include_str!("../filter_mitigations.json");
pub static MITRE_MITIGATIONS: Lazy<Arc<MitreMitigations>> = Lazy::new(|| {
    let mitigations: Vec<MitreMitigation> =
        serde_json::from_str(MITRE_MITIGATIONS_JSON).expect("Failed to parse mitigations");
    Arc::new(MitreMitigations(mitigations))
});

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
