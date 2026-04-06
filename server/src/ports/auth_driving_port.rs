use async_trait::async_trait;
use common::domain::Error;
use common::dto::{LoginRequestDto, RegistrationRequestDto, UserDto};

#[async_trait]
pub trait AuthDrivingPort {
    async fn login_user(&self, login_request: LoginRequestDto) -> Result<UserDto, Error>;
    async fn register_user(&self, registration_request: RegistrationRequestDto) -> Result<UserDto, Error>;
}
