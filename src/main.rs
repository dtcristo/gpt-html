pub use self::error::{Error, Result};

use axum::{
    body::StreamBody,
    http::{header, Uri},
    response::IntoResponse,
    routing::{get, get_service},
    Json, Router, Server,
};
use axum_extra::middleware::option_layer;
use eventsource_stream::Eventsource;
use futures::future;
use futures::{StreamExt, TryStreamExt};
use serde::{Deserialize, Serialize};
use std::{
    env,
    io::{self, Write},
    net::SocketAddr,
    time::SystemTime,
};
use tokio::signal;
use tower_http::{services::ServeDir, validate_request::ValidateRequestHeaderLayer};

mod error;

#[tokio::main]
async fn main() {
    println!("Starting server...");

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    Server::bind(&addr)
        .serve(app().into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("server should serve");
}

fn app() -> Router {
    let auth = option_layer(
        env::var("HTTP_BASIC_AUTH_PASSWORD")
            .ok()
            .map(|password| ValidateRequestHeaderLayer::basic("user", &password)),
    );

    Router::new().route("/health", get(health)).nest_service(
        "/",
        // Handle GET.
        get_service(
            // Serve static files from "public".
            ServeDir::new("public")
                // When static file missing use handler.
                .fallback(get(handler)),
        )
        // Other methods use handler.
        .post(handler)
        .patch(handler)
        .put(handler)
        .delete(handler)
        // Apply HTTP basic auth.
        .layer(auth),
    )
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    println!("signal received, starting graceful shutdown");
}

#[derive(Debug, Serialize)]
struct HealthBody {
    time: u64,
    commit_sha: String,
    basic_auth_enabled: bool,
}

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

async fn health() -> Result<impl IntoResponse> {
    println!("\n----------");
    println!("Health");
    println!("----------");

    env::var("OPENAI_API_KEY").map_err(|_| Error::EnvironmentError)?;

    let time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|_| Error::SystemTimeError)?
        .as_secs();
    let commit_sha = env::var("COMMIT_SHA").unwrap_or_else(|_| "unknown".to_string());
    let basic_auth_enabled = env::var("HTTP_BASIC_AUTH_PASSWORD").is_ok();

    Ok(Json(HealthBody {
        time,
        basic_auth_enabled,
        commit_sha,
    }))
}

async fn handler(uri: Uri) -> Result<impl IntoResponse> {
    println!("\n----------");
    println!("Fetching: {uri}");
    println!("----------");

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

    let stream = reqwest::Client::new()
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
            let _ = io::stdout().flush();

            content
        });

    Ok((
        [(header::CONTENT_TYPE, "text/html")],
        StreamBody::new(stream),
    ))
}
