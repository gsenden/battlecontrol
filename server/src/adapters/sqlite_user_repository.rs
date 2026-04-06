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

    async fn save_user(&self, name: &str) -> Result<UserDto, Error> {
        self.sqlite.execute_with_params(
            &format!("INSERT INTO {} (name) VALUES (?)", UsersTable::table_name()),
            &[&name as &dyn rusqlite::types::ToSql],
        ).map_err(|_| db_error())?;

        let id = self.sqlite.last_insert_rowid()
            .map_err(|_| db_error())?;

        Ok(UserDto {
            id,
            name: name.to_string(),
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
    async fn save_user_can_be_found_by_name() {
        let repo = repo_in_memory();
        repo.save_user("TestPlayer").await.unwrap();
        let found = repo.find_by_name("TestPlayer").await.unwrap();
        assert!(found.is_some());
    }
}
