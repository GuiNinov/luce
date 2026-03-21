use axum::{
    http::{
        header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
        HeaderValue, Method,
    },
    Router,
};
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use tracing::Level;

use crate::handlers::tasks::task_routes;

pub struct ApiServer {
    addr: SocketAddr,
}

impl ApiServer {
    pub fn new(addr: SocketAddr) -> Self {
        Self { addr }
    }

    pub fn router() -> Router {
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
            .layer(ServiceBuilder::new().layer(trace).layer(cors))
    }

    pub async fn serve(self) -> anyhow::Result<()> {
        let app = Self::router();

        tracing::info!("Starting Luce API server on {}", self.addr);

        let listener = tokio::net::TcpListener::bind(self.addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}

pub async fn start_server(addr: SocketAddr) -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    let server = ApiServer::new(addr);
    server.serve().await
}
