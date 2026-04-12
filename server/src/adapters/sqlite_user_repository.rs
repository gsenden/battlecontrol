use async_trait::async_trait;
use common::domain::Error;
use common::domain::error::DatabaseErrorError;
use common::dto::{CreateGameRequestDto, GameDto, GamePlayerDto, JoinGameRequestDto, SaveSelectedRaceRequestDto, UserDto, UserSettingsDto};
use crate::ports::{GameRepositoryDrivenPort, UserRepositoryDrivenPort};
use super::db::{GamePlayersTable, GamesTable, PasskeysTable, SqliteAdapter, TableEntity, UserSettingsTable, UsersTable};
use uuid::Uuid;
use webauthn_rs::prelude::Passkey;

fn db_error() -> Error {
    Error::DatabaseError(DatabaseErrorError::new())
}

const STALE_GAME_TIMEOUT_SECONDS: i64 = 10 * 60;
const DEFAULT_SELECTED_RACE: &str = "human-cruiser";

#[derive(Clone)]
pub struct SqliteUserRepository {
    sqlite: SqliteAdapter,
}

impl SqliteUserRepository {
    pub fn new(sqlite: SqliteAdapter) -> Result<Self, String> {
        sqlite.ensure_table::<UsersTable>()?;
        sqlite.ensure_table::<PasskeysTable>()?;
        sqlite.ensure_table::<GamesTable>()?;
        sqlite.ensure_table::<GamePlayersTable>()?;
        sqlite.ensure_table::<UserSettingsTable>()?;
        Self::migrate_users_table(&sqlite)?;
        Self::migrate_games_table(&sqlite)?;
        Self::migrate_user_settings_table(&sqlite)?;
        Ok(Self { sqlite })
    }

    fn credential_id_string(passkey: &Passkey) -> Result<String, Error> {
        serde_json::to_value(passkey.cred_id())
            .map_err(|_| db_error())?
            .as_str()
            .map(str::to_owned)
            .ok_or_else(db_error)
    }

    fn migrate_users_table(sqlite: &SqliteAdapter) -> Result<(), String> {
        let rows = sqlite.query(&format!("PRAGMA table_info({})", UsersTable::table_name()))?;
        let has_user_handle = rows.iter().any(|row| {
            row.get::<String>("name")
                .map(|column_name| column_name == "user_handle")
                .unwrap_or(false)
        });

        if !has_user_handle {
            sqlite.execute(&format!(
                "ALTER TABLE {} ADD COLUMN user_handle TEXT",
                UsersTable::table_name()
            ))?;

            let user_rows = sqlite.query(&format!("SELECT id FROM {}", UsersTable::table_name()))?;
            for row in user_rows {
                let user_id: i64 = row.get("id")?;
                let user_handle = Uuid::new_v4().to_string();
                sqlite.execute_with_params(
                    &format!("UPDATE {} SET user_handle = ? WHERE id = ?", UsersTable::table_name()),
                    &[&user_handle as &dyn rusqlite::types::ToSql, &user_id],
                )?;
            }
        }

        let has_profile_image_url = rows.iter().any(|row| {
            row.get::<String>("name")
                .map(|column_name| column_name == "profile_image_url")
                .unwrap_or(false)
        });

        if !has_profile_image_url {
            sqlite.execute(&format!(
                "ALTER TABLE {} ADD COLUMN profile_image_url TEXT",
                UsersTable::table_name()
            ))?;
        }

        Ok(())
    }

