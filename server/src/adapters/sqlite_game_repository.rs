use async_trait::async_trait;
use common::domain::Error;
use common::domain::error::{AuthenticationFailedError, DatabaseErrorError};
use common::dto::{CreateGameRequestDto, GameDto, GamePlayerDto, JoinGameRequestDto, SaveSelectedRaceRequestDto};
use uuid::Uuid;

use crate::ports::GameRepositoryDrivenPort;

use super::db::{GamePlayersTable, GamesTable, SqliteAdapter, TableEntity, UsersTable};

const STALE_GAME_TIMEOUT_SECONDS: i64 = 10 * 60;
const DEFAULT_SELECTED_RACE: &str = "human-cruiser";

fn db_error() -> Error {
    Error::DatabaseError(DatabaseErrorError::new())
}

#[derive(Clone)]
pub struct SqliteGameRepository {
    sqlite: SqliteAdapter,
}

impl SqliteGameRepository {
    pub fn new(sqlite: SqliteAdapter) -> Result<Self, String> {
        sqlite.ensure_table::<GamesTable>()?;
        sqlite.ensure_table::<GamePlayersTable>()?;
        Self::migrate_games_table(&sqlite)?;
        Ok(Self { sqlite })
    }

    fn migrate_games_table(sqlite: &SqliteAdapter) -> Result<(), String> {
        let rows = sqlite.query(&format!("PRAGMA table_info({})", GamesTable::table_name()))?;
        let has_last_activity_at = rows.iter().any(|row| {
            row.get::<String>("name")
                .map(|existing_column_name| existing_column_name == "last_activity_at")
                .unwrap_or(false)
        });

        if !has_last_activity_at {
            sqlite.execute(&format!(
                "ALTER TABLE {} ADD COLUMN last_activity_at INTEGER NOT NULL DEFAULT 0",
                GamesTable::table_name()
            ))?;
            sqlite.execute(&format!(
                "UPDATE {} SET last_activity_at = created_at WHERE last_activity_at = 0",
                GamesTable::table_name()
            ))?;
        }

        Ok(())
    }

    fn game_from_row(&self, row: &super::db::Row) -> Result<GameDto, Error> {
        let mut game = GamesTable::from_row(row).map_err(|_| db_error())?;
        game.players = self.find_players_by_game_id(&game.id)?;
        Ok(game)
    }

    fn find_players_by_game_id(&self, game_id: &str) -> Result<Vec<GamePlayerDto>, Error> {
        let rows = self.sqlite.query_with_params(
            &format!(
                "SELECT u.id, u.name, u.profile_image_url, gp.selected_race
                 FROM {} gp
                 JOIN {} u ON u.name = gp.user_name
                 WHERE gp.game_id = ?
                 ORDER BY u.name ASC",
                GamePlayersTable::table_name(),
                UsersTable::table_name()
            ),
            &[&game_id as &dyn rusqlite::types::ToSql],
        ).map_err(|_| db_error())?;

        rows.iter()
            .map(|row| {
                Ok(GamePlayerDto {
                    user: UsersTable::from_row(row).map_err(|_| db_error())?,
                    selected_race: row.get("selected_race").ok(),
                })
            })
            .collect()
    }

    fn current_timestamp() -> Result<i64, Error> {
        Ok(std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| db_error())?
            .as_secs() as i64)
    }

    fn delete_game_rows(&self, game_id: &str) -> Result<(), Error> {
        self.sqlite.execute_with_params(
            &format!("DELETE FROM {} WHERE game_id = ?", GamePlayersTable::table_name()),
            &[&game_id as &dyn rusqlite::types::ToSql],
        ).map_err(|_| db_error())?;

        self.sqlite.execute_with_params(
            &format!("DELETE FROM {} WHERE id = ?", GamesTable::table_name()),
            &[&game_id as &dyn rusqlite::types::ToSql],
        ).map_err(|_| db_error())
    }

    fn touch_game_activity(&self, game_id: &str, timestamp: i64) -> Result<(), Error> {
        self.sqlite.execute_with_params(
            &format!("UPDATE {} SET last_activity_at = ? WHERE id = ?", GamesTable::table_name()),
            &[&timestamp as &dyn rusqlite::types::ToSql, &game_id],
        ).map_err(|_| db_error())?;

        Ok(())
    }

    async fn find_user_by_name(&self, name: &str) -> Result<Option<common::dto::UserDto>, Error> {
        let rows = self.sqlite.query_with_params(
            &format!("SELECT * FROM {} WHERE name = ?", UsersTable::table_name()),
            &[&name as &dyn rusqlite::types::ToSql],
        ).map_err(|_| db_error())?;

        match rows.first() {
            Some(row) => Ok(Some(UsersTable::from_row(row).map_err(|_| db_error())?)),
            None => Ok(None),
        }
    }
}

