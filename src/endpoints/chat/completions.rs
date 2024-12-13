use crate::dto::Request;
use crate::responders::EitherResponder;
use crate::services;
use crate::state::AppState;
use actix_web::{post, web};
use crate::services::keywords;

#[post("/completions")]
pub async fn completions(
    web::Json(request): web::Json<Request>,
    state: web::Data<AppState>,
) -> EitherResponder {
    match request {
        Request::Structured { prompt, keywords } => {
            let model = state.model.clone();
            let response = services::structured(prompt, keywords, model).await;
            EitherResponder::HttpResponse(response)
        }
        Request::Chat { prompt } => {
            let model = state.model.clone();
            let response = services::chat(prompt, model);
            EitherResponder::Sse(response)
        }
    }
}
