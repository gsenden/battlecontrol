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
        Self {
            game_repo,
            game_rooms,
        }
    }
}

#[async_trait]
impl<DP: GameLobbyDrivenPorts> GameDrivingPort for GameLobby<DP> {
    async fn create_game(
        &self,
        creator_name: String,
        request: CreateGameRequestDto,
    ) -> Result<GameDto, Error> {
        let game = self.game_repo.save_game(&creator_name, &request).await?;
        self.game_rooms.create_room(&game);
        Ok(game)
    }

    async fn join_game(
        &self,
        game_id: String,
        player_name: String,
        request: JoinGameRequestDto,
    ) -> Result<GameDto, Error> {
        let game = self
            .game_repo
            .join_game(&game_id, &player_name, &request)
            .await?
            .ok_or_else(|| Error::RoomNotFound(RoomNotFoundError::new(game_id.clone())))?;
        self.game_rooms.update_room(&game);
        Ok(game)
    }

    async fn leave_game(&self, game_id: String, player_name: String) -> Result<(), Error> {
        let game = self
            .game_repo
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
        self.game_repo
            .cancel_game(&game_id, &player_name)
            .await?
            .ok_or_else(|| Error::RoomNotFound(RoomNotFoundError::new(game_id.clone())))?;
        self.game_rooms.cancel_room(&game_id);
        Ok(())
    }

    async fn start_game(&self, game_id: String, player_name: String) -> Result<GameDto, Error> {
        let game = self
            .game_repo
            .start_game(&game_id, &player_name)
            .await?
            .ok_or_else(|| Error::RoomNotFound(RoomNotFoundError::new(game_id.clone())))?;
        Ok(game)
    }

    async fn save_selected_race(
        &self,
        game_id: String,
        player_name: String,
        request: SaveSelectedRaceRequestDto,
    ) -> Result<GameDto, Error> {
        let game = self
            .game_repo
            .save_selected_race(&game_id, &player_name, &request)
            .await?
            .ok_or_else(|| Error::RoomNotFound(RoomNotFoundError::new(game_id.clone())))?;
        self.game_rooms.update_room(&game);
        Ok(game)
    }

    async fn list_games(&self) -> Result<Vec<GameDto>, Error> {
        let games = self.game_repo.list_games().await?;
        for game in &games {
            self.game_rooms.update_room(game);
        }
        Ok(games)
    }