#[async_trait]
impl GameRepositoryDrivenPort for SqliteGameRepository {
    async fn delete_stale_games(&self) -> Result<Vec<String>, Error> {
        let stale_before = Self::current_timestamp()? - STALE_GAME_TIMEOUT_SECONDS;
        let stale_rows = self.sqlite.query_with_params(
            &format!("SELECT id FROM {} WHERE last_activity_at < ?", GamesTable::table_name()),
            &[&stale_before as &dyn rusqlite::types::ToSql],
        ).map_err(|_| db_error())?;

        let stale_game_ids = stale_rows
            .iter()
            .map(|row| row.get::<String>("id").map_err(|_| db_error()))
            .collect::<Result<Vec<_>, _>>()?;

        for game_id in &stale_game_ids {
            self.delete_game_rows(game_id)?;
        }

        Ok(stale_game_ids)
    }

    async fn save_game(&self, creator_name: &str, request: &CreateGameRequestDto) -> Result<GameDto, Error> {
        let creator = self.find_user_by_name(creator_name).await?.ok_or_else(db_error)?;
        let game_id = Uuid::new_v4().to_string();
        let now = Self::current_timestamp()?;

        self.sqlite.execute_with_params(
            &format!(
                "INSERT INTO {} (
                    id,
                    name,
                    game_type,
                    max_players,
                    is_private,
                    password,
                    creator_name,
                    creator_id,
                    creator_profile_image_url,
                    created_at,
                    last_activity_at
                 ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                GamesTable::table_name()
            ),
            &[
                &game_id as &dyn rusqlite::types::ToSql,
                &request.name,
                &"free_for_all",
                &request.max_players,
                &if request.is_private { 1 } else { 0 },
                &request.password,
                &creator.name,
                &creator.id,
                &creator.profile_image_url,
                &now,
                &now,
            ],
        ).map_err(|_| db_error())?;

        self.sqlite.execute_with_params(
            &format!(
                "INSERT INTO {} (game_id, user_name, selected_race) VALUES (?, ?, ?)",
                GamePlayersTable::table_name()
            ),
            &[&game_id as &dyn rusqlite::types::ToSql, &creator.name, &DEFAULT_SELECTED_RACE],
        ).map_err(|_| db_error())?;

        self.find_game(&game_id).await?.ok_or_else(db_error)
    }

    async fn list_games(&self) -> Result<Vec<GameDto>, Error> {
        let rows = self.sqlite.query(
            &format!(
                "SELECT * FROM {} ORDER BY created_at DESC",
                GamesTable::table_name()
            ),
        ).map_err(|_| db_error())?;

        rows.iter().map(|row| self.game_from_row(row)).collect()
    }

    async fn find_game(&self, game_id: &str) -> Result<Option<GameDto>, Error> {
        let rows = self.sqlite.query_with_params(
            &format!("SELECT * FROM {} WHERE id = ?", GamesTable::table_name()),
            &[&game_id as &dyn rusqlite::types::ToSql],
        ).map_err(|_| db_error())?;

        match rows.first() {
            Some(row) => Ok(Some(self.game_from_row(row)?)),
            None => Ok(None),
        }
    }

    async fn join_game(&self, game_id: &str, player_name: &str, request: &JoinGameRequestDto) -> Result<Option<GameDto>, Error> {
        let Some(game) = self.find_game(game_id).await? else {
            return Ok(None);
        };

        if game.is_private && game.password != request.password {
            return Err(Error::AuthenticationFailed(AuthenticationFailedError::new()));
        }

        let player_exists = self.sqlite.query_with_params(
            &format!(
                "SELECT game_id FROM {} WHERE game_id = ? AND user_name = ?",
                GamePlayersTable::table_name()
            ),
            &[&game_id as &dyn rusqlite::types::ToSql, &player_name],
        ).map_err(|_| db_error())?;

        if player_exists.is_empty() {
            self.sqlite.execute_with_params(
                &format!(
                    "INSERT INTO {} (game_id, user_name, selected_race) VALUES (?, ?, ?)",
                    GamePlayersTable::table_name()
                ),
                &[&game_id as &dyn rusqlite::types::ToSql, &player_name, &DEFAULT_SELECTED_RACE],
            ).map_err(|_| db_error())?;
        }

        self.touch_game_activity(game_id, Self::current_timestamp()?)?;
        self.find_game(game_id).await
    }

