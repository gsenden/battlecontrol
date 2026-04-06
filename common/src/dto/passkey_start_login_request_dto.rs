use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct PasskeyStartLoginRequestDto {
    pub name: String,
}
