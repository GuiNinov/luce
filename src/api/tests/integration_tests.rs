use axum::{
    body::Body,
    http::{Request, StatusCode},
    response::Response,
};
use luce_api::server::ApiServer;
use serde_json::{json, Value};
use tower::util::ServiceExt;

async fn send_request(
    app: axum::Router,
    method: &str,
    uri: &str,
    body: Option<Value>,
) -> (StatusCode, String) {
    let mut request_builder = Request::builder().method(method).uri(uri);

    let request = if let Some(body) = body {
        request_builder = request_builder.header("content-type", "application/json");
        request_builder
            .body(Body::from(serde_json::to_string(&body).unwrap()))
            .unwrap()
    } else {
        request_builder.body(Body::empty()).unwrap()
    };

    let response: Response = app.oneshot(request).await.unwrap();
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    (status, body_str)
}

#[tokio::test]
async fn test_health_check() {
    let app = ApiServer::router();
    let (status, body) = send_request(app, "GET", "/api/v1/health", None).await;

    assert_eq!(status, StatusCode::OK);
    let response: Value = serde_json::from_str(&body).unwrap();
    assert_eq!(response["status"], "healthy");
    assert!(response.get("timestamp").is_some());
}

#[tokio::test]
async fn test_create_task() {
    let app = ApiServer::router();
    let create_request = json!({
        "title": "Test Task",
        "description": "A test task",
        "dependencies": []
    });

    let (status, body) = send_request(app, "POST", "/api/v1/tasks", Some(create_request)).await;

    assert_eq!(status, StatusCode::CREATED);
    let response: Value = serde_json::from_str(&body).unwrap();
    assert_eq!(response["title"], "Test Task");
    assert_eq!(response["description"], "A test task");
    assert!(response.get("id").is_some());
}

#[tokio::test]
async fn test_create_task_invalid_body() {
    let app = ApiServer::router();
    let invalid_request = json!({
        "invalid_field": "invalid"
    });

    let (status, _) = send_request(app, "POST", "/api/v1/tasks", Some(invalid_request)).await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn test_get_task() {
    let app = ApiServer::router();
    let task_id = "550e8400-e29b-41d4-a716-446655440000";

    let (status, body) =
        send_request(app, "GET", &format!("/api/v1/tasks/{}", task_id), None).await;

    assert_eq!(status, StatusCode::OK);
    let response: Value = serde_json::from_str(&body).unwrap();
    assert!(response.get("id").is_some());
    assert!(response.get("title").is_some());
}

#[tokio::test]
async fn test_get_task_invalid_id() {
    let app = ApiServer::router();
    let invalid_id = "invalid-uuid";

    let (status, _) =
        send_request(app, "GET", &format!("/api/v1/tasks/{}", invalid_id), None).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_update_task() {
    let app = ApiServer::router();
    let task_id = "550e8400-e29b-41d4-a716-446655440000";
    let update_request = json!({
        "title": "Updated Task"
    });

    let (status, body) = send_request(
        app,
        "PUT",
        &format!("/api/v1/tasks/{}", task_id),
        Some(update_request),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let response: Value = serde_json::from_str(&body).unwrap();
    assert!(response.get("id").is_some());
}

#[tokio::test]
async fn test_delete_task() {
    let app = ApiServer::router();
    let task_id = "550e8400-e29b-41d4-a716-446655440000";

    let (status, _) =
        send_request(app, "DELETE", &format!("/api/v1/tasks/{}", task_id), None).await;

    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_list_tasks() {
    let app = ApiServer::router();

    let (status, body) = send_request(app, "GET", "/api/v1/tasks", None).await;

    assert_eq!(status, StatusCode::OK);
    let response: Value = serde_json::from_str(&body).unwrap();
    assert!(response.get("tasks").is_some());
    assert!(response.get("total_count").is_some());
    assert_eq!(response["total_count"], 0);
}

#[tokio::test]
async fn test_get_ready_tasks() {
    let app = ApiServer::router();

    let (status, body) = send_request(app, "GET", "/api/v1/tasks/ready", None).await;

    assert_eq!(status, StatusCode::OK);
    let response: Value = serde_json::from_str(&body).unwrap();
    assert!(response.is_array());
    assert_eq!(response.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_mark_task_completed() {
    let app = ApiServer::router();
    let task_id = "550e8400-e29b-41d4-a716-446655440000";

    let (status, body) = send_request(
        app,
        "POST",
        &format!("/api/v1/tasks/{}/complete", task_id),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let response: Value = serde_json::from_str(&body).unwrap();
    assert!(response.get("id").is_some());
}
