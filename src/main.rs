pub mod config;
pub mod models;
pub mod routes;
pub mod services;
pub mod structs;
pub mod utils;

use std::net::SocketAddr;

mod app;

#[tokio::main]
async fn main() {
    let app = app::create_app().await;

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    println!("listening on http://{}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
