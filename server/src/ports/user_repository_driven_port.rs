use async_trait::async_trait;
use common::domain::Error;
use common::dto::UserDto;

#[async_trait]
pub trait UserRepositoryDrivenPort: Send + Sync + 'static {
    async fn find_by_name(&self, name: &str) -> Result<Option<UserDto>, Error>;
    async fn save_user(&self, name: &str) -> Result<UserDto, Error>;
}
