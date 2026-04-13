use super::{row::Row, table_entity::TableEntity};

#[allow(dead_code)]
pub struct StoredGamePlayer {
    pub game_id: String,
    pub user_name: String,
    pub selected_race: Option<String>,
}

pub struct GamePlayersTable;

impl TableEntity for GamePlayersTable {
    type Entity = StoredGamePlayer;

    fn table_name() -> &'static str {
        "game_players"
    }

    fn schema_version() -> u32 {
        1
    }

    fn create_table_sql() -> String {
        format!(
            "CREATE TABLE IF NOT EXISTS {} (
                game_id TEXT NOT NULL,
                user_name TEXT NOT NULL,
                selected_race TEXT,
                PRIMARY KEY (game_id, user_name)
            )",
            Self::table_name()
        )
    }

    fn from_row(row: &Row) -> Result<Self::Entity, String> {
        Ok(StoredGamePlayer {
            game_id: row.get("game_id")?,
            user_name: row.get("user_name")?,
            selected_race: row.get("selected_race")?,
        })
    }
}
