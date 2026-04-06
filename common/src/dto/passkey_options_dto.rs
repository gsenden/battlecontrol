use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct PasskeyOptionsDto {
    #[serde(rename = "publicKey")]
    pub public_key: serde_json::Value,
}
