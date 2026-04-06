use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use axum::routing::{get, post};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use common::dto::{
    LoginRequestDto, PasskeyFinishLoginRequestDto, PasskeyFinishRegistrationRequestDto,
    PasskeyStartLoginRequestDto, PasskeyStartRegistrationRequestDto, RegistrationRequestDto, UserDto,
};
use uuid::Uuid;
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
    sessions: Mutex<HashMap<String, UserDto>>,
}

impl<AuthPort, Logger> ApiAdapter for AuthApiAdapter<AuthPort, Logger>
where
    AuthPort: AuthDrivingPort + Send + Sync + 'static,
    Logger: LoggerDrivingPort + Send + Sync + 'static,
{
    fn routes(self) -> axum::Router {
        axum::Router::new()
            .route(
                common::domain::Resource::AuthLogin.path(),
                post(login_user::<AuthPort, Logger>),
            )
            .route(
                common::domain::Resource::AuthMe.path(),
                get(current_user::<AuthPort, Logger>),
            )
            .route(
                common::domain::Resource::AuthLogout.path(),
                post(logout::<AuthPort, Logger>),
            )
            .route(
                common::domain::Resource::AuthUser.path(),
                post(register_user::<AuthPort, Logger>),
            )
            .route(
                common::domain::Resource::AuthPasskeyRegisterStart.path(),
                post(start_passkey_registration::<AuthPort, Logger>),
            )
            .route(
                common::domain::Resource::AuthPasskeyRegisterFinish.path(),
                post(finish_passkey_registration::<AuthPort, Logger>),
            )
            .route(
                common::domain::Resource::AuthPasskeyLoginStart.path(),
                post(start_passkey_login::<AuthPort, Logger>),
            )
            .route(
                common::domain::Resource::AuthPasskeyLoginFinish.path(),
                post(finish_passkey_login::<AuthPort, Logger>),
            )
            .with_state(Arc::new(AppState {
                auth: self.auth,
                logger: self.logger,
                sessions: Mutex::new(HashMap::new()),
            }))
    }
}

async fn login_user<AuthPort: AuthDrivingPort, Logger: LoggerDrivingPort>(
    State(state): State<Arc<AppState<AuthPort, Logger>>>,
    jar: CookieJar,
    Json(body): Json<LoginRequestDto>,
) -> Response {
    match state.auth.login_user(body).await {
        Ok(user) => login_response(jar, &state, user),
        Err(error) => {
            state.logger.log_error(&error);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
        }
    }
}

async fn start_passkey_registration<AuthPort: AuthDrivingPort, Logger: LoggerDrivingPort>(
    State(state): State<Arc<AppState<AuthPort, Logger>>>,
    Json(body): Json<PasskeyStartRegistrationRequestDto>,
) -> Response {
    match state.auth.start_passkey_registration(body).await {
        Ok(options) => Json(options).into_response(),
        Err(error) => {
            state.logger.log_error(&error);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
        }
    }
}

async fn finish_passkey_registration<AuthPort: AuthDrivingPort, Logger: LoggerDrivingPort>(
    State(state): State<Arc<AppState<AuthPort, Logger>>>,
    jar: CookieJar,
    Json(body): Json<PasskeyFinishRegistrationRequestDto>,
) -> Response {
    match state.auth.finish_passkey_registration(body).await {
        Ok(user) => login_response(jar, &state, user),
        Err(error) => {
            state.logger.log_error(&error);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
        }
    }
}

async fn start_passkey_login<AuthPort: AuthDrivingPort, Logger: LoggerDrivingPort>(
    State(state): State<Arc<AppState<AuthPort, Logger>>>,
    Json(body): Json<PasskeyStartLoginRequestDto>,
) -> Response {
    match state.auth.start_passkey_login(body).await {
        Ok(options) => Json(options).into_response(),
        Err(error) => {
            state.logger.log_error(&error);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
        }
    }
}

