use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use app::session::Session;

use tokio::sync::{Mutex};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
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
                .with_state(Arc::new(AppState {
                    views: views::init(),
                    sessions: Mutex::new(HashMap::new()),
                }))
                .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()))
                .into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
}

// Define your application shared state
pub struct AppState {
    pub views: handlebars::Handlebars<'static>,
    pub sessions: Mutex<HashMap<Uuid, Session>>,
}
