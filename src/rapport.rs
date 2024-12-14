use kalosm::language::*;
use serde::{Deserialize, Serialize};

#[derive(Parse, Debug, Clone, PartialOrd, PartialEq, Serialize, Deserialize, Schema)]
pub struct Rapport {
    summary: String,
    items: Vec<RapportItem>,
}

#[derive(Parse, Debug, Clone, PartialOrd, PartialEq, Serialize, Deserialize, Schema)]
pub struct RapportItem {
    name: String,
    description: String,
    mitigations: Vec<Mitigation>,
}

#[derive(Parse, Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize, Schema)]
pub struct Mitigation {
    name: String,
    description: String,
}