async fn finish_passkey_login<AuthPort: AuthDrivingPort, Logger: LoggerDrivingPort>(
    State(state): State<Arc<AppState<AuthPort, Logger>>>,
    jar: CookieJar,
    Json(body): Json<PasskeyFinishLoginRequestDto>,
) -> Response {
    match state.auth.finish_passkey_login(body).await {
        Ok(user) => login_response(jar, &state, user),
        Err(error) => {
            state.logger.log_error(&error);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
        }
    }
}

async fn register_user<AuthPort: AuthDrivingPort, Logger: LoggerDrivingPort>(
    State(state): State<Arc<AppState<AuthPort, Logger>>>,
    jar: CookieJar,
    Json(body): Json<RegistrationRequestDto>,
) -> Response {
    match state.auth.register_user(body).await {
        Ok(user) => login_response(jar, &state, user),
        Err(error) => {
            state.logger.log_error(&error);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
        }
    }
}

async fn current_user<AuthPort: AuthDrivingPort, Logger: LoggerDrivingPort>(
    State(state): State<Arc<AppState<AuthPort, Logger>>>,
    jar: CookieJar,
) -> Response {
    match session_user(&state, &jar) {
        Some(user) => Json(user).into_response(),
        None => StatusCode::UNAUTHORIZED.into_response(),
    }
}

async fn logout<AuthPort: AuthDrivingPort, Logger: LoggerDrivingPort>(
    State(state): State<Arc<AppState<AuthPort, Logger>>>,
    jar: CookieJar,
) -> Response {
    if let Some(session_id) = session_id(&jar) {
        state.sessions.lock().unwrap().remove(&session_id);
    }

    let cleared_jar = jar.remove(session_cookie(""));
    (cleared_jar, StatusCode::NO_CONTENT).into_response()
}

fn login_response<AuthPort: AuthDrivingPort, Logger: LoggerDrivingPort>(
    jar: CookieJar,
    state: &AppState<AuthPort, Logger>,
    user: UserDto,
) -> Response {
    let session_id = Uuid::new_v4().to_string();
    state.sessions.lock().unwrap().insert(session_id.clone(), user.clone());
    let jar = jar.add(session_cookie(&session_id));
    (jar, Json(user)).into_response()
}

fn session_user<AuthPort: AuthDrivingPort, Logger: LoggerDrivingPort>(
    state: &AppState<AuthPort, Logger>,
    jar: &CookieJar,
) -> Option<UserDto> {
    let session_id = session_id(jar)?;
    state.sessions.lock().unwrap().get(&session_id).cloned()
}

fn session_id(jar: &CookieJar) -> Option<String> {
    jar.get("battlecontrol-session")
        .map(|cookie| cookie.value().to_string())
}

