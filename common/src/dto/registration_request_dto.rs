use serde::{Deserialize, Serialize};
#[derive(Clone, Serialize, Deserialize)]
pub struct RegistrationRequestDto {
    pub name: String,
}
