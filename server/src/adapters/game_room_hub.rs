use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::extract::ws::Message;
use common::dto::GameDto;
use serde::Serialize;
use tokio::sync::broadcast;

use crate::ports::GameRoomDrivenPort;

#[derive(Clone)]
pub struct GameRoomHub {
    rooms: Arc<Mutex<HashMap<String, RoomEntry>>>,
    global_sender: broadcast::Sender<GameRoomEvent>,
}

#[derive(Clone)]
struct RoomEntry {
    sender: broadcast::Sender<GameRoomEvent>,
    current_game: Option<GameDto>,
    started_game: Option<GameDto>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct GameRoomEvent {
    pub kind: GameRoomEventKind,
    pub game_id: String,
    pub game: Option<GameDto>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GameRoomEventKind {
    Snapshot,
    Started,
    Cancelled,
}

impl GameRoomHub {
    pub fn new() -> Self {
        let (global_sender, _) = broadcast::channel(64);
        Self {
            rooms: Arc::new(Mutex::new(HashMap::new())),
            global_sender,
        }
    }

    pub fn subscribe(&self, game_id: &str) -> broadcast::Receiver<GameRoomEvent> {
        let mut rooms = self.rooms.lock().expect("game rooms lock poisoned");
        let entry = rooms
            .entry(game_id.to_string())
            .or_insert_with(Self::new_room_entry);
        entry.sender.subscribe()
    }

    pub fn current_message(&self, game_id: &str) -> Option<Message> {
        let rooms = self.rooms.lock().expect("game rooms lock poisoned");
        let entry = rooms.get(game_id)?;
        let event = if let Some(game) = entry.current_game.clone() {
            Some(GameRoomEvent {
                kind: GameRoomEventKind::Snapshot,
                game_id: game_id.to_string(),
                game: Some(game),
            })
        } else {
            entry.started_game.clone().map(|game| GameRoomEvent {
                kind: GameRoomEventKind::Started,
                game_id: game_id.to_string(),
                game: Some(game),
            })
        }?;
        Some(Self::message_for(event))
    }

    pub fn game(&self, game_id: &str) -> Option<GameDto> {
        let rooms = self.rooms.lock().expect("game rooms lock poisoned");
        let entry = rooms.get(game_id)?;
        entry.current_game.clone().or_else(|| entry.started_game.clone())
    }

    pub fn subscribe_all(&self) -> broadcast::Receiver<GameRoomEvent> {
        self.global_sender.subscribe()
    }

    fn new_room_entry() -> RoomEntry {
        let (sender, _) = broadcast::channel(32);
        RoomEntry {
            sender,
            current_game: None,
            started_game: None,
        }
    }

    fn message_for(event: GameRoomEvent) -> Message {
        Message::Text(serde_json::to_string(&event).expect("Failed to serialize game room event").into())
    }

    fn broadcast(entry: &RoomEntry, event: GameRoomEvent) {
        let _ = entry.sender.send(event);
    }

    fn broadcast_global(&self, event: GameRoomEvent) {
        let _ = self.global_sender.send(event);
    }
}

impl GameRoomDrivenPort for GameRoomHub {
    fn create_room(&self, game: &GameDto) {
        let mut rooms = self.rooms.lock().expect("game rooms lock poisoned");
        let entry = rooms
            .entry(game.id.clone())
            .or_insert_with(Self::new_room_entry);
        entry.current_game = Some(game.clone());
        entry.started_game = None;
        let event = GameRoomEvent {
            kind: GameRoomEventKind::Snapshot,
            game_id: game.id.clone(),
            game: Some(game.clone()),
        };
        Self::broadcast(entry, event.clone());
        self.broadcast_global(event);
    }

    fn update_room(&self, game: &GameDto) {
        self.create_room(game);
    }

    fn start_room(&self, game: &GameDto) {
        let mut rooms = self.rooms.lock().expect("game rooms lock poisoned");
        let entry = rooms
            .entry(game.id.clone())
            .or_insert_with(Self::new_room_entry);
        entry.current_game = None;
        entry.started_game = Some(game.clone());
        let event = GameRoomEvent {
            kind: GameRoomEventKind::Started,
            game_id: game.id.clone(),
            game: Some(game.clone()),
        };
        Self::broadcast(entry, event.clone());
        self.broadcast_global(event);
    }

    fn cancel_room(&self, game_id: &str) {
        let mut rooms = self.rooms.lock().expect("game rooms lock poisoned");
        if let Some(entry) = rooms.remove(game_id) {
            let event = GameRoomEvent {
                kind: GameRoomEventKind::Cancelled,
                game_id: game_id.to_string(),
                game: None,
            };
            Self::broadcast(&entry, event.clone());
            self.broadcast_global(event);
        }
    }

    fn remove_rooms(&self, game_ids: &[String]) {
        for game_id in game_ids {
            self.cancel_room(game_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use axum::extract::ws::Message;

    use super::*;
    use crate::ports::GameRoomDrivenPort;
    use crate::test_helpers::sample_data::test_game;

    #[test]
    fn create_room_sets_current_message() {
        let hub = GameRoomHub::new();
        let game = test_game();

        hub.create_room(&game);

        assert!(hub.current_message(&game.id).is_some());
    }

    #[test]
    fn cancel_room_clears_current_message() {
        let hub = GameRoomHub::new();
        let game = test_game();
        hub.create_room(&game);

        hub.cancel_room(&game.id);

        assert!(hub.current_message(&game.id).is_none());
    }

    #[test]
    fn start_room_broadcasts_started_kind() {
        let hub = GameRoomHub::new();
        let game = test_game();

        hub.start_room(&game);
        let message = hub.current_message(&game.id).expect("room message");
        let Message::Text(payload) = message else {
            panic!("expected text message");
        };

        assert!(payload.contains("\"kind\":\"started\""));
    }

    #[tokio::test]
    async fn subscribe_all_receives_snapshot_event() {
        let hub = GameRoomHub::new();
        let mut receiver = hub.subscribe_all();
        let game = test_game();

        hub.create_room(&game);
        let event = receiver.recv().await.expect("global event");

        assert!(matches!(event.kind, GameRoomEventKind::Snapshot));
    }
}
