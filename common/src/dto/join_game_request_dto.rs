use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct JoinGameRequestDto {
    pub password: Option<String>,
}
