use crate::chat::chat_service;
use crate::state::AppState;
use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use error::ServerError;
use logging::init_tracing;
use tracing_actix_web::TracingLogger;

mod chat;
mod error;
mod logging;
mod state;

#[actix_web::main]
async fn main() -> Result<(), ServerError> {
    init_tracing()?;

    let state = web::Data::new(AppState::new().await);

    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .wrap(TracingLogger::default())
            .app_data(state.clone())
            .service(chat_service())
    })
    .bind("[::1]:8000")?
    .run()
    .await?;

    Ok(())
}
