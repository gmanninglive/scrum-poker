use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use app::session::Session;
use uuid::Uuid;

mod app;
mod prelude;
mod views;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    axum::Server::bind(&"127.0.0.1:3000".parse().unwrap())
        .serve(
            app::router()
                .with_state(AppState {
                    views: views::init(),
                    sessions: Arc::new(RwLock::new(HashMap::new())),
                })
                .into_make_service(),
        )
        .await
        .unwrap();
}

// Define your application shared state
#[derive(Clone)]
pub struct AppState {
    pub views: handlebars::Handlebars<'static>,
    pub sessions: Arc<RwLock<HashMap<Uuid, Session>>>,
}
