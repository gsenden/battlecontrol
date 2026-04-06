use async_trait::async_trait;
use common::domain::Error;
use common::domain::error::DatabaseErrorError;
use common::dto::UserDto;
use crate::ports::UserRepositoryDrivenPort;
use super::db::{SqliteAdapter, TableEntity, UsersTable};

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
        Ok(Self { sqlite })
    }
}

#[async_trait]
impl UserRepositoryDrivenPort for SqliteUserRepository {
    async fn find_by_email(&self, email: &str) -> Result<Option<UserDto>, Error> {
        let rows = self.sqlite.query_with_params(
            &format!("SELECT * FROM {} WHERE email = ?", UsersTable::table_name()),
            &[&email as &dyn rusqlite::types::ToSql],
        ).map_err(|_| db_error())?;

        match rows.first() {
            Some(row) => Ok(Some(UsersTable::from_row(row)
                .map_err(|_| db_error())?)),
            None => Ok(None),
        }
    }

    async fn save_user(&self, name: &str, email: &str) -> Result<UserDto, Error> {
        self.sqlite.execute_with_params(
            &format!("INSERT INTO {} (name, email) VALUES (?, ?)", UsersTable::table_name()),
            &[&name as &dyn rusqlite::types::ToSql, &email],
        ).map_err(|_| db_error())?;

        let id = self.sqlite.last_insert_rowid()
            .map_err(|_| db_error())?;

        Ok(UserDto {
            id,
            name: name.to_string(),
            email: email.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo_in_memory() -> SqliteUserRepository {
        let sqlite = SqliteAdapter::new(":memory:").unwrap();
        SqliteUserRepository::new(sqlite).unwrap()
    }

    #[tokio::test]
    async fn save_user_can_be_found_by_email() {
        let repo = repo_in_memory();
        repo.save_user("TestPlayer", "test@test.com").await.unwrap();
        let found = repo.find_by_email("test@test.com").await.unwrap();
        assert!(found.is_some());
    }

    #[tokio::test]
    async fn find_by_email_returns_none_when_not_found() {
        let repo = repo_in_memory();
        let found = repo.find_by_email("nobody@test.com").await.unwrap();
        assert!(found.is_none());
    }
}
