use crate::models::{ChatMessageChunk, ChatRole};
use crate::rapport::Rapport;
use crate::state::AppState;
use actix_web::{web, HttpResponse};
use actix_web_lab::__reexports::futures_util::stream::BoxStream;
use actix_web_lab::__reexports::futures_util::StreamExt;
use actix_web_lab::sse;
use actix_web_lab::sse::{Event, Sse};
use kalosm::language::{Chat, Llama, Task};
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing_subscriber::fmt::format;
use ulid::Ulid;

pub fn keywords(app_state: web::Data<AppState>, prompt: &str) -> String {
    let keyword_to_mitigation: HashMap<&str, Vec<&str>> = HashMap::from([
        ("website", vec!["M1036", "M1048", "M1057"]),
        ("web", vec!["M1036", "M1047", "M1050"]),
        ("database", vec!["M1049", "M1028", "M1032"]),
        ("backend", vec!["M1015", "M1042", "M1025"]),
        ("credentials", vec!["M1043", "M1034", "M1033"]),
        ("security", vec!["M1050", "M1041", "M1038"]),
        ("network", vec!["M1037", "M1035", "M1030"]),
        ("authentication", vec!["M1032", "M1018", "M1026"]),
        ("permissions", vec!["M1024", "M1022", "M1039"]),
        ("encryption", vec!["M1041", "M1051", "M1029"]),
    ]);

    let mitigations = app_state
        .get_data("mitre_mitigations")
        .cloned()
        .unwrap_or_default();
    let mitigations: Vec<Value> =
        serde_json::from_str(&mitigations).expect("Failed to parse mitigations JSON");

    let lowercase_prompt = prompt.to_lowercase().replace(".", "").replace(",", "");

    let words_in_prompt: Vec<&str> = lowercase_prompt.split_whitespace().collect();

    let mut relevant_mitigation_ids = Vec::new();
    for word in words_in_prompt {
        if let Some(mitigation_ids) = keyword_to_mitigation.get(word) {
            relevant_mitigation_ids.extend_from_slice(mitigation_ids);
        }
    }

    relevant_mitigation_ids.sort();
    relevant_mitigation_ids.dedup();

    let selected_mitigations: Vec<&Value> = mitigations
        .iter()
        .filter(|m| {
            if let Some(id) = m["ID"].as_str() {
                relevant_mitigation_ids.contains(&id)
            } else {
                false
            }
        })
        .collect();

    let formatted_mitigations: Vec<String> = selected_mitigations
        .iter()
        .map(|mitigation| {
            format!(
                "-name: {}, description: {}\n  url: {}",
                mitigation["name"].as_str().unwrap_or("N/A"),
                mitigation["description"].as_str().unwrap_or("N/A"),
                mitigation["url"].as_str().unwrap_or("N/A")
            )
        })
        .collect();

    let formatted_mitigations_str = formatted_mitigations.join("\n");
    println!("{}", formatted_mitigations_str);
    return formatted_mitigations_str;
}

pub fn chat(
    prompt: String,
    app_state: web::Data<AppState>,
    model: Arc<Llama>,
) -> Sse<BoxStream<'static, Result<Event, Infallible>>> {
    let mut chat = Chat::builder((*model).clone()).build();
    let formatted_mitigations_str = keywords(app_state, &prompt);
    println!("{}", formatted_mitigations_str);

    let analysis_prompt = format!(
        "You are a cybersecurity assistant. Your task is to analyze the user's input and determine the required action. 
        If the user is asking about threat modeling, respond with a question: \'Would you like to perform a threat modeling analysis for a specific connection, or an overall analysis of the system?' \
        If the user answers the question of complete analysis or component analysis, then analyze the connections provided and provide top 10 threats and mitigations.
        Reference Mitre Atlas if needed.
        User input: {}. Extra data: {}", prompt, formatted_mitigations_str
    );

    let stream = chat.add_message(analysis_prompt);

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

pub async fn structured(
    prompt: String,
    app_state: web::Data<AppState>,
    model: Arc<Llama>,
) -> HttpResponse {
    let key_words = keywords(app_state.clone(), &prompt);

    let final_prompt = format!("User input: {}. Extra Data: {}", prompt, key_words);
    println!("{}", serde_json::to_string_pretty(&final_prompt).unwrap());

    let task = Task::builder_for::<Rapport>(
        " You are a security threat analyzer. Analyze the following system or scenario and provide a list of up to 10 identified threats, each with a clear description and actionable mitigations. Structure your response in the following format:
    Threat Name
    Description: [Detailed description of the threat]
    Mitigation(s):
        [Actionable step 1]
        [Actionable step 2 (if applicable)]"
    )
    .build();

    let res = task.run(final_prompt.to_string(), &*app_state.model);
    let text = res.text().await;

    let parsed: Value = serde_json::from_str(&text).unwrap_or_else(|_| {
        serde_json::json!({
            "error": "Failed to parse LLM response"
        })
    });

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
