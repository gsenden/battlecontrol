use super::row::Row;
use super::table_entity::TableEntity;
use common::dto::UserSettingsDto;

pub struct UserSettingsTable;

impl TableEntity for UserSettingsTable {
    type Entity = UserSettingsDto;

    fn table_name() -> &'static str {
        "user_settings"
    }

    fn schema_version() -> u32 {
        2
    }

    fn create_table_sql() -> String {
        format!(
            "CREATE TABLE IF NOT EXISTS {} (
                user_name TEXT PRIMARY KEY,
                turn_left_key TEXT NOT NULL,
                turn_right_key TEXT NOT NULL,
                thrust_key TEXT NOT NULL,
                music_enabled INTEGER NOT NULL DEFAULT 1,
                music_volume INTEGER NOT NULL DEFAULT 45,
                sound_effects_enabled INTEGER NOT NULL DEFAULT 1,
                sound_effects_volume INTEGER NOT NULL DEFAULT 60
            )",
            Self::table_name()
        )
    }

    fn from_row(row: &Row) -> Result<Self::Entity, String> {
        Ok(UserSettingsDto {
            turn_left_key: row.get("turn_left_key")?,
            turn_right_key: row.get("turn_right_key")?,
            thrust_key: row.get("thrust_key")?,
            music_enabled: row.get::<i64>("music_enabled")? != 0,
            music_volume: row.get("music_volume")?,
            sound_effects_enabled: row.get::<i64>("sound_effects_enabled")? != 0,
            sound_effects_volume: row.get("sound_effects_volume")?,
        })
    }
}
