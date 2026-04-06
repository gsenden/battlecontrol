use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct PasskeyFinishLoginRequestDto {
    pub name: String,
    pub credential: serde_json::Value,
}
