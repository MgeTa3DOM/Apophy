//! API Server — Axum HTTP for Apophy.
//!
//! RESTful API for workflow execution, memory management, and model routing.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{delete, get, post},
    Json, Router,
};
use memory_engine::store::{MemoryFragment, MemoryStore};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Shared application state.
pub struct AppState {
    pub memory: MemoryStore,
}

/// Health check response.
#[derive(Serialize)]
pub struct HealthStatus {
    pub status: String,
    pub version: String,
}

/// Fragment insertion request.
#[derive(Deserialize)]
pub struct InsertFragmentRequest {
    pub id: String,
    pub session_id: String,
    pub content: String,
}

/// Search query parameters.
#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
}

/// Standard API response envelope.
#[derive(Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub ok: bool,
    pub data: T,
}

/// Model info for API response.
#[derive(Serialize)]
pub struct ModelInfo {
    pub name: String,
    pub role: String,
    pub resource: String,
    pub max_tokens: usize,
}

/// Router info response.
#[derive(Serialize)]
pub struct RouterInfo {
    pub models: Vec<ModelInfo>,
    pub total_memory_mb: u32,
}

/// Creates the base router (stateless endpoints).
pub fn create_router() -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/v1/router/info", get(router_info))
        .route("/api/v1/router/models", get(router_models))
}

/// Creates the full router with memory state.
pub fn create_router_with_memory(state: Arc<Mutex<AppState>>) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/v1/router/info", get(router_info))
        .route("/api/v1/router/models", get(router_models))
        .route("/api/v1/memory/fragments", post(insert_fragment))
        .route("/api/v1/memory/fragments/:id", get(get_fragment))
        .route("/api/v1/memory/fragments/:id", delete(delete_fragment))
        .route("/api/v1/memory/sessions", get(list_sessions))
        .route(
            "/api/v1/memory/sessions/:session_id/fragments",
            get(get_session_fragments),
        )
        .route("/api/v1/memory/search", get(search_fragments))
        .route("/api/v1/memory/count", get(count_fragments))
        .with_state(state)
}

async fn health() -> Json<HealthStatus> {
    Json(HealthStatus {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

async fn router_info() -> Json<RouterInfo> {
    let reg = llm_router::ModelRegistry::with_defaults();
    let models = reg
        .all()
        .iter()
        .map(|m| ModelInfo {
            name: m.name.clone(),
            role: m.role.clone(),
            resource: format!("{:?}", m.resource),
            max_tokens: m.max_tokens,
        })
        .collect();

    Json(RouterInfo {
        models,
        total_memory_mb: reg.total_memory_mb(),
    })
}

async fn router_models() -> Json<ApiResponse<Vec<ModelInfo>>> {
    let reg = llm_router::ModelRegistry::with_defaults();
    let models = reg
        .all()
        .iter()
        .map(|m| ModelInfo {
            name: m.name.clone(),
            role: m.role.clone(),
            resource: format!("{:?}", m.resource),
            max_tokens: m.max_tokens,
        })
        .collect();

    Json(ApiResponse {
        ok: true,
        data: models,
    })
}

async fn insert_fragment(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(req): Json<InsertFragmentRequest>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    let state = state.lock().await;
    let fragment = MemoryFragment {
        id: req.id.clone(),
        session_id: req.session_id,
        content: req.content,
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    state
        .memory
        .insert(&fragment)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse {
        ok: true,
        data: req.id,
    }))
}

async fn get_fragment(
    State(state): State<Arc<Mutex<AppState>>>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<Option<MemoryFragment>>>, StatusCode> {
    let state = state.lock().await;
    let fragment = state
        .memory
        .get_by_id(&id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse {
        ok: true,
        data: fragment,
    }))
}

async fn delete_fragment(
    State(state): State<Arc<Mutex<AppState>>>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<bool>>, StatusCode> {
    let state = state.lock().await;
    let deleted = state
        .memory
        .delete(&id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse {
        ok: true,
        data: deleted,
    }))
}

async fn list_sessions(
    State(state): State<Arc<Mutex<AppState>>>,
) -> Result<Json<ApiResponse<Vec<String>>>, StatusCode> {
    let state = state.lock().await;
    let sessions = state
        .memory
        .list_sessions()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse {
        ok: true,
        data: sessions,
    }))
}

async fn get_session_fragments(
    State(state): State<Arc<Mutex<AppState>>>,
    Path(session_id): Path<String>,
) -> Result<Json<ApiResponse<Vec<MemoryFragment>>>, StatusCode> {
    let state = state.lock().await;
    let fragments = state
        .memory
        .get_by_session(&session_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse {
        ok: true,
        data: fragments,
    }))
}

async fn search_fragments(
    State(state): State<Arc<Mutex<AppState>>>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<ApiResponse<Vec<MemoryFragment>>>, StatusCode> {
    let state = state.lock().await;
    let results = state
        .memory
        .search(&query.q)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse {
        ok: true,
        data: results,
    }))
}

async fn count_fragments(
    State(state): State<Arc<Mutex<AppState>>>,
) -> Result<Json<ApiResponse<usize>>, StatusCode> {
    let state = state.lock().await;
    let count = state
        .memory
        .count()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse {
        ok: true,
        data: count,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    #[tokio::test]
    async fn health_endpoint() {
        let app = create_router();
        let req = Request::builder()
            .uri("/health")
            .body(Body::empty())
            .expect("request build failed");

        let resp = app.oneshot(req).await.expect("request failed");
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn router_info_endpoint() {
        let app = create_router();
        let req = Request::builder()
            .uri("/api/v1/router/info")
            .body(Body::empty())
            .expect("request build failed");

        let resp = app.oneshot(req).await.expect("request failed");
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn router_models_endpoint() {
        let app = create_router();
        let req = Request::builder()
            .uri("/api/v1/router/models")
            .body(Body::empty())
            .expect("request build failed");

        let resp = app.oneshot(req).await.expect("request failed");
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
