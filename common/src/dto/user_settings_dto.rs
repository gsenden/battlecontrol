use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct UserSettingsDto {
    pub turn_left_key: String,
    pub turn_right_key: String,
    pub thrust_key: String,
    pub music_enabled: bool,
    pub music_volume: u8,
    pub sound_effects_enabled: bool,
    pub sound_effects_volume: u8,
}
