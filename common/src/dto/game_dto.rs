use serde::{Deserialize, Serialize};

use super::{GamePlayerDto, UserDto};

#[derive(Clone, Serialize, Deserialize)]
pub struct GameDto {
    pub id: String,
    pub name: String,
    pub game_type: String,
    pub max_players: u8,
    pub is_private: bool,
    pub password: Option<String>,
    pub creator: UserDto,
    pub players: Vec<GamePlayerDto>,
}
