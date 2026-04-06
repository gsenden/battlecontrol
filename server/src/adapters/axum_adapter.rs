use tower_http::services::{ServeDir, ServeFile};
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

    pub fn serve_spa(self, static_dir: &str) -> Self {
        let fallback = ServeFile::new(format!("{static_dir}/index.html"));
        let serve_dir = ServeDir::new(static_dir).fallback(fallback);
        AxumAdapter {
            router: self.router.fallback_service(serve_dir),
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
}
