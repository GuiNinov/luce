use crate::handlers::TaskHandler;
use crate::protocol::{McpRequest, McpResponse};
use anyhow::Result;
use async_stream::stream;
use axum::{
    extract::{Query, State},
    http::{
        header::{ACCEPT, ORIGIN},
        HeaderMap, HeaderValue, StatusCode,
    },
    response::{IntoResponse, Response, Sse},
    routing::{delete, get, post},
    Json, Router,
};
use futures_util::Stream;
use serde::Deserialize;
use std::{collections::HashMap, convert::Infallible, sync::Arc, time::Duration};
use tokio::sync::{broadcast, Mutex, RwLock};
use tower_http::cors::CorsLayer;
use uuid::Uuid;

const PROTOCOL_VERSION: &str = "2025-11-25";
const SESSION_HEADER: &str = "MCP-Session-Id";
const PROTOCOL_VERSION_HEADER: &str = "MCP-Protocol-Version";

#[derive(Clone)]
pub struct HttpServer {
    sessions: Arc<RwLock<HashMap<String, Session>>>,
    cors_origins: Vec<String>,
}

#[derive(Clone)]
struct Session {
    handler: Arc<Mutex<TaskHandler>>,
    event_sender: broadcast::Sender<SseEvent>,
    last_event_id: Arc<Mutex<u64>>,
}

#[derive(Clone, Debug)]
struct SseEvent {
    id: String,
    data: String,
    event_type: Option<String>,
    retry: Option<Duration>,
}

#[derive(Deserialize)]
struct SseQuery {
    #[serde(rename = "Last-Event-ID")]
    #[allow(dead_code)]
    last_event_id: Option<String>,
}

