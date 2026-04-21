use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use std::sync::Arc;
use tower_http::services::ServeDir;

use crate::models::{Project, ProjectsResponse};

pub type SharedState = Arc<Vec<Project>>;

pub fn app(state: SharedState) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/api/projects", get(projects))
        .nest_service("/static", ServeDir::new("static"))
        .with_state(state)
}

async fn index() -> impl IntoResponse {
    match tokio::fs::read_to_string("static/index.html").await {
        Ok(html) => (
            StatusCode::OK,
            [("content-type", "text/html; charset=utf-8")],
            html,
        )
            .into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "index.html missing").into_response(),
    }
}

async fn projects(State(projects): State<SharedState>) -> Json<ProjectsResponse> {
    Json(ProjectsResponse {
        projects: (*projects).clone(),
    })
}
