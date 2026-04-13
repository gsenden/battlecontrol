use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use common::domain::Error;
use common::dto::{CreateGameRequestDto, GameDto, JoinGameRequestDto, SaveSelectedRaceRequestDto};

use crate::ports::GameRepositoryDrivenPort;

#[derive(Clone)]
pub struct FakeGameRepository {
    stale_game_ids: Vec<String>,
    created_game: Option<GameDto>,
    joined_game: Option<GameDto>,
    left_game: Option<GameDto>,
    cancelled_game: Option<GameDto>,
    started_game: Option<GameDto>,
    selected_race_game: Option<GameDto>,
    listed_games: Vec<GameDto>,
    found_game: Option<GameDto>,
    create_game_calls: Arc<Mutex<Vec<(String, CreateGameRequestDto)>>>,
    join_game_calls: Arc<Mutex<Vec<(String, String, JoinGameRequestDto)>>>,
    leave_game_calls: Arc<Mutex<Vec<(String, String)>>>,
}

impl FakeGameRepository {
    pub fn new() -> Self {
        Self {
            stale_game_ids: Vec::new(),
            created_game: None,
            joined_game: None,
            left_game: None,
            cancelled_game: None,
            started_game: None,
            selected_race_game: None,
            listed_games: Vec::new(),
            found_game: None,
            create_game_calls: Arc::new(Mutex::new(Vec::new())),
            join_game_calls: Arc::new(Mutex::new(Vec::new())),
            leave_game_calls: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn with_created_game(mut self, game: GameDto) -> Self {
        self.created_game = Some(game);
        self
    }

    pub fn with_joined_game(mut self, game: Option<GameDto>) -> Self {
        self.joined_game = game;
        self
    }

    pub fn with_left_game(mut self, game: Option<GameDto>) -> Self {
        self.left_game = game;
        self
    }

    pub fn with_cancelled_game(mut self, game: Option<GameDto>) -> Self {
        self.cancelled_game = game;
        self
    }

    pub fn with_started_game(mut self, game: Option<GameDto>) -> Self {
        self.started_game = game;
        self
    }

    pub fn with_selected_race_game(mut self, game: Option<GameDto>) -> Self {
        self.selected_race_game = game;
        self
    }

    pub fn with_listed_games(mut self, games: Vec<GameDto>) -> Self {
        self.listed_games = games;
        self
    }

    pub fn with_found_game(mut self, game: Option<GameDto>) -> Self {
        self.found_game = game;
        self
    }

}

#[async_trait]
impl GameRepositoryDrivenPort for FakeGameRepository {
    async fn delete_stale_games(&self) -> Result<Vec<String>, Error> {
        Ok(self.stale_game_ids.clone())
    }

    async fn save_game(&self, creator_name: &str, request: &CreateGameRequestDto) -> Result<GameDto, Error> {
        self.create_game_calls
            .lock()
            .unwrap()
            .push((creator_name.to_string(), request.clone()));
        Ok(self.created_game.clone().expect("created game not configured"))
    }

    async fn join_game(&self, game_id: &str, player_name: &str, request: &JoinGameRequestDto) -> Result<Option<GameDto>, Error> {
        self.join_game_calls
            .lock()
            .unwrap()
            .push((game_id.to_string(), player_name.to_string(), request.clone()));
        Ok(self.joined_game.clone())
    }

    async fn leave_game(&self, game_id: &str, player_name: &str) -> Result<Option<GameDto>, Error> {
        self.leave_game_calls
            .lock()
            .unwrap()
            .push((game_id.to_string(), player_name.to_string()));
        Ok(self.left_game.clone())
    }

    async fn cancel_game(&self, _game_id: &str, _player_name: &str) -> Result<Option<GameDto>, Error> {
        Ok(self.cancelled_game.clone())
    }

    async fn start_game(&self, _game_id: &str, _player_name: &str) -> Result<Option<GameDto>, Error> {
        Ok(self.started_game.clone())
    }

    async fn save_selected_race(&self, _game_id: &str, _player_name: &str, _request: &SaveSelectedRaceRequestDto) -> Result<Option<GameDto>, Error> {
        Ok(self.selected_race_game.clone())
    }

    async fn list_games(&self) -> Result<Vec<GameDto>, Error> {
        Ok(self.listed_games.clone())
    }

    async fn find_game(&self, _game_id: &str) -> Result<Option<GameDto>, Error> {
        Ok(self.found_game.clone())
    }
}
