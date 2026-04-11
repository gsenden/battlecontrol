use common::dto::{RegistrationRequestDto, UserDto, UserSettingsDto};

pub const TEST_PLAYER_NAME: &str = "TestPlayer";
pub const TEST_USER_ID: i64 = 1;

pub fn test_registration_request() -> RegistrationRequestDto {
    RegistrationRequestDto {
        name: TEST_PLAYER_NAME.to_string(),
    }
}

pub fn test_user() -> UserDto {
    UserDto {
        id: TEST_USER_ID,
        name: TEST_PLAYER_NAME.to_string(),
        profile_image_url: None,
    }
}

pub fn test_user_settings() -> UserSettingsDto {
    UserSettingsDto {
        turn_left_key: "A".to_string(),
        turn_right_key: "D".to_string(),
        thrust_key: "W".to_string(),
        music_enabled: true,
        music_volume: 45,
        sound_effects_enabled: true,
        sound_effects_volume: 60,
    }
}
