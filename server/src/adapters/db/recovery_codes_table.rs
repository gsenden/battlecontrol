use super::row::Row;
use super::table_entity::TableEntity;

#[derive(Clone)]
#[allow(dead_code)]
pub struct StoredRecoveryCode {
    pub recovery_code: String,
    pub user_name: String,
    pub expires_at: i64,
    pub used_at: Option<i64>,
}

pub struct RecoveryCodesTable;

impl TableEntity for RecoveryCodesTable {
    type Entity = StoredRecoveryCode;

    fn table_name() -> &'static str {
        "recovery_codes"
    }

    fn schema_version() -> u32 {
        1
    }

    fn create_table_sql() -> String {
        format!(
            "CREATE TABLE IF NOT EXISTS {} (
                recovery_code TEXT PRIMARY KEY,
                user_name TEXT NOT NULL,
                expires_at INTEGER NOT NULL,
                used_at INTEGER
            )",
            Self::table_name()
        )
    }

    fn from_row(row: &Row) -> Result<Self::Entity, String> {
        Ok(StoredRecoveryCode {
            recovery_code: row.get("recovery_code")?,
            user_name: row.get("user_name")?,
            expires_at: row.get("expires_at")?,
            used_at: row.get("used_at").ok(),
        })
    }
}
