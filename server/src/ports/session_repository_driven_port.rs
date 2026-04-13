use common::dto::UserDto;

pub trait SessionRepositoryDrivenPort: Send + Sync + 'static {
    fn load_session_user(&self, session_id: &str) -> Result<Option<UserDto>, String>;
}
