use kalosm::language::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Parse, Clone)]
pub struct Actor {
    pub id: String,
    pub name: String,
    pub description: String,
    pub url: String,
    pub associated_names: Vec<String>,
}