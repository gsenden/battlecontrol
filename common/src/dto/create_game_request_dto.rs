use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct CreateGameRequestDto {
    pub name: String,
    pub game_type: String,
    pub max_players: u8,
    pub is_private: bool,
    pub password: Option<String>,
}
