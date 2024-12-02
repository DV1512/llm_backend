use crate::endpoints::chat::completions::*;
use actix_web::dev::HttpServiceFactory;
use actix_web::web;

mod completions;

pub fn chat_service() -> impl HttpServiceFactory {
    web::scope("/chat").service(completions)
}
