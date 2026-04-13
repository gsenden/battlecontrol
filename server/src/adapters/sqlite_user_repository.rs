use super::db::{
    PasskeysTable, RecoveryCodesTable, SqliteAdapter, TableEntity, UserSettingsTable, UsersTable,
};
use crate::ports::UserRepositoryDrivenPort;
use async_trait::async_trait;
use common::domain::Error;
use common::domain::error::DatabaseErrorError;
use common::dto::{UserDto, UserSettingsDto};
use uuid::Uuid;
use webauthn_rs::prelude::Passkey;

fn db_error() -> Error {
    Error::DatabaseError(DatabaseErrorError::new())
}

#[derive(Clone)]
pub struct SqliteUserRepository {
    sqlite: SqliteAdapter,
}

impl SqliteUserRepository {
    pub fn new(sqlite: SqliteAdapter) -> Result<Self, String> {
        sqlite.ensure_table::<UsersTable>()?;
        sqlite.ensure_table::<PasskeysTable>()?;
        sqlite.ensure_table::<RecoveryCodesTable>()?;
        sqlite.ensure_table::<UserSettingsTable>()?;
        Self::migrate_users_table(&sqlite)?;
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

            let user_rows =
                sqlite.query(&format!("SELECT id FROM {}", UsersTable::table_name()))?;
            for row in user_rows {
                let user_id: i64 = row.get("id")?;
                let user_handle = Uuid::new_v4().to_string();
                sqlite.execute_with_params(
                    &format!(
                        "UPDATE {} SET user_handle = ? WHERE id = ?",
                        UsersTable::table_name()
                    ),
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
        Self::ensure_user_settings_column(
            sqlite,
            "sound_effects_enabled",
            "INTEGER NOT NULL DEFAULT 1",
        )?;
        Self::ensure_user_settings_column(
            sqlite,
            "sound_effects_volume",
            "INTEGER NOT NULL DEFAULT 60",
        )?;
        Ok(())
    }

    fn ensure_user_settings_column(
        sqlite: &SqliteAdapter,
        column_name: &str,
        column_sql: &str,
    ) -> Result<(), String> {
        let rows = sqlite.query(&format!(
            "PRAGMA table_info({})",
            UserSettingsTable::table_name()
        ))?;
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

    fn current_timestamp() -> Result<i64, Error> {
        Ok(std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| db_error())?
            .as_secs() as i64)
    }
}

#[async_trait]
impl UserRepositoryDrivenPort for SqliteUserRepository {
    async fn find_by_name(&self, name: &str) -> Result<Option<UserDto>, Error> {
        let rows = self
            .sqlite
            .query_with_params(
                &format!("SELECT * FROM {} WHERE name = ?", UsersTable::table_name()),
                &[&name as &dyn rusqlite::types::ToSql],
            )
            .map_err(|_| db_error())?;

        match rows.first() {
            Some(row) => Ok(Some(UsersTable::from_row(row).map_err(|_| db_error())?)),
            None => Ok(None),
        }
    }

    async fn save_user(&self, name: &str, user_handle: Uuid) -> Result<UserDto, Error> {
        self.sqlite
            .execute_with_params(
                &format!(
                    "INSERT INTO {} (name, user_handle) VALUES (?, ?)",
                    UsersTable::table_name()
                ),
                &[
                    &name as &dyn rusqlite::types::ToSql,
                    &user_handle.to_string(),
                ],
            )
            .map_err(|_| db_error())?;

        let id = self.sqlite.last_insert_rowid().map_err(|_| db_error())?;

        Ok(UserDto {
            id,
            name: name.to_string(),
            profile_image_url: None,
        })
    }

    async fn list_passkeys_by_name(&self, name: &str) -> Result<Vec<Passkey>, Error> {
        let rows = self
            .sqlite
            .query_with_params(
                &format!(
                    "SELECT * FROM {} WHERE user_name = ?",
                    PasskeysTable::table_name()
                ),
                &[&name as &dyn rusqlite::types::ToSql],
            )
            .map_err(|_| db_error())?;

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
        self.sqlite
            .execute_with_params(
                &format!(
                    "INSERT INTO {} (user_name, credential_id, passkey_json) VALUES (?, ?, ?)",
                    PasskeysTable::table_name()
                ),
                &[
                    &name as &dyn rusqlite::types::ToSql,
                    &credential_id,
                    &passkey_json,
                ],
            )
            .map_err(|_| db_error())?;
        Ok(())
    }

    async fn update_passkey(&self, name: &str, passkey: &Passkey) -> Result<(), Error> {
        let credential_id = Self::credential_id_string(passkey)?;
        let passkey_json = serde_json::to_string(passkey).map_err(|_| db_error())?;
        self.sqlite
            .execute_with_params(
                &format!(
                    "UPDATE {} SET passkey_json = ?, user_name = ? WHERE credential_id = ?",
                    PasskeysTable::table_name()
                ),
                &[
                    &passkey_json as &dyn rusqlite::types::ToSql,
                    &name,
                    &credential_id,
                ],
            )
            .map_err(|_| db_error())?;
        Ok(())
    }

    async fn create_recovery_code(
        &self,
        user_name: &str,
        recovery_code: &str,
        expires_at: i64,
    ) -> Result<(), Error> {
        self.sqlite
            .execute_with_params(
                &format!(
                    "DELETE FROM {} WHERE user_name = ?",
                    RecoveryCodesTable::table_name()
                ),
                &[&user_name as &dyn rusqlite::types::ToSql],
            )
            .map_err(|_| db_error())?;

        self.sqlite.execute_with_params(
            &format!(
                "INSERT INTO {} (recovery_code, user_name, expires_at, used_at) VALUES (?, ?, ?, NULL)",
                RecoveryCodesTable::table_name()
            ),
            &[&recovery_code as &dyn rusqlite::types::ToSql, &user_name, &expires_at],
        ).map_err(|_| db_error())?;

        Ok(())
    }

    async fn find_by_recovery_code(
        &self,
        recovery_code: &str,
        now: i64,
    ) -> Result<Option<UserDto>, Error> {
        self.sqlite
            .execute_with_params(
                &format!(
                    "DELETE FROM {} WHERE expires_at < ? OR used_at IS NOT NULL",
                    RecoveryCodesTable::table_name()
                ),
                &[&now as &dyn rusqlite::types::ToSql],
            )
            .map_err(|_| db_error())?;

        let rows = self.sqlite.query_with_params(
            &format!(
                "SELECT u.* FROM {} rc JOIN {} u ON u.name = rc.user_name WHERE rc.recovery_code = ? AND rc.expires_at >= ? AND rc.used_at IS NULL",
                RecoveryCodesTable::table_name(),
                UsersTable::table_name()
            ),
            &[&recovery_code as &dyn rusqlite::types::ToSql, &now],
        ).map_err(|_| db_error())?;

        match rows.first() {
            Some(row) => Ok(Some(UsersTable::from_row(row).map_err(|_| db_error())?)),
            None => Ok(None),
        }
    }

    async fn mark_recovery_code_used(&self, recovery_code: &str) -> Result<(), Error> {
        self.sqlite
            .execute_with_params(
                &format!(
                    "UPDATE {} SET used_at = ? WHERE recovery_code = ?",
                    RecoveryCodesTable::table_name()
                ),
                &[
                    &Self::current_timestamp()? as &dyn rusqlite::types::ToSql,
                    &recovery_code,
                ],
            )
            .map_err(|_| db_error())?;

        Ok(())
    }

    async fn update_user_profile(
        &self,
        current_name: &str,
        name: &str,
        profile_image_url: &str,
    ) -> Result<UserDto, Error> {
        self.sqlite
            .execute_with_params(
                &format!(
                    "UPDATE {} SET name = ?, profile_image_url = ? WHERE name = ?",
                    UsersTable::table_name()
                ),
                &[
                    &name as &dyn rusqlite::types::ToSql,
                    &profile_image_url,
                    &current_name,
                ],
            )
            .map_err(|_| db_error())?;

        self.find_by_name(name).await?.ok_or_else(db_error)
    }

    async fn find_settings_by_name(&self, name: &str) -> Result<Option<UserSettingsDto>, Error> {
        let rows = self
            .sqlite
            .query_with_params(
                &format!(
                    "SELECT * FROM {} WHERE user_name = ?",
                    UserSettingsTable::table_name()
                ),
                &[&name as &dyn rusqlite::types::ToSql],
            )
            .map_err(|_| db_error())?;

        match rows.first() {
            Some(row) => Ok(Some(
                UserSettingsTable::from_row(row).map_err(|_| db_error())?,
            )),
            None => Ok(None),
        }
    }

    async fn save_settings(
        &self,
        name: &str,
        settings: &UserSettingsDto,
    ) -> Result<UserSettingsDto, Error> {
        self.sqlite
            .execute_with_params(
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
            )
            .map_err(|_| db_error())?;

        Ok(settings.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::sample_data::{TEST_PLAYER_NAME, test_user_settings};

    fn repo_in_memory() -> SqliteUserRepository {
        let sqlite = SqliteAdapter::new(":memory:").unwrap();
        SqliteUserRepository::new(sqlite).unwrap()
    }

    #[tokio::test]
    async fn create_recovery_code_can_be_found_by_code() {
        let repo = repo_in_memory();
        let user = repo.save_user("TestPlayer", Uuid::new_v4()).await.unwrap();
        repo.create_recovery_code(&user.name, "RECOVERY123", 9_999_999_999)
            .await
            .unwrap();
        let found = repo.find_by_recovery_code("RECOVERY123", 0).await.unwrap();
        assert_eq!(found.unwrap().name, user.name);
    }

    #[tokio::test]
    async fn save_settings_persists_mute_flags() {
        let repo = repo_in_memory();
        let mut expected_settings = test_user_settings();
        expected_settings.music_enabled = false;
        expected_settings.sound_effects_enabled = false;
        repo.save_settings(TEST_PLAYER_NAME, &expected_settings)
            .await
            .unwrap();
        let found = repo
            .find_settings_by_name(TEST_PLAYER_NAME)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            (found.music_enabled, found.sound_effects_enabled),
            (false, false),
        );
    }

    #[tokio::test]
    async fn save_settings_can_be_found_by_name() {
        let repo = repo_in_memory();
        let expected_settings = test_user_settings();
        repo.save_settings(TEST_PLAYER_NAME, &expected_settings)
            .await
            .unwrap();
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
}
