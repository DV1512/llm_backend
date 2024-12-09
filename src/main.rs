use crate::endpoints::chat::chat_service;
use crate::state::AppState;
use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use error::ServerError;
use logging::init_tracing;
use tracing_actix_web::TracingLogger;

mod dto;
mod endpoints;
mod error;
mod logging;
mod models;
pub(crate) mod rapport;
mod responders;
mod services;
mod state;
#[actix_web::main]
async fn main() -> Result<(), ServerError> {
    init_tracing()?;

    let state = web::Data::new(AppState::new().await?);

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .service(chat_service())
            .wrap(Cors::permissive())
            .wrap(TracingLogger::default())
    })
    .bind("0.0.0.0:8000")?
    .run()
    .await?;

    Ok(())
}
