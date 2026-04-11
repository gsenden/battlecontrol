use common::dto::UserDto;
use super::row::Row;
use super::table_entity::TableEntity;

pub struct UsersTable;

impl TableEntity for UsersTable {
    type Entity = UserDto;

    fn table_name() -> &'static str {
        "users"
    }

    fn schema_version() -> u32 {
        3
    }

    fn create_table_sql() -> String {
        format!(
            "CREATE TABLE IF NOT EXISTS {} (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                user_handle TEXT NOT NULL UNIQUE,
                profile_image_url TEXT
            )",
            Self::table_name()
        )
    }

    fn from_row(row: &Row) -> Result<Self::Entity, String> {
        Ok(UserDto {
            id: row.get("id")?,
            name: row.get("name")?,
            profile_image_url: row.get("profile_image_url").ok(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::db::SqliteAdapter;

    #[test]
    fn ensure_table_creates_users_table() {
        let adapter = SqliteAdapter::new(":memory:").unwrap();
        adapter.ensure_table::<UsersTable>().unwrap();
        let rows = adapter.query("SELECT name FROM sqlite_master WHERE type='table' AND name='users'").unwrap();
        assert_eq!(rows.len(), 1);
    }
}
