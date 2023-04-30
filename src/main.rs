use std::net::SocketAddr;

use axum::Server;

mod env;
mod error;
mod prelude;
mod signal;
mod web;

#[tokio::main]
async fn main() {
    println!("Starting server...");

    let addr = if env::docker() {
        SocketAddr::from(([0, 0, 0, 0], 8080))
    } else {
        SocketAddr::from(([127, 0, 0, 1], 8080))
    };

    Server::bind(&addr)
        .serve(web::app().into_make_service())
        .with_graceful_shutdown(signal::shutdown())
        .await
        .expect("server should serve");
}
