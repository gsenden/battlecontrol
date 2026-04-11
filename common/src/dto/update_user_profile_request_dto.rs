use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct UpdateUserProfileRequestDto {
    pub name: String,
    pub profile_image_url: String,
}
