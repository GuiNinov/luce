use axum::{
    http::{
        header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
        HeaderValue, Method,
    },
    Extension, Router,
};
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::{net::SocketAddr, sync::Arc};
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use tracing::Level;

use crate::{
    handlers::{
        attachments::attachment_routes, credentials::credential_routes, 
        integrations::integration_routes, tasks::task_routes,
    },
    services::TaskService,
};

pub struct ApiServer {
    addr: SocketAddr,
    pool: SqlitePool,
}

impl ApiServer {
    pub fn new(addr: SocketAddr, pool: SqlitePool) -> Self {
        Self { addr, pool }
    }

    pub fn router(task_service: Arc<TaskService>) -> Router {
        let cors = CorsLayer::new()
            .allow_origin("*".parse::<HeaderValue>().unwrap())
            .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
            .allow_headers([ACCEPT, AUTHORIZATION, CONTENT_TYPE]);

        let trace = TraceLayer::new_for_http()
            .make_span_with(DefaultMakeSpan::new().include_headers(true))
            .on_request(DefaultOnRequest::new().level(Level::INFO))
            .on_response(DefaultOnResponse::new().level(Level::INFO));

        Router::new()
            .nest("/api/v1", task_routes())
            .nest("/api/v1", attachment_routes())
            .nest("/api/v1", credential_routes())
            .nest("/api/v1", integration_routes())
            .layer(Extension(task_service))
            .layer(ServiceBuilder::new().layer(trace).layer(cors))
    }

    pub async fn serve(self) -> anyhow::Result<()> {
        let task_service = Arc::new(
            TaskService::new(self.pool)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to create task service: {:?}", e))?,
        );
        let app = Self::router(task_service);

        tracing::info!("Starting Luce API server on {}", self.addr);

        let listener = tokio::net::TcpListener::bind(self.addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}

pub async fn start_server(addr: SocketAddr, database_url: Option<&str>) -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();



    let database_url = database_url.map(|s| s.to_string()).unwrap_or_else(|| {
        std::env::current_dir()
            .map(|dir| format!("sqlite:{}/luce.db", dir.display()))
            .unwrap_or_else(|_| "luce.db".to_string())
    });

    tracing::info!("Connecting to database: {}", database_url);
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    // Run migrations
    // sqlx::migrate!("../migrations/migrations")
       // .run(&pool)
        // .await?;

    let server = ApiServer::new(addr, pool);
    server.serve().await
}
