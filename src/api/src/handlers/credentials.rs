use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post, put},
    Router,
};
use luce_shared::{CredentialData, IntegrationType};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::services::TaskService;

#[derive(Serialize, Deserialize)]
pub struct CreateCredentialRequest {
    pub name: String,
    pub integration_type: IntegrationType,
    pub credential_data: CredentialData,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateCredentialRequest {
    pub name: Option<String>,
    pub active: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListCredentialsQuery {
    pub integration_type: Option<IntegrationType>,
    pub include_inactive: Option<bool>,
}

pub fn credential_routes() -> Router {
    Router::new()
        .route("/credentials", get(list_credentials))
        .route("/credentials", post(create_credential))
        .route("/credentials/:id", get(get_credential))
        .route("/credentials/:id", put(update_credential))
        .route("/credentials/:id", delete(delete_credential))
        .route("/credentials/:id/test", post(test_credential))
}

async fn list_credentials(
    Extension(_service): Extension<Arc<TaskService>>,
    Query(_params): Query<ListCredentialsQuery>,
) -> Result<Json<Vec<serde_json::Value>>, (StatusCode, Json<serde_json::Value>)> {
    // TODO: Implement credential listing
    // For now, return empty list
    Ok(Json(vec![]))
}

async fn create_credential(
    Extension(_service): Extension<Arc<TaskService>>,
    Json(_request): Json<CreateCredentialRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    // TODO: Implement credential creation
    Err((
        StatusCode::NOT_IMPLEMENTED,
        Json(serde_json::json!({
            "error": "Credential creation not yet implemented"
        })),
    ))
}

async fn get_credential(
    Extension(_service): Extension<Arc<TaskService>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let _credential_id = parse_credential_id(&id)?;
    
    // TODO: Implement credential retrieval
    Err((
        StatusCode::NOT_IMPLEMENTED,
        Json(serde_json::json!({
            "error": "Credential retrieval not yet implemented"
        })),
    ))
}

async fn update_credential(
    Extension(_service): Extension<Arc<TaskService>>,
    Path(id): Path<String>,
    Json(_request): Json<UpdateCredentialRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let _credential_id = parse_credential_id(&id)?;
    
    // TODO: Implement credential update
    Err((
        StatusCode::NOT_IMPLEMENTED,
        Json(serde_json::json!({
            "error": "Credential update not yet implemented"
        })),
    ))
}

async fn delete_credential(
    Extension(_service): Extension<Arc<TaskService>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let _credential_id = parse_credential_id(&id)?;
    
    // TODO: Implement credential deletion
    Err((
        StatusCode::NOT_IMPLEMENTED,
        Json(serde_json::json!({
            "error": "Credential deletion not yet implemented"
        })),
    ))
}

async fn test_credential(
    Extension(_service): Extension<Arc<TaskService>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let _credential_id = parse_credential_id(&id)?;
    
    // TODO: Implement credential testing
    Err((
        StatusCode::NOT_IMPLEMENTED,
        Json(serde_json::json!({
            "error": "Credential testing not yet implemented"
        })),
    ))
}

fn parse_credential_id(id: &str) -> Result<Uuid, (StatusCode, Json<serde_json::Value>)> {
    Uuid::parse_str(id).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Invalid credential ID format"
            })),
        )
    })
}