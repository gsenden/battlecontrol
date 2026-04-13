use std::fs;
use std::path::PathBuf;
use axum::extract::Multipart;
use std::sync::Arc;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use axum::routing::{get, post, put};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use cookie::time::Duration;
use common::dto::{
    LoginRequestDto, PasskeyFinishLoginRequestDto, PasskeyFinishRegistrationRequestDto,
    PasskeyStartLoginRequestDto, PasskeyStartRegistrationRequestDto, ProfileImageUploadDto,
    RecoverUserRequestDto, RegistrationRequestDto, UserDto, UpdateUserProfileRequestDto, UserSettingsDto,
};
use image::codecs::webp::WebPEncoder;
use image::{ExtendedColorType, ImageEncoder};
use uuid::Uuid;
use super::ApiAdapter;
use crate::adapters::db::{SessionsTable, SqliteAdapter, TableEntity, TrustedPlayersTable, UsersTable};
use crate::ports::{AuthDrivingPort, LoggerDrivingPort};

const SESSION_INACTIVITY_TIMEOUT_SECONDS: i64 = 8 * 24 * 60 * 60;
const BROWSER_COOKIE_NAME: &str = "battlecontrol-browser";

pub struct AuthApiAdapter<AuthPort: AuthDrivingPort, Logger: LoggerDrivingPort> {
    auth: AuthPort,
    logger: Logger,
    sqlite: SqliteAdapter,
}

impl<AuthPort: AuthDrivingPort, Logger: LoggerDrivingPort> AuthApiAdapter<AuthPort, Logger> {
    pub fn new(auth: AuthPort, logger: Logger, sqlite: SqliteAdapter) -> Self {
        sqlite.ensure_table::<UsersTable>()
            .expect("Failed to initialize users table");
        sqlite.ensure_table::<SessionsTable>()
            .expect("Failed to initialize sessions table");
        sqlite.ensure_table::<TrustedPlayersTable>()
            .expect("Failed to initialize trusted players table");
        migrate_sessions_table(&sqlite)
            .expect("Failed to migrate sessions table");
        Self { auth, logger, sqlite }
    }
}

struct AppState<AuthPort: AuthDrivingPort, Logger: LoggerDrivingPort> {
    auth: AuthPort,
    logger: Logger,
    sqlite: SqliteAdapter,
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
                "/auth/profile",
                put(save_user_profile::<AuthPort, Logger>),
            )
            .route(
                "/auth/profile-image",
                post(upload_profile_image::<AuthPort, Logger>),
            )
            .route(
                common::domain::Resource::AuthLogout.path(),
                post(logout::<AuthPort, Logger>),
            )
            .route(
                common::domain::Resource::AuthSettings.path(),
                get(current_user_settings::<AuthPort, Logger>).put(save_user_settings::<AuthPort, Logger>),
            )
            .route(
                common::domain::Resource::AuthRecoveryCode.path(),
                post(create_recovery_code::<AuthPort, Logger>),
            )
            .route(
                common::domain::Resource::AuthRecover.path(),
                post(recover_user::<AuthPort, Logger>),
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
                sqlite: self.sqlite,
            }))
    }
}