impl HttpServer {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            cors_origins: vec!["http://localhost:3000".to_string()],
        }
    }

    pub fn with_cors_origins(mut self, origins: Vec<String>) -> Self {
        self.cors_origins = origins;
        self
    }

    pub fn router(&self) -> Router {
        Router::new()
            .route("/mcp", post(Self::handle_post))
            .route("/mcp", get(Self::handle_get))
            .route("/mcp", delete(Self::handle_delete))
            .with_state(self.clone())
            .layer(CorsLayer::permissive())
    }

    async fn validate_origin(&self, headers: &HeaderMap) -> Result<(), StatusCode> {
        if let Some(origin) = headers.get(ORIGIN) {
            let origin_str = origin.to_str().map_err(|_| StatusCode::BAD_REQUEST)?;
            if !self.cors_origins.contains(&origin_str.to_string()) {
                return Err(StatusCode::FORBIDDEN);
            }
        }
        Ok(())
    }

    async fn validate_protocol_version(headers: &HeaderMap) -> Result<(), StatusCode> {
        if let Some(version) = headers.get(PROTOCOL_VERSION_HEADER) {
            let version_str = version.to_str().map_err(|_| StatusCode::BAD_REQUEST)?;
            if version_str != PROTOCOL_VERSION {
                return Err(StatusCode::BAD_REQUEST);
            }
        }
        Ok(())
    }

    async fn get_or_create_session(&self, session_id: Option<&str>) -> Result<String, StatusCode> {
        match session_id {
            Some(id) => {
                let sessions = self.sessions.read().await;
                if sessions.contains_key(id) {
                    Ok(id.to_string())
                } else {
                    Err(StatusCode::NOT_FOUND)
                }
            }
            None => {
                let session_id = Uuid::new_v4().to_string();
                let (sender, _) = broadcast::channel(100);
                let session = Session {
                    handler: Arc::new(Mutex::new(TaskHandler::new())),
                    event_sender: sender,
                    last_event_id: Arc::new(Mutex::new(0)),
                };

                let mut sessions = self.sessions.write().await;
                sessions.insert(session_id.clone(), session);
                Ok(session_id)
            }
        }
    }

    async fn handle_post(
        State(server): State<HttpServer>,
        headers: HeaderMap,
        Json(request): Json<McpRequest>,
    ) -> Result<Response, StatusCode> {
        server.validate_origin(&headers).await?;
        Self::validate_protocol_version(&headers).await?;

        let session_id_header = headers.get(SESSION_HEADER);
        let session_id = session_id_header.and_then(|h| h.to_str().ok());

        let session_id = server.get_or_create_session(session_id).await?;

        let accept_header = headers.get(ACCEPT);
        let wants_sse = accept_header
            .and_then(|h| h.to_str().ok())
            .map(|s| s.contains("text/event-stream"))
            .unwrap_or(false);

        let session = {
            let sessions = server.sessions.read().await;
            sessions
                .get(&session_id)
                .cloned()
                .ok_or(StatusCode::NOT_FOUND)?
        };

        // Handle the request
        let response = {
            let mut handler = session.handler.lock().await;
            handler.handle_request(request)
        };

        if wants_sse {
            // Return SSE stream
            let stream = server
                .create_sse_stream(session_id.clone(), Some(response))
                .await;
            let mut response = Sse::new(stream).into_response();

            if session_id_header.is_none() {
                response.headers_mut().insert(
                    SESSION_HEADER,
                    HeaderValue::from_str(&session_id)
                        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
                );
            }

            Ok(response)
        } else {
            // Return JSON response
            let mut json_response = Json(response).into_response();

            if session_id_header.is_none() {
                json_response.headers_mut().insert(
                    SESSION_HEADER,
                    HeaderValue::from_str(&session_id)
                        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
                );
            }

            Ok(json_response)
        }
    }

    async fn handle_get(
        State(server): State<HttpServer>,
        headers: HeaderMap,
        Query(_query): Query<SseQuery>,
    ) -> Result<Response, StatusCode> {
        server.validate_origin(&headers).await?;
        Self::validate_protocol_version(&headers).await?;

        let session_id = headers
            .get(SESSION_HEADER)
            .and_then(|h| h.to_str().ok())
            .ok_or(StatusCode::BAD_REQUEST)?;

        let accept_header = headers.get(ACCEPT);
        let accepts_sse = accept_header
            .and_then(|h| h.to_str().ok())
            .map(|s| s.contains("text/event-stream"))
            .unwrap_or(false);

        if !accepts_sse {
            return Err(StatusCode::METHOD_NOT_ALLOWED);
        }

        // Validate session exists
        {
            let sessions = server.sessions.read().await;
            if !sessions.contains_key(session_id) {
                return Err(StatusCode::NOT_FOUND);
            }
        }

        let stream = server.create_sse_stream(session_id.to_string(), None).await;
        Ok(Sse::new(stream).into_response())
    }

    async fn handle_delete(
        State(server): State<HttpServer>,
        headers: HeaderMap,
    ) -> Result<StatusCode, StatusCode> {
        server.validate_origin(&headers).await?;
        Self::validate_protocol_version(&headers).await?;

        let session_id = headers
            .get(SESSION_HEADER)
            .and_then(|h| h.to_str().ok())
            .ok_or(StatusCode::BAD_REQUEST)?;

        let mut sessions = server.sessions.write().await;
        if sessions.remove(session_id).is_some() {
            Ok(StatusCode::NO_CONTENT)
        } else {
            Err(StatusCode::NOT_FOUND)
        }
    }

    async fn create_sse_stream(
        &self,
        session_id: String,
        initial_response: Option<McpResponse>,
    ) -> impl Stream<Item = Result<axum::response::sse::Event, Infallible>> {
        let session = {
            let sessions = self.sessions.read().await;
            sessions.get(&session_id).cloned()
        };

        stream! {
            if let Some(session) = session {
                // Send initial event ID to prime reconnection
                let mut event_id = {
                    let mut counter = session.last_event_id.lock().await;
                    *counter += 1;
                    *counter
                };

                yield Ok(axum::response::sse::Event::default()
                    .id(event_id.to_string())
                    .data(""));

                // Send initial response if provided
                if let Some(response) = initial_response {
                    let mut event_id_counter = session.last_event_id.lock().await;
                    *event_id_counter += 1;
                    event_id = *event_id_counter;

                    let data = serde_json::to_string(&response).unwrap_or_default();
                    yield Ok(axum::response::sse::Event::default()
                        .id(event_id.to_string())
                        .data(data));
                }

                // Listen for additional events
                let mut receiver = session.event_sender.subscribe();

                loop {
                    match receiver.recv().await {
                        Ok(event) => {
                            let mut sse_event = axum::response::sse::Event::default()
                                .id(&event.id)
                                .data(&event.data);

                            if let Some(event_type) = &event.event_type {
                                sse_event = sse_event.event(event_type);
                            }

                            if let Some(retry) = event.retry {
                                sse_event = sse_event.retry(retry);
                            }

                            yield Ok(sse_event);
                        }
                        Err(broadcast::error::RecvError::Lagged(_)) => {
                            // Client is behind, continue
                            continue;
                        }
                        Err(broadcast::error::RecvError::Closed) => {
                            // Channel closed, end stream
                            break;
                        }
                    }
                }
            }
        }
    }

    pub async fn run(&self, addr: &str) -> Result<()> {
        let listener = tokio::net::TcpListener::bind(addr).await?;
        println!("MCP HTTP server listening on {}", addr);

        axum::serve(listener, self.router()).await?;

        Ok(())
    }
}

