use std::env;

pub fn commit_sha() -> String {
    env::var("COMMIT_SHA").unwrap_or_else(|_| "unknown".to_string())
}

pub fn docker() -> bool {
    let var = env::var("DOCKER");
    var.is_ok() && var.unwrap() == "true"
}

pub fn http_basic_auth_password() -> Option<String> {
    env::var("HTTP_BASIC_AUTH_PASSWORD").ok()
}

pub fn openai_api_key() -> Option<String> {
    env::var("OPENAI_API_KEY").ok()
}
