//! API Server — Axum HTTP + WebSocket pour Apophy.
//!
//! Expose le workflow hybride et la mémoire via REST.

use axum::{routing::get, Json, Router};
use serde::Serialize;

/// Status de santé de l'API.
#[derive(Serialize)]
pub struct HealthStatus {
    pub status: String,
    pub version: String,
}

/// Crée le routeur Axum principal.
pub fn create_router() -> Router {
    Router::new()
        .route("/health", get(health))
}

async fn health() -> Json<HealthStatus> {
    Json(HealthStatus {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
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
}
