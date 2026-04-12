use async_trait::async_trait;
use common::domain::Error;
use common::domain::error::RoomNotFoundError;
use common::dto::{CreateGameRequestDto, GameDto, JoinGameRequestDto, SaveSelectedRaceRequestDto};

use crate::ports::{GameDrivingPort, GameRepositoryDrivenPort};

pub trait GameLobbyDrivenPorts: Send + Sync + 'static {
    type GameRepo: GameRepositoryDrivenPort;
}

pub struct GameLobby<DP: GameLobbyDrivenPorts> {
    game_repo: DP::GameRepo,
}

impl<DP: GameLobbyDrivenPorts> GameLobby<DP> {
    pub fn new(game_repo: DP::GameRepo) -> Self {
        Self { game_repo }
    }
}

#[async_trait]
impl<DP: GameLobbyDrivenPorts> GameDrivingPort for GameLobby<DP> {
    async fn create_game(&self, creator_name: String, request: CreateGameRequestDto) -> Result<GameDto, Error> {
        self.game_repo.save_game(&creator_name, &request).await
    }

    async fn join_game(&self, game_id: String, player_name: String, request: JoinGameRequestDto) -> Result<GameDto, Error> {
        self.game_repo
            .join_game(&game_id, &player_name, &request)
            .await?
            .ok_or_else(|| Error::RoomNotFound(RoomNotFoundError::new(game_id)))
    }

    async fn save_selected_race(&self, game_id: String, player_name: String, request: SaveSelectedRaceRequestDto) -> Result<GameDto, Error> {
        self.game_repo
            .save_selected_race(&game_id, &player_name, &request)
            .await?
            .ok_or_else(|| Error::RoomNotFound(RoomNotFoundError::new(game_id)))
    }

    async fn list_games(&self) -> Result<Vec<GameDto>, Error> {
        self.game_repo.list_games().await
    }

    async fn find_game(&self, game_id: String) -> Result<GameDto, Error> {
        self.game_repo
            .find_game(&game_id)
            .await?
            .ok_or_else(|| Error::RoomNotFound(RoomNotFoundError::new(game_id)))
    }
}
