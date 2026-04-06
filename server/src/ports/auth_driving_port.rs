use async_trait::async_trait;
use common::domain::Error;
use common::dto::{RegistrationRequestDto, UserDto};

#[async_trait]
pub trait AuthDrivingPort {
    async fn register_user(&self, registration_request: RegistrationRequestDto) -> Result<UserDto, Error>;
}