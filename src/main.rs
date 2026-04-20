use fusion_tracker::{data, routes};
use std::net::SocketAddr;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let dataset = data::load_dataset_from_path("data/companies.json");
    let app = routes::app(Arc::new(dataset));

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3000);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    println!("listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
