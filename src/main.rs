use fusion_tracker::{data, routes};
use std::net::SocketAddr;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let projects = data::load_projects_from_path("data/projects.json");
    let app = routes::app(Arc::new(projects));

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3000);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    println!("listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
