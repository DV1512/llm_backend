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
    match request {
        Request::Structured { data } => {
            let model = state.model.clone();
            let response = services::structured(data, state.clone(), model).await;
            EitherResponder::HttpResponse(response)
        }
        Request::Chat { prompt } => {
            let model = state.model.clone();
            let response = services::chat(prompt, state.clone(), model);
            EitherResponder::Sse(response)
        }
    }
}
