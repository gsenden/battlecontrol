use std::collections::HashMap;
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
    player_name: String,
    target_name: String,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum BattleClientMessage {
    SetInput { input: ShipInputDto },
    SetWeaponTargetPoint { x: f64, y: f64 },
    SetWeaponTargetShip,
    SetSpecialTargetPoint { x: f64, y: f64 },
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
        if game.players.len() != 2 {
            return Err("Only 2-player battles are currently supported".to_string());
        }

        let player = &game.players[0];
        let target = &game.players[1];
        let battle = CoreBattle::new(
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

        let session = Arc::new(BattleSession {
            stopped: AtomicBool::new(false),
            runtime: Mutex::new(BattleRuntime {
                battle,
                player_name: player.user.name.clone(),
                target_name: target.user.name.clone(),
            }),
        });

        self.sessions
            .lock()
            .expect("battle sessions lock poisoned")
            .insert(game.id.clone(), session.clone());

        tokio::spawn(async move {
            while !session.stopped.load(Ordering::Relaxed) {
                if let Ok(mut runtime) = session.runtime.lock() {
                    runtime.battle.tick(PHYSICS_DELTA);
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
        let snapshot = runtime.battle.snapshot();
        if runtime.player_name == user_name {
            Some(to_snapshot_dto(snapshot))
        } else if runtime.target_name == user_name {
            Some(to_snapshot_dto(swapped_snapshot(snapshot)))
        } else {
            None
        }
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
        let is_player = if runtime.player_name == user_name {
            true
        } else if runtime.target_name == user_name {
            false
        } else {
            return Err("player not in battle".to_string());
        };

        match message {
            BattleClientMessage::SetInput { input } => {
                let input = ShipInput {
                    left: input.left,
                    right: input.right,
                    thrust: input.thrust,
                    weapon: input.weapon,
                    special: input.special,
                };
                if is_player {
                    runtime.battle.set_player_input(input);
                } else {
                    runtime.battle.set_target_input(input);
                }
            }
            BattleClientMessage::SetWeaponTargetPoint { x, y } => {
                if is_player {
                    runtime.battle.set_player_weapon_target_point(x, y);
                } else {
                    runtime.battle.set_target_weapon_target_point(x, y);
                }
            }
            BattleClientMessage::SetWeaponTargetShip => {
                if is_player {
                    runtime.battle.set_player_weapon_target_ship();
                } else {
                    runtime.battle.set_target_weapon_target_ship();
                }
            }
            BattleClientMessage::SetSpecialTargetPoint { x, y } => {
                if is_player {
                    runtime.battle.set_player_special_target_point(x, y);
                } else {
                    runtime.battle.set_target_special_target_point(x, y);
                }
            }
            BattleClientMessage::ClearWeaponTarget => {
                if is_player {
                    runtime.battle.clear_player_weapon_target();
                } else {
                    runtime.battle.clear_target_weapon_target();
                }
            }
            BattleClientMessage::ClearSpecialTarget => {
                if is_player {
                    runtime.battle.clear_player_special_target();
                } else {
                    runtime.battle.clear_target_special_target();
                }
            }
        }

        Ok(())
    }
}

fn swapped_snapshot(snapshot: BattleSnapshot) -> BattleSnapshot {
    BattleSnapshot {
        player: snapshot.target,
        target: snapshot.player,
        meteors: snapshot.meteors,
        projectiles: snapshot.projectiles,
        explosions: snapshot.explosions,
        lasers: snapshot.lasers,
        audio_events: snapshot.audio_events,
    }
}

fn to_snapshot_dto(snapshot: BattleSnapshot) -> BattleSnapshotDto {
    BattleSnapshotDto {
        player: to_ship_dto(snapshot.player),
        target: to_ship_dto(snapshot.target),
        meteors: snapshot.meteors.into_iter().map(|meteor| MeteorSnapshotDto {
            id: meteor.id,
            x: meteor.x,
            y: meteor.y,
            vx: meteor.vx,
            vy: meteor.vy,
            frame_index: meteor.frame_index,
            texture_prefix: meteor.texture_prefix,
        }).collect(),
        projectiles: snapshot.projectiles.into_iter().map(|projectile| ProjectileSnapshotDto {
            id: projectile.id,
            x: projectile.x,
            y: projectile.y,
            vx: projectile.vx,
            vy: projectile.vy,
            life: projectile.life,
            texture_prefix: projectile.texture_prefix,
        }).collect(),
        explosions: snapshot.explosions.into_iter().map(|explosion| ExplosionSnapshotDto {
            id: explosion.id,
            x: explosion.x,
            y: explosion.y,
            frame_index: explosion.frame_index,
            texture_prefix: explosion.texture_prefix,
        }).collect(),
        lasers: snapshot.lasers.into_iter().map(|laser| LaserSnapshotDto {
            id: laser.id,
            start_x: laser.start_x,
            start_y: laser.start_y,
            end_x: laser.end_x,
            end_y: laser.end_y,
            color: laser.color,
            width: laser.width,
        }).collect(),
        audio_events: snapshot.audio_events.into_iter().map(to_audio_dto).collect(),
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
        thrusting: ship.thrusting,
        dead: ship.dead,
        cloaked: ship.cloaked,
        texture_prefix: ship.texture_prefix,
    }
}

fn to_audio_dto(event: AudioEventSnapshot) -> AudioEventSnapshotDto {
    AudioEventSnapshotDto { key: event.key }
}
