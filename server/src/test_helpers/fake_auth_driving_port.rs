use std::sync::{Arc, Mutex};
use async_trait::async_trait;
use common::domain::Error;
use common::dto::{
    LoginRequestDto, PasskeyFinishLoginRequestDto, PasskeyFinishRegistrationRequestDto,
    PasskeyOptionsDto, PasskeyStartLoginRequestDto, PasskeyStartRegistrationRequestDto,
    RegistrationRequestDto, UpdateUserProfileRequestDto, UserDto, UserSettingsDto,
};
use crate::ports::AuthDrivingPort;
use super::sample_data::test_user;

#[derive(Clone)]
pub struct FakeAuthDrivingPort {
    login_user_calls: Arc<Mutex<Vec<LoginRequestDto>>>,
    login_user_error: Option<Error>,
    register_user_calls: Arc<Mutex<Vec<RegistrationRequestDto>>>,
    register_user_error: Option<Error>,
}

impl FakeAuthDrivingPort {
    pub fn new() -> Self {
        Self {
            login_user_calls: Arc::new(Mutex::new(Vec::new())),
            login_user_error: None,
            register_user_calls: Arc::new(Mutex::new(Vec::new())),
            register_user_error: None,
        }
    }

    pub fn with_login_user_error(mut self, error: Error) -> Self {
        self.login_user_error = Some(error);
        self
    }

    pub fn with_register_user_error(mut self, error: Error) -> Self {
        self.register_user_error = Some(error);
        self
    }

    pub fn login_user_calls(&self) -> Vec<LoginRequestDto> {
        self.login_user_calls.lock().unwrap().clone()
    }

    pub fn register_user_calls(&self) -> Vec<RegistrationRequestDto> {
        self.register_user_calls.lock().unwrap().clone()
    }
}

#[async_trait]
impl AuthDrivingPort for FakeAuthDrivingPort {
    async fn login_user(&self, login_request: LoginRequestDto) -> Result<UserDto, Error> {
        self.login_user_calls.lock().unwrap().push(login_request);
        match &self.login_user_error {
            Some(error) => Err(error.clone()),
            None => Ok(test_user()),
        }
    }

    async fn register_user(&self, registration_request: RegistrationRequestDto) -> Result<UserDto, Error> {
        self.register_user_calls.lock().unwrap().push(registration_request);
        match &self.register_user_error {
            Some(error) => Err(error.clone()),
            None => Ok(test_user()),
        }
    }

    async fn start_passkey_registration(&self, _request: PasskeyStartRegistrationRequestDto) -> Result<PasskeyOptionsDto, Error> {
        Ok(PasskeyOptionsDto { public_key: serde_json::json!({}) })
    }

    async fn finish_passkey_registration(&self, _request: PasskeyFinishRegistrationRequestDto) -> Result<UserDto, Error> {
        Ok(test_user())
    }

    async fn start_passkey_login(&self, _request: PasskeyStartLoginRequestDto) -> Result<PasskeyOptionsDto, Error> {
        Ok(PasskeyOptionsDto { public_key: serde_json::json!({}) })
    }

    async fn finish_passkey_login(&self, _request: PasskeyFinishLoginRequestDto) -> Result<UserDto, Error> {
        Ok(test_user())
    }

    async fn update_user_profile(&self, _current_user_name: String, request: UpdateUserProfileRequestDto) -> Result<UserDto, Error> {
        Ok(UserDto {
            id: test_user().id,
            name: request.name,
            profile_image_url: if request.profile_image_url.is_empty() {
                None
            } else {
                Some(request.profile_image_url)
            },
        })
    }

    async fn get_user_settings(&self, _user_name: String) -> Result<UserSettingsDto, Error> {
        Ok(UserSettingsDto {
            turn_left_key: "A".to_string(),
            turn_right_key: "D".to_string(),
            thrust_key: "W".to_string(),
            music_enabled: true,
            music_volume: 45,
            sound_effects_enabled: true,
            sound_effects_volume: 60,
        })
    }

    async fn save_user_settings(&self, _user_name: String, settings: UserSettingsDto) -> Result<UserSettingsDto, Error> {
        Ok(settings)
    }
}
