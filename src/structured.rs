use crate::actor::Actor;
use crate::state::AppState;
use actix_web::{post, web, Responder};
use kalosm::language::*;
use serde::Deserialize;
use serde_json::{from_str, Value};

#[derive(Deserialize)]
struct Request {
    text: String,
}
pub async fn convert_to_string(state: &AppState) -> Result<String, serde_json::Error> {
    let all_data = &state.threat_groups;
    let serialized = serde_json::to_string(all_data)?;
    Ok(serialized)
}

#[post("/structured")]
pub async fn structured(
    web::Json(prompt): web::Json<Request>,
    state: web::Data<AppState>,
) -> impl Responder {
    let constraints = Actor::new_parser();

    let task = Task::builder(
        "You generate Json reports of which threat group is linked to the provided text.",
    )
    .with_constraints(constraints)
    .build();

    // Attempt to find a matching group.name
    let matched_group = state.threat_groups.iter().find(|group| {
        prompt.text.contains(&group.name)
            || group
                .associated_names
                .iter()
                .any(|alias| prompt.text.contains(alias))
    });
    let all_data = convert_to_string(&state).await.unwrap();

    let final_prompt = if let Some(group) = matched_group {
        let data_about_group = format!(
            r#"{{
    "id": "{}",
    "name": "{}",
    "description": "{}",
    "url": "{}",
    "associated_names": {:?}
        }}"#,
            group.id, group.name, group.description, group.url, group.associated_names
        );

        let format = r#"{"id": string, "name": string, "description": string, "url": string, "associated_names": string[]}"#;
        format!(
            "Question: {}, Answer with this format if possible: {}, A threat group name was found in the text, here is data about that group: {}, and here is further data about datagroups etc, that might be usefull for answering the question:{}.",
            prompt.text, format, data_about_group, all_data,
        )
    } else {
        prompt.text.clone()
    };

    let res = task.run(final_prompt, &*state.model.clone());

    /*let sse_stream = res.stream().map(move |item| -> Result<sse::Event, _> {
        let data = sse::Data::new(item);
        Ok::<_, Infallible>(sse::Event::Data(data))
    });
    sse::Sse::from_stream(sse_stream)*/

    let text = res.text().await;

    let json: Value = from_str(&text).unwrap();

    web::Json(json)
}
