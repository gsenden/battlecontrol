use async_trait::async_trait;
use common::domain::Error;
use common::domain::error::RoomNotFoundError;
use common::dto::{CreateGameRequestDto, GameDto, JoinGameRequestDto, SaveSelectedRaceRequestDto};

use crate::ports::{GameDrivingPort, GameRepositoryDrivenPort, GameRoomDrivenPort};

pub trait GameLobbyDrivenPorts: Send + Sync + 'static {
    type GameRepo: GameRepositoryDrivenPort;
    type GameRooms: GameRoomDrivenPort;
}

pub struct GameLobby<DP: GameLobbyDrivenPorts> {
    game_repo: DP::GameRepo,
    game_rooms: DP::GameRooms,
}

impl<DP: GameLobbyDrivenPorts> GameLobby<DP> {
    pub fn new(game_repo: DP::GameRepo, game_rooms: DP::GameRooms) -> Self {
        Self { game_repo, game_rooms }
    }

    async fn cleanup_stale_games(&self) -> Result<(), Error> {
        let stale_game_ids = self.game_repo.delete_stale_games().await?;
        self.game_rooms.remove_rooms(&stale_game_ids);
        Ok(())
    }
}

#[async_trait]
impl<DP: GameLobbyDrivenPorts> GameDrivingPort for GameLobby<DP> {
    async fn create_game(&self, creator_name: String, request: CreateGameRequestDto) -> Result<GameDto, Error> {
        self.cleanup_stale_games().await?;
        let game = self.game_repo.save_game(&creator_name, &request).await?;
        self.game_rooms.create_room(&game);
        Ok(game)
    }

    async fn join_game(&self, game_id: String, player_name: String, request: JoinGameRequestDto) -> Result<GameDto, Error> {
        self.cleanup_stale_games().await?;
        let game = self.game_repo
            .join_game(&game_id, &player_name, &request)
            .await?
            .ok_or_else(|| Error::RoomNotFound(RoomNotFoundError::new(game_id.clone())))?;
        self.game_rooms.update_room(&game);
        Ok(game)
    }

    async fn leave_game(&self, game_id: String, player_name: String) -> Result<(), Error> {
        self.cleanup_stale_games().await?;
        let game = self.game_repo
            .leave_game(&game_id, &player_name)
            .await?
            .ok_or_else(|| Error::RoomNotFound(RoomNotFoundError::new(game_id.clone())))?;
        if game.players.is_empty() {
            self.game_rooms.cancel_room(&game_id);
        } else {
            self.game_rooms.update_room(&game);
        }
        Ok(())
    }

    async fn cancel_game(&self, game_id: String, player_name: String) -> Result<(), Error> {
        self.cleanup_stale_games().await?;
        self.game_repo
            .cancel_game(&game_id, &player_name)
            .await?
            .ok_or_else(|| Error::RoomNotFound(RoomNotFoundError::new(game_id.clone())))?;
        self.game_rooms.cancel_room(&game_id);
        Ok(())
    }

    async fn start_game(&self, game_id: String, player_name: String) -> Result<GameDto, Error> {
        self.cleanup_stale_games().await?;
        let game = self.game_repo
            .start_game(&game_id, &player_name)
            .await?
            .ok_or_else(|| Error::RoomNotFound(RoomNotFoundError::new(game_id.clone())))?;
        self.game_rooms.start_room(&game);
        Ok(game)
    }

    async fn save_selected_race(&self, game_id: String, player_name: String, request: SaveSelectedRaceRequestDto) -> Result<GameDto, Error> {
        self.cleanup_stale_games().await?;
        let game = self.game_repo
            .save_selected_race(&game_id, &player_name, &request)
            .await?
            .ok_or_else(|| Error::RoomNotFound(RoomNotFoundError::new(game_id.clone())))?;
        self.game_rooms.update_room(&game);
        Ok(game)
    }

    async fn list_games(&self) -> Result<Vec<GameDto>, Error> {
        self.cleanup_stale_games().await?;
        let games = self.game_repo.list_games().await?;
        for game in &games {
            self.game_rooms.update_room(game);
        }
        Ok(games)
    }

    async fn find_game(&self, game_id: String) -> Result<GameDto, Error> {
        self.cleanup_stale_games().await?;
        let game = self.game_repo
            .find_game(&game_id)
            .await?
            .ok_or_else(|| Error::RoomNotFound(RoomNotFoundError::new(game_id)))?;
        self.game_rooms.update_room(&game);
        Ok(game)
    }
}
