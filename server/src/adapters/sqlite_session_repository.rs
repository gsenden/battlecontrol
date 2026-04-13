use common::dto::UserDto;

use crate::ports::SessionRepositoryDrivenPort;

use super::db::{SessionsTable, SqliteAdapter, TableEntity};

const SESSION_INACTIVITY_TIMEOUT_SECONDS: i64 = 8 * 24 * 60 * 60;

#[derive(Clone)]
pub struct SqliteSessionRepository {
    sqlite: SqliteAdapter,
}

impl SqliteSessionRepository {
    pub fn new(sqlite: SqliteAdapter) -> Result<Self, String> {
        sqlite.ensure_table::<SessionsTable>()?;
        Ok(Self { sqlite })
    }
}

impl SessionRepositoryDrivenPort for SqliteSessionRepository {
    fn load_session_user(&self, session_id: &str) -> Result<Option<UserDto>, String> {
        let rows = self.sqlite.query_with_params(
            &format!("SELECT * FROM {} WHERE session_id = ?", SessionsTable::table_name()),
            &[&session_id as &dyn rusqlite::types::ToSql],
        )?;

        match rows.first() {
            Some(row) => {
                let stored_session = SessionsTable::from_row(row)?;
                if current_timestamp() - stored_session.last_active_at > SESSION_INACTIVITY_TIMEOUT_SECONDS {
                    self.sqlite.execute_with_params(
                        &format!("DELETE FROM {} WHERE session_id = ?", SessionsTable::table_name()),
                        &[&stored_session.session_id as &dyn rusqlite::types::ToSql],
                    )?;
                    return Ok(None);
                }

                self.sqlite.execute_with_params(
                    &format!("UPDATE {} SET last_active_at = ? WHERE session_id = ?", SessionsTable::table_name()),
                    &[&current_timestamp() as &dyn rusqlite::types::ToSql, &stored_session.session_id],
                )?;

                Ok(Some(stored_session.user()?))
            }
            None => Ok(None),
        }
    }
}

fn current_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time before unix epoch")
        .as_secs() as i64
}
