use actix_web::{middleware::Logger, post, web, App, HttpServer, Responder};
use actix_web_lab::sse;
use env_logger::Env;
use kalosm::language::{Chat, Llama, LlamaSource};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::sync::Mutex;
use tokio_stream::StreamExt;
use ulid::Ulid;

#[derive(Serialize, Deserialize)]
struct ChatRequest {
    prompt: String,
}

#[post("/stream")]
async fn stream_endpoint(
    web::Json(request_data): web::Json<ChatRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let stream = state.chat.lock().unwrap().add_message(request_data.prompt);
    let ulid = Ulid::new().to_string();
    let sse_stream = stream.map(move |item| -> Result<sse::Event, _> {
        let data = sse::Data::new(item).id(ulid.clone());
        Ok::<_, Infallible>(sse::Event::Data(data))
    });
    sse::Sse::from_stream(sse_stream)
}

struct AppState {
    chat: Mutex<Chat>,
}

impl AppState {
    async fn new() -> Self {
        let model = Llama::builder()
            .with_source(LlamaSource::llama_3_1_8b_chat())
            .build()
            .await
            .unwrap();
        Self {
            chat: Mutex::new(Chat::builder(model).build()),
        }
    }
}

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    let state = web::Data::new(AppState::new().await);

    env_logger::init_from_env(Env::default().default_filter_or("info"));

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(state.clone())
            .service(stream_endpoint)
    })
    .bind("[::1]:8000")?
    .run()
    .await
}