    fn migrate_user_settings_table(sqlite: &SqliteAdapter) -> Result<(), String> {
        Self::ensure_user_settings_column(sqlite, "music_enabled", "INTEGER NOT NULL DEFAULT 1")?;
        Self::ensure_user_settings_column(sqlite, "music_volume", "INTEGER NOT NULL DEFAULT 45")?;
        Self::ensure_user_settings_column(sqlite, "sound_effects_enabled", "INTEGER NOT NULL DEFAULT 1")?;
        Self::ensure_user_settings_column(sqlite, "sound_effects_volume", "INTEGER NOT NULL DEFAULT 60")?;
        Ok(())
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

    fn ensure_user_settings_column(sqlite: &SqliteAdapter, column_name: &str, column_sql: &str) -> Result<(), String> {
        let rows = sqlite.query(&format!("PRAGMA table_info({})", UserSettingsTable::table_name()))?;
        let has_column = rows.iter().any(|row| {
            row.get::<String>("name")
                .map(|existing_column_name| existing_column_name == column_name)
                .unwrap_or(false)
        });

        if !has_column {
            sqlite.execute(&format!(
                "ALTER TABLE {} ADD COLUMN {} {}",
                UserSettingsTable::table_name(),
                column_name,
                column_sql
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
}

#[async_trait]
impl UserRepositoryDrivenPort for SqliteUserRepository {
    async fn find_by_name(&self, name: &str) -> Result<Option<UserDto>, Error> {
        let rows = self.sqlite.query_with_params(
            &format!("SELECT * FROM {} WHERE name = ?", UsersTable::table_name()),
            &[&name as &dyn rusqlite::types::ToSql],
        ).map_err(|_| db_error())?;

        match rows.first() {
            Some(row) => Ok(Some(UsersTable::from_row(row)
                .map_err(|_| db_error())?)),
            None => Ok(None),
        }
    }

    async fn find_user_handle_by_name(&self, name: &str) -> Result<Option<Uuid>, Error> {
        let rows = self.sqlite.query_with_params(
            &format!("SELECT user_handle FROM {} WHERE name = ?", UsersTable::table_name()),
            &[&name as &dyn rusqlite::types::ToSql],
        ).map_err(|_| db_error())?;

        match rows.first() {
            Some(row) => {
                let raw: String = row.get("user_handle").map_err(|_| db_error())?;
                let parsed = Uuid::parse_str(&raw).map_err(|_| db_error())?;
                Ok(Some(parsed))
            }
            None => Ok(None),
        }
    }

    async fn save_user(&self, name: &str, user_handle: Uuid) -> Result<UserDto, Error> {
        self.sqlite.execute_with_params(
            &format!("INSERT INTO {} (name, user_handle) VALUES (?, ?)", UsersTable::table_name()),
            &[&name as &dyn rusqlite::types::ToSql, &user_handle.to_string()],
        ).map_err(|_| db_error())?;

        let id = self.sqlite.last_insert_rowid()
            .map_err(|_| db_error())?;

        Ok(UserDto {
            id,
            name: name.to_string(),
            profile_image_url: None,
        })
    }

    async fn list_passkeys_by_name(&self, name: &str) -> Result<Vec<Passkey>, Error> {
        let rows = self.sqlite.query_with_params(
            &format!("SELECT * FROM {} WHERE user_name = ?", PasskeysTable::table_name()),
            &[&name as &dyn rusqlite::types::ToSql],
        ).map_err(|_| db_error())?;

        rows.iter()
            .map(|row| {
                let stored = PasskeysTable::from_row(row).map_err(|_| db_error())?;
                serde_json::from_str::<Passkey>(&stored.passkey_json).map_err(|_| db_error())
            })
            .collect()
    }

    async fn save_passkey(&self, name: &str, passkey: &Passkey) -> Result<(), Error> {
        let credential_id = Self::credential_id_string(passkey)?;
        let passkey_json = serde_json::to_string(passkey).map_err(|_| db_error())?;
        self.sqlite.execute_with_params(
            &format!(
                "INSERT INTO {} (user_name, credential_id, passkey_json) VALUES (?, ?, ?)",
                PasskeysTable::table_name()
            ),
            &[&name as &dyn rusqlite::types::ToSql, &credential_id, &passkey_json],
        ).map_err(|_| db_error())?;
        Ok(())
    }

    async fn update_passkey(&self, name: &str, passkey: &Passkey) -> Result<(), Error> {
        let credential_id = Self::credential_id_string(passkey)?;
        let passkey_json = serde_json::to_string(passkey).map_err(|_| db_error())?;
        self.sqlite.execute_with_params(
            &format!(
                "UPDATE {} SET passkey_json = ?, user_name = ? WHERE credential_id = ?",
                PasskeysTable::table_name()
            ),
            &[&passkey_json as &dyn rusqlite::types::ToSql, &name, &credential_id],
        ).map_err(|_| db_error())?;
        Ok(())
    }

    async fn update_user_profile(&self, current_name: &str, name: &str, profile_image_url: &str) -> Result<UserDto, Error> {
        self.sqlite.execute_with_params(
            &format!(
                "UPDATE {} SET name = ?, profile_image_url = ? WHERE name = ?",
                UsersTable::table_name()
            ),
            &[&name as &dyn rusqlite::types::ToSql, &profile_image_url, &current_name],
        ).map_err(|_| db_error())?;

        self.find_by_name(name).await?
            .ok_or_else(db_error)
    }

    async fn find_settings_by_name(&self, name: &str) -> Result<Option<UserSettingsDto>, Error> {
        let rows = self.sqlite.query_with_params(
            &format!("SELECT * FROM {} WHERE user_name = ?", UserSettingsTable::table_name()),
            &[&name as &dyn rusqlite::types::ToSql],
        ).map_err(|_| db_error())?;

        match rows.first() {
            Some(row) => Ok(Some(UserSettingsTable::from_row(row).map_err(|_| db_error())?)),
            None => Ok(None),
        }
    }

    async fn save_settings(&self, name: &str, settings: &UserSettingsDto) -> Result<UserSettingsDto, Error> {
        self.sqlite.execute_with_params(
            &format!(
                "INSERT INTO {} (
                    user_name,
                    turn_left_key,
                    turn_right_key,
                    thrust_key,
                    music_enabled,
                    music_volume,
                    sound_effects_enabled,
                    sound_effects_volume
                 )
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                 ON CONFLICT(user_name) DO UPDATE SET
                 turn_left_key = excluded.turn_left_key,
                 turn_right_key = excluded.turn_right_key,
                 thrust_key = excluded.thrust_key,
                 music_enabled = excluded.music_enabled,
                 music_volume = excluded.music_volume,
                 sound_effects_enabled = excluded.sound_effects_enabled,
                 sound_effects_volume = excluded.sound_effects_volume",
                UserSettingsTable::table_name()
            ),
            &[
                &name as &dyn rusqlite::types::ToSql,
                &settings.turn_left_key,
                &settings.turn_right_key,
                &settings.thrust_key,
                &if settings.music_enabled { 1 } else { 0 },
                &settings.music_volume,
                &if settings.sound_effects_enabled { 1 } else { 0 },
                &settings.sound_effects_volume,
            ],
        ).map_err(|_| db_error())?;

        Ok(settings.clone())
    }
}

#[async_trait]
impl GameRepositoryDrivenPort for SqliteUserRepository {
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
        let creator = self.find_by_name(creator_name).await?.ok_or_else(db_error)?;
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
            return Err(common::domain::Error::AuthenticationFailed(
                common::domain::error::AuthenticationFailedError::new(),
            ));
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
            return Err(common::domain::Error::AuthenticationFailed(
                common::domain::error::AuthenticationFailedError::new(),
            ));
        }

        self.delete_game_rows(game_id)?;
        Ok(Some(game))
    }

    async fn start_game(&self, game_id: &str, player_name: &str) -> Result<Option<GameDto>, Error> {
        let Some(game) = self.find_game(game_id).await? else {
            return Ok(None);
        };

        if game.creator.name != player_name {
            return Err(common::domain::Error::AuthenticationFailed(
                common::domain::error::AuthenticationFailedError::new(),
            ));
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
    use crate::test_helpers::sample_data::{test_user_settings, TEST_PLAYER_NAME};

    fn repo_in_memory() -> SqliteUserRepository {
        let sqlite = SqliteAdapter::new(":memory:").unwrap();
        SqliteUserRepository::new(sqlite).unwrap()
    }

    #[tokio::test]
    async fn list_games_removes_stale_games() {
        let repo = repo_in_memory();
        repo.save_user("Host", Uuid::new_v4()).await.unwrap();
        let game = repo.save_game("Host", &CreateGameRequestDto {
            name: "Stale".to_string(),
            game_type: "free_for_all".to_string(),
            max_players: 4,
            is_private: false,
            password: None,
        }).await.unwrap();
        repo.sqlite.execute_with_params(
            &format!("UPDATE {} SET last_activity_at = ? WHERE id = ?", GamesTable::table_name()),
            &[&0i64 as &dyn rusqlite::types::ToSql, &game.id],
        ).unwrap();

        let games = repo.delete_stale_games().await.unwrap();

        assert_eq!(games, vec![game.id]);
    }

    #[tokio::test]
    async fn start_game_removes_game_from_repository() {
        let repo = repo_in_memory();
        repo.save_user("Host", Uuid::new_v4()).await.unwrap();
        let game = repo.save_game("Host", &CreateGameRequestDto {
            name: "Ready".to_string(),
            game_type: "free_for_all".to_string(),
            max_players: 4,
            is_private: false,
            password: None,
        }).await.unwrap();

        repo.start_game(&game.id, "Host").await.unwrap();
        let found = repo.find_game(&game.id).await.unwrap();

        assert!(found.is_none());
    }

    #[tokio::test]
    async fn save_settings_persists_mute_flags() {
        let repo = repo_in_memory();
        let mut expected_settings = test_user_settings();
        expected_settings.music_enabled = false;
        expected_settings.sound_effects_enabled = false;
        repo.save_settings(TEST_PLAYER_NAME, &expected_settings).await.unwrap();
        let found = repo.find_settings_by_name(TEST_PLAYER_NAME).await.unwrap().unwrap();
        assert_eq!(
            (found.music_enabled, found.sound_effects_enabled),
            (false, false),
        );
    }

    #[tokio::test]
    async fn save_settings_can_be_found_by_name() {
        let repo = repo_in_memory();
        let expected_settings = test_user_settings();
        repo.save_settings(TEST_PLAYER_NAME, &expected_settings).await.unwrap();
        let found = repo.find_settings_by_name(TEST_PLAYER_NAME).await.unwrap();
        assert_eq!(found.unwrap().music_volume, expected_settings.music_volume);
    }

    #[test]
    fn new_migrates_existing_users_table_without_user_handle() {
        let sqlite = SqliteAdapter::new(":memory:").unwrap();
        sqlite.execute("CREATE TABLE users (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL UNIQUE)").unwrap();
        SqliteUserRepository::new(sqlite.clone()).unwrap();
        let columns = sqlite.query("PRAGMA table_info(users)").unwrap();
        let has_user_handle = columns.iter().any(|row| {
            row.get::<String>("name")
                .map(|column_name| column_name == "user_handle")
                .unwrap_or(false)
        });
        assert!(has_user_handle);
    }

    #[tokio::test]
    async fn save_user_can_be_found_by_name() {
        let repo = repo_in_memory();
        repo.save_user("TestPlayer", Uuid::new_v4()).await.unwrap();
        let found = repo.find_by_name("TestPlayer").await.unwrap();
        assert!(found.is_some());
    }

    #[tokio::test]
    async fn save_user_can_be_found_by_user_handle() {
        let repo = repo_in_memory();
        let user_handle = Uuid::new_v4();
        repo.save_user("TestPlayer", user_handle).await.unwrap();
        let found = repo.find_user_handle_by_name("TestPlayer").await.unwrap();
        assert_eq!(found, Some(user_handle));
    }
}