async fn login_user<AuthPort: AuthDrivingPort, Logger: LoggerDrivingPort>(
    State(state): State<Arc<AppState<AuthPort, Logger>>>,
    jar: CookieJar,
    Json(body): Json<LoginRequestDto>,
) -> Response {
    let browser_id = browser_id(&jar).unwrap_or_else(|| Uuid::new_v4().to_string());
    match find_user_by_name(&state.sqlite, &body.name) {
        Ok(Some(user)) if browser_trusts_user(&state.sqlite, &browser_id, &body.name) => {
            login_response_with_browser(jar, &state, user, &browser_id)
        }
        Ok(Some(_)) => {
            let error = common::domain::Error::UserAlreadyExists(
                common::domain::error::UserAlreadyExistsError::new(body.name),
            );
            state.logger.log_error(&error);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
        }
        Ok(None) => match state.auth.login_user(body).await {
            Ok(user) => {
                let _ = trust_browser_for_user(&state.sqlite, &browser_id, &user.name);
                login_response_with_browser(jar, &state, user, &browser_id)
            }
            Err(error) => {
                state.logger.log_error(&error);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
            }
        },
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
    let browser_id = browser_id(&jar).unwrap_or_else(|| Uuid::new_v4().to_string());
    match state.auth.finish_passkey_registration(body).await {
        Ok(user) => {
            let _ = trust_browser_for_user(&state.sqlite, &browser_id, &user.name);
            login_response_with_browser(jar, &state, user, &browser_id)
        }
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
    let browser_id = browser_id(&jar).unwrap_or_else(|| Uuid::new_v4().to_string());
    match state.auth.finish_passkey_login(body).await {
        Ok(user) => {
            let _ = trust_browser_for_user(&state.sqlite, &browser_id, &user.name);
            login_response_with_browser(jar, &state, user, &browser_id)
        }
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
    let browser_id = browser_id(&jar).unwrap_or_else(|| Uuid::new_v4().to_string());
    match state.auth.register_user(body).await {
        Ok(user) => {
            let _ = trust_browser_for_user(&state.sqlite, &browser_id, &user.name);
            login_response_with_browser(jar, &state, user, &browser_id)
        }
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
        let _ = state.sqlite.execute_with_params(
            &format!("DELETE FROM {} WHERE session_id = ?", SessionsTable::table_name()),
            &[&session_id as &dyn rusqlite::types::ToSql],
        );
    }

    let cleared_jar = jar.remove(session_cookie(""));
    (cleared_jar, StatusCode::NO_CONTENT).into_response()
}

async fn current_user_settings<AuthPort: AuthDrivingPort, Logger: LoggerDrivingPort>(
    State(state): State<Arc<AppState<AuthPort, Logger>>>,
    jar: CookieJar,
) -> Response {
    let Some(user) = session_user(&state, &jar) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };

    match state.auth.get_user_settings(user.name).await {
        Ok(settings) => Json(settings).into_response(),
        Err(error) => {
            state.logger.log_error(&error);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
        }
    }
}

async fn save_user_settings<AuthPort: AuthDrivingPort, Logger: LoggerDrivingPort>(
    State(state): State<Arc<AppState<AuthPort, Logger>>>,
    jar: CookieJar,
    Json(body): Json<UserSettingsDto>,
) -> Response {
    let Some(user) = session_user(&state, &jar) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };

    match state.auth.save_user_settings(user.name, body).await {
        Ok(settings) => Json(settings).into_response(),
        Err(error) => {
            state.logger.log_error(&error);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
        }
    }
}

async fn create_recovery_code<AuthPort: AuthDrivingPort, Logger: LoggerDrivingPort>(
    State(state): State<Arc<AppState<AuthPort, Logger>>>,
    jar: CookieJar,
) -> Response {
    let Some(user) = session_user(&state, &jar) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };

    match state.auth.create_recovery_code(user.name).await {
        Ok(recovery_code) => Json(recovery_code).into_response(),
        Err(error) => {
            state.logger.log_error(&error);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
        }
    }
}

async fn recover_user<AuthPort: AuthDrivingPort, Logger: LoggerDrivingPort>(
    State(state): State<Arc<AppState<AuthPort, Logger>>>,
    jar: CookieJar,
    Json(body): Json<RecoverUserRequestDto>,
) -> Response {
    match state.auth.recover_user(body).await {
        Ok(user) => {
            let browser_id = browser_id(&jar).unwrap_or_else(|| Uuid::new_v4().to_string());
            let _ = trust_browser_for_user(&state.sqlite, &browser_id, &user.name);
            login_response_with_browser(jar, &state, user, &browser_id)
        }
        Err(error) => {
            state.logger.log_error(&error);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
        }
    }
}

async fn save_user_profile<AuthPort: AuthDrivingPort, Logger: LoggerDrivingPort>(
    State(state): State<Arc<AppState<AuthPort, Logger>>>,
    jar: CookieJar,
    Json(body): Json<UpdateUserProfileRequestDto>,
) -> Response {
    let Some(user) = session_user(&state, &jar) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let current_user_name = user.name.clone();

    match state.auth.update_user_profile(current_user_name.clone(), body).await {
        Ok(updated_user) => {
            if let Some(session_id) = session_id(&jar) {
                let _ = store_session(&state.sqlite, &session_id, &updated_user);
            }
            if let Some(browser_id) = browser_id(&jar) {
                let _ = update_trusted_user_name(&state.sqlite, &browser_id, &current_user_name, &updated_user.name);
            }
            Json(updated_user).into_response()
        }
        Err(error) => {
            state.logger.log_error(&error);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
        }
    }
}

