use std::convert::Infallible;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::response::IntoResponse;
use tower_http::services::{ServeDir, ServeFile};
use tower::ServiceExt;
use super::ApiAdapter;

pub struct AxumAdapter {
    router: axum::Router,
}

impl AxumAdapter {
    pub fn new() -> Self {
        AxumAdapter {
            router: axum::Router::new(),
        }
    }

    pub fn register(mut self, adapter: impl ApiAdapter) -> Self {
        self.router = self.router.merge(adapter.routes());
        self
    }

    pub fn serve_directory(self, route_path: &str, static_dir: &str) -> Self {
        AxumAdapter {
            router: self.router.nest_service(route_path, ServeDir::new(static_dir)),
        }
    }

    pub fn serve_spa(self, static_dir: &str) -> Self {
        let fallback = ServeFile::new(format!("{static_dir}/index.html"));
        let serve_dir = ServeDir::new(static_dir).fallback(fallback);
        let static_service = tower::service_fn(move |request: Request<Body>| {
            let serve_dir = serve_dir.clone();
            async move {
                if request.uri().path().starts_with("/auth") {
                    return Ok::<_, Infallible>(StatusCode::NOT_FOUND.into_response());
                }

                let response = serve_dir
                    .oneshot(request)
                    .await
                    .map(|response| response.into_response())
                    .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response());
                Ok::<_, Infallible>(response)
            }
        });

        AxumAdapter {
            router: self.router.fallback_service(static_service),
        }
    }

    pub async fn serve(self) {
        let host = common::domain::EnvVar::ServerHost.value();
        let port = common::domain::EnvVar::ServerPort.value();
        let addr = format!("{host}:{port}");

        let listener = tokio::net::TcpListener::bind(&addr)
            .await
            .unwrap_or_else(|_| panic!("Failed to bind to {addr}"));

        println!("Server listening on {addr}");

        axum::serve(listener, self.router)
            .await
            .expect("Server failed");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use tower::ServiceExt;
    use crate::adapters::AuthApiAdapter;
    use crate::adapters::db::SqliteAdapter;
    use crate::test_helpers::{FakeAuthDrivingPort, FakeLoggerDrivingPort};

    #[test]
    fn axum_adapter_exists() {
        let _ = AxumAdapter::new();
    }

    #[test]
    fn register_accepts_api_adapter() {
        struct MockAdapter;
        impl ApiAdapter for MockAdapter {
            fn routes(self) -> axum::Router {
                axum::Router::new()
            }
        }

        let _ = AxumAdapter::new()
            .register(MockAdapter);
    }

    #[tokio::test]
    async fn serve_spa_does_not_handle_auth_paths() {
        let app = AxumAdapter::new()
            .register(AuthApiAdapter::new(
                FakeAuthDrivingPort::new(),
                FakeLoggerDrivingPort::new(),
                SqliteAdapter::new(":memory:").unwrap(),
            ))
            .serve_spa("frontend/build")
            .router;

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/auth/unknown")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert!(body.is_empty());
    }

    #[tokio::test]
    async fn serve_spa_keeps_auth_me_route_active() {
        let app = AxumAdapter::new()
            .register(AuthApiAdapter::new(
                FakeAuthDrivingPort::new(),
                FakeLoggerDrivingPort::new(),
                SqliteAdapter::new(":memory:").unwrap(),
            ))
            .serve_spa("frontend/build")
            .router;

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/auth/me")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
