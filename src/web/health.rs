use std::time::SystemTime;

use axum::{response::IntoResponse, Json};
use serde::Serialize;

use crate::{
    env,
    error::{Error, Result},
};

#[derive(Debug, Serialize)]
struct HealthBody {
    time: u64,
    commit_sha: String,
    basic_auth_enabled: bool,
}

pub async fn handler() -> Result<impl IntoResponse> {
    println!("\n----------");
    println!("Health");

    env::print();

    env::openai_api_key().ok_or(Error::EnvironmentError)?;

    let time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|_| Error::SystemTimeError)?
        .as_secs();
    let commit_sha = env::commit_sha();
    let basic_auth_enabled = env::http_basic_auth_password().is_some();

    Ok(Json(HealthBody {
        time,
        basic_auth_enabled,
        commit_sha,
    }))
}
