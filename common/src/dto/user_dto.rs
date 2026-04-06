use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct UserDto {
    pub id: i64,
    pub name: String,
    pub email: String,
}
