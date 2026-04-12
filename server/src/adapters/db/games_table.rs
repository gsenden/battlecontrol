use common::dto::{GameDto, UserDto};

use super::{row::Row, table_entity::TableEntity};

pub struct GamesTable;

impl TableEntity for GamesTable {
    type Entity = GameDto;

    fn table_name() -> &'static str {
        "games"
    }

    fn schema_version() -> u32 {
        1
    }

    fn create_table_sql() -> String {
        format!(
            "CREATE TABLE IF NOT EXISTS {} (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                game_type TEXT NOT NULL,
                max_players INTEGER NOT NULL,
                is_private INTEGER NOT NULL,
                password TEXT,
                creator_name TEXT NOT NULL,
                creator_id INTEGER NOT NULL DEFAULT 0,
                creator_profile_image_url TEXT,
                created_at INTEGER NOT NULL
            )",
            Self::table_name()
        )
    }

    fn from_row(row: &Row) -> Result<Self::Entity, String> {
        Ok(GameDto {
            id: row.get("id")?,
            name: row.get("name")?,
            game_type: row.get("game_type")?,
            max_players: row.get("max_players")?,
            is_private: row.get::<i64>("is_private")? != 0,
            password: row.get("password")?,
            creator: UserDto {
                id: row.get("creator_id")?,
                name: row.get("creator_name")?,
                profile_image_url: row.get("creator_profile_image_url")?,
            },
            players: Vec::new(),
        })
    }
}
