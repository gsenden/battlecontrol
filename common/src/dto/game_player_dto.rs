use serde::{Deserialize, Serialize};

use super::UserDto;

#[derive(Clone, Serialize, Deserialize)]
pub struct GamePlayerDto {
    pub user: UserDto,
    pub selected_race: Option<String>,
}
