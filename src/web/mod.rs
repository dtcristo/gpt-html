use axum::{
    routing::{get, get_service},
    Router,
};
use axum_extra::middleware::option_layer;
use tower_http::{services::ServeDir, validate_request::ValidateRequestHeaderLayer};

use crate::env;

pub mod generate;
pub mod health;

pub fn app() -> Router {
    Router::new()
        .route("/health", get(health::handler))
        .nest_service(
            "/",
            // Handle GET.
            get_service(
                // Serve static files from "public".
                ServeDir::new("public")
                    // When static file missing use handler.
                    .fallback(get(generate::handler)),
            )
            // Other methods use handler.
            .post(generate::handler)
            .patch(generate::handler)
            .put(generate::handler)
            .delete(generate::handler)
            // Optionally apply HTTP basic auth.
            .layer(option_layer(env::http_basic_auth_password().map(
                |password| ValidateRequestHeaderLayer::basic("user", &password),
            ))),
        )
}
