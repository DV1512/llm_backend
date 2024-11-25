use std::convert::Infallible;
use actix_web::{post, web, Responder};
use actix_web_lab::sse;
use kalosm::language::*;
use serde::Deserialize;
use serde_json::{from_str, Value};
use crate::actor::Actor;
use crate::state::AppState;

#[derive(Deserialize)]
struct Request {
    text: String,
}

#[post("/structured")]
pub async fn structured(web::Json(prompt): web::Json<Request>, state: web::Data<AppState>) -> impl Responder {
    let constraints = Actor::new_parser();

    let task = Task::builder("You generate Json reports of which threat group is linked to the provided text.")
        .with_constraints(constraints)
        .build();

    let format = r#"{"id": string, "name": string, "description": string, "url": string, "associated_names": string[]}"#;
    let prompt = format!("{} answer with this format: {}", prompt.text, format);

    let mut res = task.run(prompt, &*state.model.clone());

    /*let sse_stream = res.stream().map(move |item| -> Result<sse::Event, _> {
        let data = sse::Data::new(item);
        Ok::<_, Infallible>(sse::Event::Data(data))
    });
    sse::Sse::from_stream(sse_stream)*/

    let text = res.text().await;

    let json: Value = from_str(&text).unwrap();

    web::Json(json)
}