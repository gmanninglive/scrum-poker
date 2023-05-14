pub mod session;
mod ws;

use std::sync::Arc;

use axum::{
    extract::State,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use serde_json::json;
use tower_http::services::ServeDir;
use ws::ws_handler;

use crate::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .fallback(render_404)
        .nest_service("/assets", ServeDir::new("assets"))
        .merge(session::router())
        .route("/ws/:id", get(ws_handler))
}

async fn render_404(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let data = json!({
        "parent": "layout"
    });

    Html(state.views.render("404", &data).unwrap())
}
