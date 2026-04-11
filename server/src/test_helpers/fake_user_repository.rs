use std::sync::{Arc, Mutex};
use async_trait::async_trait;
use common::domain::Error;
use common::dto::{UserDto, UserSettingsDto};
use crate::ports::UserRepositoryDrivenPort;
use super::sample_data::test_user;
use uuid::Uuid;
use webauthn_rs::prelude::Passkey;

#[derive(Clone)]
pub struct FakeUserRepository {
    existing_user: Option<UserDto>,
    existing_settings: Option<UserSettingsDto>,
    save_user_calls: Arc<Mutex<Vec<String>>>,
}

impl FakeUserRepository {
    pub fn new() -> Self {
        Self {
            existing_user: None,
            existing_settings: None,
            save_user_calls: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn with_existing_user(mut self, user: UserDto) -> Self {
        self.existing_user = Some(user);
        self
    }

    pub fn with_existing_settings(mut self, settings: UserSettingsDto) -> Self {
        self.existing_settings = Some(settings);
        self
    }

    pub fn save_user_calls(&self) -> Vec<String> {
        self.save_user_calls.lock().unwrap().clone()
    }
}

#[async_trait]
impl UserRepositoryDrivenPort for FakeUserRepository {
    async fn find_by_name(&self, name: &str) -> Result<Option<UserDto>, Error> {
        Ok(self.existing_user.as_ref().filter(|u| u.name == name).cloned())
    }

    async fn find_user_handle_by_name(&self, name: &str) -> Result<Option<Uuid>, Error> {
        Ok(self.existing_user
            .as_ref()
            .filter(|u| u.name == name)
            .map(|_| Uuid::nil()))
    }

    async fn save_user(&self, name: &str, _user_handle: Uuid) -> Result<UserDto, Error> {
        self.save_user_calls.lock().unwrap().push(name.to_string());
        Ok(test_user())
    }

    async fn list_passkeys_by_name(&self, _name: &str) -> Result<Vec<Passkey>, Error> {
        Ok(Vec::new())
    }

    async fn save_passkey(&self, _name: &str, _passkey: &Passkey) -> Result<(), Error> {
        Ok(())
    }

    async fn update_passkey(&self, _name: &str, _passkey: &Passkey) -> Result<(), Error> {
        Ok(())
    }

    async fn update_user_profile(&self, _current_name: &str, name: &str, profile_image_url: &str) -> Result<UserDto, Error> {
        Ok(UserDto {
            id: test_user().id,
            name: name.to_string(),
            profile_image_url: if profile_image_url.is_empty() {
                None
            } else {
                Some(profile_image_url.to_string())
            },
        })
    }

    async fn find_settings_by_name(&self, _name: &str) -> Result<Option<UserSettingsDto>, Error> {
        Ok(self.existing_settings.clone())
    }

    async fn save_settings(&self, _name: &str, settings: &UserSettingsDto) -> Result<UserSettingsDto, Error> {
        Ok(settings.clone())
    }
}
