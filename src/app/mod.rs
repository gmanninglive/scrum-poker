pub mod session;
mod ws;

use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use minijinja::Environment;
use serde::Serialize;
use tower_http::services::ServeDir;
use ws::ws_handler;

use crate::prelude::*;
use crate::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .fallback(not_found)
        .nest_service("/assets", ServeDir::new("assets"))
        .merge(session::router())
        .route("/ws/:id", get(ws_handler))
        .route("/500", get(server_error))
}

async fn not_found(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        render_html_template(&state.view_env, "404", ()),
    )
}

async fn server_error(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    render_html_template(&state.view_env, "404", ())
}

fn render_html_template<S>(env: &Environment<'_>, name: &str, ctx: S) -> Result<Html<String>>
where
    S: Serialize,
{
    let Ok(template) = env.get_template(name) else {
        return Err(Error::ServerError(
            "Couldn't find MiniJinja template"
        ));
    };

    let rendered = template
        .render(ctx)
        .map_err(|_| Error::ServerError("Failed to render MiniJinja template"))?;

    Ok(Html(rendered))
}
