mod adapters;
mod domain;
mod ports;

#[cfg(test)]
mod test_helpers;

use adapters::db::SqliteAdapter;
use adapters::{
    AuthApiAdapter, AxumAdapter, BattleSessionHub, GameApiAdapter, GameRoomHub,
    SqliteGameRepository, SqliteSessionRepository, SqliteUserRepository, TracingLoggerAdapter,
};
use domain::{Authenticator, AuthenticatorDrivenPorts, GameLobby, GameLobbyDrivenPorts};
use ports::{GameRepositoryDrivenPort, GameRoomDrivenPort};
use std::path::{Path, PathBuf};
use std::time::Duration;

fn database_path() -> String {
    common::domain::EnvVar::ServerDatabasePath.value()
}

fn uploads_path() -> String {
    let database_path = PathBuf::from(database_path());
    let base_dir = database_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    base_dir.join("uploads").to_string_lossy().to_string()
}

struct ProductionDrivenPorts;
impl AuthenticatorDrivenPorts for ProductionDrivenPorts {
    type UserRepo = SqliteUserRepository;
}

struct ProductionGameDrivenPorts;
impl GameLobbyDrivenPorts for ProductionGameDrivenPorts {
    type GameRepo = SqliteGameRepository;
    type GameRooms = GameRoomHub;
}

fn spawn_stale_game_cleanup(game_repo: SqliteGameRepository, game_rooms: GameRoomHub) {
    tokio::spawn(async move {
        loop {
            if let Ok(stale_game_ids) = game_repo.delete_stale_games().await
                && !stale_game_ids.is_empty()
            {
                game_rooms.remove_rooms(&stale_game_ids);
            }
            tokio::time::sleep(Duration::from_secs(60)).await;
        }
    });
}

#[tokio::main]
async fn main() {
    let sqlite = SqliteAdapter::new(&database_path()).expect("Failed to open database");
    let user_repo =
        SqliteUserRepository::new(sqlite.clone()).expect("Failed to initialize user repository");
    let game_repo =
        SqliteGameRepository::new(sqlite.clone()).expect("Failed to initialize game repository");
    let session_repo = SqliteSessionRepository::new(sqlite.clone())
        .expect("Failed to initialize session repository");
    let game_rooms = GameRoomHub::new();
    let battle_sessions = BattleSessionHub::new();
    let authenticator = Authenticator::<ProductionDrivenPorts>::new(user_repo.clone());
    let cleanup_game_repo = game_repo.clone();
    let game_lobby = GameLobby::<ProductionGameDrivenPorts>::new(game_repo, game_rooms.clone());
    let logger = TracingLoggerAdapter;
    spawn_stale_game_cleanup(cleanup_game_repo, game_rooms.clone());

    AxumAdapter::new()
        .register(AuthApiAdapter::new(authenticator, logger, sqlite.clone()))
        .register(GameApiAdapter::new(
            game_lobby,
            game_rooms,
            battle_sessions,
            session_repo,
            logger,
        ))
        .serve_directory("/uploads", &uploads_path())
        .serve_spa("frontend/build")
        .serve()
        .await;
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_DATABASE_PATH: &str = "/data/battlecontrol.db";

    #[test]
    fn database_path_returns_env_var() {
        unsafe {
            std::env::set_var("MATTER_SERVER_DATABASE_PATH", TEST_DATABASE_PATH);
        }
        assert_eq!(database_path(), TEST_DATABASE_PATH);
        unsafe {
            std::env::remove_var("MATTER_SERVER_DATABASE_PATH");
        }
    }

    #[test]
    fn database_path_returns_default() {
        assert_eq!(database_path(), "battlecontrol.db");
    }

    #[test]
    fn database_path_exists() {
        let _ = database_path();
    }
}
