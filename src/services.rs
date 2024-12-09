use crate::models::{ChatMessageChunk, ChatRole};
use crate::rapport::Rapport;
use crate::state::AppState;
use actix_web::{web, HttpResponse};
use actix_web_lab::__reexports::futures_util::stream::BoxStream;
use actix_web_lab::__reexports::futures_util::StreamExt;
use actix_web_lab::sse;
use actix_web_lab::sse::{Event, Sse};
use kalosm::language::{Chat, Llama, Task};
use serde_json::Value;
use std::collections::HashMap;
use std::collections::HashSet;
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
pub async fn structured(
    prompt: String,
    app_state: web::Data<AppState>,
    model: Arc<Llama>,
) -> HttpResponse {
    // Predefined keyword-to-mitigation mapping
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

    // Lowercase the user input for consistent matching
    let lowercase_prompt = prompt.to_lowercase().replace(".", "").replace(",", "");

    let words_in_prompt: Vec<&str> = lowercase_prompt.split_whitespace().collect();

    // Find relevant mitigation IDs based on keywords
    let mut relevant_mitigation_ids = Vec::new();
    for word in words_in_prompt {
        if let Some(mitigation_ids) = keyword_to_mitigation.get(word) {
            relevant_mitigation_ids.extend_from_slice(mitigation_ids);
        }
    }

    // Deduplicate IDs to avoid repetition
    relevant_mitigation_ids.sort();
    relevant_mitigation_ids.dedup();

    // Filter the mitigations using the selected IDs
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

    // Prepare mitigations for the prompt
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

    // Construct the final prompt
    let final_prompt = format!(
        r#"
    You are a cybersecurity expert. Your task is to generate a detailed report analyzing potential vulnerabilities in the provided system and suggesting mitigation strategies.

    User Input: {}

    Instructions:
    1. Analyze the input to identify potential vulnerabilities and attack surfaces.
    2. For each vulnerability:
    - Assign a unique, descriptive `name`.
    - Write a clear `description` explaining the vulnerability.
    - Provide a list of `mitigations` for this vulnerability. Each mitigation must include:
      - A `name` describing the mitigation (e.g., "Implement Account Lockout Policy").
      - A `description` explaining how the mitigation addresses the vulnerability.
      - A `url` for additional information or documentation about the mitigation.
    3. Ensure that each vulnerability and mitigation is specific, relevant, and complete.
    
    Relevant Data: {}

    **Example Output:**
{{
    "summary": "This report outlines potential vulnerabilities and mitigation strategies for the provided system.",
    "items": [
        {{
            "name": "SQL Injection Vulnerability",
            "description": "The system is vulnerable to SQL injection due to improper sanitization of user inputs.",
            "mitigations": [
                {{
                    "name": "Use Parameterized Queries",
                    "description": "Ensure all SQL queries use parameterized statements to avoid injection.",
                    "url": "https://example.com/mitigation/sql-injection"
                }},
                {{
                    "name": "Input Validation",
                    "description": "Implement robust input validation to sanitize user inputs.",
                    "url": "https://example.com/mitigation/input-validation"
                }}
            ]
        }},
        {{
            "name": "Cross-Site Scripting (XSS)",
            "description": "The system lacks proper input sanitization, making it vulnerable to XSS attacks.",
            "mitigations": [
                {{
                    "name": "Content Security Policy",
                    "description": "Implement a Content Security Policy to restrict loaded resources.",
                    "url": "https://example.com/mitigation/csp"
                }},
                {{
                    "name": "Escape Special Characters",
                    "description": "Sanitize user inputs by escaping special characters.",
                    "url": "https://example.com/mitigation/xss-escape"
                }}
            ]
        }}
    ]
}}

    Report: You make a rapport with each threat, mitigation in a similiar manner to the example output.
    "#,
        prompt,
        formatted_mitigations.join("\n")
    );
    println!("{}", final_prompt);

    // Run the task
    let task = Task::builder_for::<Rapport>(
        "You threat model a given input and generate a JSON rapport with vulnerabilities and mitigations."
    )
    .build();

    let res = task.run(final_prompt, &*model);
    let text = res.text().await;

    // Parse the response
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
/*
pub async fn structured(
    prompt: String,
    app_state: web::Data<AppState>,
    model: Arc<Llama>,
) -> HttpResponse {
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

    let lowercase_prompt = prompt
        .to_lowercase()
        .replace(".", "") // Remove periods
        .replace(",", ""); // Remove commas

    let words_in_prompt: Vec<&str> = lowercase_prompt.split_whitespace().collect();
    //println!("Combined Keywords: {:?}", words_in_prompt);

    let mut relevant_mitigation_ids: HashSet<&str> = HashSet::new();
    for word in &words_in_prompt {
        if let Some(mitigation_ids) = keyword_to_mitigation.get(word) {
            relevant_mitigation_ids.extend(mitigation_ids);
        }
        println!("{}", word);
    }
    //println!("Combined Keywords: {:?}", relevant_mitigation_ids);

    let selected_mitigations: Vec<&Value> = mitigations
        .iter()
        .filter(|m| {
            if let Some(id) = m["ID"].as_str() {
                relevant_mitigation_ids.contains(id)
            } else {
                false
            }
        })
        .collect();
    //println!("Combined Keywords: {:?}", selected_mitigations);
    let formatted_mitigations: Vec<String> = selected_mitigations
        .iter()
        .map(|mitigation| {
            format!(
                "- {}: {}\n",
                mitigation["name"].as_str().unwrap_or("N/A"),
                mitigation["description"].as_str().unwrap_or("N/A")
            )
        })
        .collect();

    let final_prompt = format!(
        r#"
    You are a cybersecurity expert. Your task is to analyze potential vulnerabilities in the provided system and suggesting mitigations.

    **User Input:**
    {}

    **Instructions:**
    "1. Identify 10 potential vulnerabilities based on the user input.",
    "2. For each vulnerability, provide a detailed description and associated mitigations.",
    "3. Make a Report with each threat and vulnerability and the mitigation associated to the threat.",


    **Relevant Data:**
    {}
    And your own data.
    "#,
        prompt,
        formatted_mitigations.join("\n")
    );
    println!("{}", final_prompt);

    let task = Task::builder_for::<Rapport>(
        "You threat model a given input and generate a report with top ten vulnerabilities and mitigations."
    )
    .build();

    let res = task.run(final_prompt, &*model);
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
*/
