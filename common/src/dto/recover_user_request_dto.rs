use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct RecoverUserRequestDto {
    pub recovery_code: String,
}