    async fn leave_game(&self, game_id: &str, player_name: &str) -> Result<Option<GameDto>, Error> {
        let Some(game) = self.find_game(game_id).await? else {
            return Ok(None);
        };

        self.sqlite.execute_with_params(
            &format!("DELETE FROM {} WHERE game_id = ? AND user_name = ?", GamePlayersTable::table_name()),
            &[&game_id as &dyn rusqlite::types::ToSql, &player_name],
        ).map_err(|_| db_error())?;

        if game.creator.name == player_name {
            self.delete_game_rows(game_id)?;
            return Ok(Some(GameDto {
                players: Vec::new(),
                ..game
            }));
        }

        self.touch_game_activity(game_id, Self::current_timestamp()?)?;
        self.find_game(game_id).await
    }

    async fn cancel_game(&self, game_id: &str, player_name: &str) -> Result<Option<GameDto>, Error> {
        let Some(game) = self.find_game(game_id).await? else {
            return Ok(None);
        };

        if game.creator.name != player_name {
            return Err(Error::AuthenticationFailed(AuthenticationFailedError::new()));
        }

        self.delete_game_rows(game_id)?;
        Ok(Some(game))
    }

    async fn start_game(&self, game_id: &str, player_name: &str) -> Result<Option<GameDto>, Error> {
        let Some(game) = self.find_game(game_id).await? else {
            return Ok(None);
        };

        if game.creator.name != player_name {
            return Err(Error::AuthenticationFailed(AuthenticationFailedError::new()));
        }

        self.delete_game_rows(game_id)?;
        Ok(Some(game))
    }

    async fn save_selected_race(&self, game_id: &str, player_name: &str, request: &SaveSelectedRaceRequestDto) -> Result<Option<GameDto>, Error> {
        let player_exists = self.sqlite.query_with_params(
            &format!(
                "SELECT game_id FROM {} WHERE game_id = ? AND user_name = ?",
                GamePlayersTable::table_name()
            ),
            &[&game_id as &dyn rusqlite::types::ToSql, &player_name],
        ).map_err(|_| db_error())?;

        if player_exists.is_empty() {
            return Ok(None);
        }

        self.sqlite.execute_with_params(
            &format!(
                "UPDATE {} SET selected_race = ? WHERE game_id = ? AND user_name = ?",
                GamePlayersTable::table_name()
            ),
            &[&request.selected_race as &dyn rusqlite::types::ToSql, &game_id, &player_name],
        ).map_err(|_| db_error())?;

        self.touch_game_activity(game_id, Self::current_timestamp()?)?;
        self.find_game(game_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::SqliteUserRepository;
    use crate::adapters::db::SqliteAdapter;
    use crate::ports::UserRepositoryDrivenPort;

    fn repos_in_memory() -> (SqliteUserRepository, SqliteGameRepository) {
        let sqlite = SqliteAdapter::new(":memory:").unwrap();
        let user_repo = SqliteUserRepository::new(sqlite.clone()).unwrap();
        let game_repo = SqliteGameRepository::new(sqlite).unwrap();
        (user_repo, game_repo)
    }

    #[tokio::test]
    async fn list_games_removes_stale_games() {
        let (user_repo, game_repo) = repos_in_memory();
        user_repo.save_user("Host", Uuid::new_v4()).await.unwrap();
        let game = game_repo.save_game("Host", &CreateGameRequestDto {
            name: "Stale".to_string(),
            game_type: "free_for_all".to_string(),
            max_players: 4,
            is_private: false,
            password: None,
        }).await.unwrap();
        game_repo.sqlite.execute_with_params(
            &format!("UPDATE {} SET last_activity_at = ? WHERE id = ?", GamesTable::table_name()),
            &[&0i64 as &dyn rusqlite::types::ToSql, &game.id],
        ).unwrap();

        let games = game_repo.delete_stale_games().await.unwrap();

        assert_eq!(games, vec![game.id]);
    }

    #[tokio::test]
    async fn start_game_removes_game_from_repository() {
        let (user_repo, game_repo) = repos_in_memory();
        user_repo.save_user("Host", Uuid::new_v4()).await.unwrap();
        let game = game_repo.save_game("Host", &CreateGameRequestDto {
            name: "Ready".to_string(),
            game_type: "free_for_all".to_string(),
            max_players: 4,
            is_private: false,
            password: None,
        }).await.unwrap();

        game_repo.start_game(&game.id, "Host").await.unwrap();
        let found = game_repo.find_game(&game.id).await.unwrap();

        assert!(found.is_none());
    }
}
