use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct SaveSelectedRaceRequestDto {
    pub selected_race: String,
}