async fn upload_profile_image<AuthPort: AuthDrivingPort, Logger: LoggerDrivingPort>(
    State(state): State<Arc<AppState<AuthPort, Logger>>>,
    jar: CookieJar,
    mut multipart: Multipart,
) -> Response {
    let Some(_user) = session_user(&state, &jar) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };

    let mut uploaded_image = None;
    while let Ok(Some(field)) = multipart.next_field().await {
        if field.name() != Some("image") {
            continue;
        }

        match field.bytes().await {
            Ok(bytes) => {
                uploaded_image = Some(bytes);
                break;
            }
            Err(_) => {
                let error = common::domain::Error::DatabaseError(common::domain::error::DatabaseErrorError::new());
                state.logger.log_error(&error);
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response();
            }
        }
    }

    let Some(image_bytes) = uploaded_image else {
        return StatusCode::BAD_REQUEST.into_response();
    };

    match persist_profile_image(&image_bytes) {
        Ok(profile_image_url) => Json(ProfileImageUploadDto { profile_image_url }).into_response(),
        Err(error) => {
            state.logger.log_error(&error);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
        }
    }
}

fn persist_profile_image(bytes: &[u8]) -> Result<String, common::domain::Error> {
    let source_image = image::load_from_memory(bytes)
        .map_err(|_| common::domain::Error::DatabaseError(common::domain::error::DatabaseErrorError::new()))?;
    let resized_image = source_image.thumbnail(256, 256).to_rgba8();
    let image_id = Uuid::new_v4().to_string();
    let relative_path = format!("profile-images/{image_id}.webp");
    let uploads_dir = PathBuf::from(common::domain::EnvVar::ServerDatabasePath.value())
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| std::path::Path::new("."))
        .join("uploads")
        .join("profile-images");
    fs::create_dir_all(&uploads_dir)
        .map_err(|_| common::domain::Error::DatabaseError(common::domain::error::DatabaseErrorError::new()))?;
    let output_path = uploads_dir.join(format!("{image_id}.webp"));
    let mut output_file = fs::File::create(&output_path)
        .map_err(|_| common::domain::Error::DatabaseError(common::domain::error::DatabaseErrorError::new()))?;
    let encoder = WebPEncoder::new_lossless(&mut output_file);
    encoder.write_image(
        resized_image.as_raw(),
        resized_image.width(),
        resized_image.height(),
        ExtendedColorType::Rgba8,
    ).map_err(|_| common::domain::Error::DatabaseError(common::domain::error::DatabaseErrorError::new()))?;

    Ok(format!("/uploads/{relative_path}"))
}

fn login_response_with_browser<AuthPort: AuthDrivingPort, Logger: LoggerDrivingPort>(
    jar: CookieJar,
    state: &AppState<AuthPort, Logger>,
    user: UserDto,
    browser_id: &str,
) -> Response {
    let session_id = Uuid::new_v4().to_string();
    store_session(&state.sqlite, &session_id, &user)
        .expect("Failed to store session");
    let jar = jar
        .add(session_cookie(&session_id))
        .add(browser_cookie(browser_id));
    (jar, Json(user)).into_response()
}

fn session_user<AuthPort: AuthDrivingPort, Logger: LoggerDrivingPort>(
    state: &AppState<AuthPort, Logger>,
    jar: &CookieJar,
) -> Option<UserDto> {
    let session_id = session_id(jar)?;
    load_session_user(&state.sqlite, &session_id).ok().flatten()
}

fn session_id(jar: &CookieJar) -> Option<String> {
    jar.get("battlecontrol-session")
        .map(|cookie| cookie.value().to_string())
}

fn browser_id(jar: &CookieJar) -> Option<String> {
    jar.get(BROWSER_COOKIE_NAME)
        .map(|cookie| cookie.value().to_string())
}