impl Default for HttpServer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use serde_json::json;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_http_server_creation() {
        let server = HttpServer::new();
        let _router = server.router();
        // Just verify it can be created without panicking
    }

    #[tokio::test]
    async fn test_post_json_response() {
        let server = HttpServer::new();
        let router = server.router();

        let request_body = json!({
            "method": "tasks/list"
        });

        let request = Request::builder()
            .method("POST")
            .uri("/mcp")
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .header("MCP-Protocol-Version", PROTOCOL_VERSION)
            .body(Body::from(request_body.to_string()))
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // Should have session header
        assert!(response.headers().contains_key(SESSION_HEADER));
    }

    #[tokio::test]
    async fn test_invalid_protocol_version() {
        let server = HttpServer::new();
        let router = server.router();

        let request_body = json!({
            "method": "tasks/list"
        });

        let request = Request::builder()
            .method("POST")
            .uri("/mcp")
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .header("MCP-Protocol-Version", "invalid-version")
            .body(Body::from(request_body.to_string()))
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_session_management() {
        let server = HttpServer::new();
        let router = server.router();

        // First request should create a session
        let request_body = json!({
            "method": "tasks/list"
        });

        let request = Request::builder()
            .method("POST")
            .uri("/mcp")
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .header("MCP-Protocol-Version", PROTOCOL_VERSION)
            .body(Body::from(request_body.to_string()))
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let session_id = response
            .headers()
            .get(SESSION_HEADER)
            .and_then(|h| h.to_str().ok())
            .unwrap();

        // Second request with session ID should work
        let router2 = server.router();
        let request2 = Request::builder()
            .method("POST")
            .uri("/mcp")
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .header("MCP-Protocol-Version", PROTOCOL_VERSION)
            .header("MCP-Session-Id", session_id)
            .body(Body::from(request_body.to_string()))
            .unwrap();

        let response2 = router2.oneshot(request2).await.unwrap();
        assert_eq!(response2.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_delete_session() {
        let server = HttpServer::new();

        // Create a session first
        let session_id = server.get_or_create_session(None).await.unwrap();

        let router = server.router();
        let request = Request::builder()
            .method("DELETE")
            .uri("/mcp")
            .header("MCP-Protocol-Version", PROTOCOL_VERSION)
            .header("MCP-Session-Id", &session_id)
            .body(Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn test_get_sse_without_session() {
        let server = HttpServer::new();
        let router = server.router();

        let request = Request::builder()
            .method("GET")
            .uri("/mcp")
            .header("Accept", "text/event-stream")
            .header("MCP-Protocol-Version", PROTOCOL_VERSION)
            .body(Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
