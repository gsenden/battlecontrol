use std::collections::{HashMap, HashSet};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;

use common::dto::GameDto;
use game_logic::battle::{AudioEventSnapshot, Battle as CoreBattle, BattleSnapshot};
use game_logic::ship_input::ShipInput;
use serde::{Deserialize, Serialize};
use tokio::time::sleep;

use crate::ports::BattleSessionDrivenPort;

const BATTLE_WIDTH: f64 = 7680.0;
const BATTLE_HEIGHT: f64 = 4320.0;
const PHYSICS_DELTA: f64 = 1000.0 / 24.0;

#[derive(Clone)]
pub struct BattleSessionHub {
    sessions: Arc<Mutex<HashMap<String, Arc<BattleSession>>>>,
}

struct BattleSession {
    stopped: AtomicBool,
    runtime: Mutex<BattleRuntime>,
}

struct BattleRuntime {
    battle: CoreBattle,
    participant_names: HashSet<String>,
    active_players: Vec<String>,
    ready_players: HashSet<String>,
    total_players: usize,
    battle_started: bool,
    battle_completed: bool,
    winner_name: Option<String>,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum BattleClientMessage {
    PlayerReady,
    SetInput { input: ShipInputDto },
    SetWeaponTargetPoint { x: f64, y: f64 },
    SetWeaponTargetShip { ship_id: Option<u64> },
    SetSpecialTargetPoint { x: f64, y: f64 },
    SetSpecialTargetShip { ship_id: Option<u64> },
    ClearWeaponTarget,
    ClearSpecialTarget,
}

#[derive(Deserialize)]
pub struct ShipInputDto {
    pub left: bool,
    pub right: bool,
    pub thrust: bool,
    pub weapon: bool,
    pub special: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BattleServerMessage {
    #[serde(rename = "type")]
    pub message_type: &'static str,
    pub snapshot: BattleSnapshotDto,
    pub battle_started: bool,
    pub ready_players: usize,
    pub total_players: usize,
    pub player_active: bool,
    pub battle_completed: bool,
    pub winner_name: Option<String>,
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BattleShipSnapshotDto {
    pub id: u64,
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub crew: i32,
    pub energy: i32,
    pub facing: f64,
    pub turret_facing: f64,
    pub thrusting: bool,
    pub dead: bool,
    pub cloaked: bool,
    pub texture_prefix: &'static str,
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectileSnapshotDto {
    pub id: u64,
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub life: i32,
    pub texture_prefix: &'static str,
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MeteorSnapshotDto {
    pub id: u64,
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub frame_index: i32,
    pub texture_prefix: &'static str,
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExplosionSnapshotDto {
    pub id: u64,
    pub x: f64,
    pub y: f64,
    pub frame_index: i32,
    pub texture_prefix: &'static str,
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LaserSnapshotDto {
    pub id: u64,
    pub start_x: f64,
    pub start_y: f64,
    pub end_x: f64,
    pub end_y: f64,
    pub color: u32,
    pub width: f64,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioEventSnapshotDto {
    pub key: &'static str,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BattleSnapshotDto {
    pub ships: Vec<BattleShipSnapshotDto>,
    pub player: BattleShipSnapshotDto,
    pub target: BattleShipSnapshotDto,
    pub meteors: Vec<MeteorSnapshotDto>,
    pub projectiles: Vec<ProjectileSnapshotDto>,
    pub explosions: Vec<ExplosionSnapshotDto>,
    pub lasers: Vec<LaserSnapshotDto>,
    pub audio_events: Vec<AudioEventSnapshotDto>,
}

impl BattleSessionHub {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn start_battle(&self, game: &GameDto) -> Result<(), String> {
        if game.players.len() < 2 {
            return Err("At least 2 players are required".to_string());
        }

        let player = &game.players[0];
        let target = &game.players[1];
        let participant_names = game
            .players
            .iter()
            .map(|player| player.user.name.clone())
            .collect::<HashSet<_>>();
        let mut battle = CoreBattle::new(
            player.selected_race.as_deref().unwrap_or("human-cruiser"),
            target.selected_race.as_deref().unwrap_or("human-cruiser"),
            (BATTLE_WIDTH / 2.0) + 800.0,
            BATTLE_HEIGHT / 2.0,
            (BATTLE_WIDTH / 2.0) + 2600.0,
            (BATTLE_HEIGHT / 2.0) - 500.0,
            BATTLE_WIDTH / 2.0,
            BATTLE_HEIGHT / 2.0,
            BATTLE_WIDTH,
            BATTLE_HEIGHT,
        )?;
        let active_players = game
            .players
            .iter()
            .map(|participant| participant.user.name.clone())
            .collect::<Vec<_>>();

        for (index, participant) in game.players.iter().enumerate().skip(2) {
            let extra_ship_count = game.players.len().saturating_sub(2).max(1);
            let angle = ((index - 2) as f64 / extra_ship_count as f64) * std::f64::consts::TAU;
            let radius = 1800.0;
            let x = (BATTLE_WIDTH / 2.0) + (radius * angle.cos());
            let y = (BATTLE_HEIGHT / 2.0) + (radius * angle.sin());
            battle.add_active_ship(
                participant
                    .selected_race
                    .as_deref()
                    .unwrap_or("human-cruiser"),
                x,
                y,
            )?;
        }

        let session = Arc::new(BattleSession {
            stopped: AtomicBool::new(false),
            runtime: Mutex::new(BattleRuntime {
                battle,
                participant_names,
                active_players,
                ready_players: HashSet::new(),
                total_players: game.players.len(),
                battle_started: false,
                battle_completed: false,
                winner_name: None,
            }),
        });

        self.sessions
            .lock()
            .expect("battle sessions lock poisoned")
            .insert(game.id.clone(), session.clone());

        tokio::spawn(async move {
            while !session.stopped.load(Ordering::Relaxed) {
                if let Ok(mut runtime) = session.runtime.lock()
                    && runtime.battle_started
                {
                    runtime.battle.tick(PHYSICS_DELTA);
                    runtime.resolve_battle_if_needed();
                }
                sleep(Duration::from_millis(PHYSICS_DELTA as u64)).await;
            }
        });

        Ok(())
    }

    pub fn remove_battle(&self, game_id: &str) {
        let session = self
            .sessions
            .lock()
            .expect("battle sessions lock poisoned")
            .remove(game_id);
        if let Some(session) = session {
            session.stopped.store(true, Ordering::Relaxed);
        }
    }

    pub fn has_battle(&self, game_id: &str) -> bool {
        self.sessions
            .lock()
            .expect("battle sessions lock poisoned")
            .contains_key(game_id)
    }

    pub fn snapshot_for(&self, game_id: &str, user_name: &str) -> Option<BattleSnapshotDto> {
        let session = self
            .sessions
            .lock()
            .expect("battle sessions lock poisoned")
            .get(game_id)
            .cloned()?;
        let runtime = session.runtime.lock().ok()?;
        runtime.snapshot_for(user_name).map(to_snapshot_dto)
    }

    pub fn apply_message(
        &self,
        game_id: &str,
        user_name: &str,
        message: BattleClientMessage,
    ) -> Result<(), String> {
        let session = self
            .sessions
            .lock()
            .expect("battle sessions lock poisoned")
            .get(game_id)
            .cloned()
            .ok_or_else(|| "battle not found".to_string())?;
        let mut runtime = session
            .runtime
            .lock()
            .map_err(|_| "battle runtime lock poisoned".to_string())?;
        if !runtime.participant_names.contains(user_name) {
            return Err("player not in battle".to_string());
        }

        match message {
            BattleClientMessage::PlayerReady => {
                runtime.ready_players.insert(user_name.to_string());
                if runtime.ready_players.len() >= runtime.total_players {
                    runtime.battle_started = true;
                }
            }
            BattleClientMessage::SetInput { input } => {
                let input = ShipInput {
                    left: input.left,
                    right: input.right,
                    thrust: input.thrust,
                    weapon: input.weapon,
                    special: input.special,
                };
                if let Some(ship_id) = runtime.controlled_ship_id_for(user_name) {
                    runtime.battle.set_input_for_ship(ship_id, input)?;
                }
            }
            BattleClientMessage::SetWeaponTargetPoint { x, y } => {
                if let Some(ship_id) = runtime.controlled_ship_id_for(user_name) {
                    runtime.battle.set_weapon_target_point_for(ship_id, x, y)?;
                }
            }
            BattleClientMessage::SetWeaponTargetShip { ship_id } => {
                if let (Some(ship_id), Some(target_ship_id)) = (
                    runtime.controlled_ship_id_for(user_name),
                    runtime.target_ship_id_for(user_name, ship_id),
                ) {
                    runtime
                        .battle
                        .set_weapon_target_ship_for(ship_id, target_ship_id)?;
                }
            }
            BattleClientMessage::SetSpecialTargetPoint { x, y } => {
                if let Some(ship_id) = runtime.controlled_ship_id_for(user_name) {
                    runtime.battle.set_special_target_point_for(ship_id, x, y)?;
                }
            }
            BattleClientMessage::SetSpecialTargetShip { ship_id } => {
                if let (Some(ship_id), Some(target_ship_id)) = (
                    runtime.controlled_ship_id_for(user_name),
                    runtime.target_ship_id_for(user_name, ship_id),
                ) {
                    runtime
                        .battle
                        .set_special_target_ship_for(ship_id, target_ship_id)?;
                }
            }
            BattleClientMessage::ClearWeaponTarget => {
                if let Some(ship_id) = runtime.controlled_ship_id_for(user_name) {
                    runtime.battle.clear_weapon_target_for(ship_id)?;
                }
            }
            BattleClientMessage::ClearSpecialTarget => {
                if let Some(ship_id) = runtime.controlled_ship_id_for(user_name) {
                    runtime.battle.clear_special_target_for(ship_id)?;
                }
            }
        }

        Ok(())
    }

    pub fn ready_state_for(&self, game_id: &str, user_name: &str) -> Option<BattleReadyStateDto> {
        let session = self
            .sessions
            .lock()
            .expect("battle sessions lock poisoned")
            .get(game_id)
            .cloned()?;
        let runtime = session.runtime.lock().ok()?;
        if !runtime.participant_names.contains(user_name) {
            return None;
        }
        Some(BattleReadyStateDto {
            battle_started: runtime.battle_started,
            ready_players: runtime.ready_players.len(),
            total_players: runtime.total_players,
            player_active: runtime.player_active_for(user_name),
            battle_completed: runtime.battle_completed,
            winner_name: runtime.winner_name.clone(),
        })
    }
}

impl BattleRuntime {
    fn snapshot_for(&self, user_name: &str) -> Option<BattleSnapshot> {
        if !self.participant_names.contains(user_name) {
            return None;
        }
        let snapshot = self.battle.snapshot();
        let controlled_ship_id = self.controlled_ship_id_for(user_name)?;
        perspective_snapshot(snapshot, controlled_ship_id)
    }

    fn player_active_for(&self, user_name: &str) -> bool {
        let Some(ship_id) = self.controlled_ship_id_for(user_name) else {
            return false;
        };
        self.battle
            .snapshot()
            .ships
            .into_iter()
            .find(|ship| ship.id == ship_id)
            .map(|ship| !ship.dead)
            .unwrap_or(false)
    }

    fn controlled_ship_id_for(&self, user_name: &str) -> Option<u64> {
        let snapshot = self.battle.snapshot();
        let player_index = self.active_players.iter().position(|name| name == user_name)?;
        snapshot.ships.get(player_index).map(|ship| ship.id)
    }

    fn opponent_ship_id_for(&self, user_name: &str) -> Option<u64> {
        let snapshot = self.battle.snapshot();
        let player_index = self.active_players.iter().position(|name| name == user_name)?;
        snapshot
            .ships
            .iter()
            .enumerate()
            .find(|(index, ship)| *index != player_index && !ship.dead)
            .or_else(|| {
                snapshot
                    .ships
                    .iter()
                    .enumerate()
                    .find(|(index, _)| *index != player_index)
            })
            .map(|(_, ship)| ship.id)
    }

    fn target_ship_id_for(&self, user_name: &str, requested_ship_id: Option<u64>) -> Option<u64> {
        let controlled_ship_id = self.controlled_ship_id_for(user_name)?;
        if let Some(ship_id) = requested_ship_id {
            let ship_exists = self
                .battle
                .snapshot()
                .ships
                .iter()
                .any(|ship| ship.id == ship_id && ship.id != controlled_ship_id);
            if ship_exists {
                return Some(ship_id);
            }
        }
        self.opponent_ship_id_for(user_name)
    }

    fn resolve_battle_if_needed(&mut self) {
        if self.battle_completed {
            return;
        }
        let snapshot = self.battle.snapshot();
        let alive_players = self
            .active_players
            .iter()
            .zip(snapshot.ships.iter())
            .filter(|(_, ship)| !ship.dead)
            .map(|(name, _)| name.clone())
            .collect::<Vec<_>>();
        if alive_players.len() > 1 {
            return;
        }
        self.battle_completed = true;
        self.winner_name = alive_players.into_iter().next();
    }
}

#[derive(Clone)]
pub struct BattleReadyStateDto {
    pub battle_started: bool,
    pub ready_players: usize,
    pub total_players: usize,
    pub player_active: bool,
    pub battle_completed: bool,
    pub winner_name: Option<String>,
}

impl BattleSessionDrivenPort for BattleSessionHub {
    fn start_battle(&self, game: &GameDto) -> Result<(), String> {
        BattleSessionHub::start_battle(self, game)
    }

    fn remove_battle(&self, game_id: &str) {
        BattleSessionHub::remove_battle(self, game_id);
    }

    fn has_battle(&self, game_id: &str) -> bool {
        BattleSessionHub::has_battle(self, game_id)
    }

    fn snapshot_for(&self, game_id: &str, user_name: &str) -> Option<BattleSnapshotDto> {
        BattleSessionHub::snapshot_for(self, game_id, user_name)
    }

    fn ready_state_for(&self, game_id: &str, user_name: &str) -> Option<BattleReadyStateDto> {
        BattleSessionHub::ready_state_for(self, game_id, user_name)
    }

    fn apply_message(
        &self,
        game_id: &str,
        user_name: &str,
        message: BattleClientMessage,
    ) -> Result<(), String> {
        BattleSessionHub::apply_message(self, game_id, user_name, message)
    }
}

fn perspective_snapshot(snapshot: BattleSnapshot, player_ship_id: u64) -> Option<BattleSnapshot> {
    let player = snapshot
        .ships
        .iter()
        .find(|ship| ship.id == player_ship_id)
        .copied()?;
    let target = snapshot
        .ships
        .iter()
        .find(|ship| ship.id != player_ship_id && !ship.dead)
        .or_else(|| snapshot.ships.iter().find(|ship| ship.id != player_ship_id))
        .copied()
        .unwrap_or(player);
    Some(BattleSnapshot {
        ships: snapshot.ships,
        player,
        target,
        meteors: snapshot.meteors,
        projectiles: snapshot.projectiles,
        explosions: snapshot.explosions,
        lasers: snapshot.lasers,
        audio_events: snapshot.audio_events,
    })
}

fn to_snapshot_dto(snapshot: BattleSnapshot) -> BattleSnapshotDto {
    BattleSnapshotDto {
        ships: snapshot.ships.into_iter().map(to_ship_dto).collect(),
        player: to_ship_dto(snapshot.player),
        target: to_ship_dto(snapshot.target),
        meteors: snapshot
            .meteors
            .into_iter()
            .map(|meteor| MeteorSnapshotDto {
                id: meteor.id,
                x: meteor.x,
                y: meteor.y,
                vx: meteor.vx,
                vy: meteor.vy,
                frame_index: meteor.frame_index,
                texture_prefix: meteor.texture_prefix,
            })
            .collect(),
        projectiles: snapshot
            .projectiles
            .into_iter()
            .map(|projectile| ProjectileSnapshotDto {
                id: projectile.id,
                x: projectile.x,
                y: projectile.y,
                vx: projectile.vx,
                vy: projectile.vy,
                life: projectile.life,
                texture_prefix: projectile.texture_prefix,
            })
            .collect(),
        explosions: snapshot
            .explosions
            .into_iter()
            .map(|explosion| ExplosionSnapshotDto {
                id: explosion.id,
                x: explosion.x,
                y: explosion.y,
                frame_index: explosion.frame_index,
                texture_prefix: explosion.texture_prefix,
            })
            .collect(),
        lasers: snapshot
            .lasers
            .into_iter()
            .map(|laser| LaserSnapshotDto {
                id: laser.id,
                start_x: laser.start_x,
                start_y: laser.start_y,
                end_x: laser.end_x,
                end_y: laser.end_y,
                color: laser.color,
                width: laser.width,
            })
            .collect(),
        audio_events: snapshot
            .audio_events
            .into_iter()
            .map(to_audio_dto)
            .collect(),
    }
}

fn to_ship_dto(ship: game_logic::battle::BattleShipSnapshot) -> BattleShipSnapshotDto {
    BattleShipSnapshotDto {
        id: ship.id,
        x: ship.x,
        y: ship.y,
        vx: ship.vx,
        vy: ship.vy,
        crew: ship.crew,
        energy: ship.energy,
        facing: ship.facing,
        turret_facing: ship.turret_facing,
        thrusting: ship.thrusting,
        dead: ship.dead,
        cloaked: ship.cloaked,
        texture_prefix: ship.texture_prefix,
    }
}

fn to_audio_dto(event: AudioEventSnapshot) -> AudioEventSnapshotDto {
    AudioEventSnapshotDto { key: event.key }
}

#[cfg(test)]
mod tests {
    use common::dto::{GameDto, GamePlayerDto, UserDto};

    use super::*;

    #[tokio::test]
    async fn additional_player_snapshot_uses_their_own_ship_as_player() {
        let (hub, game) = BattleSessionTestBuilder::new()
            .with_waiting_player("PilotThree", "androsynth-guardian")
            .build();
        hub.start_battle(&game).expect("start battle");

        let snapshot = hub
            .snapshot_for(&game.id, "PilotThree")
            .expect("player snapshot");
        hub.remove_battle(&game.id);

        assert_eq!(snapshot.player.texture_prefix, "androsynth-guardian");
    }

    #[tokio::test]
    async fn ready_state_marks_additional_player_active() {
        let (hub, game) = BattleSessionTestBuilder::new()
            .with_waiting_player("PilotThree", "androsynth-guardian")
            .build();
        hub.start_battle(&game).expect("start battle");

        let ready_state = hub
            .ready_state_for(&game.id, "PilotThree")
            .expect("ready state");
        hub.remove_battle(&game.id);

        assert!(ready_state.player_active);
    }

    #[tokio::test]
    async fn start_battle_adds_all_players_as_active_ships() {
        let (hub, game) = BattleSessionTestBuilder::new()
            .with_waiting_player("PilotThree", "androsynth-guardian")
            .build();
        hub.start_battle(&game).expect("start battle");

        let snapshot = hub.snapshot_for(&game.id, "PilotOne").expect("snapshot");
        hub.remove_battle(&game.id);

        assert_eq!(snapshot.ships.len(), 3);
    }

    #[tokio::test]
    async fn snapshot_dto_contains_ship_turret_facing() {
        let hub = BattleSessionHub::new();
        let game = two_player_game();
        hub.start_battle(&game).expect("start battle");

        let snapshot = hub.snapshot_for(&game.id, "PilotOne").expect("snapshot");
        let json = serde_json::to_string(&snapshot).expect("serialize snapshot");
        let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        hub.remove_battle(&game.id);

        assert!(value["player"]["turretFacing"].is_number());
    }

    struct BattleSessionTestBuilder {
        hub: BattleSessionHub,
        game: GameDto,
    }

    impl BattleSessionTestBuilder {
        fn new() -> Self {
            Self {
                hub: BattleSessionHub::new(),
                game: two_player_game(),
            }
        }

        fn with_waiting_player(mut self, name: &str, race: &str) -> Self {
            self.game.players.push(GamePlayerDto {
                user: user(3, name),
                selected_race: Some(race.to_string()),
            });
            self
        }

        fn build(self) -> (BattleSessionHub, GameDto) {
            (self.hub, self.game)
        }
    }

    fn user(id: i64, name: &str) -> UserDto {
        UserDto {
            id,
            name: name.to_string(),
            profile_image_url: None,
        }
    }

    fn two_player_game() -> GameDto {
        GameDto {
            id: "game-2p".to_string(),
            name: "Test 2P".to_string(),
            game_type: "free_for_all".to_string(),
            max_players: 2,
            is_private: false,
            password: None,
            creator: user(1, "PilotOne"),
            players: vec![
                GamePlayerDto {
                    user: user(1, "PilotOne"),
                    selected_race: Some("human-cruiser".to_string()),
                },
                GamePlayerDto {
                    user: user(2, "PilotTwo"),
                    selected_race: Some("arilou-skiff".to_string()),
                },
            ],
        }
    }

    fn one_player_game() -> GameDto {
        let mut game = two_player_game();
        game.id = "game-1p".to_string();
        game.players.truncate(1);
        game
    }

    #[tokio::test]
    async fn snapshot_for_additional_player_returns_some() {
        let hub = BattleSessionHub::new();
        let mut game = two_player_game();
        game.players.push(GamePlayerDto {
            user: user(3, "PilotThree"),
            selected_race: Some("androsynth-guardian".to_string()),
        });
        hub.start_battle(&game).expect("start battle");

        let snapshot = hub.snapshot_for(&game.id, "PilotThree");
        hub.remove_battle(&game.id);

        assert!(snapshot.is_some());
    }

    #[tokio::test]
    async fn ready_state_counts_all_game_players() {
        let hub = BattleSessionHub::new();
        let mut game = two_player_game();
        game.players.push(GamePlayerDto {
            user: user(3, "PilotThree"),
            selected_race: Some("androsynth-guardian".to_string()),
        });
        hub.start_battle(&game).expect("start battle");

        hub.apply_message(&game.id, "PilotOne", BattleClientMessage::PlayerReady)
            .expect("player one ready");
        hub.apply_message(&game.id, "PilotTwo", BattleClientMessage::PlayerReady)
            .expect("player two ready");
        let ready_state = hub
            .ready_state_for(&game.id, "PilotThree")
            .expect("ready state");
        hub.remove_battle(&game.id);

        assert_eq!(
            (ready_state.ready_players, ready_state.total_players),
            (2, 3)
        );
    }

    #[tokio::test]
    async fn start_battle_requires_at_least_two_players() {
        let hub = BattleSessionHub::new();
        let game = one_player_game();

        let result = hub.start_battle(&game);

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn start_battle_accepts_more_than_two_players() {
        let hub = BattleSessionHub::new();
        let mut game = two_player_game();
        game.players.push(GamePlayerDto {
            user: user(3, "PilotThree"),
            selected_race: Some("androsynth-guardian".to_string()),
        });

        let result = hub.start_battle(&game);
        hub.remove_battle(&game.id);

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn start_battle_registers_session() {
        let hub = BattleSessionHub::new();
        let game = two_player_game();
        hub.start_battle(&game).expect("start battle");

        let has_battle = hub.has_battle(&game.id);
        hub.remove_battle(&game.id);

        assert!(has_battle);
    }

    #[tokio::test]
    async fn remove_battle_clears_session() {
        let hub = BattleSessionHub::new();
        let game = two_player_game();
        hub.start_battle(&game).expect("start battle");

        hub.remove_battle(&game.id);

        assert!(!hub.has_battle(&game.id));
    }

    #[tokio::test]
    async fn snapshot_for_unknown_player_returns_none() {
        let hub = BattleSessionHub::new();
        let game = two_player_game();
        hub.start_battle(&game).expect("start battle");

        let snapshot = hub.snapshot_for(&game.id, "UnknownPilot");
        hub.remove_battle(&game.id);

        assert!(snapshot.is_none());
    }
}
