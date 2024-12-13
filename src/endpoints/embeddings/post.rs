use crate::{
    dto::{EmbeddingQuery, EmbeddingsRequest, EmbeddingsResponse, SearchEmbeddingsRequest},
    models::Embedding,
    services::{compute_single_embedding, find_closest_embeddings},
    state::AppState,
};

use actix_web::{
    error::ErrorInternalServerError,
    post,
    web::{self},
    HttpResponse, Responder,
};
use kalosm::language::EmbedderExt;

const MAIN_BACKEND_ADD_EMBEDDINGS_URL: &str = "http://localhost:9999/api/v1/embeddings";

#[post("")]
pub async fn add_embeddings(
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
pub async fn search_embeddings(
    web::Json(req): web::Json<EmbeddingQuery>,
    state: web::Data<AppState>,
) -> Result<impl Responder, actix_web::Error> {
    let embedding_model = state.embedding_model.clone();
    let embedding = compute_single_embedding(req.query, embedding_model).await?;
    let neighbors = find_closest_embeddings(SearchEmbeddingsRequest {
        embedding,
        num_neighbors: req.num_neighbors,
        entry_type: req.entry_type,
    })
    .await?;
    Ok(HttpResponse::Ok().json(neighbors))
}
