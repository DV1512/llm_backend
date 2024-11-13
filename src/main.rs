use actix_web::{middleware::Logger, post, web, App, HttpServer, Responder};
use actix_web_lab::sse;
use env_logger::Env;
use kalosm::language::{Chat, Llama, LlamaSource};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
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

    let ulid = Ulid::new();
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards?")
        .as_secs();

    let sse_stream = stream.map(move |item| -> Result<sse::Event, _> {
        let chunk = ChatMessageChunk::new(ulid, timestamp, ChatRole::User, item);
        let chunk_str = serde_json::to_string(&chunk).unwrap();
        let data = sse::Data::new(chunk_str);
        Ok::<_, Infallible>(sse::Event::Data(data))
    });
    sse::Sse::from_stream(sse_stream)
}

#[derive(Serialize)]
enum ChatRole {
    System,
    User,
    Assistant,
}

/// A single chunk of a chat message.
#[derive(Serialize)]
struct ChatMessageChunk {
    /// A unique identifier for the chat completion.
    /// Each chunk has the same ID.
    id: Ulid,
    /// The Unix timestamp (in seconds) of when the chat completion was created.
    /// Each chunk has the same timestamp.
    timestamp: u64,
    /// The role of the author of this message.
    role: ChatRole,
    /// The contents of the chunk message.
    contents: String,
}

impl ChatMessageChunk {
    fn new(id: ulid::Ulid, timestamp: u64, role: ChatRole, contents: String) -> Self {
        Self {
            id,
            timestamp,
            role,
            contents,
        }
    }
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
