use std::net::SocketAddr;

use axum::Server;

pub use self::error::{Error, Result};

mod env;
mod error;
mod signal;
mod web;

#[tokio::main]
async fn main() {
    println!("Starting server...");

    let addr = if env::docker() {
        env::print();
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
