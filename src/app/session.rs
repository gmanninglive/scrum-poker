use std::{collections::HashSet, net::SocketAddr, sync::Arc};

use axum::{
    extract::{ConnectInfo, Path, State},
    http::{
        header::{LOCATION, SET_COOKIE},
        HeaderName, StatusCode,
    },
    response::{AppendHeaders, Html, IntoResponse},
    routing::{get, post},
    Form, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

use crate::prelude::*;
use crate::AppState;

use super::render_html_template;

pub const USER_COOKIE: &str = "sp_user";
fn set_user_cookie(user_name: String) -> (HeaderName, String) {
    (SET_COOKIE, format!("{USER_COOKIE}={user_name}"))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(new_session))
        .route("/session", post(create_session))
        .route("/session/:id", get(get_session).delete(delete_session))
}

async fn new_session(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    render_html_template(&state.view_env, "index", ())
}

#[derive(Debug, Deserialize)]
struct CreateSession {
    display_name: String,
}

async fn create_session(
    State(state): State<Arc<AppState>>,
    ConnectInfo(_addr): ConnectInfo<SocketAddr>,
    Form(form): Form<CreateSession>,
) -> impl IntoResponse {
    let mut sessions = state.sessions.lock().await;

    let session_id = Uuid::new_v4();

    let (tx, _rx) = broadcast::channel(100);

    sessions.insert(
        session_id,
        Session {
            user_set: RwLock::new(HashSet::new()),
            tx,
        },
    );

    let location = format!("/session/{session_id}");

    (
        StatusCode::SEE_OTHER,
        AppendHeaders([(LOCATION, location), set_user_cookie(form.display_name)]),
    )
}

async fn delete_session() -> impl IntoResponse {
    StatusCode::OK
}

#[derive(Debug)]
pub struct Session {
    pub user_set: RwLock<HashSet<String>>,
    // Channel used to send messages to all connected clients.
    pub tx: broadcast::Sender<String>,
}

#[derive(Serialize)]
struct PageData {
    members: Vec<String>,
}

async fn get_session(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Html<String>> {
    if let Some(session) = state.sessions.lock().await.get(&id) {
        let user_vec = session
            .user_set
            .read()
            .await
            .iter()
            .map(|s| s.to_owned())
            .collect::<Vec<_>>();

        let _data = json!(PageData { members: user_vec });

        let data = json!({
            "title": "Example 1",
            "parent": "layout",
            "cards": [1, 2, 3, 5, 8, 12],
        });

        render_html_template(&state.view_env, "session_show", data)
    } else {
        Err(Error::NotFound("Not Found"))
    }
}
