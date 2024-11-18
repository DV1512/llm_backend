use crate::chat::chat_service;
use crate::state::AppState;
use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use env_logger::Env;

mod chat;
mod state;

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    let state = web::Data::new(AppState::new().await);

    env_logger::init_from_env(Env::default().default_filter_or("info"));

    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .wrap(Logger::default())
            .app_data(state.clone())
            .service(chat_service())
    })
    .bind("[::1]:8000")?
    .run()
    .await
}
