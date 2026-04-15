use std::sync::Arc;

use axum::Json;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post, put};
use axum_extra::extract::cookie::CookieJar;
use common::dto::{CreateGameRequestDto, JoinGameRequestDto, SaveSelectedRaceRequestDto};

use super::{ApiAdapter, GameRoomHub};
use crate::adapters::battle_session_hub::{BattleClientMessage, BattleServerMessage};
use crate::ports::{
    BattleSessionDrivenPort, GameDrivingPort, GameRoomDrivenPort, LoggerDrivingPort,
    SessionRepositoryDrivenPort,
};

pub struct GameApiAdapter<
    GamePort: GameDrivingPort,
    BattlePort: BattleSessionDrivenPort,
    SessionPort: SessionRepositoryDrivenPort,
    Logger: LoggerDrivingPort,
> {
    game: GamePort,
    game_rooms: GameRoomHub,
    battle_sessions: BattlePort,
    session_repo: SessionPort,
    logger: Logger,
}

impl<
    GamePort: GameDrivingPort,
    BattlePort: BattleSessionDrivenPort,
    SessionPort: SessionRepositoryDrivenPort,
    Logger: LoggerDrivingPort,
> GameApiAdapter<GamePort, BattlePort, SessionPort, Logger>
{
    pub fn new(
        game: GamePort,
        game_rooms: GameRoomHub,
        battle_sessions: BattlePort,
        session_repo: SessionPort,
        logger: Logger,
    ) -> Self {
        Self {
            game,
            game_rooms,
            battle_sessions,
            session_repo,
            logger,
        }
    }
}

struct AppState<
    GamePort: GameDrivingPort,
    BattlePort: BattleSessionDrivenPort,
    SessionPort: SessionRepositoryDrivenPort,
    Logger: LoggerDrivingPort,
> {
    game: GamePort,
    game_rooms: GameRoomHub,
    battle_sessions: BattlePort,
    session_repo: SessionPort,
    logger: Logger,
}

impl<GamePort, BattlePort, SessionPort, Logger> ApiAdapter
    for GameApiAdapter<GamePort, BattlePort, SessionPort, Logger>
where
    GamePort: GameDrivingPort + Send + Sync + 'static,
    BattlePort: BattleSessionDrivenPort + Send + Sync + Clone + 'static,
    SessionPort: SessionRepositoryDrivenPort + Send + Sync + Clone + 'static,
    Logger: LoggerDrivingPort + Send + Sync + 'static,
{
    fn routes(self) -> axum::Router {
        axum::Router::new()
            .route(
                "/games",
                post(create_game::<GamePort, BattlePort, SessionPort, Logger>)
                    .get(list_games::<GamePort, BattlePort, SessionPort, Logger>),
            )
            .route(
                "/games/events",
                get(connect_lobby_events::<GamePort, BattlePort, SessionPort, Logger>),
            )
            .route(
                "/games/{game_id}",
                get(find_game::<GamePort, BattlePort, SessionPort, Logger>),
            )
            .route(
                "/games/{game_id}/instance",
                get(find_game_instance::<GamePort, BattlePort, SessionPort, Logger>),
            )
            .route(
                "/games/{game_id}/events",
                get(connect_game_events::<GamePort, BattlePort, SessionPort, Logger>),
            )
            .route(
                "/games/{game_id}/battle",
                get(connect_battle::<GamePort, BattlePort, SessionPort, Logger>),
            )
            .route(
                "/games/{game_id}/join",
                post(join_game::<GamePort, BattlePort, SessionPort, Logger>),
            )
            .route(
                "/games/{game_id}/leave",
                post(leave_game::<GamePort, BattlePort, SessionPort, Logger>),
            )
            .route(
                "/games/{game_id}/cancel",
                post(cancel_game::<GamePort, BattlePort, SessionPort, Logger>),
            )
            .route(
                "/games/{game_id}/start",
                post(start_game::<GamePort, BattlePort, SessionPort, Logger>),
            )
            .route(
                "/games/{game_id}/complete",
                post(complete_game::<GamePort, BattlePort, SessionPort, Logger>),
            )
            .route(
                "/games/{game_id}/race",
                put(save_selected_race::<GamePort, BattlePort, SessionPort, Logger>),
            )
            .with_state(Arc::new(AppState {
                game: self.game,
                game_rooms: self.game_rooms,
                battle_sessions: self.battle_sessions,
                session_repo: self.session_repo,
                logger: self.logger,
            }))
    }
}

