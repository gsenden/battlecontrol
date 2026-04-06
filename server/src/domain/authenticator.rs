use async_trait::async_trait;
use common::domain::Error;
use common::domain::error::UserNotFoundError;
use common::dto::{LoginRequestDto, RegistrationRequestDto, UserDto};
use crate::ports::{AuthDrivingPort, UserRepositoryDrivenPort};

pub trait AuthenticatorDrivenPorts: Send + Sync + 'static {
    type UserRepo: UserRepositoryDrivenPort;
}

pub struct Authenticator<DP: AuthenticatorDrivenPorts> {
    user_repo: DP::UserRepo,
}

impl<DP: AuthenticatorDrivenPorts> Authenticator<DP> {
    pub fn new(user_repo: DP::UserRepo) -> Self {
        Self { user_repo }
    }
}

#[async_trait]
impl<DP: AuthenticatorDrivenPorts> AuthDrivingPort for Authenticator<DP> {
    async fn login_user(&self, login_request: LoginRequestDto) -> Result<UserDto, Error> {
        self.user_repo
            .find_by_name(&login_request.name)
            .await?
            .ok_or_else(|| Error::UserNotFound(UserNotFoundError::new(login_request.name)))
    }

    async fn register_user(&self, registration_request: RegistrationRequestDto) -> Result<UserDto, Error> {
        if self.user_repo.find_by_name(&registration_request.name).await?.is_some() {
            return Err(Error::UserAlreadyExists(
                common::domain::error::UserAlreadyExistsError::new(registration_request.name),
            ));
        }
        self.user_repo.save_user(&registration_request.name).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::AuthDrivingPort;
    use crate::test_helpers::FakeUserRepository;
    use crate::test_helpers::sample_data::{test_registration_request, TEST_PLAYER_NAME};

    struct FakeDrivenPorts;
    impl AuthenticatorDrivenPorts for FakeDrivenPorts {
        type UserRepo = FakeUserRepository;
    }

    #[tokio::test]
    async fn login_user_returns_existing_user() {
        let repo = FakeUserRepository::new()
            .with_existing_user(crate::test_helpers::sample_data::test_user());
        let auth = Authenticator::<FakeDrivenPorts>::new(repo);
        let result = auth
            .login_user(LoginRequestDto { name: TEST_PLAYER_NAME.to_string() })
            .await
            .unwrap();
        assert_eq!(result.name, TEST_PLAYER_NAME);
    }

    #[tokio::test]
    async fn login_user_returns_error_when_user_does_not_exist() {
        let repo = FakeUserRepository::new();
        let auth = Authenticator::<FakeDrivenPorts>::new(repo);
        let result = auth
            .login_user(LoginRequestDto { name: TEST_PLAYER_NAME.to_string() })
            .await;
        assert!(matches!(result, Err(Error::UserNotFound(_))));
    }

    #[tokio::test]
    async fn register_user_does_not_save_when_user_exists() {
        let repo = FakeUserRepository::new()
            .with_existing_user(crate::test_helpers::sample_data::test_user());
        let repo_clone = repo.clone();
        let auth = Authenticator::<FakeDrivenPorts>::new(repo);
        let _ = auth.register_user(test_registration_request()).await;
        assert_eq!(repo_clone.save_user_calls().len(), 0);
    }

    #[tokio::test]
    async fn register_user_returns_error_when_user_exists() {
        let repo = FakeUserRepository::new()
            .with_existing_user(crate::test_helpers::sample_data::test_user());
        let auth = Authenticator::<FakeDrivenPorts>::new(repo);
        let result = auth.register_user(test_registration_request()).await;
        assert!(matches!(result, Err(Error::UserAlreadyExists(_))));
    }

    #[tokio::test]
    async fn register_user_saves_user_when_not_exists() {
        let repo = FakeUserRepository::new();
        let repo_clone = repo.clone();
        let auth = Authenticator::<FakeDrivenPorts>::new(repo);
        auth.register_user(test_registration_request()).await.unwrap();
        let calls = repo_clone.save_user_calls();
        assert_eq!(calls.len(), 1);
    }
}
