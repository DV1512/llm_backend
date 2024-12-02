use std::sync::Arc;
use kalosm::language::{Chat, Llama, Parse, Task};
use actix_web::HttpResponse;
use serde_json::Value;
use ulid::Ulid;
use std::time::{SystemTime, UNIX_EPOCH};
use actix_web_lab::sse::{Event, Sse};
use actix_web_lab::__reexports::futures_util::stream::BoxStream;
use std::convert::Infallible;
use actix_web_lab::sse;
use actix_web_lab::__reexports::futures_util::StreamExt;
use crate::models::{ChatMessageChunk, ChatRole};
use crate::rapport::Rapport;

pub fn chat(prompt: String, model: Arc<Llama>) -> Sse<BoxStream<'static, Result<Event, Infallible>>> {
    let mut chat = Chat::builder((*model).clone()).build();
    let stream = chat.add_message(prompt);

    let ulid = Ulid::new();
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards?")
        .as_secs();

    let sse_stream = stream
        .map(move |item| -> Result<Event, _> {
            let chunk =
                ChatMessageChunk::new(ulid, timestamp, ChatRole::Assistant, Value::String(item));
            let chunk_str = serde_json::to_string(&chunk).unwrap();
            let data = sse::Data::new(chunk_str);
            Ok::<_, Infallible>(Event::Data(data))
        })
        .boxed();

    sse::Sse::from_stream(sse_stream)
}

pub async fn structured(prompt: String, model: Arc<Llama>) -> HttpResponse {
    let constraints = Rapport::new_parser();
    let task = Task::builder("You threat model a given input and generate a JSON rapport with vulnerabilities and mitigations.")
        .with_constraints(constraints)
        .build();
    let format_prompt = r#"""
    {
        "summary": string,
        "items": {
            "name": string,
            "description": string,
            "mitigations: {
                "name": string,
                "description": string,
                "url": url,
                "citations": string
            }[]
            "likelihood": float
        }[]
    }
    """#;

    let final_prompt = format!("{prompt}. \n Answer using this format {format_prompt}...");
    let res = task.run(final_prompt, &*model);
    let text = res.text().await;

    let parsed: Value = serde_json::from_str(&text).unwrap();

    let chunk = ChatMessageChunk::new_serialized(
        Ulid::new(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards?")
            .as_secs(),
        ChatRole::Assistant,
        parsed,
    );

    HttpResponse::Ok().json(chunk)
}