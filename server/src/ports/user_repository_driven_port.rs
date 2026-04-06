use async_trait::async_trait;
use common::domain::Error;
use common::dto::UserDto;
use uuid::Uuid;
use webauthn_rs::prelude::Passkey;

#[async_trait]
pub trait UserRepositoryDrivenPort: Send + Sync + 'static {
    async fn find_by_name(&self, name: &str) -> Result<Option<UserDto>, Error>;
    async fn find_user_handle_by_name(&self, name: &str) -> Result<Option<Uuid>, Error>;
    async fn save_user(&self, name: &str, user_handle: Uuid) -> Result<UserDto, Error>;
    async fn list_passkeys_by_name(&self, name: &str) -> Result<Vec<Passkey>, Error>;
    async fn save_passkey(&self, name: &str, passkey: &Passkey) -> Result<(), Error>;
    async fn update_passkey(&self, name: &str, passkey: &Passkey) -> Result<(), Error>;
}
