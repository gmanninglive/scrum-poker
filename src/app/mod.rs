pub mod session;
mod ws;

use std::sync::Arc;

use axum::{
    extract::State,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use minijinja::Environment;
use serde::Serialize;
use serde_json::json;
use ws::ws_handler;

use crate::prelude::*;
use crate::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/500",
            get(|s| async { render_error(s, Error::ServerError("")).await }),
        )
        .fallback(|s| async { render_error(s, Error::NotFound("")).await })
        .merge(session::router())
        .route("/ws/:id", get(ws_handler))
}

async fn render_error(State(state): State<Arc<AppState>>, error: Error) -> impl IntoResponse {
    let data = match error {
        Error::NotFound(_) => {
            json!({
                "title": "Page not found",
                "message": "Sorry, we couldn't find the page you're looking for.",
                "status": 404})
        }
        _ => {
            json!({
        "title": "Internal Server Error",
        "message": "Sorry, an error occured on the server.",
        "status": 500})
        }
    };

    render_html_template(&state.view_env, "error", data)
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
