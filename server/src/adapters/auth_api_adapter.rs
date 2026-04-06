use std::sync::Arc;
use axum::extract::State;
use axum::Json;
use axum::routing::post;
use common::dto::RegistrationRequestDto;
use super::ApiAdapter;
use crate::ports::{AuthDrivingPort, LoggerDrivingPort};

pub struct AuthApiAdapter<AuthPort: AuthDrivingPort, Logger: LoggerDrivingPort> {
    auth: AuthPort,
    logger: Logger,
}

impl<AuthPort: AuthDrivingPort, Logger: LoggerDrivingPort> AuthApiAdapter<AuthPort, Logger> {
    pub fn new(auth: AuthPort, logger: Logger) -> Self {
        Self { auth, logger }
    }
}

struct AppState<AuthPort: AuthDrivingPort, Logger: LoggerDrivingPort> {
    auth: AuthPort,
    logger: Logger,
}

impl<AuthPort, Logger> ApiAdapter for AuthApiAdapter<AuthPort, Logger>
where
    AuthPort: AuthDrivingPort + Send + Sync + 'static,
    Logger: LoggerDrivingPort + Send + Sync + 'static,
{
    fn routes(self) -> axum::Router {
        axum::Router::new()
            .route(
                common::domain::Resource::AuthUser.path(),
                post(register_user::<AuthPort, Logger>),
            )
            .with_state(Arc::new(AppState {
                auth: self.auth,
                logger: self.logger,
            }))
    }
}

async fn register_user<AuthPort: AuthDrivingPort, Logger: LoggerDrivingPort>(
    State(state): State<Arc<AppState<AuthPort, Logger>>>,
    Json(body): Json<RegistrationRequestDto>,
) -> axum::response::Response {
    match state.auth.register_user(body).await {
        Ok(user) => axum::response::IntoResponse::into_response(Json(user)),
        Err(error) => {
            state.logger.log_error(&error);
            axum::response::IntoResponse::into_response((
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(error),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use tower::ServiceExt;
    use common::domain::ErrorTrait;
    use crate::adapters::ApiAdapter;
    use crate::test_helpers::{FakeAuthDrivingPort, FakeLoggerDrivingPort};
    use crate::test_helpers::sample_data::{TEST_EMAIL, TEST_PLAYER_NAME, TEST_USER_ID};

    fn register_request() -> axum::http::Request<Body> {
        let body = format!(r#"{{"name":"{TEST_PLAYER_NAME}","email":"{TEST_EMAIL}"}}"#);
        axum::http::Request::builder()
            .method("POST")
            .uri(common::domain::Resource::AuthUser.path())
            .header("content-type", "application/json")
            .body(Body::from(body))
            .unwrap()
    }

    async fn post_register_user() -> (axum::http::Response<Body>, FakeAuthDrivingPort) {
        let fake_port = FakeAuthDrivingPort::new();
        let port_clone = fake_port.clone();
        let adapter = AuthApiAdapter { auth: fake_port, logger: FakeLoggerDrivingPort::new() };
        let response = adapter.routes().oneshot(register_request()).await.unwrap();
        (response, port_clone)
    }

    async fn post_register_user_with_error(error: common::domain::Error) -> (axum::http::Response<Body>, FakeLoggerDrivingPort) {
        let fake_port = FakeAuthDrivingPort::new().with_register_user_error(error);
        let fake_logger = FakeLoggerDrivingPort::new();
        let logger_clone = fake_logger.clone();
        let adapter = AuthApiAdapter { auth: fake_port, logger: fake_logger };
        let response = adapter.routes().oneshot(register_request()).await.unwrap();
        (response, logger_clone)
    }

    #[tokio::test]
    async fn register_user_error_logs_error() {
        let error = common::domain::error::RoomNotFoundError::new("lobby".to_string());
        let (_, logger) = post_register_user_with_error(common::domain::Error::RoomNotFound(error)).await;
        assert_eq!(logger.logged_errors()[0].key(), common::domain::ErrorCode::RoomNotFound);
    }

    #[tokio::test]
    async fn register_user_error_returns_error_code() {
        let error = common::domain::error::RoomNotFoundError::new("lobby".to_string());
        let (response, _) = post_register_user_with_error(common::domain::Error::RoomNotFound(error)).await;
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["code"], "RoomNotFound");
    }

    #[tokio::test]
    async fn register_user_error_returns_500() {
        let error = common::domain::error::RoomNotFoundError::new("lobby".to_string());
        let (response, _) = post_register_user_with_error(common::domain::Error::RoomNotFound(error)).await;
        assert_eq!(response.status(), axum::http::StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn register_user_returns_user_id() {
        let (response, _) = post_register_user().await;
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let user: common::dto::UserDto = serde_json::from_slice(&body).unwrap();
        assert_eq!(user.id, TEST_USER_ID);
    }

    #[tokio::test]
    async fn register_user_passes_email_to_port() {
        let (_, port) = post_register_user().await;
        assert_eq!(port.register_user_calls()[0].email, TEST_EMAIL);
    }

    #[tokio::test]
    async fn register_user_passes_name_to_port() {
        let (_, port) = post_register_user().await;
        assert_eq!(port.register_user_calls()[0].name, TEST_PLAYER_NAME);
    }

    #[tokio::test]
    async fn routes_registers_post_auth_user() {
        let (response, _) = post_register_user().await;
        assert_ne!(response.status(), axum::http::StatusCode::NOT_FOUND);
    }

    #[test]
    fn implements_api_adapter() {
        fn assert_api_adapter<T: ApiAdapter>() {}
        assert_api_adapter::<AuthApiAdapter<FakeAuthDrivingPort, FakeLoggerDrivingPort>>();
    }
}
