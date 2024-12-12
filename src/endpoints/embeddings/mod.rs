use crate::state::AppState;
use actix_web::dev::HttpServiceFactory;
use actix_web::error::ErrorInternalServerError;
use actix_web::{post, web, HttpResponse, Responder};
use kalosm::language::EmbedderExt;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct EmbeddingsRequest {
    data: Vec<String>,
}

#[post("")]
async fn embeddings(
    web::Json(req): web::Json<EmbeddingsRequest>,
    state: web::Data<AppState>,
) -> Result<impl Responder, actix_web::Error> {
    let embeddings_result = state.embedding_model.embed_batch(req.data).await;
    match embeddings_result {
        Ok(embeddings) => {
            let embedding_vecs: Vec<Vec<f32>> = embeddings.iter().map(|emb| emb.to_vec()).collect();
            Ok(HttpResponse::Ok().json(embedding_vecs))
        }
        Err(err) => Err(ErrorInternalServerError(err.to_string())),
    }
}

pub fn embeddings_service() -> impl HttpServiceFactory {
    web::scope("/embeddings").service(embeddings)
}
