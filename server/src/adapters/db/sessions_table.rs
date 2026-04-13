use super::row::Row;
use super::table_entity::TableEntity;
use common::dto::UserDto;

#[derive(Clone)]
pub struct StoredSession {
    pub session_id: String,
    pub user_json: String,
    pub last_active_at: i64,
}

pub struct SessionsTable;

impl TableEntity for SessionsTable {
    type Entity = StoredSession;

    fn table_name() -> &'static str {
        "sessions"
    }

    fn schema_version() -> u32 {
        2
    }

    fn create_table_sql() -> String {
        format!(
            "CREATE TABLE IF NOT EXISTS {} (
                session_id TEXT PRIMARY KEY,
                user_json TEXT NOT NULL,
                last_active_at INTEGER NOT NULL
            )",
            Self::table_name()
        )
    }

    fn from_row(row: &Row) -> Result<Self::Entity, String> {
        Ok(StoredSession {
            session_id: row.get("session_id")?,
            user_json: row.get("user_json")?,
            last_active_at: row.get("last_active_at")?,
        })
    }
}

impl StoredSession {
    pub fn user(&self) -> Result<UserDto, String> {
        serde_json::from_str(&self.user_json).map_err(|error| error.to_string())
    }
}
