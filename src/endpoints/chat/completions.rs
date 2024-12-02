use crate::state::AppState;
use actix_web::{post, web};
use crate::dto::Request;
use crate::responders::EitherResponder;
use crate::services;

#[post("/completions")]
pub async fn completions(
    web::Json(request): web::Json<Request>,
    state: web::Data<AppState>,
) -> EitherResponder {
    let model = state.into_inner().model.clone();

    match request {
        Request::Structured { data } => {
            let response = services::structured(data, model).await;
            EitherResponder::HttpResponse(response)
        }
        Request::Chat { prompt } => {
            let response = services::chat(prompt, model);
            EitherResponder::Sse(response)
        }
    }
}
