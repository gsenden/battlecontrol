use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct PasskeyStartRegistrationRequestDto {
    pub name: String,
}
