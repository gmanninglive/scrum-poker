use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    response::IntoResponse,
};

use futures::{sink::SinkExt, stream::StreamExt};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

use crate::AppState;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<Uuid>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| websocket(socket, state, session_id))
}

// This function deals with a single websocket connection, i.e., a single
// connected client / user, for which we will spawn two independent tasks (for
// receiving / sending chat messages).
async fn websocket(stream: WebSocket, state: Arc<AppState>, session_id: Uuid) {
    // By splitting, we can send and receive at the same time.
    let (mut sender, mut receiver) = stream.split();

    // Username gets set in the receive loop, if it's valid.
    let mut username = String::new();
    // Loop until a text message is found.
    while let Some(Ok(message)) = receiver.next().await {
        if let Message::Text(name) = message {
            // If username that is sent by client is not taken, fill username string.
            check_username(&state, &mut username, &name, session_id).await;

            // If not empty we want to quit the loop else we want to quit function.
            if !username.is_empty() {
                break;
            } else {
                // Only send our client that username is taken.
                let _ = sender
                    .send(Message::Text(String::from("Username already taken.")))
                    .await;

                return;
            }
        }
    }

    // We subscribe *before* sending the "joined" message, so that we will also
    // display it to our client.
    let mut rx = state
        .sessions
        .lock()
        .await
        .get(&session_id)
        .unwrap()
        .tx
        .subscribe();

    let user_vec = get_user_vec(&state, session_id);

    // Now send the "joined" message to all subscribers.
    let msg =
        json!({"type": "user_joined", "payload": { "username": username }, "users": user_vec.await })
            .to_string();

    tracing::debug!("{}", msg);

    let _ = state
        .sessions
        .lock()
        .await
        .get(&session_id)
        .unwrap()
        .tx
        .send(msg);

    // Spawn the first task that will receive broadcast messages and send text
    // messages over the websocket to our client.
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            // In any websocket error, break loop.
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    // Clone things we want to pass (move) to the receiving task.
    let tx = state
        .sessions
        .lock()
        .await
        .get(&session_id)
        .unwrap()
        .tx
        .clone();
    let name = username.clone();
    let user_vec = get_user_vec(&state, session_id).await;

    // Spawn a task that takes messages from the websocket, prepends the user
    // name, and sends them to all broadcast subscribers.
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(vote))) = receiver.next().await {
            // Add username before message.
            let _ = tx.send(
                json!({"type": "user_voted", "payload": { "username": name, "vote": vote }, "users": user_vec })
                    .to_string(),
            );
        }
    });

    // If any one of the tasks run to completion, we abort the other.
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };

    // Send "user left" message (similar to "joined" above).
    let msg = format!("{} left.", username);
    tracing::debug!("{}", msg);
    let _ = state
        .sessions
        .lock()
        .await
        .get(&session_id)
        .unwrap()
        .tx
        .send(msg);

    remove_user(&state, session_id, username).await;
}

async fn check_username(state: &Arc<AppState>, string: &mut String, name: &str, session_id: Uuid) {
    let db = state.sessions.lock().await;
    let mut user_set = db.get(&session_id).unwrap().user_set.write().await;

    if !user_set.contains(name) {
        user_set.insert(name.to_owned());

        string.push_str(name);
    }
}

async fn get_user_vec(state: &Arc<AppState>, session_id: Uuid) -> Vec<String> {
    let user_vec = state.sessions.lock().await;
    let user_vec = user_vec.get(&session_id).unwrap().user_set.read().await;
    user_vec.iter().map(|s| s.to_owned()).collect::<Vec<_>>()
}

async fn remove_user(state: &Arc<AppState>, session_id: Uuid, username: String) {
    // Remove username from map so new clients can take it again.
    let sessions = state.sessions.lock().await;
    sessions
        .get(&session_id)
        .unwrap()
        .user_set
        .write()
        .await
        .remove(&username);
}
