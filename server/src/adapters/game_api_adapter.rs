use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post, put};
use axum_extra::extract::cookie::CookieJar;
use common::dto::{CreateGameRequestDto, JoinGameRequestDto, SaveSelectedRaceRequestDto, UserDto};

use super::ApiAdapter;
use crate::adapters::db::{SessionsTable, SqliteAdapter, TableEntity};
use crate::ports::{GameDrivingPort, LoggerDrivingPort};

const SESSION_INACTIVITY_TIMEOUT_SECONDS: i64 = 8 * 24 * 60 * 60;

pub struct GameApiAdapter<GamePort: GameDrivingPort, Logger: LoggerDrivingPort> {
    game: GamePort,
    logger: Logger,
    sqlite: SqliteAdapter,
}

impl<GamePort: GameDrivingPort, Logger: LoggerDrivingPort> GameApiAdapter<GamePort, Logger> {
    pub fn new(game: GamePort, logger: Logger, sqlite: SqliteAdapter) -> Self {
        sqlite.ensure_table::<SessionsTable>()
            .expect("Failed to initialize sessions table");
        Self { game, logger, sqlite }
    }
}

struct AppState<GamePort: GameDrivingPort, Logger: LoggerDrivingPort> {
    game: GamePort,
    logger: Logger,
    sqlite: SqliteAdapter,
}

impl<GamePort, Logger> ApiAdapter for GameApiAdapter<GamePort, Logger>
where
    GamePort: GameDrivingPort + Send + Sync + 'static,
    Logger: LoggerDrivingPort + Send + Sync + 'static,
{
    fn routes(self) -> axum::Router {
        axum::Router::new()
            .route("/games", post(create_game::<GamePort, Logger>).get(list_games::<GamePort, Logger>))
            .route("/games/{game_id}", get(find_game::<GamePort, Logger>))
            .route("/games/{game_id}/join", post(join_game::<GamePort, Logger>))
            .route("/games/{game_id}/race", put(save_selected_race::<GamePort, Logger>))
            .with_state(Arc::new(AppState {
                game: self.game,
                logger: self.logger,
                sqlite: self.sqlite,
            }))
    }
}

async fn create_game<GamePort: GameDrivingPort, Logger: LoggerDrivingPort>(
    State(state): State<Arc<AppState<GamePort, Logger>>>,
    jar: CookieJar,
    Json(body): Json<CreateGameRequestDto>,
) -> Response {
    let Some(user) = session_user(&state, &jar) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };

    match state.game.create_game(user.name, body).await {
        Ok(game) => (StatusCode::CREATED, Json(game)).into_response(),
        Err(error) => {
            state.logger.log_error(&error);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
        }
    }
}

async fn list_games<GamePort: GameDrivingPort, Logger: LoggerDrivingPort>(
    State(state): State<Arc<AppState<GamePort, Logger>>>,
    jar: CookieJar,
) -> Response {
    if session_user(&state, &jar).is_none() {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    match state.game.list_games().await {
        Ok(games) => Json(games).into_response(),
        Err(error) => {
            state.logger.log_error(&error);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
        }
    }
}

async fn find_game<GamePort: GameDrivingPort, Logger: LoggerDrivingPort>(
    State(state): State<Arc<AppState<GamePort, Logger>>>,
    jar: CookieJar,
    Path(game_id): Path<String>,
) -> Response {
    if session_user(&state, &jar).is_none() {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    match state.game.find_game(game_id).await {
        Ok(game) => Json(game).into_response(),
        Err(error) => {
            state.logger.log_error(&error);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
        }
    }
}

async fn join_game<GamePort: GameDrivingPort, Logger: LoggerDrivingPort>(
    State(state): State<Arc<AppState<GamePort, Logger>>>,
    jar: CookieJar,
    Path(game_id): Path<String>,
    Json(body): Json<JoinGameRequestDto>,
) -> Response {
    let Some(user) = session_user(&state, &jar) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };

    match state.game.join_game(game_id, user.name, body).await {
        Ok(game) => Json(game).into_response(),
        Err(error) => {
            state.logger.log_error(&error);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
        }
    }
}

async fn save_selected_race<GamePort: GameDrivingPort, Logger: LoggerDrivingPort>(
    State(state): State<Arc<AppState<GamePort, Logger>>>,
    jar: CookieJar,
    Path(game_id): Path<String>,
    Json(body): Json<SaveSelectedRaceRequestDto>,
) -> Response {
    let Some(user) = session_user(&state, &jar) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };

    match state.game.save_selected_race(game_id, user.name, body).await {
        Ok(game) => Json(game).into_response(),
        Err(error) => {
            state.logger.log_error(&error);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
        }
    }
}

fn session_user<GamePort: GameDrivingPort, Logger: LoggerDrivingPort>(
    state: &AppState<GamePort, Logger>,
    jar: &CookieJar,
) -> Option<UserDto> {
    let session_id = jar
        .get("battlecontrol-session")
        .map(|cookie| cookie.value().to_string())?;

    load_session_user(&state.sqlite, &session_id).ok().flatten()
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

fn current_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time before unix epoch")
        .as_secs() as i64
}
