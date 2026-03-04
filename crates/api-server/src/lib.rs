//! API Server — Axum HTTP + WebSocket pour Apophy.
//!
//! Expose le workflow hybride, la mémoire et le routeur via REST.

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

/// État partagé de l'API.
pub struct AppState {
    pub memory: MemoryStore,
}

/// Status de santé de l'API.
#[derive(Serialize)]
pub struct HealthStatus {
    pub status: String,
    pub version: String,
    pub crates: Vec<String>,
}

/// Requête d'insertion mémoire.
#[derive(Deserialize)]
pub struct InsertFragmentRequest {
    pub id: String,
    pub session_id: String,
    pub content: String,
}

/// Requête de recherche.
#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
}

/// Réponse standard.
#[derive(Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub ok: bool,
    pub data: T,
}

/// Info du routeur.
#[derive(Serialize)]
pub struct RouterInfo {
    pub models: Vec<ModelInfo>,
    pub thermal_limit: u32,
    pub gpu_temp: u32,
}

#[derive(Serialize)]
pub struct ModelInfo {
    pub name: String,
    pub role: String,
    pub resource: String,
    pub max_tokens: usize,
}

/// Crée le routeur Axum principal.
pub fn create_router() -> Router {
    let router = Router::new()
        .route("/health", get(health))
        .route("/api/v1/router/info", get(router_info))
        .route("/api/v1/router/models", get(router_models));

    router
}

/// Crée le routeur avec state mémoire (pour tests ou production).
pub fn create_router_with_memory(state: Arc<Mutex<AppState>>) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/v1/router/info", get(router_info))
        .route("/api/v1/router/models", get(router_models))
        .route("/api/v1/memory/fragments", post(insert_fragment))
        .route("/api/v1/memory/fragments/:id", get(get_fragment))
        .route("/api/v1/memory/fragments/:id", delete(delete_fragment))
        .route("/api/v1/memory/sessions", get(list_sessions))
        .route("/api/v1/memory/sessions/:session_id/fragments", get(get_session_fragments))
        .route("/api/v1/memory/search", get(search_fragments))
        .route("/api/v1/memory/count", get(count_fragments))
        .with_state(state)
}

async fn health() -> Json<HealthStatus> {
    Json(HealthStatus {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        crates: vec![
            "toon-core".to_string(),
            "memory-engine".to_string(),
            "llm-router".to_string(),
            "prompt-engine".to_string(),
            "refine-loop".to_string(),
            "agent-runtime".to_string(),
            "api-server".to_string(),
        ],
    })
}

async fn router_info() -> Json<RouterInfo> {
    let reg = llm_router::ModelRegistry::apophy_default();
    let gpu_temp = llm_router::thermal::read_gpu_temp();

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
        thermal_limit: reg.thermal_limit(),
        gpu_temp,
    })
}

async fn router_models() -> Json<ApiResponse<Vec<ModelInfo>>> {
    let reg = llm_router::ModelRegistry::apophy_default();
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
