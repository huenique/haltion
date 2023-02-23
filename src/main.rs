mod app;
pub mod config;
pub mod models;
pub mod routes;
pub mod services;
pub mod structs;
pub mod utils;

use std::env;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    // Load environment variables from .env file if it exists
    if let Err(_) = dotenvy::dotenv() {
        // If .env file does not exist, load environment variables from system environment
        let _ = env::vars();
    }

    let app = app::create_app().await;

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    println!("listening on http://{}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
