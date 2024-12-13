use crate::state::AppState;
use actix_web::{
    dev::HttpServiceFactory, error::ErrorInternalServerError, post, web, HttpResponse, Responder,
};
use kalosm::language::{Embedder, EmbedderExt};
use serde::{Deserialize, Serialize};

const MAIN_BACKEND_ADD_EMBEDDINGS_URL: &str = "http://localhost:9999/api/v1/embeddings";
const MAIN_BACKEND_SEARCH_EMBEDDINGS_URL: &str = "http://localhost:9999/api/v1/embeddings/search";

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

#[derive(Serialize, Deserialize)]
struct EmbeddingQuery {
    #[serde(rename = "type")]
    entry_type: EntryType,

    query: String,
    num_neighbors: u32,
}

#[derive(Serialize, Deserialize)]
struct SearchEmbeddingsRequest {
    #[serde(rename = "type")]
    entry_type: EntryType,

    embedding: Vec<f32>,
    num_neighbors: u32,
}

#[post("")]
async fn add_embeddings(
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
                .post(MAIN_BACKEND_ADD_EMBEDDINGS_URL)
                .json(&request_body)
                .send()
                .await
            {
                return Err(ErrorInternalServerError(err.to_string()));
            }

            Ok(HttpResponse::Created())
        }
        Err(err) => Err(ErrorInternalServerError(err.to_string())),
    }
}

#[post("/search")]
async fn search_embeddings(
    web::Json(req): web::Json<EmbeddingQuery>,
    state: web::Data<AppState>,
) -> Result<impl Responder, actix_web::Error> {
    let Ok(embedding) = state.embedding_model.embed_string(req.query).await else {
        return Err(ErrorInternalServerError(
            "Error computing query embedding".to_string(),
        ));
    };
    let embedding_vec = embedding.to_vec();

    let client = reqwest::Client::new();
    let req_body = SearchEmbeddingsRequest {
        entry_type: req.entry_type,
        embedding: embedding_vec,
        num_neighbors: req.num_neighbors,
    };
    let Ok(res) = client
        .post(MAIN_BACKEND_SEARCH_EMBEDDINGS_URL)
        .json(&req_body)
        .send()
        .await
    else {
        return Err(ErrorInternalServerError(
            "Error connecting to main backend".to_string(),
        ));
    };

    let Ok(res_body) = res.text().await else {
        return Err(ErrorInternalServerError(
            "Error reading body from request to main backend".to_string(),
        ));
    };

    Ok(HttpResponse::Ok().body(res_body))
}

pub fn embeddings_service() -> impl HttpServiceFactory {
    web::scope("/embeddings")
        .service(add_embeddings)
        .service(search_embeddings)
}
