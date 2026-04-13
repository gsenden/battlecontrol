use async_trait::async_trait;
use common::domain::Error;
use common::dto::{UserDto, UserSettingsDto};
use uuid::Uuid;
use webauthn_rs::prelude::Passkey;

#[async_trait]
pub trait UserRepositoryDrivenPort: Send + Sync + 'static {
    async fn find_by_name(&self, name: &str) -> Result<Option<UserDto>, Error>;
    async fn save_user(&self, name: &str, user_handle: Uuid) -> Result<UserDto, Error>;
    async fn list_passkeys_by_name(&self, name: &str) -> Result<Vec<Passkey>, Error>;
    async fn save_passkey(&self, name: &str, passkey: &Passkey) -> Result<(), Error>;
    async fn update_passkey(&self, name: &str, passkey: &Passkey) -> Result<(), Error>;
    async fn create_recovery_code(
        &self,
        user_name: &str,
        recovery_code: &str,
        expires_at: i64,
    ) -> Result<(), Error>;
    async fn find_by_recovery_code(
        &self,
        recovery_code: &str,
        now: i64,
    ) -> Result<Option<UserDto>, Error>;
    async fn mark_recovery_code_used(&self, recovery_code: &str) -> Result<(), Error>;
    async fn update_user_profile(
        &self,
        current_name: &str,
        name: &str,
        profile_image_url: &str,
    ) -> Result<UserDto, Error>;
    async fn find_settings_by_name(&self, name: &str) -> Result<Option<UserSettingsDto>, Error>;
    async fn save_settings(
        &self,
        name: &str,
        settings: &UserSettingsDto,
    ) -> Result<UserSettingsDto, Error>;
}
