use std::sync::{Arc, Mutex};
use async_trait::async_trait;
use common::domain::Error;
use common::dto::{LoginRequestDto, RegistrationRequestDto, UserDto};
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
}
