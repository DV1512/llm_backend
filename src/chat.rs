use crate::state::AppState;
use crate::structured::structured;
use actix_web::dev::HttpServiceFactory;
use actix_web::{post, web, Responder};
use actix_web_lab::sse;
use kalosm::language::Chat;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::convert::Infallible;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio_stream::StreamExt;
use ulid::Ulid;

#[derive(Serialize, Deserialize)]
struct ChatRequest {
    prompt: String,
}

#[derive(Serialize, Deserialize)]
struct Prompt {
    name: String,
    prompt_template: String,
}

pub struct Report {
    pub contents: String,
}

impl Report {
    pub fn new() -> Self {
        Report {
            contents: String::new(),
        }
    }
    pub fn append(&mut self, data: &str) {
        if !self.contents.is_empty() && !data.trim().is_empty() {
            self.contents.push(' ');
        }
        self.contents.push_str(data.trim());
    }
    pub fn format(&mut self, raw_data: &String) {
        for chunk in raw_data.split('}') {
            let json_str = format!("{}}}", chunk);

            if let Ok(Value::Object(parsed)) = serde_json::from_str::<Value>(&json_str) {
                if let Some(Value::String(content)) = parsed.get("contents") {
                    self.append(content);
                }
            }
        }
        self.contents = self.formatting();
    }
    fn formatting(&self) -> String {
        let mut formatted = String::new();
        let mut capitalize_next = true;

        for c in self.contents.chars() {
            match c {
                '.' | ',' | ')' => {
                    formatted = formatted.trim_end().to_string();
                    formatted.push(c);
                    capitalize_next = c == '.';
                }
                ' ' => {
                    if !formatted.ends_with(' ') {
                        formatted.push(c);
                    }
                }
                '\'' => {
                    if formatted.ends_with(' ') {
                        formatted.pop();
                    }
                    formatted.push(c);
                }
                _ if capitalize_next && c.is_alphabetic() => {
                    formatted.push(c.to_ascii_uppercase());
                    capitalize_next = false;
                }
                _ => formatted.push(c),
            }
        }

        formatted
    }

    pub fn save(&self) {
        if !self.contents.is_empty() {
            let mut file = OpenOptions::new()
                .create(true)
                .write(true)
                .open("text.txt")
                .expect("Failed to open text.txt");

            writeln!(file, "{}", self.contents).expect("Failed to write to text.txt");
        }
    }
}

#[post("/templated")]
async fn templated(
    web::Json(request_data): web::Json<Prompt>,
    state: web::Data<AppState>,
) -> Result<impl Responder, actix_web::Error> {
    let prompt_data = state
        .prompts
        .iter()
        .find(|p| p.name == "example")
        .ok_or_else(|| {
            actix_web::error::ErrorNotFound(format!("Prompt not found for {}", request_data.name))
        })?;

    let group = state
        .threat_groups
        .iter()
        .find(|group| group.name == request_data.name)
        .ok_or_else(|| {
            actix_web::error::ErrorNotFound(format!(
                "Threat group {} not found.",
                request_data.name
            ))
        })?;

    let prompt = prompt_data
        .prompt_template
        .replace("{name}", &group.name)
        .replace("{description}", &group.description)
        .replace("{associated_names}", &group.associated_names.join(", "))
        .replace("{template_question}", &request_data.prompt_template);

    let mut chat = Chat::builder((*state.model).clone()).build();
    let stream = chat.add_message(prompt);

    let ulid = Ulid::new();
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| actix_web::error::ErrorInternalServerError("time went backwards?"))?
        .as_secs();

    let return_report = Arc::new(Mutex::new(Report::new()));
    let return_report_clone = Arc::clone(&return_report);

    let sse_stream = stream.map(move |item| -> Result<sse::Event, Infallible> {
        let chunk = ChatMessageChunk::new(ulid, timestamp, ChatRole::User, item);
        let chunk_str = serde_json::to_string(&chunk).unwrap();
        {
            let mut report = return_report_clone.lock().unwrap();
            report.format(&chunk_str);
            report.save();
        }

        let data = sse::Data::new(chunk_str);
        Ok(sse::Event::Data(data))
    });

    Ok(sse::Sse::from_stream(sse_stream))
}

#[post("/completions")]
async fn completions(
    web::Json(request_data): web::Json<ChatRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let mut chat = Chat::builder((*state.model).clone()).build();
    let stream = chat.add_message(request_data.prompt);

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
    web::scope("/chat")
        .service(completions)
        .service(templated)
        .service(structured)
}

#[derive(Serialize)]
pub enum ChatRole {
    System,
    User,
    Assistant,
}

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
