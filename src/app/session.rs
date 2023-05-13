use axum::{
    extract::{Path, State},
    headers::Location,
    http::{HeaderValue, StatusCode},
    response::{Html, IntoResponse, Redirect},
    routing::{delete, get, post},
    Form, Router, TypedHeader,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::prelude::*;
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(new_session))
        .route("/create", post(create_session))
        .route("/delete", delete(delete_session))
        .route("/:id", get(get_session))
}

async fn new_session(State(state): State<AppState>) -> impl IntoResponse {
    let data = json!({
        "title": "Example 1",
        "parent": "layout"
    });

    Html(state.views.render("session_form", &data).unwrap())
}

async fn create_session(
    State(state): State<AppState>,
    Form(form): Form<Member>,
) -> impl IntoResponse {
    let mut sessions = state.sessions.write().unwrap();
    let id = Uuid::new_v4();
    sessions.insert(
        id,
        Session {
            members: vec![Member {
                display_name: form.display_name,
            }],
        },
    );

    let location = format!("/session/{id}");

    Redirect::to(&location)
}

async fn delete_session() -> impl IntoResponse {
    StatusCode::OK
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Session {
    members: Vec<Member>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Member {
    display_name: String,
}

#[axum::debug_handler]
async fn get_session(State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<Html<String>> {
    let sessions = state.sessions.read().unwrap();

    if let Some(session) = sessions.get(&id) {
        let data = json!({
            "title": "Example 1",
            "parent": "layout",
            "cards": [1, 2, 3, 5, 8, 12],
            "members": session.members
        });

        Ok(Html(state.views.render("session_show", &data).unwrap()))
    } else {
        Err(Error::NotFound("Not Found"))
    }
}
