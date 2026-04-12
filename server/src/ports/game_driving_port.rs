use async_trait::async_trait;
use common::domain::Error;
use common::dto::{CreateGameRequestDto, GameDto, JoinGameRequestDto, SaveSelectedRaceRequestDto};

#[async_trait]
pub trait GameDrivingPort {
    async fn create_game(&self, creator_name: String, request: CreateGameRequestDto) -> Result<GameDto, Error>;
    async fn join_game(&self, game_id: String, player_name: String, request: JoinGameRequestDto) -> Result<GameDto, Error>;
    async fn save_selected_race(&self, game_id: String, player_name: String, request: SaveSelectedRaceRequestDto) -> Result<GameDto, Error>;
    async fn list_games(&self) -> Result<Vec<GameDto>, Error>;
    async fn find_game(&self, game_id: String) -> Result<GameDto, Error>;
}
