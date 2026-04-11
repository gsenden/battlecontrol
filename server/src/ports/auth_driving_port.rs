use async_trait::async_trait;
use common::domain::Error;
use common::dto::{
    LoginRequestDto, PasskeyFinishLoginRequestDto, PasskeyFinishRegistrationRequestDto,
    PasskeyOptionsDto, PasskeyStartLoginRequestDto, PasskeyStartRegistrationRequestDto,
    RegistrationRequestDto, UpdateUserProfileRequestDto, UserDto, UserSettingsDto,
};

#[async_trait]
pub trait AuthDrivingPort {
    async fn login_user(&self, login_request: LoginRequestDto) -> Result<UserDto, Error>;
    async fn register_user(&self, registration_request: RegistrationRequestDto) -> Result<UserDto, Error>;
    async fn start_passkey_registration(&self, request: PasskeyStartRegistrationRequestDto) -> Result<PasskeyOptionsDto, Error>;
    async fn finish_passkey_registration(&self, request: PasskeyFinishRegistrationRequestDto) -> Result<UserDto, Error>;
    async fn start_passkey_login(&self, request: PasskeyStartLoginRequestDto) -> Result<PasskeyOptionsDto, Error>;
    async fn finish_passkey_login(&self, request: PasskeyFinishLoginRequestDto) -> Result<UserDto, Error>;
    async fn update_user_profile(&self, current_user_name: String, request: UpdateUserProfileRequestDto) -> Result<UserDto, Error>;
    async fn get_user_settings(&self, user_name: String) -> Result<UserSettingsDto, Error>;
    async fn save_user_settings(&self, user_name: String, settings: UserSettingsDto) -> Result<UserSettingsDto, Error>;
}
