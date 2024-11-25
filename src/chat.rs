use crate::state::AppState;
use actix_web::dev::HttpServiceFactory;
use actix_web::{post, web, Responder};
use actix_web_lab::sse;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio_stream::StreamExt;
use ulid::Ulid;

#[derive(Debug, Serialize, Deserialize)]
struct ChatRequest {
    prompt: String,
}

#[tracing::instrument(skip(state))]
#[post("/completions")]
async fn completions(
    web::Json(request_data): web::Json<ChatRequest>,
    state: web::Data<AppState>,
) -> Result<impl Responder, actix_web::Error> {
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
    Ok(sse::Sse::from_stream(sse_stream))
}

#[tracing::instrument]
pub fn chat_service() -> impl HttpServiceFactory {
    web::scope("/chat").service(completions)
}

#[derive(Serialize)]
pub enum ChatRole {
    System,
    User,
    Assistant,
}

/// A single chunk of a chat message.
#[derive(Serialize)]
pub struct ChatMessageChunk {
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
    pub fn new(id: ulid::Ulid, timestamp: u64, role: ChatRole, contents: String) -> Self {
        Self {
            id,
            timestamp,
            role,
            contents,
        }
    }
}
