use std::sync::{Arc, Mutex};
use async_trait::async_trait;
use common::domain::Error;
use common::dto::UserDto;
use crate::ports::UserRepositoryDrivenPort;
use super::sample_data::test_user;

#[derive(Clone)]
pub struct FakeUserRepository {
    existing_user: Option<UserDto>,
    save_user_calls: Arc<Mutex<Vec<(String, String)>>>,
}

impl FakeUserRepository {
    pub fn new() -> Self {
        Self {
            existing_user: None,
            save_user_calls: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn with_existing_user(mut self, user: UserDto) -> Self {
        self.existing_user = Some(user);
        self
    }

    pub fn save_user_calls(&self) -> Vec<(String, String)> {
        self.save_user_calls.lock().unwrap().clone()
    }
}

#[async_trait]
impl UserRepositoryDrivenPort for FakeUserRepository {
    async fn find_by_email(&self, email: &str) -> Result<Option<UserDto>, Error> {
        Ok(self.existing_user.as_ref().filter(|u| u.email == email).cloned())
    }

    async fn save_user(&self, name: &str, email: &str) -> Result<UserDto, Error> {
        self.save_user_calls.lock().unwrap().push((name.to_string(), email.to_string()));
        Ok(test_user())
    }
}
