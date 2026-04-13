use common::dto::{
    CreateGameRequestDto,
    GameDto,
    GamePlayerDto,
    JoinGameRequestDto,
    RegistrationRequestDto,
    SaveSelectedRaceRequestDto,
    UserDto,
    UserSettingsDto,
};

pub const TEST_PLAYER_NAME: &str = "TestPlayer";
pub const TEST_USER_ID: i64 = 1;
pub const TEST_GAME_ID: &str = "game-1";

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

pub fn test_create_game_request() -> CreateGameRequestDto {
    CreateGameRequestDto {
        name: "Test Game".to_string(),
        game_type: "free_for_all".to_string(),
        max_players: 2,
        is_private: false,
        password: None,
    }
}

pub fn test_join_game_request() -> JoinGameRequestDto {
    JoinGameRequestDto {
        password: None,
    }
}

pub fn test_save_selected_race_request() -> SaveSelectedRaceRequestDto {
    SaveSelectedRaceRequestDto {
        selected_race: "human-cruiser".to_string(),
    }
}

pub fn test_game() -> GameDto {
    GameDto {
        id: TEST_GAME_ID.to_string(),
        name: "Test Game".to_string(),
        game_type: "free_for_all".to_string(),
        max_players: 2,
        is_private: false,
        password: None,
        creator: test_user(),
        players: vec![GamePlayerDto {
            user: test_user(),
            selected_race: Some("human-cruiser".to_string()),
        }],
    }
}
