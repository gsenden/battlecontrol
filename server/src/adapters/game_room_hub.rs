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
        Self {
            rooms: Arc::new(Mutex::new(HashMap::new())),
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
}

impl GameRoomDrivenPort for GameRoomHub {
    fn create_room(&self, game: &GameDto) {
        let mut rooms = self.rooms.lock().expect("game rooms lock poisoned");
        let entry = rooms
            .entry(game.id.clone())
            .or_insert_with(Self::new_room_entry);
        entry.current_game = Some(game.clone());
        entry.started_game = None;
        Self::broadcast(
            entry,
            GameRoomEvent {
                kind: GameRoomEventKind::Snapshot,
                game_id: game.id.clone(),
                game: Some(game.clone()),
            },
        );
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
        Self::broadcast(
            entry,
            GameRoomEvent {
                kind: GameRoomEventKind::Started,
                game_id: game.id.clone(),
                game: Some(game.clone()),
            },
        );
    }

    fn cancel_room(&self, game_id: &str) {
        let mut rooms = self.rooms.lock().expect("game rooms lock poisoned");
        if let Some(entry) = rooms.remove(game_id) {
            Self::broadcast(
                &entry,
                GameRoomEvent {
                    kind: GameRoomEventKind::Cancelled,
                    game_id: game_id.to_string(),
                    game: None,
                },
            );
        }
    }

    fn remove_rooms(&self, game_ids: &[String]) {
        for game_id in game_ids {
            self.cancel_room(game_id);
        }
    }
}
