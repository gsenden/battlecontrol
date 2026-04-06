use common::dto::{RegistrationRequestDto, UserDto};

pub const TEST_PLAYER_NAME: &str = "TestPlayer";
pub const TEST_EMAIL: &str = "test@matter-rs.com";
pub const TEST_USER_ID: i64 = 1;

pub fn test_registration_request() -> RegistrationRequestDto {
    RegistrationRequestDto {
        name: TEST_PLAYER_NAME.to_string(),
        email: TEST_EMAIL.to_string(),
    }
}

pub fn test_user() -> UserDto {
    UserDto {
        id: TEST_USER_ID,
        name: TEST_PLAYER_NAME.to_string(),
        email: TEST_EMAIL.to_string(),
    }
}
