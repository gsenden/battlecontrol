use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct ProfileImageUploadDto {
    pub profile_image_url: String,
}
