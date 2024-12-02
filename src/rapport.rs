use kalosm::language::*;
use serde::{Deserialize, Serialize};

#[derive(Parse, Debug, Clone, PartialOrd, PartialEq, Serialize, Deserialize)]
pub struct Rapport {
    summary: String,
    items: Vec<RapportItem>,
}

#[derive(Parse, Debug, Clone, PartialOrd, PartialEq, Serialize, Deserialize)]
pub struct RapportItem {
    name: String,
    description: String,
    mitigations: Vec<Mitigation>,
    #[parse(with = FloatParser::new(0.0..=1.0))]
    likelihood: f64,
}

#[derive(Parse, Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize)]
pub struct Mitigation {
    name: String,
    description: String,
    url: String,
    citations: String,
}
