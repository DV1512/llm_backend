use crate::state::AppState;
use actix_web::dev::HttpServiceFactory;
use actix_web::{post, web, Responder};
use actix_web_lab::sse;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio_stream::StreamExt;
use ulid::Ulid;

#[derive(Serialize, Deserialize)]
struct ChatRequest {
    prompt: String, // The custom prompt the user provides
}

#[derive(Serialize, Deserialize)]
struct Prompt {
    name: String,
    prompt_template: String,
}

fn load_prompts() -> Vec<Prompt> {
    let file_data = fs::read_to_string("prompt.json").expect("unable to read prompt.json");
    serde_json::from_str(&file_data).expect("unable to parse prompt.json")
}

#[post("/templated")]
async fn templated(
    web::Json(request_data): web::Json<Prompt>,
    state: web::Data<AppState>,
) -> impl Responder {
    let prompts = load_prompts();

    let prompt_data = prompts
        .iter()
        .find(|p| p.name == request_data.name)
        .expect("Prompt not found for the given name");

    let group = state
        .threat_groups
        .iter()
        .find(|group| group.name == request_data.name)
        .expect("Group not found in the state");

    let prompt = prompt_data
        .prompt_template
        .replace("{name}", &group.name)
        .replace("{description}", &group.description)
        .replace("{associated_names}", &group.associated_names.join(", "));

    let stream = state.chat.lock().unwrap().add_message(prompt);

    let ulid = Ulid::new();
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards?")
        .as_secs();

    let sse_stream = stream.map(move |item| -> Result<sse::Event, Infallible> {
        let chunk = ChatMessageChunk::new(ulid, timestamp, ChatRole::User, item);
        let chunk_str = serde_json::to_string(&chunk).unwrap();
        let data = sse::Data::new(chunk_str);
        Ok(sse::Event::Data(data))
    });

    sse::Sse::from_stream(sse_stream)
}

#[post("/completions")]
async fn completions(
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

pub fn chat_service() -> impl HttpServiceFactory {
    web::scope("/chat").service(completions).service(templated)
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
    id: Ulid,
    timestamp: u64,
    role: ChatRole,
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
