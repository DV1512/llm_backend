use crate::dto::SearchEmbeddingsRequest;
use crate::models::{ChatMessageChunk, ChatRole, Entry};
use crate::rapport::Rapport;
use actix_web::error::ErrorInternalServerError;
use actix_web::HttpResponse;
use actix_web_lab::__reexports::futures_util::stream::BoxStream;
use actix_web_lab::__reexports::futures_util::StreamExt;
use actix_web_lab::sse;
use actix_web_lab::sse::{Event, Sse};
use kalosm::language::{Bert, Chat, Embedder, Llama, Parse, Task};
use serde_json::Value;
use std::convert::Infallible;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use ulid::Ulid;

pub fn chat(
    prompt: String,
    model: Arc<Llama>,
) -> Sse<BoxStream<'static, Result<Event, Infallible>>> {
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

const FALLBACK_MAIN_BACKEND_SEARCH_EMBEDDINGS_URL: &str =
    "http://localhost:9999/api/v1/embeddings/search";

pub async fn compute_single_embedding(
    query: String,
    model: Arc<Bert>,
) -> Result<Vec<f32>, actix_web::Error> {
    match model.embed_string(query).await {
        Ok(embedding) => Ok(embedding.to_vec()),
        Err(err) => Err(ErrorInternalServerError(err)),
    }
}

pub async fn find_closest_embeddings(
    search_request: SearchEmbeddingsRequest,
) -> Result<Vec<Entry>, actix_web::Error> {
    let main_backend_search_embeddings_url = std::env::var("MAIN_BACKEND_SEARCH_EMBEDDINGS_URL")
        .unwrap_or(FALLBACK_MAIN_BACKEND_SEARCH_EMBEDDINGS_URL.to_string());

    let client = reqwest::Client::new();
    let Ok(res) = client
        .post(main_backend_search_embeddings_url)
        .json(&search_request)
        .send()
        .await
    else {
        return Err(ErrorInternalServerError(
            "Error connecting to main backend".to_string(),
        ));
    };

    let Ok(res_body) = res.json::<Vec<Entry>>().await else {
        return Err(ErrorInternalServerError(
            "Error reading body from request to main backend".to_string(),
        ));
    };

    Ok(res_body)
}
