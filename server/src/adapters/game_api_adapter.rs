use std::sync::Arc;

use axum::Json;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post, put};
use axum_extra::extract::cookie::CookieJar;
use common::dto::{CreateGameRequestDto, JoinGameRequestDto, SaveSelectedRaceRequestDto, UserDto};

use super::{ApiAdapter, BattleSessionHub, GameRoomHub};
use crate::adapters::db::{SessionsTable, SqliteAdapter, TableEntity};
use crate::ports::{GameDrivingPort, GameRoomDrivenPort, LoggerDrivingPort};
use crate::adapters::battle_session_hub::{BattleClientMessage, BattleServerMessage};

const SESSION_INACTIVITY_TIMEOUT_SECONDS: i64 = 8 * 24 * 60 * 60;

pub struct GameApiAdapter<GamePort: GameDrivingPort, Logger: LoggerDrivingPort> {
    game: GamePort,
    game_rooms: GameRoomHub,
    battle_sessions: BattleSessionHub,
    logger: Logger,
    sqlite: SqliteAdapter,
}

impl<GamePort: GameDrivingPort, Logger: LoggerDrivingPort> GameApiAdapter<GamePort, Logger> {
    pub fn new(game: GamePort, game_rooms: GameRoomHub, battle_sessions: BattleSessionHub, logger: Logger, sqlite: SqliteAdapter) -> Self {
        sqlite.ensure_table::<SessionsTable>()
            .expect("Failed to initialize sessions table");
        Self { game, game_rooms, battle_sessions, logger, sqlite }
    }
}