fn session_cookie(session_id: &str) -> Cookie<'static> {
    Cookie::build(("battlecontrol-session", session_id.to_string()))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::header::SET_COOKIE;
    use tower::ServiceExt;
    use common::domain::ErrorTrait;
    use crate::adapters::ApiAdapter;
    use crate::test_helpers::{FakeAuthDrivingPort, FakeLoggerDrivingPort};
    use crate::test_helpers::sample_data::{TEST_PLAYER_NAME, TEST_USER_ID};

    fn login_request() -> axum::http::Request<Body> {
        let body = format!(r#"{{"name":"{TEST_PLAYER_NAME}"}}"#);
        axum::http::Request::builder()
            .method("POST")
            .uri(common::domain::Resource::AuthLogin.path())
            .header("content-type", "application/json")
            .body(Body::from(body))
            .unwrap()
    }

    fn register_request() -> axum::http::Request<Body> {
        let body = format!(r#"{{"name":"{TEST_PLAYER_NAME}"}}"#);
        axum::http::Request::builder()
            .method("POST")
            .uri(common::domain::Resource::AuthUser.path())
            .header("content-type", "application/json")
            .body(Body::from(body))
            .unwrap()
    }

    fn auth_me_request(session_cookie: &str) -> axum::http::Request<Body> {
        axum::http::Request::builder()
            .method("GET")
            .uri(common::domain::Resource::AuthMe.path())
            .header("cookie", session_cookie)
            .body(Body::empty())
            .unwrap()
    }

    async fn post_login_user() -> (axum::http::Response<Body>, FakeAuthDrivingPort) {
        let fake_port = FakeAuthDrivingPort::new();
        let port_clone = fake_port.clone();
        let adapter = AuthApiAdapter { auth: fake_port, logger: FakeLoggerDrivingPort::new() };
        let response = adapter.routes().oneshot(login_request()).await.unwrap();
        (response, port_clone)
    }

    async fn post_login_user_with_error(error: common::domain::Error) -> (axum::http::Response<Body>, FakeLoggerDrivingPort) {
        let fake_port = FakeAuthDrivingPort::new().with_login_user_error(error);
        let fake_logger = FakeLoggerDrivingPort::new();
        let logger_clone = fake_logger.clone();
        let adapter = AuthApiAdapter { auth: fake_port, logger: fake_logger };
        let response = adapter.routes().oneshot(login_request()).await.unwrap();
        (response, logger_clone)
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
    async fn login_user_sets_session_cookie() {
        let (response, _) = post_login_user().await;
        let set_cookie = response.headers().get(SET_COOKIE).unwrap().to_str().unwrap();
        assert!(set_cookie.contains("battlecontrol-session="));
    }

    #[tokio::test]
    async fn auth_me_returns_logged_in_user() {
        let fake_port = FakeAuthDrivingPort::new();
        let adapter = AuthApiAdapter { auth: fake_port, logger: FakeLoggerDrivingPort::new() };
        let app = adapter.routes();

        let login_response = app.clone().oneshot(login_request()).await.unwrap();
        let set_cookie = login_response.headers().get(SET_COOKIE).unwrap().to_str().unwrap();
        let session_cookie = set_cookie.split(';').next().unwrap();

        let response = app.oneshot(auth_me_request(session_cookie)).await.unwrap();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let user: common::dto::UserDto = serde_json::from_slice(&body).unwrap();
        assert_eq!(user.id, TEST_USER_ID);
    }

    #[tokio::test]
    async fn login_user_error_logs_error() {
        let error = common::domain::error::UserNotFoundError::new(TEST_PLAYER_NAME.to_string());
        let (_, logger) = post_login_user_with_error(common::domain::Error::UserNotFound(error)).await;
        assert_eq!(logger.logged_errors()[0].key(), common::domain::ErrorCode::UserNotFound);
    }

    #[tokio::test]
    async fn login_user_error_returns_error_code() {
        let error = common::domain::error::UserNotFoundError::new(TEST_PLAYER_NAME.to_string());
        let (response, _) = post_login_user_with_error(common::domain::Error::UserNotFound(error)).await;
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["code"], "UserNotFound");
    }

    #[tokio::test]
    async fn login_user_returns_user_id() {
        let (response, _) = post_login_user().await;
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let user: common::dto::UserDto = serde_json::from_slice(&body).unwrap();
        assert_eq!(user.id, TEST_USER_ID);
    }

    #[tokio::test]
    async fn login_user_passes_name_to_port() {
        let (_, port) = post_login_user().await;
        assert_eq!(port.login_user_calls()[0].name, TEST_PLAYER_NAME);
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
    async fn register_user_passes_name_to_port() {
        let (_, port) = post_register_user().await;
        assert_eq!(port.register_user_calls()[0].name, TEST_PLAYER_NAME);
    }

    #[tokio::test]
    async fn routes_registers_post_auth_user() {
        let (response, _) = post_register_user().await;
        assert_ne!(response.status(), axum::http::StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn routes_registers_post_auth_login() {
        let (response, _) = post_login_user().await;
        assert_ne!(response.status(), axum::http::StatusCode::NOT_FOUND);
    }

    #[test]
    fn implements_api_adapter() {
        fn assert_api_adapter<T: ApiAdapter>() {}
        assert_api_adapter::<AuthApiAdapter<FakeAuthDrivingPort, FakeLoggerDrivingPort>>();
    }
}
