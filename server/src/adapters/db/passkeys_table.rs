use super::row::Row;
use super::table_entity::TableEntity;

#[derive(Clone)]
#[allow(dead_code)]
pub struct StoredPasskey {
    pub id: i64,
    pub user_name: String,
    pub credential_id: String,
    pub passkey_json: String,
}

pub struct PasskeysTable;

impl TableEntity for PasskeysTable {
    type Entity = StoredPasskey;

    fn table_name() -> &'static str {
        "passkeys"
    }

    fn schema_version() -> u32 {
        1
    }

    fn create_table_sql() -> String {
        format!(
            "CREATE TABLE IF NOT EXISTS {} (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_name TEXT NOT NULL,
                credential_id TEXT NOT NULL UNIQUE,
                passkey_json TEXT NOT NULL
            )",
            Self::table_name()
        )
    }

    fn from_row(row: &Row) -> Result<Self::Entity, String> {
        Ok(StoredPasskey {
            id: row.get("id")?,
            user_name: row.get("user_name")?,
            credential_id: row.get("credential_id")?,
            passkey_json: row.get("passkey_json")?,
        })
    }
}