struct AppState<GamePort: GameDrivingPort, Logger: LoggerDrivingPort> {
    game: GamePort,
    game_rooms: GameRoomHub,
    battle_sessions: BattleSessionHub,
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
            .route("/games/{game_id}/instance", get(find_game_instance::<GamePort, Logger>))
            .route("/games/{game_id}/events", get(connect_game_events::<GamePort, Logger>))
            .route("/games/{game_id}/battle", get(connect_battle::<GamePort, Logger>))
            .route("/games/{game_id}/join", post(join_game::<GamePort, Logger>))
            .route("/games/{game_id}/leave", post(leave_game::<GamePort, Logger>))
            .route("/games/{game_id}/cancel", post(cancel_game::<GamePort, Logger>))
            .route("/games/{game_id}/start", post(start_game::<GamePort, Logger>))
            .route("/games/{game_id}/complete", post(complete_game::<GamePort, Logger>))
            .route("/games/{game_id}/race", put(save_selected_race::<GamePort, Logger>))
            .with_state(Arc::new(AppState {
                game: self.game,
                game_rooms: self.game_rooms,
                battle_sessions: self.battle_sessions,
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

async fn connect_game_events<GamePort: GameDrivingPort, Logger: LoggerDrivingPort>(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState<GamePort, Logger>>>,
    jar: CookieJar,
    Path(game_id): Path<String>,
) -> Response {
    if session_user(&state, &jar).is_none() {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    let game_rooms = state.game_rooms.clone();

    ws.on_upgrade(move |socket| game_events_socket(socket, game_rooms, game_id))
        .into_response()
}

async fn connect_battle<GamePort: GameDrivingPort, Logger: LoggerDrivingPort>(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState<GamePort, Logger>>>,
    jar: CookieJar,
    Path(game_id): Path<String>,
) -> Response {
    let Some(user) = session_user(&state, &jar) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };

    if !state.battle_sessions.has_battle(&game_id) {
        return StatusCode::NOT_FOUND.into_response();
    }

    let battle_sessions = state.battle_sessions.clone();
    ws.on_upgrade(move |socket| battle_socket(socket, battle_sessions, game_id, user.name))
        .into_response()
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

async fn find_game_instance<GamePort: GameDrivingPort, Logger: LoggerDrivingPort>(
    State(state): State<Arc<AppState<GamePort, Logger>>>,
    jar: CookieJar,
    Path(game_id): Path<String>,
) -> Response {
    if session_user(&state, &jar).is_none() {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    match state.game_rooms.game(&game_id) {
        Some(game) => Json(game).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
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

async fn leave_game<GamePort: GameDrivingPort, Logger: LoggerDrivingPort>(
    State(state): State<Arc<AppState<GamePort, Logger>>>,
    jar: CookieJar,
    Path(game_id): Path<String>,
) -> Response {
    let Some(user) = session_user(&state, &jar) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };

    match state.game.leave_game(game_id, user.name).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(error) => {
            state.logger.log_error(&error);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
        }
    }
}

async fn cancel_game<GamePort: GameDrivingPort, Logger: LoggerDrivingPort>(
    State(state): State<Arc<AppState<GamePort, Logger>>>,
    jar: CookieJar,
    Path(game_id): Path<String>,
) -> Response {
    let Some(user) = session_user(&state, &jar) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };

    match state.game.cancel_game(game_id, user.name).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(error) => {
            state.logger.log_error(&error);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
        }
    }
}

async fn start_game<GamePort: GameDrivingPort, Logger: LoggerDrivingPort>(
    State(state): State<Arc<AppState<GamePort, Logger>>>,
    jar: CookieJar,
    Path(game_id): Path<String>,
) -> Response {
    let Some(user) = session_user(&state, &jar) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };

    match state.game.start_game(game_id, user.name).await {
        Ok(game) => {
            if let Err(error) = state.battle_sessions.start_battle(&game) {
                return (StatusCode::INTERNAL_SERVER_ERROR, error).into_response();
            }
            Json(game).into_response()
        }
        Err(error) => {
            state.logger.log_error(&error);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
        }
    }
}

async fn complete_game<GamePort: GameDrivingPort, Logger: LoggerDrivingPort>(
    State(state): State<Arc<AppState<GamePort, Logger>>>,
    jar: CookieJar,
    Path(game_id): Path<String>,
) -> Response {
    if session_user(&state, &jar).is_none() {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    state.battle_sessions.remove_battle(&game_id);
    state.game_rooms.cancel_room(&game_id);
    StatusCode::NO_CONTENT.into_response()
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

async fn game_events_socket(socket: WebSocket, game_rooms: GameRoomHub, game_id: String) {
    let mut socket = socket;
    let mut receiver = game_rooms.subscribe(&game_id);

    if let Some(message) = game_rooms.current_message(&game_id) {
        if socket.send(message).await.is_err() {
            return;
        }
    }

    loop {
        tokio::select! {
            maybe_event = receiver.recv() => {
                let Ok(event) = maybe_event else {
                    break;
                };

                let message = axum::extract::ws::Message::Text(
                    serde_json::to_string(&event)
                        .expect("Failed to serialize game room event")
                        .into(),
                );

                if socket.send(message).await.is_err() {
                    break;
                }
            }
            maybe_message = socket.recv() => {
                match maybe_message {
                    Some(Ok(axum::extract::ws::Message::Close(_))) | None => break,
                    Some(Err(_)) => break,
                    _ => {}
                }
            }
        }
    }
}

async fn battle_socket(
    socket: WebSocket,
    battle_sessions: BattleSessionHub,
    game_id: String,
    user_name: String,
) {
    let mut socket = socket;

    loop {
        if let Some(snapshot) = battle_sessions.snapshot_for(&game_id, &user_name) {
            let message = BattleServerMessage {
                message_type: "snapshot",
                snapshot,
            };
            if socket
                .send(Message::Text(
                    serde_json::to_string(&message)
                        .expect("Failed to serialize battle server message")
                        .into(),
                ))
                .await
                .is_err()
            {
                break;
            }
        } else {
            break;
        }

        tokio::select! {
            maybe_message = socket.recv() => {
                match maybe_message {
                    Some(Ok(Message::Text(text))) => {
                        let Ok(message) = serde_json::from_str::<BattleClientMessage>(&text) else {
                            break;
                        };
                        if battle_sessions.apply_message(&game_id, &user_name, message).is_err() {
                            break;
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Err(_)) => break,
                    _ => {}
                }
            }
            _ = tokio::time::sleep(std::time::Duration::from_millis(1000 / 24)) => {}
        }
    }
}
