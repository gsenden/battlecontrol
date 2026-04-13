use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct RecoveryCodeDto {
    pub recovery_code: String,
    pub expires_at: i64,
}
