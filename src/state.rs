use kalosm::language::{Llama, LlamaSource};
use serde::{Deserialize, Serialize};
use std::fs;
use std::sync::Arc;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::Surreal;

#[derive(Serialize, Deserialize, Debug)]
pub struct ThreatActor {
    pub id: String,
    pub name: String,
    pub description: String,
    pub url: String,
    pub associated_names: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Prompt {
    pub name: String,
    pub prompt_template: String,
}

pub struct AppState {
    pub model: Arc<Llama>,
    pub threat_groups: Vec<ThreatActor>,
    pub prompts: Vec<Prompt>,
    pub db: Arc<Surreal<Client>>,
}

fn load_prompts() -> Vec<Prompt> {
    let file_data = fs::read_to_string("prompt.json").expect("unable to read prompt.json");
    serde_json::from_str(&file_data).expect("unable to parse prompt.json")
}

pub async fn db(ns: &str, db: &str) -> Surreal<Client> {
    let db_url = "localhost:7352";
    let db_user = "root";
    let db_pass = "root";

    let database = Surreal::new::<Ws>(db_url)
        .await
        .expect("Failed to connect to database");

    database
        .signin(surrealdb::opt::auth::Root {
            username: db_user,
            password: db_pass,
        })
        .await
        .expect("Failed to sign in to database");

    database
        .use_ns(ns)
        .use_db(db)
        .await
        .expect("Failed to use database");

    database
}

impl AppState {
    pub async fn new() -> Self {
        let model = Arc::new(
            Llama::builder()
                .with_source(LlamaSource::llama_3_1_8b_chat())
                .build()
                .await
                .unwrap(),
        );

        let json_data = fs::read_to_string("data.json").expect("Failed to read");
        let threat_groups: Vec<ThreatActor> =
            serde_json::from_str(&json_data).expect("JSON failed");

        let prompts = load_prompts();

        let db = Arc::new(db("default", "llm").await);

        Self {
            model,
            db,
            threat_groups,
            prompts,
        }
    }
}
