use async_trait::async_trait;
use common::domain::Error;
use common::domain::error::DatabaseErrorError;
use common::dto::UserDto;
use crate::ports::UserRepositoryDrivenPort;
use super::db::{PasskeysTable, SqliteAdapter, TableEntity, UsersTable};
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
        Self::migrate_users_table(&sqlite)?;
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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo_in_memory() -> SqliteUserRepository {
        let sqlite = SqliteAdapter::new(":memory:").unwrap();
        SqliteUserRepository::new(sqlite).unwrap()
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
