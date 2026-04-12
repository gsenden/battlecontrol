mod adapters;
mod ports;
mod domain;

#[cfg(test)]
mod test_helpers;

use adapters::{AuthApiAdapter, AxumAdapter, GameApiAdapter, SqliteUserRepository, TracingLoggerAdapter};
use adapters::db::SqliteAdapter;
use domain::{Authenticator, AuthenticatorDrivenPorts, GameLobby, GameLobbyDrivenPorts};
use std::path::{Path, PathBuf};

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
    type GameRepo = SqliteUserRepository;
}

#[tokio::main]
async fn main() {
    let sqlite = SqliteAdapter::new(&database_path())
        .expect("Failed to open database");
    let user_repo = SqliteUserRepository::new(sqlite.clone())
        .expect("Failed to initialize user repository");
    let authenticator = Authenticator::<ProductionDrivenPorts>::new(user_repo.clone());
    let game_lobby = GameLobby::<ProductionGameDrivenPorts>::new(user_repo);
    let logger = TracingLoggerAdapter;

    AxumAdapter::new()
        .register(AuthApiAdapter::new(authenticator, logger.clone(), sqlite.clone()))
        .register(GameApiAdapter::new(game_lobby, logger, sqlite))
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
