use crate::dto::Request;
use crate::responders::EitherResponder;
use crate::services;
use crate::state::AppState;
use actix_web::{post, web};

#[post("/completions")]
pub async fn completions(
    web::Json(request): web::Json<Request>,
    state: web::Data<AppState>,
) -> EitherResponder {
    let model = state.model.clone();
    match request {
        Request::Structured { prompt, keywords } => {
            let response = services::structured(prompt, keywords, model).await;
            EitherResponder::HttpResponse(response)
        }
        Request::Chat { prompt } => {
            let response = services::chat(prompt, model);
            EitherResponder::Sse(response)
        }
    }
}
