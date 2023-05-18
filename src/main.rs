mod app;
mod prelude;
mod views;

use app::session::Session;
use prelude::*;
use std::{
    collections::{HashMap, HashSet},
    net::SocketAddr,
    sync::Arc,
};
use tokio::sync::{broadcast, Mutex, RwLock};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use uuid::Uuid;

pub struct AppState {
    pub view_env: minijinja::Environment<'static>,
    pub sessions: Mutex<HashMap<Uuid, Session>>,
    pub tx: broadcast::Sender<Job>,
}

#[derive(Debug, Clone)]
pub enum TaskKind {
    KeepAlive,
    QueueDelete,
    Delete,
}

#[derive(Debug, Clone)]
pub enum Job {
    KeepAlive(Uuid),
    QueueDelete(Uuid),
    Drain,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    // Channel for broadcasting session jobs
    let (tx, mut rx) = broadcast::channel::<Job>(100);

    let app_state = Arc::new(AppState {
        view_env: views::init(),
        sessions: Mutex::new(HashMap::new()),
        tx,
    });

    let job_runner = JobRunner {
        state: app_state.clone(),
        queued: Arc::new(RwLock::new(HashSet::new())),
    };

    // spawn drain job once every hour
    tokio::spawn({
        let mut interval_timer =
            tokio::time::interval(chrono::Duration::hours(1).to_std().unwrap());
        let runner = job_runner.clone();
        async move {
            loop {
                // Wait for the next interval tick
                interval_timer.tick().await;
                let _ = runner.run(Job::Drain).await;
            }
        }
    });

    // listen for and execute jobs from receiver
    tokio::spawn({
        async move {
            while let Ok(job) = rx.recv().await {
                let _ = job_runner.run(job).await;
            }
        }
    });

    axum::Server::bind(&"127.0.0.1:3000".parse().unwrap())
        .serve(
            app::router()
                .with_state(app_state)
                .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()))
                .into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
}

#[derive(Clone)]
struct JobRunner {
    state: Arc<AppState>,
    // sessions queued for deletion
    queued: Arc<RwLock<HashSet<Uuid>>>,
}

impl JobRunner {
    async fn run(&self, job: Job) -> Result<()> {
        tracing::debug!("{:?}", job);
        match job {
            Job::Drain => {
                let mut sessions = self.state.sessions.lock().await;
                for id in self.queued.write().await.drain() {
                    tracing::debug!("Deleted: {:?}", id);
                    sessions.remove(&id);
                }
                Ok(())
            }
            Job::KeepAlive(id) => {
                self.queued.write().await.remove(&id);
                Ok(())
            }
            Job::QueueDelete(id) => {
                self.queued.write().await.insert(id);
                Ok(())
            }
        }
    }
}
