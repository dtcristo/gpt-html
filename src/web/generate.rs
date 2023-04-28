use std::{env, future};

use axum::{body::StreamBody, http::Uri, response::IntoResponse};
use eventsource_stream::Eventsource;
use futures::{StreamExt, TryStreamExt};
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

#[derive(Debug, Serialize)]
struct ChatCompletionsBody {
    model: String,
    stream: bool,
    messages: Vec<Message>,
}

#[derive(Debug, Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct Event {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    delta: Delta,
}

#[derive(Debug, Deserialize)]
struct Delta {
    content: String,
}

pub async fn handler(uri: Uri) -> Result<impl IntoResponse> {
    println!("\n----------");
    println!("Fetching: {uri}");

    let prompt = r#"
Output a valid HTML document for the webpage that could be located at the URL path provided by the user. Include general navigation anchor tags as well as relative anchor tags to other related pages. Include a minimal amount of inline styles to improve the look of the page. Make the text content quite long with a decent amount of interesting content. Do not use any dummy text on the page.

Start the reponse with the following exact characters:

<!doctype html>
<html>"#;

    let body = ChatCompletionsBody {
        model: "gpt-3.5-turbo".to_string(),
        stream: true,
        messages: vec![
            Message {
                role: "system".to_string(),
                content: prompt.to_string(),
            },
            Message {
                role: "user".to_string(),
                content: uri.to_string(),
            },
        ],
    };

    let stream = Client::new()
        .post("https://api.openai.com/v1/chat/completions")
        .header("content-type", "application/json")
        .header(
            "authorization",
            &format!(
                "Bearer {}",
                env::var("OPENAI_API_KEY").map_err(|_| Error::EnvironmentError)?
            ),
        )
        .body(serde_json::to_string(&body).map_err(|_| Error::SerializationError)?)
        .send()
        .await
        .map_err(|_| Error::RequestError)?
        .bytes_stream()
        .eventsource()
        .map(|r| match r {
            Ok(e) => {
                serde_json::from_str::<Event>(&e.data).map_err(|_| Error::DeserializationError)
            }
            _ => Err(Error::StreamError),
        })
        // Discard errors (will most likely be `Error::JsonError`).
        .filter(|r| future::ready(r.is_ok()))
        .map_ok(|event| {
            let content = event
                .choices
                .into_iter()
                .next()
                .expect("event should have at least one choice")
                .delta
                .content;

            // Debug log.
            print!("{}", content);

            content
        });

    Ok((
        [(header::CONTENT_TYPE, "text/html")],
        StreamBody::new(stream),
    ))
}
