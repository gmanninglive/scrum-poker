pub mod session;
mod ws;

use axum::{
    extract::State,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use serde_json::json;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use ws::ws_handler;

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(home))
        .nest("/session", session::router())
        .route("/ws", get(ws_handler))
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()))
}

async fn home(State(state): State<AppState>) -> impl IntoResponse {
    let data = json!({
        "title": "Example 1",
        "parent": "layout"
    });

    Html(state.views.render("index", &data).unwrap())
}
