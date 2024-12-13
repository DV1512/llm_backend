use crate::models::EntryType;
use actix_web::{dev::HttpServiceFactory, web};
use std::fmt::Display;

mod post;
use post::*;

impl Display for EntryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntryType::Threat => write!(f, "threat"),
            EntryType::Mitigation => write!(f, "mitigation"),
        }
    }
}

pub fn embeddings_service() -> impl HttpServiceFactory {
    web::scope("/embeddings")
        .service(add_embeddings)
        .service(search_embeddings)
}
