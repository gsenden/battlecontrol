use async_trait::async_trait;
use common::domain::Error;
use common::dto::{CreateGameRequestDto, GameDto, JoinGameRequestDto, SaveSelectedRaceRequestDto};

#[async_trait]
pub trait GameRepositoryDrivenPort: Send + Sync + 'static {
    async fn save_game(&self, creator_name: &str, request: &CreateGameRequestDto) -> Result<GameDto, Error>;
    async fn join_game(&self, game_id: &str, player_name: &str, request: &JoinGameRequestDto) -> Result<Option<GameDto>, Error>;
    async fn save_selected_race(&self, game_id: &str, player_name: &str, request: &SaveSelectedRaceRequestDto) -> Result<Option<GameDto>, Error>;
    async fn list_games(&self) -> Result<Vec<GameDto>, Error>;
    async fn find_game(&self, game_id: &str) -> Result<Option<GameDto>, Error>;
}
