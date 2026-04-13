use std::sync::{Arc, Mutex};

use common::dto::GameDto;

use crate::ports::GameRoomDrivenPort;

#[derive(Clone)]
pub struct FakeGameRoomDrivenPort {
    created_room_ids: Arc<Mutex<Vec<String>>>,
    updated_room_ids: Arc<Mutex<Vec<String>>>,
    cancelled_room_ids: Arc<Mutex<Vec<String>>>,
    removed_room_ids: Arc<Mutex<Vec<Vec<String>>>>,
}

impl FakeGameRoomDrivenPort {
    pub fn new() -> Self {
        Self {
            created_room_ids: Arc::new(Mutex::new(Vec::new())),
            updated_room_ids: Arc::new(Mutex::new(Vec::new())),
            cancelled_room_ids: Arc::new(Mutex::new(Vec::new())),
            removed_room_ids: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn created_room_ids(&self) -> Vec<String> {
        self.created_room_ids.lock().unwrap().clone()
    }

    pub fn updated_room_ids(&self) -> Vec<String> {
        self.updated_room_ids.lock().unwrap().clone()
    }

    pub fn cancelled_room_ids(&self) -> Vec<String> {
        self.cancelled_room_ids.lock().unwrap().clone()
    }

}

impl GameRoomDrivenPort for FakeGameRoomDrivenPort {
    fn create_room(&self, game: &GameDto) {
        self.created_room_ids.lock().unwrap().push(game.id.clone());
    }

    fn update_room(&self, game: &GameDto) {
        self.updated_room_ids.lock().unwrap().push(game.id.clone());
    }

    fn start_room(&self, _game: &GameDto) {}

    fn cancel_room(&self, game_id: &str) {
        self.cancelled_room_ids.lock().unwrap().push(game_id.to_string());
    }

    fn remove_rooms(&self, game_ids: &[String]) {
        self.removed_room_ids.lock().unwrap().push(game_ids.to_vec());
    }
}