async fn create_game<
    GamePort: GameDrivingPort,
    BattlePort: BattleSessionDrivenPort,
    SessionPort: SessionRepositoryDrivenPort,
    Logger: LoggerDrivingPort,
>(
    State(state): State<Arc<AppState<GamePort, BattlePort, SessionPort, Logger>>>,
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

async fn connect_game_events<
    GamePort: GameDrivingPort,
    BattlePort: BattleSessionDrivenPort,
    SessionPort: SessionRepositoryDrivenPort,
    Logger: LoggerDrivingPort,
>(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState<GamePort, BattlePort, SessionPort, Logger>>>,
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

async fn connect_lobby_events<
    GamePort: GameDrivingPort,
    BattlePort: BattleSessionDrivenPort,
    SessionPort: SessionRepositoryDrivenPort,
    Logger: LoggerDrivingPort,
>(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState<GamePort, BattlePort, SessionPort, Logger>>>,
    jar: CookieJar,
) -> Response {
    if session_user(&state, &jar).is_none() {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    let game_rooms = state.game_rooms.clone();

    ws.on_upgrade(move |socket| lobby_events_socket(socket, game_rooms))
        .into_response()
}

async fn connect_battle<
    GamePort: GameDrivingPort,
    BattlePort: BattleSessionDrivenPort + Clone + Send + Sync + 'static,
    SessionPort: SessionRepositoryDrivenPort,
    Logger: LoggerDrivingPort,
>(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState<GamePort, BattlePort, SessionPort, Logger>>>,
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

async fn list_games<
    GamePort: GameDrivingPort,
    BattlePort: BattleSessionDrivenPort,
    SessionPort: SessionRepositoryDrivenPort,
    Logger: LoggerDrivingPort,
>(
    State(state): State<Arc<AppState<GamePort, BattlePort, SessionPort, Logger>>>,
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

async fn find_game<
    GamePort: GameDrivingPort,
    BattlePort: BattleSessionDrivenPort,
    SessionPort: SessionRepositoryDrivenPort,
    Logger: LoggerDrivingPort,
>(
    State(state): State<Arc<AppState<GamePort, BattlePort, SessionPort, Logger>>>,
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

async fn find_game_instance<
    GamePort: GameDrivingPort,
    BattlePort: BattleSessionDrivenPort,
    SessionPort: SessionRepositoryDrivenPort,
    Logger: LoggerDrivingPort,
>(
    State(state): State<Arc<AppState<GamePort, BattlePort, SessionPort, Logger>>>,
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

async fn join_game<
    GamePort: GameDrivingPort,
    BattlePort: BattleSessionDrivenPort,
    SessionPort: SessionRepositoryDrivenPort,
    Logger: LoggerDrivingPort,
>(
    State(state): State<Arc<AppState<GamePort, BattlePort, SessionPort, Logger>>>,
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

async fn leave_game<
    GamePort: GameDrivingPort,
    BattlePort: BattleSessionDrivenPort,
    SessionPort: SessionRepositoryDrivenPort,
    Logger: LoggerDrivingPort,
>(
    State(state): State<Arc<AppState<GamePort, BattlePort, SessionPort, Logger>>>,
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

async fn cancel_game<
    GamePort: GameDrivingPort,
    BattlePort: BattleSessionDrivenPort,
    SessionPort: SessionRepositoryDrivenPort,
    Logger: LoggerDrivingPort,
>(
    State(state): State<Arc<AppState<GamePort, BattlePort, SessionPort, Logger>>>,
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

async fn start_game<
    GamePort: GameDrivingPort,
    BattlePort: BattleSessionDrivenPort,
    SessionPort: SessionRepositoryDrivenPort,
    Logger: LoggerDrivingPort,
>(
    State(state): State<Arc<AppState<GamePort, BattlePort, SessionPort, Logger>>>,
    jar: CookieJar,
    Path(game_id): Path<String>,
) -> Response {
    let Some(user) = session_user(&state, &jar) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };

    match state.game.start_game(game_id, user.name).await {
        Ok(game) => {
            if let Err(error) = state.battle_sessions.start_battle(&game) {
                return (StatusCode::BAD_REQUEST, Json(error)).into_response();
            }
            state.game_rooms.start_room(&game);
            Json(game).into_response()
        }
        Err(error) => {
            state.logger.log_error(&error);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
        }
    }
}

async fn complete_game<
    GamePort: GameDrivingPort,
    BattlePort: BattleSessionDrivenPort,
    SessionPort: SessionRepositoryDrivenPort,
    Logger: LoggerDrivingPort,
>(
    State(state): State<Arc<AppState<GamePort, BattlePort, SessionPort, Logger>>>,
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

async fn save_selected_race<
    GamePort: GameDrivingPort,
    BattlePort: BattleSessionDrivenPort,
    SessionPort: SessionRepositoryDrivenPort,
    Logger: LoggerDrivingPort,
>(
    State(state): State<Arc<AppState<GamePort, BattlePort, SessionPort, Logger>>>,
    jar: CookieJar,
    Path(game_id): Path<String>,
    Json(body): Json<SaveSelectedRaceRequestDto>,
) -> Response {
    let Some(user) = session_user(&state, &jar) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };

    match state
        .game
        .save_selected_race(game_id, user.name, body)
        .await
    {
        Ok(game) => Json(game).into_response(),
        Err(error) => {
            state.logger.log_error(&error);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
        }
    }
}

fn session_user<
    GamePort: GameDrivingPort,
    BattlePort: BattleSessionDrivenPort,
    SessionPort: SessionRepositoryDrivenPort,
    Logger: LoggerDrivingPort,
>(
    state: &AppState<GamePort, BattlePort, SessionPort, Logger>,
    jar: &CookieJar,
) -> Option<common::dto::UserDto> {
    let session_id = jar
        .get("battlecontrol-session")
        .map(|cookie| cookie.value().to_string())?;

    state
        .session_repo
        .load_session_user(&session_id)
        .ok()
        .flatten()
}

async fn game_events_socket(socket: WebSocket, game_rooms: GameRoomHub, game_id: String) {
    let mut socket = socket;
    let mut receiver = game_rooms.subscribe(&game_id);

    if let Some(message) = game_rooms.current_message(&game_id)
        && socket.send(message).await.is_err()
    {
        return;
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

async fn lobby_events_socket(socket: WebSocket, game_rooms: GameRoomHub) {
    let mut socket = socket;
    let mut receiver = game_rooms.subscribe_all();

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

async fn battle_socket<BattlePort: BattleSessionDrivenPort + Send + Sync + 'static>(
    socket: WebSocket,
    battle_sessions: BattlePort,
    game_id: String,
    user_name: String,
) {
    let mut socket = socket;

    while let Some(snapshot) = battle_sessions.snapshot_for(&game_id, &user_name) {
        let Some(ready_state) = battle_sessions.ready_state_for(&game_id, &user_name) else {
            break;
        };
        let message = BattleServerMessage {
            message_type: "snapshot",
            snapshot,
            battle_started: ready_state.battle_started,
            ready_players: ready_state.ready_players,
            total_players: ready_state.total_players,
            battle_completed: ready_state.battle_completed,
            winner_name: ready_state.winner_name,
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

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use crate::adapters::BattleSessionHub;
    use async_trait::async_trait;
    use axum::body::Body;
    use axum::http::StatusCode;
    use common::domain::Error;
    use common::dto::{
        CreateGameRequestDto, GameDto, JoinGameRequestDto, SaveSelectedRaceRequestDto, UserDto,
    };
    use tower::ServiceExt;

    use super::*;
    use crate::adapters::db::{SessionsTable, SqliteAdapter, TableEntity, UsersTable};
    use crate::adapters::{ApiAdapter, SqliteSessionRepository};
    use crate::ports::{GameDrivingPort, LoggerDrivingPort, SessionRepositoryDrivenPort};
    use crate::test_helpers::FakeLoggerDrivingPort;
    use crate::test_helpers::sample_data::{TEST_PLAYER_NAME, test_create_game_request, test_game};

    const TEST_SESSION_ID: &str = "session-1";

    #[derive(Clone)]
    struct FakeGameDrivingPort {
        create_game_calls: Arc<Mutex<Vec<String>>>,
        games: Arc<Mutex<Vec<GameDto>>>,
    }

    impl FakeGameDrivingPort {
        fn new() -> Self {
            Self {
                create_game_calls: Arc::new(Mutex::new(Vec::new())),
                games: Arc::new(Mutex::new(vec![test_game()])),
            }
        }

        fn create_game_calls(&self) -> Vec<String> {
            self.create_game_calls
                .lock()
                .expect("create_game_calls lock poisoned")
                .clone()
        }
    }

    #[async_trait]
    impl GameDrivingPort for FakeGameDrivingPort {
        async fn create_game(
            &self,
            creator_name: String,
            _request: CreateGameRequestDto,
        ) -> Result<GameDto, Error> {
            self.create_game_calls
                .lock()
                .expect("create_game_calls lock poisoned")
                .push(creator_name);
            Ok(test_game())
        }

        async fn join_game(
            &self,
            _game_id: String,
            _player_name: String,
            _request: JoinGameRequestDto,
        ) -> Result<GameDto, Error> {
            Ok(test_game())
        }

        async fn leave_game(&self, _game_id: String, _player_name: String) -> Result<(), Error> {
            Ok(())
        }

        async fn cancel_game(&self, _game_id: String, _player_name: String) -> Result<(), Error> {
            Ok(())
        }

        async fn start_game(
            &self,
            _game_id: String,
            _player_name: String,
        ) -> Result<GameDto, Error> {
            Ok(test_game())
        }

        async fn save_selected_race(
            &self,
            _game_id: String,
            _player_name: String,
            _request: SaveSelectedRaceRequestDto,
        ) -> Result<GameDto, Error> {
            Ok(test_game())
        }

        async fn list_games(&self) -> Result<Vec<GameDto>, Error> {
            Ok(self.games.lock().expect("games lock poisoned").clone())
        }

        async fn find_game(&self, _game_id: String) -> Result<GameDto, Error> {
            Ok(test_game())
        }
    }

    struct TestGameApiAdapterBuilder {
        sqlite: SqliteAdapter,
        game: FakeGameDrivingPort,
        logger: FakeLoggerDrivingPort,
        with_session: bool,
    }

    impl TestGameApiAdapterBuilder {
        fn new() -> Self {
            let sqlite = SqliteAdapter::new(":memory:").expect("in-memory sqlite");
            sqlite
                .ensure_table::<UsersTable>()
                .expect("initialize users table");
            sqlite
                .ensure_table::<SessionsTable>()
                .expect("initialize sessions table");
            Self {
                sqlite,
                game: FakeGameDrivingPort::new(),
                logger: FakeLoggerDrivingPort::new(),
                with_session: false,
            }
        }

        fn with_session(mut self) -> Self {
            self.with_session = true;
            self
        }

        fn build(self) -> (axum::Router, FakeGameDrivingPort) {
            if self.with_session {
                insert_session(&self.sqlite, TEST_SESSION_ID, test_user());
            }
            let session_repo =
                SqliteSessionRepository::new(self.sqlite.clone()).expect("session repo");
            let app = GameApiAdapter::new(
                self.game.clone(),
                GameRoomHub::new(),
                BattleSessionHub::new(),
                session_repo,
                self.logger,
            )
            .routes();
            (app, self.game)
        }
    }

    fn test_user() -> UserDto {
        UserDto {
            id: 1,
            name: TEST_PLAYER_NAME.to_string(),
            profile_image_url: None,
        }
    }

    fn current_timestamp() -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_secs() as i64
    }

    fn insert_session(sqlite: &SqliteAdapter, session_id: &str, user: UserDto) {
        let user_json = serde_json::to_string(&user).expect("serialize user");
        sqlite
            .execute_with_params(
                &format!(
                    "INSERT INTO {} (session_id, user_json, last_active_at) VALUES (?, ?, ?)",
                    SessionsTable::table_name()
                ),
                &[
                    &session_id as &dyn rusqlite::types::ToSql,
                    &user_json,
                    &current_timestamp(),
                ],
            )
            .expect("insert session");
    }

    fn cookie_header() -> String {
        format!("battlecontrol-session={TEST_SESSION_ID}")
    }

    #[tokio::test]
    async fn list_games_requires_session_cookie() {
        let (app, _) = TestGameApiAdapterBuilder::new().build();

        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .method("GET")
                    .uri("/games")
                    .body(Body::empty())
                    .expect("build request"),
            )
            .await
            .expect("request response");

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn list_games_returns_games_for_valid_session() {
        let (app, _) = TestGameApiAdapterBuilder::new().with_session().build();

        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .method("GET")
                    .uri("/games")
                    .header("cookie", cookie_header())
                    .body(Body::empty())
                    .expect("build request"),
            )
            .await
            .expect("request response");
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("read body");
        let games: Vec<GameDto> = serde_json::from_slice(&body).expect("parse games");

        assert_eq!(games.len(), 1);
    }

    #[tokio::test]
    async fn create_game_uses_session_user_name_as_creator() {
        let (app, fake_game) = TestGameApiAdapterBuilder::new().with_session().build();
        let request_json =
            serde_json::to_string(&test_create_game_request()).expect("serialize request");

        let _ = app
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/games")
                    .header("content-type", "application/json")
                    .header("cookie", cookie_header())
                    .body(Body::from(request_json))
                    .expect("build request"),
            )
            .await
            .expect("request response");

        assert_eq!(
            fake_game.create_game_calls(),
            vec![TEST_PLAYER_NAME.to_string()]
        );
    }

    #[test]
    fn load_session_user_returns_none_for_stale_session() {
        let sqlite = SqliteAdapter::new(":memory:").expect("in-memory sqlite");
        sqlite
            .ensure_table::<SessionsTable>()
            .expect("initialize sessions table");
        let session_repo = SqliteSessionRepository::new(sqlite.clone()).expect("session repo");
        let user_json = serde_json::to_string(&test_user()).expect("serialize user");
        let stale_last_active_at = current_timestamp() - (8 * 24 * 60 * 60) - 1;
        sqlite
            .execute_with_params(
                &format!(
                    "INSERT INTO {} (session_id, user_json, last_active_at) VALUES (?, ?, ?)",
                    SessionsTable::table_name()
                ),
                &[
                    &TEST_SESSION_ID as &dyn rusqlite::types::ToSql,
                    &user_json,
                    &stale_last_active_at,
                ],
            )
            .expect("insert stale session");

        let session_user = session_repo
            .load_session_user(TEST_SESSION_ID)
            .expect("load session result");

        assert!(session_user.is_none());
    }

    #[test]
    fn implements_api_adapter() {
        fn assert_api_adapter<T: ApiAdapter>() {}
        fn assert_logger_port<T: LoggerDrivingPort>() {}
        assert_api_adapter::<
            GameApiAdapter<
                FakeGameDrivingPort,
                BattleSessionHub,
                SqliteSessionRepository,
                FakeLoggerDrivingPort,
            >,
        >();
        assert_logger_port::<FakeLoggerDrivingPort>();
    }
}
