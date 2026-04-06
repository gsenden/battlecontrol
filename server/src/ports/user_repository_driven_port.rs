use async_trait::async_trait;
use common::domain::Error;
use common::dto::UserDto;

#[async_trait]
pub trait UserRepositoryDrivenPort: Send + Sync + 'static {
    async fn find_by_email(&self, email: &str) -> Result<Option<UserDto>, Error>;
    async fn save_user(&self, name: &str, email: &str) -> Result<UserDto, Error>;
}