fn session_cookie(session_id: &str) -> Cookie<'static> {
    Cookie::build(("battlecontrol-session", session_id.to_string()))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(Duration::days(30))
        .build()
}

fn browser_cookie(browser_id: &str) -> Cookie<'static> {
    Cookie::build((BROWSER_COOKIE_NAME, browser_id.to_string()))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(Duration::days(365))
        .build()
}

fn find_user_by_name(sqlite: &SqliteAdapter, name: &str) -> Result<Option<UserDto>, common::domain::Error> {
    let rows = sqlite.query_with_params(
        &format!("SELECT * FROM {} WHERE name = ?", UsersTable::table_name()),
        &[&name as &dyn rusqlite::types::ToSql],
    ).map_err(|_| common::domain::Error::DatabaseError(common::domain::error::DatabaseErrorError::new()))?;

    match rows.first() {
        Some(row) => Ok(Some(UsersTable::from_row(row)
            .map_err(|_| common::domain::Error::DatabaseError(common::domain::error::DatabaseErrorError::new()))?)),
        None => Ok(None),
    }
}

fn browser_trusts_user(sqlite: &SqliteAdapter, browser_id: &str, user_name: &str) -> bool {
    sqlite.query_with_params(
        &format!(
            "SELECT * FROM {} WHERE browser_id = ? AND user_name = ?",
            TrustedPlayersTable::table_name()
        ),
        &[&browser_id as &dyn rusqlite::types::ToSql, &user_name],
    ).map(|rows| !rows.is_empty()).unwrap_or(false)
}

fn trust_browser_for_user(sqlite: &SqliteAdapter, browser_id: &str, user_name: &str) -> Result<(), String> {
    sqlite.execute_with_params(
        &format!(
            "INSERT OR IGNORE INTO {} (browser_id, user_name) VALUES (?, ?)",
            TrustedPlayersTable::table_name()
        ),
        &[&browser_id as &dyn rusqlite::types::ToSql, &user_name],
    )
}

fn update_trusted_user_name(sqlite: &SqliteAdapter, browser_id: &str, current_name: &str, updated_name: &str) -> Result<(), String> {
    sqlite.execute_with_params(
        &format!(
            "UPDATE {} SET user_name = ? WHERE browser_id = ? AND user_name = ?",
            TrustedPlayersTable::table_name()
        ),
        &[&updated_name as &dyn rusqlite::types::ToSql, &browser_id, &current_name],
    )
}

fn store_session(sqlite: &SqliteAdapter, session_id: &str, user: &UserDto) -> Result<(), String> {
    let user_json = serde_json::to_string(user).map_err(|error| error.to_string())?;
    let now = current_timestamp();
    sqlite.execute_with_params(
        &format!(
            "INSERT INTO {} (session_id, user_json, last_active_at) VALUES (?, ?, ?)
             ON CONFLICT(session_id) DO UPDATE SET
             user_json = excluded.user_json,
             last_active_at = excluded.last_active_at",
            SessionsTable::table_name()
        ),
        &[&session_id as &dyn rusqlite::types::ToSql, &user_json, &now],
    )
}

fn load_session_user(sqlite: &SqliteAdapter, session_id: &str) -> Result<Option<UserDto>, String> {
    let rows = sqlite.query_with_params(
        &format!("SELECT * FROM {} WHERE session_id = ?", SessionsTable::table_name()),
        &[&session_id as &dyn rusqlite::types::ToSql],
    )?;

    match rows.first() {
        Some(row) => {
            let stored_session = SessionsTable::from_row(row)?;
            if current_timestamp() - stored_session.last_active_at > SESSION_INACTIVITY_TIMEOUT_SECONDS {
                sqlite.execute_with_params(
                    &format!("DELETE FROM {} WHERE session_id = ?", SessionsTable::table_name()),
                    &[&stored_session.session_id as &dyn rusqlite::types::ToSql],
                )?;
                return Ok(None);
            }

            sqlite.execute_with_params(
                &format!("UPDATE {} SET last_active_at = ? WHERE session_id = ?", SessionsTable::table_name()),
                &[&current_timestamp() as &dyn rusqlite::types::ToSql, &stored_session.session_id],
            )?;

            Ok(Some(stored_session.user()?))
        }
        None => Ok(None),
    }
}