    async fn find_game(&self, game_id: String) -> Result<GameDto, Error> {
        let game = self
            .game_repo
            .find_game(&game_id)
            .await?
            .ok_or_else(|| Error::RoomNotFound(RoomNotFoundError::new(game_id)))?;
        self.game_rooms.update_room(&game);
        Ok(game)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::{
        FakeGameRepository, FakeGameRoomDrivenPort,
        sample_data::{
            TEST_GAME_ID, TEST_PLAYER_NAME, test_create_game_request, test_game,
            test_join_game_request, test_save_selected_race_request,
        },
    };

    struct TestGameLobbyDrivenPorts;

    impl GameLobbyDrivenPorts for TestGameLobbyDrivenPorts {
        type GameRepo = FakeGameRepository;
        type GameRooms = FakeGameRoomDrivenPort;
    }

    struct TestGameLobbyBuilder {
        game_repo: FakeGameRepository,
        game_rooms: FakeGameRoomDrivenPort,
    }

    impl TestGameLobbyBuilder {
        fn new() -> Self {
            Self {
                game_repo: FakeGameRepository::new(),
                game_rooms: FakeGameRoomDrivenPort::new(),
            }
        }

        fn with_game_repo(mut self, game_repo: FakeGameRepository) -> Self {
            self.game_repo = game_repo;
            self
        }

        fn build(
            self,
        ) -> (
            GameLobby<TestGameLobbyDrivenPorts>,
            FakeGameRepository,
            FakeGameRoomDrivenPort,
        ) {
            let game_repo = self.game_repo;
            let game_rooms = self.game_rooms;
            (
                GameLobby::<TestGameLobbyDrivenPorts>::new(game_repo.clone(), game_rooms.clone()),
                game_repo,
                game_rooms,
            )
        }
    }

    #[tokio::test]
    async fn create_game_creates_room() {
        let game = test_game();
        let (lobby, _, game_rooms) = TestGameLobbyBuilder::new()
            .with_game_repo(FakeGameRepository::new().with_created_game(game))
            .build();

        lobby
            .create_game(TEST_PLAYER_NAME.to_string(), test_create_game_request())
            .await
            .unwrap();

        assert_eq!(
            game_rooms.created_room_ids(),
            vec![TEST_GAME_ID.to_string()]
        );
    }

    #[tokio::test]
    async fn join_game_updates_room() {
        let game = test_game();
        let (lobby, _, game_rooms) = TestGameLobbyBuilder::new()
            .with_game_repo(FakeGameRepository::new().with_joined_game(Some(game)))
            .build();

        lobby
            .join_game(
                TEST_GAME_ID.to_string(),
                TEST_PLAYER_NAME.to_string(),
                test_join_game_request(),
            )
            .await
            .unwrap();

        assert_eq!(
            game_rooms.updated_room_ids(),
            vec![TEST_GAME_ID.to_string()]
        );
    }

    #[tokio::test]
    async fn leave_game_cancels_room_when_last_player_left() {
        let mut empty_game = test_game();
        empty_game.players = Vec::new();
        let (lobby, _, game_rooms) = TestGameLobbyBuilder::new()
            .with_game_repo(FakeGameRepository::new().with_left_game(Some(empty_game)))
            .build();

        lobby
            .leave_game(TEST_GAME_ID.to_string(), TEST_PLAYER_NAME.to_string())
            .await
            .unwrap();

        assert_eq!(
            game_rooms.cancelled_room_ids(),
            vec![TEST_GAME_ID.to_string()]
        );
    }

    #[tokio::test]
    async fn cancel_game_cancels_room() {
        let (lobby, _, game_rooms) = TestGameLobbyBuilder::new()
            .with_game_repo(FakeGameRepository::new().with_cancelled_game(Some(test_game())))
            .build();

        lobby
            .cancel_game(TEST_GAME_ID.to_string(), TEST_PLAYER_NAME.to_string())
            .await
            .unwrap();

        assert_eq!(
            game_rooms.cancelled_room_ids(),
            vec![TEST_GAME_ID.to_string()]
        );
    }

    #[tokio::test]
    async fn start_game_returns_started_game() {
        let (lobby, _, _) = TestGameLobbyBuilder::new()
            .with_game_repo(FakeGameRepository::new().with_started_game(Some(test_game())))
            .build();

        let game = lobby
            .start_game(TEST_GAME_ID.to_string(), TEST_PLAYER_NAME.to_string())
            .await
            .unwrap();

        assert_eq!(game.id, TEST_GAME_ID);
    }

    #[tokio::test]
    async fn save_selected_race_updates_room() {
        let (lobby, _, game_rooms) = TestGameLobbyBuilder::new()
            .with_game_repo(FakeGameRepository::new().with_selected_race_game(Some(test_game())))
            .build();

        lobby
            .save_selected_race(
                TEST_GAME_ID.to_string(),
                TEST_PLAYER_NAME.to_string(),
                test_save_selected_race_request(),
            )
            .await
            .unwrap();

        assert_eq!(
            game_rooms.updated_room_ids(),
            vec![TEST_GAME_ID.to_string()]
        );
    }

    #[tokio::test]
    async fn list_games_updates_each_room() {
        let listed_games = vec![test_game()];
        let (lobby, _, game_rooms) = TestGameLobbyBuilder::new()
            .with_game_repo(FakeGameRepository::new().with_listed_games(listed_games))
            .build();

        lobby.list_games().await.unwrap();

        assert_eq!(
            game_rooms.updated_room_ids(),
            vec![TEST_GAME_ID.to_string()]
        );
    }

    #[tokio::test]
    async fn find_game_updates_room() {
        let (lobby, _, game_rooms) = TestGameLobbyBuilder::new()
            .with_game_repo(FakeGameRepository::new().with_found_game(Some(test_game())))
            .build();

        lobby.find_game(TEST_GAME_ID.to_string()).await.unwrap();

        assert_eq!(
            game_rooms.updated_room_ids(),
            vec![TEST_GAME_ID.to_string()]
        );
    }
}
