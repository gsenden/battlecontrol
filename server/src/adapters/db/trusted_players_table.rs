use super::row::Row;
use super::table_entity::TableEntity;

#[derive(Clone)]
pub struct StoredTrustedPlayer {
    pub browser_id: String,
    pub user_name: String,
}

pub struct TrustedPlayersTable;

impl TableEntity for TrustedPlayersTable {
    type Entity = StoredTrustedPlayer;

    fn table_name() -> &'static str {
        "trusted_players"
    }

    fn schema_version() -> u32 {
        1
    }

    fn create_table_sql() -> String {
        format!(
            "CREATE TABLE IF NOT EXISTS {} (
                browser_id TEXT NOT NULL,
                user_name TEXT NOT NULL,
                PRIMARY KEY (browser_id, user_name)
            )",
            Self::table_name()
        )
    }

    fn from_row(row: &Row) -> Result<Self::Entity, String> {
        Ok(StoredTrustedPlayer {
            browser_id: row.get("browser_id")?,
            user_name: row.get("user_name")?,
        })
    }
}