fn migrate_sessions_table(sqlite: &SqliteAdapter) -> Result<(), String> {
    let rows = sqlite.query(&format!("PRAGMA table_info({})", SessionsTable::table_name()))?;
    let has_last_active_at = rows.iter().any(|row| {
        row.get::<String>("name")
            .map(|column_name| column_name == "last_active_at")
            .unwrap_or(false)
    });

    if !has_last_active_at {
        sqlite.execute(&format!(
            "ALTER TABLE {} ADD COLUMN last_active_at INTEGER NOT NULL DEFAULT 0",
            SessionsTable::table_name()
        ))?;
        sqlite.execute(&format!(
            "UPDATE {} SET last_active_at = {}",
            SessionsTable::table_name(),
            current_timestamp()
        ))?;
    }

    Ok(())
}

fn current_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time before unix epoch")
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::header::SET_COOKIE;
    use tower::ServiceExt;
    use common::domain::ErrorTrait;
    use crate::adapters::ApiAdapter;
    use crate::adapters::db::SqliteAdapter;
    use crate::test_helpers::{FakeAuthDrivingPort, FakeLoggerDrivingPort};
    use crate::test_helpers::sample_data::{TEST_PLAYER_NAME, TEST_USER_ID};

    fn sqlite_in_memory() -> SqliteAdapter {
        SqliteAdapter::new(":memory:").unwrap()
    }

    fn login_request() -> axum::http::Request<Body> {
        let body = format!(r#"{{"name":"{TEST_PLAYER_NAME}"}}"#);
        axum::http::Request::builder()
            .method("POST")
            .uri(common::domain::Resource::AuthLogin.path())
            .header("content-type", "application/json")
            .body(Body::from(body))
            .unwrap()
    }

    fn login_request_with_browser(name: &str, browser_id: &str) -> axum::http::Request<Body> {
        let body = format!(r#"{{"name":"{name}"}}"#);
        axum::http::Request::builder()
            .method("POST")
            .uri(common::domain::Resource::AuthLogin.path())
            .header("content-type", "application/json")
            .header("cookie", format!("{BROWSER_COOKIE_NAME}={browser_id}"))
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

    fn extract_session_cookie(response: &axum::http::Response<Body>) -> String {
        response
            .headers()
            .get_all(SET_COOKIE)
            .iter()
            .filter_map(|value| value.to_str().ok())
            .find(|value| value.starts_with("battlecontrol-session="))
            .and_then(|value| value.split(';').next())
            .unwrap()
            .to_string()
    }

    async fn post_login_user() -> (axum::http::Response<Body>, FakeAuthDrivingPort) {
        let fake_port = FakeAuthDrivingPort::new();
        let port_clone = fake_port.clone();
        let adapter = AuthApiAdapter::new(fake_port, FakeLoggerDrivingPort::new(), sqlite_in_memory());
        let response = adapter.routes().oneshot(login_request()).await.unwrap();
        (response, port_clone)
    }

    async fn post_login_user_with_error(error: common::domain::Error) -> (axum::http::Response<Body>, FakeLoggerDrivingPort) {
        let fake_port = FakeAuthDrivingPort::new().with_login_user_error(error);
        let fake_logger = FakeLoggerDrivingPort::new();
        let logger_clone = fake_logger.clone();
        let adapter = AuthApiAdapter::new(fake_port, fake_logger, sqlite_in_memory());
        let response = adapter.routes().oneshot(login_request()).await.unwrap();
        (response, logger_clone)
    }

    async fn post_register_user() -> (axum::http::Response<Body>, FakeAuthDrivingPort) {
        let fake_port = FakeAuthDrivingPort::new();
        let port_clone = fake_port.clone();
        let adapter = AuthApiAdapter::new(fake_port, FakeLoggerDrivingPort::new(), sqlite_in_memory());
        let response = adapter.routes().oneshot(register_request()).await.unwrap();
        (response, port_clone)
    }

    async fn post_register_user_with_error(error: common::domain::Error) -> (axum::http::Response<Body>, FakeLoggerDrivingPort) {
        let fake_port = FakeAuthDrivingPort::new().with_register_user_error(error);
        let fake_logger = FakeLoggerDrivingPort::new();
        let logger_clone = fake_logger.clone();
        let adapter = AuthApiAdapter::new(fake_port, fake_logger, sqlite_in_memory());
        let response = adapter.routes().oneshot(register_request()).await.unwrap();
        (response, logger_clone)
    }

    #[tokio::test]
    async fn login_user_skips_port_for_trusted_browser() {
        let sqlite = sqlite_in_memory();
        sqlite.ensure_table::<UsersTable>().unwrap();
        sqlite.ensure_table::<TrustedPlayersTable>().unwrap();
        sqlite.execute_with_params(
            "INSERT INTO users (name, user_handle) VALUES (?, ?)",
            &[&TEST_PLAYER_NAME as &dyn rusqlite::types::ToSql, &Uuid::new_v4().to_string()],
        ).unwrap();
        sqlite.execute_with_params(
            "INSERT INTO trusted_players (browser_id, user_name) VALUES (?, ?)",
            &[&"browser-1" as &dyn rusqlite::types::ToSql, &TEST_PLAYER_NAME],
        ).unwrap();
        let fake_port = FakeAuthDrivingPort::new();
        let port_clone = fake_port.clone();
        let app = AuthApiAdapter::new(fake_port, FakeLoggerDrivingPort::new(), sqlite).routes();

        let _ = app.oneshot(login_request_with_browser(TEST_PLAYER_NAME, "browser-1")).await.unwrap();

        assert_eq!(port_clone.login_user_calls().len(), 0);
    }

    #[tokio::test]
    async fn login_user_rejects_existing_name_for_other_browser() {
        let sqlite = sqlite_in_memory();
        sqlite.ensure_table::<UsersTable>().unwrap();
        sqlite.ensure_table::<TrustedPlayersTable>().unwrap();
        sqlite.execute_with_params(
            "INSERT INTO users (name, user_handle) VALUES (?, ?)",
            &[&TEST_PLAYER_NAME as &dyn rusqlite::types::ToSql, &Uuid::new_v4().to_string()],
        ).unwrap();
        sqlite.execute_with_params(
            "INSERT INTO trusted_players (browser_id, user_name) VALUES (?, ?)",
            &[&"browser-1" as &dyn rusqlite::types::ToSql, &TEST_PLAYER_NAME],
        ).unwrap();
        let app = AuthApiAdapter::new(FakeAuthDrivingPort::new(), FakeLoggerDrivingPort::new(), sqlite).routes();

        let response = app.oneshot(login_request_with_browser(TEST_PLAYER_NAME, "browser-2")).await.unwrap();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["code"], "UserAlreadyExists");
    }

    #[tokio::test]
    async fn login_user_sets_session_cookie() {
        let (response, _) = post_login_user().await;
        let session_cookie = extract_session_cookie(&response);
        assert!(session_cookie.starts_with("battlecontrol-session="));
    }

    #[tokio::test]
    async fn auth_me_returns_logged_in_user() {
        let fake_port = FakeAuthDrivingPort::new();
        let adapter = AuthApiAdapter::new(fake_port, FakeLoggerDrivingPort::new(), sqlite_in_memory());
        let app = adapter.routes();

        let login_response = app.clone().oneshot(login_request()).await.unwrap();
        let session_cookie = extract_session_cookie(&login_response);

        let response = app.oneshot(auth_me_request(&session_cookie)).await.unwrap();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let user: common::dto::UserDto = serde_json::from_slice(&body).unwrap();
        assert_eq!(user.id, TEST_USER_ID);
    }

    #[tokio::test]
    async fn auth_me_returns_logged_in_user_after_rebuilding_adapter() {
        let sqlite = sqlite_in_memory();
        let first_app = AuthApiAdapter::new(FakeAuthDrivingPort::new(), FakeLoggerDrivingPort::new(), sqlite.clone()).routes();

        let login_response = first_app.oneshot(login_request()).await.unwrap();
        let session_cookie = extract_session_cookie(&login_response);

        let second_app = AuthApiAdapter::new(FakeAuthDrivingPort::new(), FakeLoggerDrivingPort::new(), sqlite).routes();
        let response = second_app.oneshot(auth_me_request(&session_cookie)).await.unwrap();
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
