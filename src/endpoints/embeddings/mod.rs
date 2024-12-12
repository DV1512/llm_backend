use crate::state::AppState;
use actix_web::{
    dev::HttpServiceFactory, error::ErrorInternalServerError, post, web, HttpResponse, Responder,
};
use kalosm::language::EmbedderExt;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Entry {
    pub mitre_id: String,
    pub mitre_name: String,
    pub mitre_description: String,
    pub mitre_url: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EntryType {
    Threat,
    Mitigation,
}

impl From<EntryType> for String {
    fn from(t: EntryType) -> Self {
        let s = match t {
            EntryType::Threat => "threat",
            EntryType::Mitigation => "mitigation",
        };
        String::from(s)
    }
}

#[derive(Serialize, Deserialize)]
struct EmbeddingsRequest {
    #[serde(rename = "type")]
    entry_type: EntryType,
    entries: Vec<Entry>,
}

#[derive(Serialize, Deserialize)]
struct Embedding {
    #[serde(flatten)]
    entry: Entry,

    embedding: Vec<f32>,
}

#[derive(Serialize, Deserialize)]
struct EmbeddingsResponse {
    #[serde(rename = "type")]
    entry_type: EntryType,
    entries: Vec<Embedding>,
}

#[post("")]
async fn embeddings(
    web::Json(req): web::Json<EmbeddingsRequest>,
    state: web::Data<AppState>,
) -> Result<impl Responder, actix_web::Error> {
    let inputs: Vec<_> = req
        .entries
        .iter()
        .map(|entry| &entry.mitre_description)
        .collect();

    let embeddings_result = state.embedding_model.embed_batch(inputs).await;
    match embeddings_result {
        Ok(embeddings) => {
            let embedding_vecs: Vec<Vec<f32>> = embeddings.iter().map(|emb| emb.to_vec()).collect();

            let embeddings: Vec<Embedding> = req
                .entries
                .iter()
                .zip(embeddings)
                .map(|(entry, embedding)| Embedding {
                    entry: entry.clone(),
                    embedding: embedding.to_vec(),
                })
                .collect();

            let request_body = EmbeddingsResponse {
                entries: embeddings,
                entry_type: req.entry_type,
            };

            let client = reqwest::Client::new();
            if let Err(err) = client
                .post("http://localhost:9999/api/v1/embeddings")
                .json(&request_body)
                .send()
                .await
            {
                return Err(ErrorInternalServerError(err.to_string()));
            }

            Ok(HttpResponse::Ok().json(embedding_vecs))
        }
        Err(err) => Err(ErrorInternalServerError(err.to_string())),
    }
}

pub fn embeddings_service() -> impl HttpServiceFactory {
    web::scope("/embeddings").service(embeddings)
}
