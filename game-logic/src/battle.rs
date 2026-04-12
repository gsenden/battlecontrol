use crate::matter_world::{MatterBodyState, MatterWorld};
use crate::physics_command::PhysicsCommand;
use crate::ship_input::ShipInput;
use crate::traits::game_object::GameObject;
use crate::traits::ship_trait::{
    CrewDrainTransferSpec, CrewToEnergySpec, HitPolygonPoint, InstantLaserSpec,
    PlanetHarvestSpec, PrimaryProjectileSpec, ProjectileBehaviorSpec, ProjectileCollisionSpec,
    ProjectileSpawnSpec, ProjectileTargetMode, ProjectileVolleySpec, SecondaryProjectileSpec,
    SelfDestructSpec, SoundOnlySpec, SpecialAbilitySpec, TeleportSpecialSpec, TransformSpec,
};
use crate::ships::{apply_collision_between, build_ship, AnyShip};
use crate::velocity_vector::VelocityVector;
use crate::wrap::{shortest_wrapped_delta, wrap_axis};
use matter_js_rs::geometry::Vec2;

const INITIAL_PLAYER_ID: u64 = 1;
const INITIAL_TARGET_ID: u64 = 2;
const INITIAL_NEXT_GAME_OBJECT_ID: u64 = 3;
const INITIAL_METEOR_LAYOUT: [(f64, f64, f64, f64, i32); 3] = [
    (0.12, 0.18, 4.0, 2.0, 0),
    (0.78, 0.28, -3.0, 4.0, 7),
    (0.52, 0.82, -4.5, -2.5, 13),
];
const SHIP_DEATH_EXPLOSION_START_FRAME: i32 = 0;
const SHIP_DEATH_EXPLOSION_END_FRAME: i32 = 8;
const ANDROSYNTH_BUBBLE_RANDOM_SEED: u32 = 0x00C0FFEE;
const METEOR_DAMAGE: i32 = 1;
const METEOR_FRAME_COUNT: i32 = 21;
const METEOR_HIT_RADIUS: f64 = 44.0;
const METEOR_IMPACT_PUSH: f64 = 2.5;
const METEOR_TEXTURE_PREFIX: &str = "battle-asteroid";
const PROJECTILE_FACINGS: f64 = 16.0;
const EXPLOSION_TEXTURE_BATTLE_BOOM: &str = "battle-boom";
const AUDIO_SHIP_DEATH: &str = "battle-shipdies";
const SC2_SINE_TABLE: [i32; 64] = [
    -16384, -16305, -16069, -15679, -15137, -14449, -13623, -12665,
    -11585, -10394, -9102, -7723, -6270, -4756, -3196, -1606,
    0, 1606, 3196, 4756, 6270, 7723, 9102, 10394,
    11585, 12665, 13623, 14449, 15137, 15679, 16069, 16305,
    16384, 16305, 16069, 15679, 15137, 14449, 13623, 12665,
    11585, 10394, 9102, 7723, 6270, 4756, 3196, 1606,
    0, -1606, -3196, -4756, -6270, -7723, -9102, -10394,
    -11585, -12665, -13623, -14449, -15137, -15679, -16069, -16305,
];

#[derive(Clone, Copy)]
pub struct BattleShipSnapshot {
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

#[derive(Clone, Copy)]
pub struct ProjectileSnapshot {
    pub id: u64,
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub life: i32,
    pub texture_prefix: &'static str,
    damage: i32,
    impact_texture_prefix: &'static str,
    impact_start_frame: i32,
    impact_end_frame: i32,
    impact_sound_key: &'static str,
    track_wait: i32,
    behavior: ProjectileBehaviorSpec,
    collision: ProjectileCollisionSpec,
    facing: f64,
    acceleration: f64,
    max_speed: f64,
    turn_wait: i32,
    facing_index: i32,
    speed: i32,
    raw_vx: i32,
    raw_vy: i32,
    velocity_width: i32,
    velocity_height: i32,
    velocity_fract_width: i32,
    velocity_fract_height: i32,
    velocity_error_width: i32,
    velocity_error_height: i32,
    velocity_sign_width: i32,
    velocity_sign_height: i32,
    bubble_rng: u32,
    owner_is_player: bool,
    target: ProjectileTarget,
}

#[derive(Clone, Copy)]
pub struct ExplosionSnapshot {
    pub id: u64,
    pub x: f64,
    pub y: f64,
    pub frame_index: i32,
    pub end_frame: i32,
    pub texture_prefix: &'static str,
}

#[derive(Clone, Copy)]
pub struct MeteorSnapshot {
    pub id: u64,
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub frame_index: i32,
    pub texture_prefix: &'static str,
    radius: f64,
    frame_count: i32,
    spin_step: i32,
    player_contacting: bool,
    target_contacting: bool,
}

#[derive(Clone, Copy)]
pub struct LaserSnapshot {
    pub id: u64,
    pub start_x: f64,
    pub start_y: f64,
    pub end_x: f64,
    pub end_y: f64,
    pub color: u32,
    pub width: f64,
}

#[derive(Clone, Copy)]
pub struct AudioEventSnapshot {
    pub key: &'static str,
}

pub struct BattleSnapshot {
    pub player: BattleShipSnapshot,
    pub target: BattleShipSnapshot,
    pub meteors: Vec<MeteorSnapshot>,
    pub projectiles: Vec<ProjectileSnapshot>,
    pub explosions: Vec<ExplosionSnapshot>,
    pub lasers: Vec<LaserSnapshot>,
    pub audio_events: Vec<AudioEventSnapshot>,
}

#[derive(Clone, Copy)]
enum ProjectileTarget {
    None,
    Point { x: f64, y: f64 },
    PlayerShip,
    TargetShip,
}

#[derive(Clone, Copy)]
enum SpecialTarget {
    None,
    Point { x: f64, y: f64 },
}

struct BattleShipState {
    id: u64,
    ship_id: usize,
    body_id: usize,
    thrusting: bool,
    dead: bool,
    special_active: bool,
    special_contacting: bool,
    previous_x: f64,
    previous_y: f64,
}

pub struct Battle {
    ships: Vec<AnyShip>,
    meteors: Vec<MeteorSnapshot>,
    projectiles: Vec<ProjectileSnapshot>,
    explosions: Vec<ExplosionSnapshot>,
    lasers: Vec<LaserSnapshot>,
    audio_events: Vec<AudioEventSnapshot>,
    matter_world: MatterWorld,
    player: BattleShipState,
    target: BattleShipState,
    player_input: ShipInput,
    target_input: ShipInput,
    queued_target_weapon: bool,
    player_weapon_target: ProjectileTarget,
    target_weapon_target: ProjectileTarget,
    player_special_target: SpecialTarget,
    target_special_target: SpecialTarget,
    bubble_rng_state: u32,
    planet_x: f64,
    planet_y: f64,
    width: f64,
    height: f64,
    next_game_object_id: u64,
}

impl Battle {
    pub fn new(
        player_ship_type: &str,
        target_ship_type: &str,
        player_x: f64,
        player_y: f64,
        target_x: f64,
        target_y: f64,
        planet_x: f64,
        planet_y: f64,
        width: f64,
        height: f64,
    ) -> Result<Self, String> {
        let mut ships = Vec::new();
        let mut matter_world = MatterWorld::new();

        let player_ship = build_ship(player_ship_type)
            .ok_or_else(|| format!("unknown ship type: {player_ship_type}"))?;
        let player_ship_id = ships.len();
        let player_body_id = create_ship_body_for(
            &mut matter_world,
            &player_ship,
            player_x,
            player_y,
            false,
        );
        ships.push(player_ship);

        let target_ship = build_ship(target_ship_type)
            .ok_or_else(|| format!("unknown ship type: {target_ship_type}"))?;
        let target_ship_id = ships.len();
        let target_body_id = create_ship_body_for(
            &mut matter_world,
            &target_ship,
            target_x,
            target_y,
            false,
        );
        ships.push(target_ship);

        let mut battle = Self {
            ships,
            meteors: Vec::new(),
            projectiles: Vec::new(),
            explosions: Vec::new(),
            lasers: Vec::new(),
            audio_events: Vec::new(),
            matter_world,
            player: BattleShipState {
                id: INITIAL_PLAYER_ID,
                ship_id: player_ship_id,
                body_id: player_body_id,
                thrusting: false,
                dead: false,
                special_active: false,
                special_contacting: false,
                previous_x: player_x,
                previous_y: player_y,
            },
            target: BattleShipState {
                id: INITIAL_TARGET_ID,
                ship_id: target_ship_id,
                body_id: target_body_id,
                thrusting: false,
                dead: false,
                special_active: false,
                special_contacting: false,
                previous_x: target_x,
                previous_y: target_y,
            },
            player_input: ShipInput {
                left: false,
                right: false,
                thrust: false,
                weapon: false,
                special: false,
            },
            target_input: ShipInput {
                left: false,
                right: false,
                thrust: false,
                weapon: false,
                special: false,
            },
            queued_target_weapon: false,
            player_weapon_target: ProjectileTarget::None,
            target_weapon_target: ProjectileTarget::None,
            player_special_target: SpecialTarget::None,
            target_special_target: SpecialTarget::None,
            bubble_rng_state: ANDROSYNTH_BUBBLE_RANDOM_SEED,
            planet_x,
            planet_y,
            width,
            height,
            next_game_object_id: INITIAL_NEXT_GAME_OBJECT_ID,
        };
        battle.spawn_initial_meteors();
        Ok(battle)
    }

    fn spawn_initial_meteors(&mut self) {
        for (x_ratio, y_ratio, vx, vy, frame_index) in INITIAL_METEOR_LAYOUT {
            let meteor_id = self.next_game_object_id();
            self.meteors.push(MeteorSnapshot {
                id: meteor_id,
                x: self.width * x_ratio,
                y: self.height * y_ratio,
                vx,
                vy,
                frame_index,
                texture_prefix: METEOR_TEXTURE_PREFIX,
                radius: METEOR_HIT_RADIUS,
                frame_count: METEOR_FRAME_COUNT,
                spin_step: 1,
                player_contacting: false,
                target_contacting: false,
            });
        }
    }

    pub fn set_player_input(&mut self, input: ShipInput) {
        self.player_input = input;
    }

    pub fn set_target_input(&mut self, input: ShipInput) {
        self.target_input = input;
    }

    pub fn trigger_target_weapon(&mut self) {
        self.queued_target_weapon = true;
    }

    pub fn set_player_weapon_target_point(&mut self, x: f64, y: f64) {
        self.player_weapon_target = ProjectileTarget::Point { x, y };
    }

    pub fn set_player_weapon_target_ship(&mut self) {
        self.player_weapon_target = ProjectileTarget::TargetShip;
    }

    pub fn clear_player_weapon_target(&mut self) {
        self.player_weapon_target = ProjectileTarget::None;
    }

    pub fn set_target_weapon_target_point(&mut self, x: f64, y: f64) {
        self.target_weapon_target = ProjectileTarget::Point { x, y };
    }

    pub fn set_target_weapon_target_ship(&mut self) {
        self.target_weapon_target = ProjectileTarget::PlayerShip;
    }

    pub fn clear_target_weapon_target(&mut self) {
        self.target_weapon_target = ProjectileTarget::None;
    }

    pub fn set_player_special_target_point(&mut self, x: f64, y: f64) {
        self.player_special_target = SpecialTarget::Point { x, y };
    }

    pub fn clear_player_special_target(&mut self) {
        self.player_special_target = SpecialTarget::None;
    }

    pub fn set_target_special_target_point(&mut self, x: f64, y: f64) {
        self.target_special_target = SpecialTarget::Point { x, y };
    }

    pub fn clear_target_special_target(&mut self) {
        self.target_special_target = SpecialTarget::None;
    }

    pub fn switch_player_ship(&mut self, ship_type: &str) -> Result<(), String> {
        let current = self
            .matter_world
            .body_state(self.player.body_id)
            .ok_or_else(|| "missing player body state".to_string())?;
        let next_ship = build_ship(ship_type).ok_or_else(|| format!("unknown ship type: {ship_type}"))?;
        let next_body_id = create_ship_body_for(
            &mut self.matter_world,
            &next_ship,
            current.x,
            current.y,
            self.player.special_active,
        );
        self.matter_world.set_body_velocity(next_body_id, current.vx, current.vy);
        self.matter_world.disable_body(self.player.body_id);
        self.ships[self.player.ship_id] = next_ship;
        self.player.body_id = next_body_id;
        self.player.thrusting = false;
        Ok(())
    }

    pub fn tick(&mut self, delta: f64) {
        self.audio_events.clear();
        self.lasers.clear();

        for explosion in &mut self.explosions {
            explosion.frame_index += 1;
        }
        self.explosions
            .retain(|explosion| explosion.frame_index <= explosion.end_frame);
        self.step_meteors();

        let player_target_body = self.matter_world.body_state(self.player.body_id);
        let target_target_body = self.matter_world.body_state(self.target.body_id);

        for projectile in &mut self.projectiles {
            if matches!(projectile.behavior, ProjectileBehaviorSpec::WobbleTracking { .. }) {
                step_wobble_tracking_projectile(projectile, player_target_body, target_target_body);
                continue;
            }

            match projectile.target {
                ProjectileTarget::None => {}
                ProjectileTarget::Point { x, y } => {
                    if projectile.turn_wait > 0 {
                        projectile.turn_wait -= 1;
                    } else {
                        projectile.facing_index = turn_projectile_toward_target(
                            projectile.facing_index,
                            projectile.x,
                            projectile.y,
                            x,
                            y,
                        );
                        projectile.facing = facing_index_to_radians(projectile.facing_index);
                        projectile.turn_wait = projectile.track_wait;
                    }
                }
                ProjectileTarget::PlayerShip | ProjectileTarget::TargetShip => {
                    if projectile.turn_wait > 0 {
                        projectile.turn_wait -= 1;
                    } else if let Some(target) = match projectile.target {
                        ProjectileTarget::PlayerShip => player_target_body,
                        ProjectileTarget::TargetShip => target_target_body,
                        ProjectileTarget::None | ProjectileTarget::Point { .. } => None,
                    } {
                        projectile.facing_index = turn_projectile_toward_target(
                            projectile.facing_index,
                            projectile.x,
                            projectile.y,
                            target.x,
                            target.y,
                        );
                        projectile.facing = facing_index_to_radians(projectile.facing_index);
                        projectile.turn_wait = projectile.track_wait;
                    }
                }
            }
            projectile.speed = (projectile.speed + projectile.acceleration as i32).min(projectile.max_speed as i32);
            let (raw_vx, raw_vy) = projectile_velocity_for_facing(projectile.facing_index, projectile.speed);
            set_projectile_velocity_components(projectile, raw_vx, raw_vy);
            advance_projectile_position(projectile);
            projectile.life -= 1;
        }

        self.step_ship(self.player.ship_id, self.player.body_id, self.player_input, true);
        let target_input = ShipInput {
            weapon: self.target_input.weapon || self.queued_target_weapon,
            ..self.target_input
        };
        self.step_ship(
            self.target.ship_id,
            self.target.body_id,
            target_input,
            false,
        );
        self.queued_target_weapon = false;

        self.handle_human_point_defense(true, self.player_input);
        self.handle_human_point_defense(false, target_input);

        let mut hit_projectile_indexes = Vec::new();
        let mut hit_explosions = Vec::new();
        let mut player_died = false;
        let mut target_died = false;
        let mut player_won = false;
        let mut target_won = false;

        for (index, projectile) in self.projectiles.iter().enumerate() {
            let Some(is_player) = self.projectile_hit_target(projectile) else {
                continue;
            };

            if !self.ship_blocks_damage(is_player) {
                let damage = projectile.damage;
                if is_player {
                    let died = self.ships[self.player.ship_id].take_damage(damage);
                    player_died |= died;
                    target_won |= died && !projectile.owner_is_player;
                } else {
                    let died = self.ships[self.target.ship_id].take_damage(damage);
                    target_died |= died;
                    player_won |= died && projectile.owner_is_player;
                }
            }
            hit_explosions.push(ExplosionSnapshot {
                id: 0,
                x: projectile.x,
                y: projectile.y,
                frame_index: projectile.impact_start_frame,
                end_frame: projectile.impact_end_frame,
                texture_prefix: projectile.impact_texture_prefix,
            });
            self.audio_events.push(AudioEventSnapshot {
                key: projectile.impact_sound_key,
            });
            hit_projectile_indexes.push(index);
        }

        for index in hit_projectile_indexes.into_iter().rev() {
            self.projectiles.remove(index);
        }

        self.handle_meteor_collisions();

        for mut explosion in hit_explosions {
            explosion.id = self.next_game_object_id();
            self.explosions.push(explosion);
        }

        self.projectiles.retain(|projectile| projectile.life >= 0);

        let player_body_before_step = self.matter_world.body_state(self.player.body_id);
        let target_body_before_step = self.matter_world.body_state(self.target.body_id);
        let state = self.matter_world.step(delta);
        let mut player_blazer_hit_applied = false;
        let mut target_blazer_hit_applied = false;

        let player_blazer_spec = self.blazer_spec_for(true);
        let target_blazer_spec = self.blazer_spec_for(false);
        let player_blazer_hits = player_blazer_spec.is_some()
            && self.player.special_active
            && !self.target.dead
            && self.androsynth_blazer_hits_other_ship(true);
        if player_blazer_hits && !self.player.special_contacting {
            if let (Some(pb), Some(tb)) = (player_body_before_step, target_body_before_step) {
                self.apply_blazer_collision_velocity(
                    player_blazer_spec.expect("blazer hit must have spec").mass,
                    self.player.body_id, self.target.body_id,
                    &pb, &tb, self.ships[self.target.ship_id].mass(),
                );
            }
            apply_collision_between(&mut self.ships, self.player.ship_id, self.target.ship_id);
            if !self.ship_blocks_damage(false) {
                let died = self.ships[self.target.ship_id]
                    .take_damage(player_blazer_spec.expect("blazer hit must have spec").damage);
                target_died |= died;
                player_won |= died;
            }
            player_blazer_hit_applied = true;
            self.audio_events.push(AudioEventSnapshot {
                key: player_blazer_spec.expect("blazer hit must have spec").impact_sound_key,
            });
        }
        self.player.special_contacting = player_blazer_hits;

        let target_blazer_hits = target_blazer_spec.is_some()
            && self.target.special_active
            && !self.player.dead
            && self.androsynth_blazer_hits_other_ship(false);
        if target_blazer_hits && !self.target.special_contacting {
            if let (Some(pb), Some(tb)) = (player_body_before_step, target_body_before_step) {
                self.apply_blazer_collision_velocity(
                    target_blazer_spec.expect("blazer hit must have spec").mass,
                    self.target.body_id, self.player.body_id,
                    &tb, &pb, self.ships[self.player.ship_id].mass(),
                );
            }
            apply_collision_between(&mut self.ships, self.player.ship_id, self.target.ship_id);
            if !self.ship_blocks_damage(true) {
                let died = self.ships[self.player.ship_id]
                    .take_damage(target_blazer_spec.expect("blazer hit must have spec").damage);
                player_died |= died;
                target_won |= died;
            }
            target_blazer_hit_applied = true;
            self.audio_events.push(AudioEventSnapshot {
                key: target_blazer_spec.expect("blazer hit must have spec").impact_sound_key,
            });
        }
        self.target.special_contacting = target_blazer_hits;

        for collision in state.collisions {
            let ids = [collision.body_a, collision.body_b];
            if !ids.contains(&self.player.body_id) || !ids.contains(&self.target.body_id) {
                continue;
            }

            apply_collision_between(&mut self.ships, self.player.ship_id, self.target.ship_id);
            if self.player.special_active && !self.target.dead && !player_blazer_hit_applied {
                if let (Some(player_before), Some(target_before)) =
                    (player_body_before_step, target_body_before_step)
                {
                    self.apply_blazer_collision_velocity(
                        player_blazer_spec.expect("active blazer must have spec").mass,
                        self.player.body_id,
                        self.target.body_id,
                        &player_before,
                        &target_before,
                        self.ships[self.target.ship_id].mass(),
                    );
                }
            } else if self.target.special_active && !self.player.dead && !target_blazer_hit_applied {
                if let (Some(player_before), Some(target_before)) =
                    (player_body_before_step, target_body_before_step)
                {
                    self.apply_blazer_collision_velocity(
                        target_blazer_spec.expect("active blazer must have spec").mass,
                        self.target.body_id,
                        self.player.body_id,
                        &target_before,
                        &player_before,
                        self.ships[self.player.ship_id].mass(),
                    );
                }
            } else if !player_blazer_hits && !target_blazer_hits {
                if let (Some(player_before), Some(target_before)) =
                    (player_body_before_step, target_body_before_step)
                {
                    let player_after = state
                        .bodies
                        .iter()
                        .find(|body| body.id == self.player.body_id)
                        .copied()
                        .unwrap_or(player_before);
                    let target_after = state
                        .bodies
                        .iter()
                        .find(|body| body.id == self.target.body_id)
                        .copied()
                        .unwrap_or(target_before);
                    let ((player_vx, player_vy), (target_vx, target_vy)) =
                        resolve_collision_velocity(
                            player_after.x,
                            player_after.y,
                            player_before.vx,
                            player_before.vy,
                            self.ships[self.player.ship_id].mass(),
                            target_after.x,
                            target_after.y,
                            target_before.vx,
                            target_before.vy,
                            self.ships[self.target.ship_id].mass(),
                        );
                    self.matter_world
                        .set_body_velocity(self.player.body_id, player_vx, player_vy);
                    self.matter_world
                        .set_body_velocity(self.target.body_id, target_vx, target_vy);
                }
            }
            if self.player.special_active && !self.target.dead && !player_blazer_hit_applied {
                let player_blazer_spec = player_blazer_spec.expect("active blazer must have spec");
                if !self.ship_blocks_damage(false) {
                    let died = self.ships[self.target.ship_id].take_damage(player_blazer_spec.damage);
                    target_died |= died;
                    player_won |= died;
                }
                self.audio_events.push(AudioEventSnapshot {
                    key: player_blazer_spec.impact_sound_key,
                });
            }
            if self.target.special_active && !self.player.dead && !target_blazer_hit_applied {
                let target_blazer_spec = target_blazer_spec.expect("active blazer must have spec");
                if !self.ship_blocks_damage(true) {
                    let died = self.ships[self.player.ship_id].take_damage(target_blazer_spec.damage);
                    player_died |= died;
                    target_won |= died;
                }
                self.audio_events.push(AudioEventSnapshot {
                    key: target_blazer_spec.impact_sound_key,
                });
            }
        }

        if player_died {
            self.mark_ship_dead(true);
        }
        if target_died {
            self.mark_ship_dead(false);
        }
        if player_won
            && let Some(key) = self.ships[self.player.ship_id].victory_sound_key()
        {
            self.audio_events.push(AudioEventSnapshot {
                key,
            });
        }
        if target_won
            && let Some(key) = self.ships[self.target.ship_id].victory_sound_key()
        {
            self.audio_events.push(AudioEventSnapshot {
                key,
            });
        }

        let _ = self.matter_world.wrap_body(self.player.body_id, self.width, self.height);
        let _ = self.matter_world.wrap_body(self.target.body_id, self.width, self.height);
    }

    pub fn snapshot(&self) -> BattleSnapshot {
        BattleSnapshot {
            player: self.snapshot_for(&self.player),
            target: self.snapshot_for(&self.target),
            meteors: self.meteors.clone(),
            projectiles: self.projectiles.clone(),
            explosions: self.explosions.clone(),
            lasers: self.lasers.clone(),
            audio_events: self.audio_events.clone(),
        }
    }

    fn step_ship(&mut self, ship_id: usize, body_id: usize, input: ShipInput, is_player: bool) {
        if self.ship_state(is_player).dead {
            return;
        }

        if self.ship_state(is_player).special_active && self.blazer_spec_for(is_player).is_some() {
            self.step_androsynth_blazer(ship_id, body_id, input, is_player);
            return;
        }

        let Some(body) = self.matter_world.body_state(body_id) else {
            return;
        };
        let ship_state = self.ship_state_mut(is_player);
        ship_state.previous_x = body.x;
        ship_state.previous_y = body.y;

        self.apply_gravity(ship_id, body_id, body);
        let current = self
            .matter_world
            .body_state(body_id)
            .unwrap_or(body);
        let in_gravity_well = self.in_gravity_well(current.x, current.y);
        let energy_before = self.ships[ship_id].energy();
        let weapon_counter_before = self.ships[ship_id].weapon_counter();
        let special_counter_before = self.ships[ship_id].special_counter();
        let mut commands = self.ships[ship_id].update(
            &input,
            &VelocityVector {
                x: current.vx,
                y: current.vy,
            },
            in_gravity_well,
        );
        if let SpecialAbilitySpec::Blazer(blazer_spec) = self.ships[ship_id].special_ability_spec() {
            if input.special && self.ships[ship_id].energy() < energy_before {
                let should_play_special_sound = !self.ship_state(is_player).special_active;
                if should_play_special_sound {
                    self.audio_events.push(AudioEventSnapshot {
                        key: blazer_spec.activation_sound_key,
                    });
                }
                self.matter_world.set_body_mass(body_id, blazer_spec.mass);
                let ship_state = self.ship_state_mut(is_player);
                ship_state.special_active = true;
                let energy_wait = self.ships[ship_id].energy_wait();
                self.ships[ship_id].set_energy_counter(energy_wait - 1);
            }
            if self.ship_state(is_player).special_active {
                let facing = self.ships[ship_id].facing();
                commands.push(PhysicsCommand::SetVelocity {
                    vx: facing.cos() * blazer_spec.speed,
                    vy: facing.sin() * blazer_spec.speed,
                });
            }
        }
        let weapon_triggered = input.weapon
            && weapon_counter_before == 0
            && self.ships[ship_id].weapon_counter() == self.ships[ship_id].weapon_wait()
            && (self.ships[ship_id].weapon_wait() > 0
                || self.ships[ship_id].weapon_energy_cost() == 0
                || self.ships[ship_id].energy() < energy_before);
        let special_active = self.ship_state(is_player).special_active;
        if weapon_triggered {
            if self.ships[ship_id].is_cloaked(special_active) {
                self.ship_state_mut(is_player).special_active = false;
            }

            if let Some(volley_spec) = self.ships[ship_id].primary_volley_spec_for_state(special_active) {
                self.spawn_projectile_volley(current, is_player, volley_spec);
            } else if let Some(projectile_spec) = self.ships[ship_id].primary_projectile_spec_for_state(special_active) {
                self.spawn_projectile_from_spec(
                    current,
                    is_player,
                    projectile_spec,
                    ProjectileSpawnSpec {
                        facing_offset: 0,
                        forward_offset: projectile_spec.offset,
                        lateral_offset: 0.0,
                    },
                    self.projectile_target_for(is_player),
                );
                if !projectile_spec.sound_key.is_empty() {
                    self.audio_events.push(AudioEventSnapshot {
                        key: projectile_spec.sound_key,
                    });
                }
            } else if let Some(laser_spec) = self.ships[ship_id].primary_instant_laser_spec_for_state(special_active) {
                self.fire_instant_laser(current, is_player, laser_spec);
            }
        }

        let special_triggered = input.special
            && special_counter_before == 0
            && self.ships[ship_id].special_counter() == self.ships[ship_id].special_wait()
            && (self.ships[ship_id].special_wait() > 0
                || self.ships[ship_id].special_energy_cost() == 0
                || self.ships[ship_id].energy() < energy_before);
        if special_triggered {
            match self.ships[ship_id].special_ability_spec() {
                SpecialAbilitySpec::Teleport(spec) => {
                    self.activate_teleport_special(is_player, body_id, spec);
                    self.ship_state_mut(is_player).special_active = true;
                }
                SpecialAbilitySpec::InstantLaser(spec) => {
                    self.fire_instant_laser(current, is_player, spec);
                }
                SpecialAbilitySpec::Shield(spec) => {
                    if !spec.sound_key.is_empty() {
                        self.audio_events.push(AudioEventSnapshot { key: spec.sound_key });
                    }
                    self.ship_state_mut(is_player).special_active = true;
                }
                SpecialAbilitySpec::DirectionalThrust(spec) => {
                    if !spec.sound_key.is_empty() {
                        self.audio_events.push(AudioEventSnapshot { key: spec.sound_key });
                    }
                    let facing = self.ships[ship_id].facing() + spec.facing_offset;
                    commands.push(PhysicsCommand::SetVelocity {
                        vx: facing.cos() * spec.speed,
                        vy: facing.sin() * spec.speed,
                    });
                }
                SpecialAbilitySpec::Projectile(SecondaryProjectileSpec { volley }) => {
                    self.spawn_projectile_volley(current, is_player, volley);
                }
                SpecialAbilitySpec::CrewRegeneration(spec) => {
                    if !spec.sound_key.is_empty() {
                        self.audio_events.push(AudioEventSnapshot { key: spec.sound_key });
                    }
                    let max_crew = self.ships[ship_id].max_crew();
                    let next_crew = (self.ships[ship_id].crew() + spec.amount).min(max_crew);
                    self.ships[ship_id].set_crew(next_crew);
                }
                SpecialAbilitySpec::CrewToEnergy(spec) => {
                    self.activate_crew_to_energy_special(current, ship_id, spec, &mut commands);
                }
                SpecialAbilitySpec::SelfDestruct(spec) => {
                    self.activate_self_destruct_special(current, is_player, spec);
                }
                SpecialAbilitySpec::SoundOnly(spec) => {
                    self.activate_sound_only_special(spec);
                }
                SpecialAbilitySpec::Cloak(spec) => {
                    self.activate_cloak_special(is_player, spec);
                }
                SpecialAbilitySpec::Transform(spec) => {
                    self.activate_transform_special(is_player, spec);
                }
                SpecialAbilitySpec::CrewDrainTransfer(spec) => {
                    self.activate_crew_drain_special(is_player, spec);
                }
                SpecialAbilitySpec::PlanetHarvest(spec) => {
                    self.activate_planet_harvest_special(current, ship_id, spec);
                }
                SpecialAbilitySpec::None
                | SpecialAbilitySpec::PointDefense(_)
                | SpecialAbilitySpec::Blazer(_) => {}
            }
        }
        let thrusting = apply_commands(&mut self.matter_world, body_id, commands);

        if !self.ships[ship_id].special_state_persists_after_cooldown()
            && !matches!(self.ships[ship_id].special_ability_spec(), SpecialAbilitySpec::Blazer(_))
        {
            let special_active = self.ship_state(is_player).special_active;
            if special_active && self.ships[ship_id].special_counter() == 0 {
                self.ship_state_mut(is_player).special_active = false;
            }
        }

        if is_player {
            self.player.thrusting = thrusting;
        } else {
            self.target.thrusting = thrusting;
        }

        sync_ship_body_angle(&mut self.matter_world, body_id, &self.ships[ship_id]);
    }

    fn step_androsynth_blazer(&mut self, ship_id: usize, body_id: usize, input: ShipInput, is_player: bool) {
        let SpecialAbilitySpec::Blazer(blazer_spec) = self.ships[ship_id].special_ability_spec() else {
            return;
        };
        if self.ships[ship_id].energy() <= 0 {
            self.matter_world.set_body_mass(body_id, self.ships[ship_id].mass());
            self.ship_state_mut(is_player).special_active = false;
            let _ = apply_commands(&mut self.matter_world, body_id, vec![PhysicsCommand::SetVelocity {
                vx: 0.0,
                vy: 0.0,
            }]);
            if is_player {
                self.player.thrusting = false;
            } else {
                self.target.thrusting = false;
            }
            return;
        }

        if self.ships[ship_id].turn_counter() > 0 {
            self.ships[ship_id].decrease_turn_counter(1);
        } else if input.left || input.right {
            let rate = self.ships[ship_id].turn_rate();
            if input.left {
                self.ships[ship_id].decrease_facing(rate);
            } else {
                self.ships[ship_id].increase_facing(rate);
            }
            self.ships[ship_id].set_turn_counter(1);
        }

        if self.ships[ship_id].energy_counter() > 0 {
            self.ships[ship_id].decrease_energy_counter(1);
        } else {
            self.ships[ship_id].decrease_energy(1);
            let energy_wait = self.ships[ship_id].energy_wait();
            self.ships[ship_id].set_energy_counter(energy_wait);
        }

        let thrusting = if self.ships[ship_id].thrust_counter() > 0 {
            self.ships[ship_id].decrease_thrust_counter(1);
            false
        } else {
            let facing = self.ships[ship_id].facing();
            apply_commands(&mut self.matter_world, body_id, vec![PhysicsCommand::SetVelocity {
                vx: facing.cos() * blazer_spec.speed,
                vy: facing.sin() * blazer_spec.speed,
            }])
        };

        if is_player {
            self.player.thrusting = thrusting;
        } else {
            self.target.thrusting = thrusting;
        }

        sync_ship_body_angle(&mut self.matter_world, body_id, &self.ships[ship_id]);
    }

    fn apply_gravity(&mut self, ship_id: usize, body_id: usize, body: MatterBodyState) {
        let dx = shortest_wrapped_delta(body.x, self.planet_x, self.width);
        let dy = shortest_wrapped_delta(body.y, self.planet_y, self.height);
        if let Some(command) = self.ships[ship_id].gravity_command(dx, dy) {
            let _ = apply_commands(&mut self.matter_world, body_id, vec![command]);
        }
    }

    fn in_gravity_well(&self, x: f64, y: f64) -> bool {
        shortest_wrapped_delta(x, self.planet_x, self.width).abs() <= 420.0
            && shortest_wrapped_delta(y, self.planet_y, self.height).abs() <= 420.0
    }

    fn handle_human_point_defense(&mut self, is_player: bool, input: ShipInput) {
        if !input.special {
            return;
        }

        let ship_state = if is_player { &self.player } else { &self.target };
        if ship_state.dead {
            return;
        }

        let ship_id = ship_state.ship_id;
        let SpecialAbilitySpec::PointDefense(point_defense_spec) = self.ships[ship_id].special_ability_spec() else {
            return;
        };

        let special_wait = self.ships[ship_id].special_wait();
        if self.ships[ship_id].special_counter() != special_wait {
            return;
        }

        let special_cost = self.ships[ship_id].special_energy_cost();
        let energy_before_refund = self.ships[ship_id].energy() + special_cost;
        let Some(body) = self.matter_world.body_state(ship_state.body_id) else {
            return;
        };

        let target_indexes = self.find_point_defense_targets(is_player, body.x, body.y, point_defense_spec.range);
        if target_indexes.is_empty() {
            self.ships[ship_id].set_special_counter(0);
            self.ships[ship_id].set_energy(energy_before_refund);
            return;
        }

        self.audio_events.push(AudioEventSnapshot {
            key: point_defense_spec.sound_key,
        });

        for index in target_indexes.into_iter().rev() {
            let projectile = self.projectiles.remove(index);
            let laser_id = self.next_game_object_id();
            self.lasers.push(LaserSnapshot {
                id: laser_id,
                start_x: body.x,
                start_y: body.y,
                end_x: projectile.x,
                end_y: projectile.y,
                color: 0xffffff,
                width: 3.0,
            });
        }
    }

    fn find_point_defense_targets(&self, is_player: bool, ship_x: f64, ship_y: f64, range: f64) -> Vec<usize> {
        self.projectiles
            .iter()
            .enumerate()
            .filter(|(_, projectile)| projectile.owner_is_player != is_player)
            .filter_map(|(index, projectile)| {
                let dx = shortest_wrapped_delta(ship_x, projectile.x, self.width);
                let dy = shortest_wrapped_delta(ship_y, projectile.y, self.height);
                let distance = ((dx * dx) + (dy * dy)).sqrt();
                (distance <= range).then_some((index, distance))
            })
            .collect::<Vec<_>>()
            .into_iter()
            .map(|(index, _)| index)
            .collect()
    }

    fn projectile_target_for(&self, is_player: bool) -> ProjectileTarget {
        let ship_id = self.ship_state(is_player).ship_id;
        let special_active = self.ship_state(is_player).special_active;
        match self.ships[ship_id].primary_projectile_target_mode_for_state(special_active) {
            ProjectileTargetMode::None => {
                if is_player {
                    if matches!(self.player_weapon_target, ProjectileTarget::TargetShip)
                        && !self.ship_is_targetable(false)
                    {
                        ProjectileTarget::None
                    } else {
                        self.player_weapon_target
                    }
                } else {
                    if matches!(self.target_weapon_target, ProjectileTarget::PlayerShip)
                        && !self.ship_is_targetable(true)
                    {
                        ProjectileTarget::None
                    } else {
                        self.target_weapon_target
                    }
                }
            }
            ProjectileTargetMode::EnemyShip => self.default_enemy_target_for(is_player),
            ProjectileTargetMode::PlayerSelectedOrEnemyShip => {
                if is_player && !matches!(self.player_weapon_target, ProjectileTarget::None) {
                    if matches!(self.player_weapon_target, ProjectileTarget::TargetShip)
                        && !self.ship_is_targetable(false)
                    {
                        ProjectileTarget::None
                    } else {
                        self.player_weapon_target
                    }
                } else if is_player {
                    self.default_enemy_target_for(true)
                } else if !matches!(self.target_weapon_target, ProjectileTarget::None) {
                    if matches!(self.target_weapon_target, ProjectileTarget::PlayerShip)
                        && !self.ship_is_targetable(true)
                    {
                        ProjectileTarget::None
                    } else {
                        self.target_weapon_target
                    }
                } else {
                    self.default_enemy_target_for(false)
                }
            }
            ProjectileTargetMode::PlayerSelectedPointOrForward => {
                if is_player {
                    match self.player_weapon_target {
                        ProjectileTarget::Point { .. } => self.player_weapon_target,
                        _ => ProjectileTarget::None,
                    }
                } else {
                    match self.target_weapon_target {
                        ProjectileTarget::Point { .. } => self.target_weapon_target,
                        _ => ProjectileTarget::None,
                    }
                }
            }
        }
    }

    fn projectile_target_for_mode(&self, is_player: bool, mode: ProjectileTargetMode) -> ProjectileTarget {
        match mode {
            ProjectileTargetMode::None => {
                if is_player {
                    if matches!(self.player_weapon_target, ProjectileTarget::TargetShip)
                        && !self.ship_is_targetable(false)
                    {
                        ProjectileTarget::None
                    } else {
                        self.player_weapon_target
                    }
                } else {
                    if matches!(self.target_weapon_target, ProjectileTarget::PlayerShip)
                        && !self.ship_is_targetable(true)
                    {
                        ProjectileTarget::None
                    } else {
                        self.target_weapon_target
                    }
                }
            }
            ProjectileTargetMode::EnemyShip => self.default_enemy_target_for(is_player),
            ProjectileTargetMode::PlayerSelectedOrEnemyShip => {
                if is_player && !matches!(self.player_weapon_target, ProjectileTarget::None) {
                    if matches!(self.player_weapon_target, ProjectileTarget::TargetShip)
                        && !self.ship_is_targetable(false)
                    {
                        ProjectileTarget::None
                    } else {
                        self.player_weapon_target
                    }
                } else if is_player {
                    self.default_enemy_target_for(true)
                } else if !matches!(self.target_weapon_target, ProjectileTarget::None) {
                    if matches!(self.target_weapon_target, ProjectileTarget::PlayerShip)
                        && !self.ship_is_targetable(true)
                    {
                        ProjectileTarget::None
                    } else {
                        self.target_weapon_target
                    }
                } else {
                    self.default_enemy_target_for(false)
                }
            }
            ProjectileTargetMode::PlayerSelectedPointOrForward => {
                if is_player {
                    match self.player_weapon_target {
                        ProjectileTarget::Point { .. } => self.player_weapon_target,
                        _ => ProjectileTarget::None,
                    }
                } else {
                    match self.target_weapon_target {
                        ProjectileTarget::Point { .. } => self.target_weapon_target,
                        _ => ProjectileTarget::None,
                    }
                }
            }
        }
    }

    fn default_enemy_target_for(&self, is_player: bool) -> ProjectileTarget {
        if !self.ship_is_targetable(!is_player) {
            ProjectileTarget::None
        } else if is_player {
            ProjectileTarget::TargetShip
        } else {
            ProjectileTarget::PlayerShip
        }
    }

    fn spawn_projectile_volley(
        &mut self,
        current: MatterBodyState,
        is_player: bool,
        volley_spec: ProjectileVolleySpec,
    ) {
        for spawn in volley_spec.spawns {
            self.spawn_projectile_from_spec(
                current,
                is_player,
                volley_spec.projectile,
                *spawn,
                self.projectile_target_for_mode(is_player, volley_spec.target_mode),
            );
        }

        if !volley_spec.sound_key.is_empty() {
            self.audio_events.push(AudioEventSnapshot {
                key: volley_spec.sound_key,
            });
        }
    }

    fn spawn_projectile_from_spec(
        &mut self,
        current: MatterBodyState,
        is_player: bool,
        projectile_spec: PrimaryProjectileSpec,
        spawn: ProjectileSpawnSpec,
        target: ProjectileTarget,
    ) {
        let base_facing = self.ships[self.ship_state(is_player).ship_id].facing();
        let base_facing_index = radians_to_facing_index(base_facing);
        let facing_index = (base_facing_index + spawn.facing_offset).rem_euclid(PROJECTILE_FACINGS as i32);
        let facing = facing_index_to_radians(facing_index);
        let (projectile_raw_vx, projectile_raw_vy) =
            projectile_velocity_for_facing(facing_index, projectile_spec.speed as i32);
        let (spawn_rewind_x, spawn_rewind_y) = match projectile_spec.behavior {
            ProjectileBehaviorSpec::WobbleTracking { spawn_rewind_divisor, .. } => (
                projectile_raw_vx as f64 / spawn_rewind_divisor,
                projectile_raw_vy as f64 / spawn_rewind_divisor,
            ),
            ProjectileBehaviorSpec::Tracking => (0.0, 0.0),
        };
        let lateral_facing = facing + std::f64::consts::FRAC_PI_2;
        let projectile_id = self.next_game_object_id();
        self.projectiles.push(ProjectileSnapshot {
            id: projectile_id,
            x: current.x
                + (facing.cos() * spawn.forward_offset)
                + (lateral_facing.cos() * spawn.lateral_offset)
                - spawn_rewind_x,
            y: current.y
                + (facing.sin() * spawn.forward_offset)
                + (lateral_facing.sin() * spawn.lateral_offset)
                - spawn_rewind_y,
            vx: projectile_raw_vx as f64 / 32.0,
            vy: projectile_raw_vy as f64 / 32.0,
            life: projectile_spec.life - 1,
            texture_prefix: projectile_spec.texture_prefix,
            damage: projectile_spec.impact.damage,
            impact_texture_prefix: projectile_spec.impact.texture_prefix,
            impact_start_frame: projectile_spec.impact.start_frame,
            impact_end_frame: projectile_spec.impact.end_frame,
            impact_sound_key: projectile_spec.impact.sound_key,
            track_wait: projectile_spec.turn_wait,
            behavior: projectile_spec.behavior,
            collision: projectile_spec.collision,
            facing,
            acceleration: projectile_spec.acceleration,
            max_speed: projectile_spec.max_speed,
            turn_wait: projectile_spec.turn_wait,
            facing_index,
            speed: projectile_spec.speed as i32,
            raw_vx: projectile_raw_vx,
            raw_vy: projectile_raw_vy,
            velocity_width: 0,
            velocity_height: 0,
            velocity_fract_width: 0,
            velocity_fract_height: 0,
            velocity_error_width: 0,
            velocity_error_height: 0,
            velocity_sign_width: 1,
            velocity_sign_height: 1,
            bubble_rng: if matches!(projectile_spec.behavior, ProjectileBehaviorSpec::WobbleTracking { .. }) {
                next_androsynth_random(&mut self.bubble_rng_state) as u32
            } else {
                0
            },
            owner_is_player: is_player,
            target,
        });
        let projectile = self.projectiles.last_mut().expect("projectile just pushed");
        set_projectile_velocity_components(projectile, projectile_raw_vx, projectile_raw_vy);
    }

    fn step_meteors(&mut self) {
        for meteor in &mut self.meteors {
            meteor.x = wrap_axis(meteor.x + meteor.vx, self.width);
            meteor.y = wrap_axis(meteor.y + meteor.vy, self.height);
            meteor.frame_index = (meteor.frame_index + meteor.spin_step).rem_euclid(meteor.frame_count);
        }
    }

    fn handle_meteor_collisions(&mut self) {
        let mut hit_projectile_indexes = Vec::new();
        let mut player_died = false;
        let mut target_died = false;
        let player_hits: Vec<bool> = self.meteors.iter().map(|meteor| self.meteor_hits_ship(meteor, true)).collect();
        let target_hits: Vec<bool> = self.meteors.iter().map(|meteor| self.meteor_hits_ship(meteor, false)).collect();

        for index in 0..self.meteors.len() {
            let player_hit = player_hits[index];
            let target_hit = target_hits[index];
            let player_contacting = self.meteors[index].player_contacting;
            let target_contacting = self.meteors[index].target_contacting;

            if player_hit && !player_contacting {
                if !self.ship_blocks_damage(true) {
                    player_died |= self.ships[self.player.ship_id].take_damage(METEOR_DAMAGE);
                    let explosion_id = self.next_game_object_id();
                    self.explosions.push(ExplosionSnapshot {
                        id: explosion_id,
                        x: self.meteors[index].x,
                        y: self.meteors[index].y,
                        frame_index: 0,
                        end_frame: 7,
                        texture_prefix: "battle-blast",
                    });
                    self.audio_events.push(AudioEventSnapshot {
                        key: "battle-boom-23",
                    });
                    self.push_ship_from_meteor(true, self.meteors[index].x, self.meteors[index].y);
                }
                self.meteors[index].vx = -self.meteors[index].vx;
                self.meteors[index].vy = -self.meteors[index].vy;
            }

            if target_hit && !target_contacting {
                if !self.ship_blocks_damage(false) {
                    target_died |= self.ships[self.target.ship_id].take_damage(METEOR_DAMAGE);
                    let explosion_id = self.next_game_object_id();
                    self.explosions.push(ExplosionSnapshot {
                        id: explosion_id,
                        x: self.meteors[index].x,
                        y: self.meteors[index].y,
                        frame_index: 0,
                        end_frame: 7,
                        texture_prefix: "battle-blast",
                    });
                    self.audio_events.push(AudioEventSnapshot {
                        key: "battle-boom-23",
                    });
                    self.push_ship_from_meteor(false, self.meteors[index].x, self.meteors[index].y);
                }
                self.meteors[index].vx = -self.meteors[index].vx;
                self.meteors[index].vy = -self.meteors[index].vy;
            }

            self.meteors[index].player_contacting = player_hit;
            self.meteors[index].target_contacting = target_hit;
        }

        let projectile_hits: Vec<(usize, f64, f64, i32, i32, &'static str, &'static str)> = self.projectiles
            .iter()
            .enumerate()
            .filter_map(|(index, projectile)| {
                self.meteors
                    .iter()
                    .any(|meteor| self.projectile_hits_meteor(projectile, meteor))
                    .then_some((
                        index,
                        projectile.x,
                        projectile.y,
                        projectile.impact_start_frame,
                        projectile.impact_end_frame,
                        projectile.impact_texture_prefix,
                        projectile.impact_sound_key,
                    ))
            })
            .collect();

        for (index, x, y, impact_start_frame, impact_end_frame, impact_texture_prefix, impact_sound_key) in projectile_hits {
            let explosion_id = self.next_game_object_id();
                self.explosions.push(ExplosionSnapshot {
                    id: explosion_id,
                    x,
                    y,
                    frame_index: impact_start_frame,
                    end_frame: impact_end_frame,
                    texture_prefix: impact_texture_prefix,
                });
                if !impact_sound_key.is_empty() {
                    self.audio_events.push(AudioEventSnapshot {
                        key: impact_sound_key,
                    });
                }
                hit_projectile_indexes.push(index);
        }

        for index in hit_projectile_indexes.into_iter().rev() {
            self.projectiles.remove(index);
        }

        if player_died {
            self.mark_ship_dead(true);
        }
        if target_died {
            self.mark_ship_dead(false);
        }
    }

    fn push_ship_from_meteor(&mut self, is_player: bool, meteor_x: f64, meteor_y: f64) {
        let ship = self.ship_state(is_player);
        let Some(body) = self.matter_world.body_state(ship.body_id) else {
            return;
        };
        let dx = shortest_wrapped_delta(body.x, meteor_x, self.width);
        let dy = shortest_wrapped_delta(body.y, meteor_y, self.height);
        let distance = ((dx * dx) + (dy * dy)).sqrt();
        if distance <= f64::EPSILON {
            return;
        }
        self.matter_world.add_body_velocity(
            ship.body_id,
            (dx / distance) * METEOR_IMPACT_PUSH,
            (dy / distance) * METEOR_IMPACT_PUSH,
        );
    }

    fn fire_instant_laser(&mut self, current: MatterBodyState, is_player: bool, laser_spec: InstantLaserSpec) {
        let (attacker_ship_id, defender_ship_id, defender_body_id, defender_dead) = if is_player {
            (self.player.ship_id, self.target.ship_id, self.target.body_id, self.target.dead)
        } else {
            (self.target.ship_id, self.player.ship_id, self.player.body_id, self.player.dead)
        };
        let facing = self.ships[attacker_ship_id].facing();
        let start_x = current.x + (facing.cos() * laser_spec.offset);
        let start_y = current.y + (facing.sin() * laser_spec.offset);
        let (aim_end_x, aim_end_y) =
            self.instant_laser_end_for_mode(start_x, start_y, facing, is_player, laser_spec);
        let mut end_x = aim_end_x;
        let mut end_y = aim_end_y;
        let mut ship_hit = false;

        let ship_candidate = if !defender_dead && self.ship_is_targetable(!is_player) {
            self.matter_world.body_state(defender_body_id).and_then(|defender_body| {
                let defender_state = self.ship_state(!is_player);
                let defender_logic = &self.ships[defender_state.ship_id];
                let defender_facing = radians_to_facing_index(defender_logic.facing());
                let defender_hit_polygon = defender_logic.hit_polygon_for_state(
                    defender_facing,
                    defender_body.x,
                    defender_body.y,
                    defender_state.special_active,
                );
                segment_hits_polygon(start_x, start_y, aim_end_x, aim_end_y, &defender_hit_polygon, 0.0)
                    .then_some((defender_body.x, defender_body.y, segment_distance_squared_to_point(start_x, start_y, defender_body.x, defender_body.y)))
            })
        } else {
            None
        };

        let meteor_candidate = self
            .meteors
            .iter()
            .filter(|meteor| point_to_segment_distance_squared(meteor.x, meteor.y, start_x, start_y, aim_end_x, aim_end_y) <= meteor.radius * meteor.radius)
            .map(|meteor| {
                (
                    meteor.x,
                    meteor.y,
                    segment_distance_squared_to_point(start_x, start_y, meteor.x, meteor.y),
                )
            })
            .min_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));

        match (ship_candidate, meteor_candidate) {
            (Some(ship), Some(meteor)) => {
                if ship.2 <= meteor.2 {
                    end_x = ship.0;
                    end_y = ship.1;
                    ship_hit = true;
                } else {
                    end_x = meteor.0;
                    end_y = meteor.1;
                }
            }
            (Some(ship), None) => {
                end_x = ship.0;
                end_y = ship.1;
                ship_hit = true;
            }
            (None, Some(meteor)) => {
                end_x = meteor.0;
                end_y = meteor.1;
            }
            (None, None) => {}
        }

        let laser_id = self.next_game_object_id();
        self.lasers.push(LaserSnapshot {
            id: laser_id,
            start_x,
            start_y,
            end_x,
            end_y,
            color: laser_spec.color,
            width: laser_spec.width,
        });
        if !laser_spec.sound_key.is_empty() {
            self.audio_events.push(AudioEventSnapshot {
                key: laser_spec.sound_key,
            });
        }

        if !ship_hit {
            return;
        }

        if self.ship_blocks_damage(!is_player) {
            return;
        }

        let died = self.ships[defender_ship_id].take_damage(laser_spec.damage);
        let impact_x = end_x;
        let impact_y = end_y;
        let explosion_id = self.next_game_object_id();
        self.explosions.push(ExplosionSnapshot {
            id: explosion_id,
            x: impact_x,
            y: impact_y,
            frame_index: SHIP_DEATH_EXPLOSION_START_FRAME,
            end_frame: SHIP_DEATH_EXPLOSION_END_FRAME,
            texture_prefix: EXPLOSION_TEXTURE_BATTLE_BOOM,
        });
        if !laser_spec.impact_sound_key.is_empty() {
            self.audio_events.push(AudioEventSnapshot {
                key: laser_spec.impact_sound_key,
            });
        }
        if died {
            self.mark_ship_dead(!is_player);
            if let Some(key) = self.ships[attacker_ship_id].victory_sound_key() {
                self.audio_events.push(AudioEventSnapshot { key });
            }
        }
    }

    fn instant_laser_end_for_mode(
        &self,
        start_x: f64,
        start_y: f64,
        facing: f64,
        is_player: bool,
        laser_spec: InstantLaserSpec,
    ) -> (f64, f64) {
        let default_end = (
            wrap_axis(start_x + (facing.cos() * laser_spec.range), self.width),
            wrap_axis(start_y + (facing.sin() * laser_spec.range), self.height),
        );

        match laser_spec.target_mode {
            ProjectileTargetMode::None => default_end,
            ProjectileTargetMode::EnemyShip => {
                let target = self.default_enemy_target_for(is_player);
                projectile_target_position(target, self.matter_world.body_state(self.player.body_id), self.matter_world.body_state(self.target.body_id))
                    .and_then(|(x, y)| point_along_range(start_x, start_y, x, y, laser_spec.range, self.width, self.height))
                    .unwrap_or(default_end)
            }
            ProjectileTargetMode::PlayerSelectedOrEnemyShip => {
                let target = self.projectile_target_for_mode(is_player, ProjectileTargetMode::PlayerSelectedOrEnemyShip);
                projectile_target_position(target, self.matter_world.body_state(self.player.body_id), self.matter_world.body_state(self.target.body_id))
                    .and_then(|(x, y)| point_along_range(start_x, start_y, x, y, laser_spec.range, self.width, self.height))
                    .unwrap_or(default_end)
            }
            ProjectileTargetMode::PlayerSelectedPointOrForward => {
                if is_player {
                    projectile_target_position(
                        self.player_weapon_target,
                        self.matter_world.body_state(self.player.body_id),
                        self.matter_world.body_state(self.target.body_id),
                    )
                    .and_then(|(x, y)| point_along_range(start_x, start_y, x, y, laser_spec.range, self.width, self.height))
                    .unwrap_or(default_end)
                } else {
                    projectile_target_position(
                        self.target_weapon_target,
                        self.matter_world.body_state(self.player.body_id),
                        self.matter_world.body_state(self.target.body_id),
                    )
                    .and_then(|(x, y)| point_along_range(start_x, start_y, x, y, laser_spec.range, self.width, self.height))
                    .unwrap_or(default_end)
                }
            }
        }
    }

    fn activate_teleport_special(&mut self, is_player: bool, body_id: usize, spec: TeleportSpecialSpec) {
        let Some(body) = self.matter_world.body_state(body_id) else {
            return;
        };
        let old_x = body.x;
        let old_y = body.y;
        let (new_x, new_y) = match self.special_target_for(is_player) {
            SpecialTarget::Point { x, y } => (wrap_axis(x, self.width), wrap_axis(y, self.height)),
            SpecialTarget::None => (
                wrap_axis(body.x + (self.width * 0.33), self.width),
                wrap_axis(body.y + (self.height * 0.27), self.height),
            ),
        };
        let old_explosion_id = self.next_game_object_id();
        self.explosions.push(ExplosionSnapshot {
            id: old_explosion_id,
            x: old_x,
            y: old_y,
            frame_index: 0,
            end_frame: spec.end_frame,
            texture_prefix: spec.effect_texture_prefix,
        });
        self.matter_world.set_body_position(body_id, new_x, new_y);
        let new_explosion_id = self.next_game_object_id();
        self.explosions.push(ExplosionSnapshot {
            id: new_explosion_id,
            x: new_x,
            y: new_y,
            frame_index: 0,
            end_frame: spec.end_frame,
            texture_prefix: spec.effect_texture_prefix,
        });
        if !spec.sound_key.is_empty() {
            self.audio_events.push(AudioEventSnapshot {
                key: spec.sound_key,
            });
        }
    }

    fn special_target_for(&self, is_player: bool) -> SpecialTarget {
        if is_player {
            self.player_special_target
        } else {
            self.target_special_target
        }
    }

    fn activate_sound_only_special(&mut self, spec: SoundOnlySpec) {
        if !spec.sound_key.is_empty() {
            self.audio_events.push(AudioEventSnapshot { key: spec.sound_key });
        }
    }

    fn activate_cloak_special(&mut self, is_player: bool, spec: crate::traits::ship_trait::CloakSpec) {
        if !spec.sound_key.is_empty() {
            self.audio_events.push(AudioEventSnapshot { key: spec.sound_key });
        }
        let ship_state = self.ship_state_mut(is_player);
        ship_state.special_active = !ship_state.special_active;
    }

    fn activate_transform_special(&mut self, is_player: bool, spec: TransformSpec) {
        if !spec.sound_key.is_empty() {
            self.audio_events.push(AudioEventSnapshot { key: spec.sound_key });
        }
        let ship_state = self.ship_state_mut(is_player);
        ship_state.special_active = !ship_state.special_active;
    }

    fn activate_crew_drain_special(&mut self, is_player: bool, spec: CrewDrainTransferSpec) {
        let defender_is_player = !is_player;
        if self.ship_state(defender_is_player).dead {
            return;
        }
        let attacker_body = match self.matter_world.body_state(self.ship_state(is_player).body_id) {
            Some(body) => body,
            None => return,
        };
        let defender_body = match self.matter_world.body_state(self.ship_state(defender_is_player).body_id) {
            Some(body) => body,
            None => return,
        };
        let dx = shortest_wrapped_delta(attacker_body.x, defender_body.x, self.width);
        let dy = shortest_wrapped_delta(attacker_body.y, defender_body.y, self.height);
        let distance = ((dx * dx) + (dy * dy)).sqrt();
        if distance > spec.range {
            return;
        }

        let attacker_ship_id = self.ship_state(is_player).ship_id;
        let defender_ship_id = self.ship_state(defender_is_player).ship_id;
        let available = (self.ships[defender_ship_id].crew() - 1).max(0);
        let capacity = self.ships[attacker_ship_id].max_crew() - self.ships[attacker_ship_id].crew();
        let transfer = spec.max_transfer.min(available).min(capacity);
        if transfer <= 0 {
            return;
        }

        let defender_next_crew = self.ships[defender_ship_id].crew() - transfer;
        let attacker_next_crew = self.ships[attacker_ship_id].crew() + transfer;
        self.ships[defender_ship_id].set_crew(defender_next_crew);
        self.ships[attacker_ship_id].set_crew(attacker_next_crew);
        if !spec.sound_key.is_empty() {
            self.audio_events.push(AudioEventSnapshot { key: spec.sound_key });
        }
    }

    fn activate_planet_harvest_special(
        &mut self,
        current: MatterBodyState,
        ship_id: usize,
        spec: PlanetHarvestSpec,
    ) {
        let dx = shortest_wrapped_delta(current.x, self.planet_x, self.width);
        let dy = shortest_wrapped_delta(current.y, self.planet_y, self.height);
        let distance = ((dx * dx) + (dy * dy)).sqrt();
        if distance > spec.range {
            return;
        }

        let next_energy = (self.ships[ship_id].energy() + spec.energy_gain).min(self.ships[ship_id].max_energy());
        if next_energy == self.ships[ship_id].energy() {
            return;
        }

        self.ships[ship_id].set_energy(next_energy);
        if !spec.sound_key.is_empty() {
            self.audio_events.push(AudioEventSnapshot { key: spec.sound_key });
        }
    }

    fn activate_crew_to_energy_special(
        &mut self,
        current: MatterBodyState,
        ship_id: usize,
        spec: CrewToEnergySpec,
        commands: &mut Vec<PhysicsCommand>,
    ) {
        if self.ships[ship_id].crew() <= spec.crew_cost {
            return;
        }

        if !spec.sound_key.is_empty() {
            self.audio_events.push(AudioEventSnapshot { key: spec.sound_key });
        }

        let next_crew = self.ships[ship_id].crew() - spec.crew_cost;
        let next_energy = (self.ships[ship_id].energy() + spec.energy_gain).min(self.ships[ship_id].max_energy());
        self.ships[ship_id].set_crew(next_crew);
        self.ships[ship_id].set_energy(next_energy);

        if spec.recoil_speed > 0.0 {
            let recoil_angle = current.angle + std::f64::consts::PI - std::f64::consts::FRAC_PI_2;
            commands.push(PhysicsCommand::AddVelocity {
                vx: recoil_angle.cos() * spec.recoil_speed,
                vy: recoil_angle.sin() * spec.recoil_speed,
            });
        }
    }

    fn activate_self_destruct_special(
        &mut self,
        current: MatterBodyState,
        is_player: bool,
        spec: SelfDestructSpec,
    ) {
        if !spec.sound_key.is_empty() {
            self.audio_events.push(AudioEventSnapshot { key: spec.sound_key });
        }

        let explosion_id = self.next_game_object_id();
        self.explosions.push(ExplosionSnapshot {
            id: explosion_id,
            x: current.x,
            y: current.y,
            frame_index: 0,
            end_frame: spec.end_frame,
            texture_prefix: spec.texture_prefix,
        });

        let defender_is_player = !is_player;
        if !self.ship_blocks_damage(defender_is_player) {
            let defender_state = self.ship_state(defender_is_player);
            if !defender_state.dead {
                if let Some(defender_body) = self.matter_world.body_state(defender_state.body_id) {
                    let dx = shortest_wrapped_delta(defender_body.x, current.x, self.width);
                    let dy = shortest_wrapped_delta(defender_body.y, current.y, self.height);
                    let distance = ((dx * dx) + (dy * dy)).sqrt();
                    if distance <= spec.radius {
                        let defender_ship_id = defender_state.ship_id;
                        if self.ships[defender_ship_id].take_damage(spec.damage) {
                            self.mark_ship_dead(defender_is_player);
                        }
                    }
                }
            }
        }

        self.remove_ship_without_death_effect(is_player);
    }

    fn remove_ship_without_death_effect(&mut self, is_player: bool) {
        let (ship_id, body_id) = {
            let ship_state = self.ship_state(is_player);
            (ship_state.ship_id, ship_state.body_id)
        };
        self.ships[ship_id].set_crew(0);
        let ship_state = self.ship_state_mut(is_player);
        ship_state.dead = true;
        ship_state.thrusting = false;
        self.matter_world.disable_body(body_id);
    }

    fn ship_blocks_damage(&self, is_player: bool) -> bool {
        let ship_state = self.ship_state(is_player);
        if !ship_state.special_active {
            return false;
        }

        matches!(self.ships[ship_state.ship_id].special_ability_spec(), SpecialAbilitySpec::Shield(_))
    }

    fn projectile_hit_target(&self, projectile: &ProjectileSnapshot) -> Option<bool> {
        let (is_player, ship_state) = if projectile.owner_is_player {
            (false, &self.target)
        } else {
            (true, &self.player)
        };
        let body = self.matter_world.body_state(ship_state.body_id)?;
        let logic = &self.ships[ship_state.ship_id];
        if !logic.is_targetable(ship_state.special_active) {
            return None;
        }
        let facing = radians_to_facing_index(logic.facing());
        let hit_polygon = logic.hit_polygon_for_state(facing, body.x, body.y, ship_state.special_active);
        if !hit_polygon.is_empty() {
            let projectile_polygon =
                projectile_hit_polygon(projectile.collision, projectile.facing_index, projectile.x, projectile.y);
            if !projectile_polygon.is_empty() {
                return polygons_intersect(&projectile_polygon, &hit_polygon).then_some(is_player);
            }
            return point_in_polygon(projectile.x, projectile.y, &hit_polygon).then_some(is_player);
        }

        None
    }

    fn snapshot_for(&self, ship: &BattleShipState) -> BattleShipSnapshot {
        let body = self
            .matter_world
            .body_state(ship.body_id)
            .unwrap_or(MatterBodyState {
                id: ship.body_id,
                x: -10000.0,
                y: -10000.0,
                vx: 0.0,
                vy: 0.0,
                angle: 0.0,
            });
        let logic = &self.ships[ship.ship_id];

        BattleShipSnapshot {
            id: ship.game_object_id(),
            x: body.x,
            y: body.y,
            vx: body.vx,
            vy: body.vy,
            crew: logic.crew(),
            energy: logic.energy(),
            facing: logic.facing(),
            thrusting: ship.thrusting,
            dead: ship.dead,
            cloaked: logic.is_cloaked(ship.special_active),
            texture_prefix: logic.active_texture_prefix(ship.special_active),
        }
    }

    fn ship_is_targetable(&self, is_player: bool) -> bool {
        let ship = self.ship_state(is_player);
        !ship.dead && self.ships[ship.ship_id].is_targetable(ship.special_active)
    }

    fn meteor_hits_ship(&self, meteor: &MeteorSnapshot, is_player: bool) -> bool {
        let ship = self.ship_state(is_player);
        if ship.dead {
            return false;
        }
        let Some(body) = self.matter_world.body_state(ship.body_id) else {
            return false;
        };
        let logic = &self.ships[ship.ship_id];
        let facing = radians_to_facing_index(logic.facing());
        let polygon = logic.hit_polygon_for_state(facing, body.x, body.y, ship.special_active);
        polygon_intersects_circle(&polygon, meteor.x, meteor.y, meteor.radius)
    }

    fn projectile_hits_meteor(&self, projectile: &ProjectileSnapshot, meteor: &MeteorSnapshot) -> bool {
        let projectile_polygon =
            projectile_hit_polygon(projectile.collision, projectile.facing_index, projectile.x, projectile.y);
        if projectile_polygon.is_empty() {
            let dx = shortest_wrapped_delta(projectile.x, meteor.x, self.width);
            let dy = shortest_wrapped_delta(projectile.y, meteor.y, self.height);
            return ((dx * dx) + (dy * dy)).sqrt() <= meteor.radius;
        }

        let wrapped_polygon: Vec<HitPolygonPoint> = projectile_polygon
            .into_iter()
            .map(|point| HitPolygonPoint {
                x: meteor.x + shortest_wrapped_delta(meteor.x, point.x, self.width),
                y: meteor.y + shortest_wrapped_delta(meteor.y, point.y, self.height),
            })
            .collect();
        polygon_intersects_circle(&wrapped_polygon, meteor.x, meteor.y, meteor.radius)
    }

    fn androsynth_blazer_hits_other_ship(&self, is_player: bool) -> bool {
        let Some(blazer_spec) = self.blazer_spec_for(is_player) else {
            return false;
        };
        let (attacker, defender) = if is_player {
            (&self.player, &self.target)
        } else {
            (&self.target, &self.player)
        };

        let Some(attacker_body) = self.matter_world.body_state(attacker.body_id) else {
            return false;
        };
        let Some(defender_body) = self.matter_world.body_state(defender.body_id) else {
            return false;
        };

        let defender_logic = &self.ships[defender.ship_id];
        let defender_facing = radians_to_facing_index(defender_logic.facing());
        let defender_hit_polygon = defender_logic.hit_polygon_for_state(
            defender_facing,
            defender_body.x,
            defender_body.y,
            defender.special_active,
        );
        let start_x = attacker.previous_x;
        let start_y = attacker.previous_y;
        let end_x = attacker_body.x;
        let end_y = attacker_body.y;
        if !defender_hit_polygon.is_empty() {
            let wrapped_end_x = start_x + shortest_wrapped_delta(start_x, end_x, self.width);
            let wrapped_end_y = start_y + shortest_wrapped_delta(start_y, end_y, self.height);
            let wrapped_polygon: Vec<HitPolygonPoint> = defender_hit_polygon
                .into_iter()
                .map(|point| HitPolygonPoint {
                    x: start_x + shortest_wrapped_delta(start_x, point.x, self.width),
                    y: start_y + shortest_wrapped_delta(start_y, point.y, self.height),
                })
                .collect();

            return segment_hits_polygon(
                start_x,
                start_y,
                wrapped_end_x,
                wrapped_end_y,
                &wrapped_polygon,
                blazer_spec.hit_radius,
            );
        }

        false
    }

    fn ship_state(&self, is_player: bool) -> &BattleShipState {
        if is_player {
            &self.player
        } else {
            &self.target
        }
    }

    fn ship_state_mut(&mut self, is_player: bool) -> &mut BattleShipState {
        if is_player {
            &mut self.player
        } else {
            &mut self.target
        }
    }

    fn apply_blazer_collision_velocity(
        &mut self,
        blazer_mass: f64,
        blazer_body_id: usize,
        victim_body_id: usize,
        blazer_before: &MatterBodyState,
        victim_before: &MatterBodyState,
        victim_mass: f64,
    ) {
        let dx = victim_before.x - blazer_before.x;
        let dy = victim_before.y - blazer_before.y;
        let distance = (dx * dx + dy * dy).sqrt();
        let (nx, ny) = if distance <= f64::EPSILON {
            (1.0, 0.0)
        } else {
            (dx / distance, dy / distance)
        };
        let blazer_speed = (blazer_before.vx.powi(2) + blazer_before.vy.powi(2)).sqrt();
        let ((blazer_vx, blazer_vy), (victim_vx, victim_vy)) = resolve_collision_velocity(
            blazer_before.x,
            blazer_before.y,
            nx * blazer_speed,
            ny * blazer_speed,
            blazer_mass,
            victim_before.x,
            victim_before.y,
            victim_before.vx,
            victim_before.vy,
            victim_mass,
        );
        self.matter_world.set_body_velocity(blazer_body_id, blazer_vx, blazer_vy);
        self.matter_world.set_body_velocity(victim_body_id, victim_vx, victim_vy);
    }

    fn blazer_spec_for(&self, is_player: bool) -> Option<crate::traits::ship_trait::BlazerSpecialSpec> {
        let ship_id = self.ship_state(is_player).ship_id;
        match self.ships[ship_id].special_ability_spec() {
            SpecialAbilitySpec::Blazer(spec) => Some(spec),
            SpecialAbilitySpec::None
            | SpecialAbilitySpec::PointDefense(_)
            | SpecialAbilitySpec::Shield(_)
            | SpecialAbilitySpec::Teleport(_)
            | SpecialAbilitySpec::InstantLaser(_)
            | SpecialAbilitySpec::DirectionalThrust(_)
            | SpecialAbilitySpec::Projectile(_)
            | SpecialAbilitySpec::CrewRegeneration(_)
            | SpecialAbilitySpec::CrewToEnergy(_)
            | SpecialAbilitySpec::SelfDestruct(_)
            | SpecialAbilitySpec::SoundOnly(_)
            | SpecialAbilitySpec::Cloak(_)
            | SpecialAbilitySpec::Transform(_)
            | SpecialAbilitySpec::CrewDrainTransfer(_)
            | SpecialAbilitySpec::PlanetHarvest(_) => None,
        }
    }

    fn next_game_object_id(&mut self) -> u64 {
        let id = self.next_game_object_id;
        self.next_game_object_id += 1;
        id
    }

    fn mark_ship_dead(&mut self, is_player: bool) {
        let (ship_dead, body_id) = if is_player {
            (self.player.dead, self.player.body_id)
        } else {
            (self.target.dead, self.target.body_id)
        };

        if ship_dead {
            return;
        }

        if let Some(body) = self.matter_world.body_state(body_id) {
            let explosion_id = self.next_game_object_id();
            self.explosions.push(ExplosionSnapshot {
                id: explosion_id,
                x: body.x,
                y: body.y,
                frame_index: SHIP_DEATH_EXPLOSION_START_FRAME,
                end_frame: SHIP_DEATH_EXPLOSION_END_FRAME,
                texture_prefix: EXPLOSION_TEXTURE_BATTLE_BOOM,
            });
        }
        self.audio_events.push(AudioEventSnapshot {
            key: AUDIO_SHIP_DEATH,
        });

        let ship = if is_player {
            &mut self.player
        } else {
            &mut self.target
        };
        ship.dead = true;
        ship.thrusting = false;
        self.matter_world.disable_body(body_id);
    }
}

fn projectile_hit_polygon(
    collision: ProjectileCollisionSpec,
    facing: i32,
    center_x: f64,
    center_y: f64,
) -> Vec<HitPolygonPoint> {
    let base_polygon = match collision {
        ProjectileCollisionSpec::None => return Vec::new(),
        ProjectileCollisionSpec::Polygon(points) => points,
    };
    rotate_polygon_points(base_polygon, facing, center_x, center_y)
}

impl GameObject for BattleShipState {
    fn game_object_id(&self) -> u64 {
        self.id
    }
}

impl GameObject for ProjectileSnapshot {
    fn game_object_id(&self) -> u64 {
        self.id
    }
}

impl GameObject for ExplosionSnapshot {
    fn game_object_id(&self) -> u64 {
        self.id
    }
}

impl GameObject for MeteorSnapshot {
    fn game_object_id(&self) -> u64 {
        self.id
    }
}

impl GameObject for LaserSnapshot {
    fn game_object_id(&self) -> u64 {
        self.id
    }
}

fn rotate_polygon_points(
    base_polygon: &[HitPolygonPoint],
    facing: i32,
    center_x: f64,
    center_y: f64,
) -> Vec<HitPolygonPoint> {
    let rotation = (facing.rem_euclid(16) as f64) * ((2.0 * std::f64::consts::PI) / 16.0);
    base_polygon
        .iter()
        .map(|point| HitPolygonPoint {
            x: center_x + ((point.x * rotation.cos()) - (point.y * rotation.sin())),
            y: center_y + ((point.x * rotation.sin()) + (point.y * rotation.cos())),
        })
        .collect()
}

fn create_ship_body_for(
    matter_world: &mut MatterWorld,
    ship: &AnyShip,
    x: f64,
    y: f64,
    special_active: bool,
) -> usize {
    let polygon = ship.hit_polygon_for_state(0, 0.0, 0.0, special_active);
    if polygon.is_empty() {
        return matter_world.create_ship_body(x, y, ship.size(), ship.mass(), 0.8);
    }

    let vertices = polygon
        .iter()
        .map(|point| Vec2 { x: point.x, y: point.y })
        .collect::<Vec<_>>();
    matter_world.create_ship_polygon_body(
        x,
        y,
        &vertices,
        ship.mass(),
        0.8,
        ship_body_angle(ship),
    )
}

fn sync_ship_body_angle(matter_world: &mut MatterWorld, body_id: usize, ship: &AnyShip) {
    matter_world.set_body_angle(body_id, ship_body_angle(ship));
}

fn ship_body_angle(ship: &AnyShip) -> f64 {
    ship.facing() + std::f64::consts::FRAC_PI_2
}

fn apply_commands(matter_world: &mut MatterWorld, body_id: usize, commands: Vec<PhysicsCommand>) -> bool {
    let mut thrusting = false;
    const THRUST_VELOCITY_EPSILON: f64 = 0.001;

    for command in commands {
        match command {
            PhysicsCommand::SetVelocity { vx, vy } => {
                if vx.abs() > THRUST_VELOCITY_EPSILON || vy.abs() > THRUST_VELOCITY_EPSILON {
                    thrusting = true;
                }
                matter_world.set_body_velocity(body_id, vx, vy);
            }
            PhysicsCommand::AddVelocity { vx, vy } => {
                matter_world.add_body_velocity(body_id, vx, vy);
            }
        }
    }

    thrusting
}

fn turn_projectile_toward_target(
    facing: i32,
    projectile_x: f64,
    projectile_y: f64,
    target_x: f64,
    target_y: f64,
) -> i32 {
    let current = facing;
    let desired = radians_to_facing_index((target_y - projectile_y).atan2(target_x - projectile_x));
    let delta = (desired - current).rem_euclid(PROJECTILE_FACINGS as i32);

    if delta == 0 {
        return facing;
    }

    if delta < (PROJECTILE_FACINGS as i32 / 2) {
        current + 1
    } else {
        current - 1
    }
    .rem_euclid(PROJECTILE_FACINGS as i32)
}

fn radians_to_facing_index(facing: f64) -> i32 {
    let angle = (facing + std::f64::consts::FRAC_PI_2).rem_euclid(std::f64::consts::TAU);
    (angle / (std::f64::consts::TAU / PROJECTILE_FACINGS)).round() as i32 % PROJECTILE_FACINGS as i32
}

fn facing_index_to_radians(facing: i32) -> f64 {
    -std::f64::consts::FRAC_PI_2 + (facing as f64 * (std::f64::consts::TAU / PROJECTILE_FACINGS))
}

fn projectile_velocity_for_facing(facing: i32, speed: i32) -> (i32, i32) {
    let angle = ((facing.rem_euclid(PROJECTILE_FACINGS as i32)) * 4) as usize;
    let magnitude = speed << 5;
    let vx = ((SC2_SINE_TABLE[(angle + 16) & 63] as i64) * magnitude as i64) >> 14;
    let vy = ((SC2_SINE_TABLE[angle & 63] as i64) * magnitude as i64) >> 14;
    (vx as i32, vy as i32)
}

fn set_projectile_velocity_components(projectile: &mut ProjectileSnapshot, raw_vx: i32, raw_vy: i32) {
    projectile.raw_vx = raw_vx;
    projectile.raw_vy = raw_vy;
    projectile.vx = raw_vx as f64 / 32.0;
    projectile.vy = raw_vy as f64 / 32.0;

    let (width, fract, sign) = split_sc2_velocity_component(raw_vx);
    projectile.velocity_width = width;
    projectile.velocity_fract_width = fract;
    projectile.velocity_error_width = 0;
    projectile.velocity_sign_width = sign;

    let (height, fract, sign) = split_sc2_velocity_component(raw_vy);
    projectile.velocity_height = height;
    projectile.velocity_fract_height = fract;
    projectile.velocity_error_height = 0;
    projectile.velocity_sign_height = sign;
}

fn split_sc2_velocity_component(raw: i32) -> (i32, i32, i32) {
    if raw >= 0 {
        (raw >> 5, raw & 31, 1)
    } else {
        let abs = -raw;
        (-(abs >> 5), abs & 31, -1)
    }
}

fn advance_projectile_position(projectile: &mut ProjectileSnapshot) {
    let dx = next_sc2_velocity_step(
        projectile.velocity_width,
        projectile.velocity_fract_width,
        &mut projectile.velocity_error_width,
        projectile.velocity_sign_width,
    );
    let dy = next_sc2_velocity_step(
        projectile.velocity_height,
        projectile.velocity_fract_height,
        &mut projectile.velocity_error_height,
        projectile.velocity_sign_height,
    );
    projectile.x += dx as f64;
    projectile.y += dy as f64;
}

fn next_sc2_velocity_step(vector: i32, fract: i32, error: &mut i32, sign: i32) -> i32 {
    let e = *error + fract;
    let step = vector + (sign * (e >> 5));
    *error = e & 31;
    step
}

fn step_wobble_tracking_projectile(
    projectile: &mut ProjectileSnapshot,
    player_target_body: Option<MatterBodyState>,
    target_target_body: Option<MatterBodyState>,
) {
    let ProjectileBehaviorSpec::WobbleTracking {
        direct_track_range,
        ..
    } = projectile.behavior else {
        return;
    };
    let mut thrust_wait = (projectile.turn_wait >> 8) & 0xff;
    let mut turn_wait = projectile.turn_wait & 0xff;

    if thrust_wait > 0 {
        thrust_wait -= 1;
    } else {
        thrust_wait = next_androsynth_random(&mut projectile.bubble_rng) & 3;
    }

    if turn_wait > 0 {
        turn_wait -= 1;
    } else if let Some((target_x, target_y)) = projectile_target_position(
        projectile.target,
        player_target_body,
        target_target_body,
    ) {
        let target_dx = target_x - projectile.x;
        let target_dy = target_y - projectile.y;
        let target_distance = ((target_dx * target_dx) + (target_dy * target_dy)).sqrt();
        let current_facing = projectile.facing_index;
        let desired_facing = vector_to_facing_index(target_dx, target_dy);
        let delta_facing = (desired_facing - current_facing).rem_euclid(PROJECTILE_FACINGS as i32);
        let random_turn = next_androsynth_random(&mut projectile.bubble_rng) & 7;

        projectile.facing_index = if target_distance <= direct_track_range {
            desired_facing
        } else if delta_facing <= 8 {
            (current_facing + random_turn).rem_euclid(PROJECTILE_FACINGS as i32)
        } else {
            (current_facing - random_turn).rem_euclid(PROJECTILE_FACINGS as i32)
        };
        projectile.facing = facing_index_to_radians(projectile.facing_index);
        turn_wait = projectile.track_wait;
    }

    projectile.turn_wait = (thrust_wait << 8) | turn_wait;
    let (raw_vx, raw_vy) = projectile_velocity_for_facing(projectile.facing_index, projectile.speed);
    set_projectile_velocity_components(projectile, raw_vx, raw_vy);
    advance_projectile_position(projectile);
    projectile.life -= 1;
}

fn projectile_target_position(
    target: ProjectileTarget,
    player_target_body: Option<MatterBodyState>,
    target_target_body: Option<MatterBodyState>,
) -> Option<(f64, f64)> {
    match target {
        ProjectileTarget::None => None,
        ProjectileTarget::Point { x, y } => Some((x, y)),
        ProjectileTarget::PlayerShip => player_target_body.map(|body| (body.x, body.y)),
        ProjectileTarget::TargetShip => target_target_body.map(|body| (body.x, body.y)),
    }
}

fn next_androsynth_random(state: &mut u32) -> i32 {
    *state = state.wrapping_mul(1103515245).wrapping_add(12345);
    ((*state >> 16) & 0x7fff) as i32
}

fn vector_to_facing_index(dx: f64, dy: f64) -> i32 {
    let mut best_facing = 0;
    let mut best_dot = f64::NEG_INFINITY;

    for facing in 0..PROJECTILE_FACINGS as i32 {
        let (vx, vy) = projectile_velocity_for_facing(facing, 1);
        let dot = (vx as f64 * dx) + (vy as f64 * dy);
        if dot > best_dot {
            best_dot = dot;
            best_facing = facing;
        }
    }

    best_facing
}

#[cfg(test)]
fn segment_hits_circle(
    start_x: f64,
    start_y: f64,
    end_x: f64,
    end_y: f64,
    circle_x: f64,
    circle_y: f64,
    radius: f64,
) -> bool {
    let dx = end_x - start_x;
    let dy = end_y - start_y;
    let length_squared = (dx * dx) + (dy * dy);

    if length_squared == 0.0 {
        let point_dx = start_x - circle_x;
        let point_dy = start_y - circle_y;
        return (point_dx * point_dx) + (point_dy * point_dy) <= radius * radius;
    }

    let t = (((circle_x - start_x) * dx) + ((circle_y - start_y) * dy)) / length_squared;
    let clamped_t = t.clamp(0.0, 1.0);
    let closest_x = start_x + (dx * clamped_t);
    let closest_y = start_y + (dy * clamped_t);
    let closest_dx = closest_x - circle_x;
    let closest_dy = closest_y - circle_y;

    (closest_dx * closest_dx) + (closest_dy * closest_dy) <= radius * radius
}

fn segment_hits_polygon(
    start_x: f64,
    start_y: f64,
    end_x: f64,
    end_y: f64,
    polygon: &[HitPolygonPoint],
    radius: f64,
) -> bool {
    if polygon.is_empty() {
        return false;
    }

    if point_in_polygon(start_x, start_y, polygon) || point_in_polygon(end_x, end_y, polygon) {
        return true;
    }

    for index in 0..polygon.len() {
        let edge_start = polygon[index];
        let edge_end = polygon[(index + 1) % polygon.len()];
        if segment_to_segment_distance_squared(
            start_x,
            start_y,
            end_x,
            end_y,
            edge_start.x,
            edge_start.y,
            edge_end.x,
            edge_end.y,
        ) <= radius * radius {
            return true;
        }
    }

    false
}

fn polygons_intersect(a: &[HitPolygonPoint], b: &[HitPolygonPoint]) -> bool {
    if a.is_empty() || b.is_empty() {
        return false;
    }

    if a.iter().any(|point| point_in_polygon(point.x, point.y, b)) {
        return true;
    }
    if b.iter().any(|point| point_in_polygon(point.x, point.y, a)) {
        return true;
    }

    for a_index in 0..a.len() {
        let a_start = a[a_index];
        let a_end = a[(a_index + 1) % a.len()];
        for b_index in 0..b.len() {
            let b_start = b[b_index];
            let b_end = b[(b_index + 1) % b.len()];
            if segments_intersect(a_start.x, a_start.y, a_end.x, a_end.y, b_start.x, b_start.y, b_end.x, b_end.y) {
                return true;
            }
        }
    }

    false
}

fn polygon_intersects_circle(
    polygon: &[HitPolygonPoint],
    circle_x: f64,
    circle_y: f64,
    radius: f64,
) -> bool {
    if polygon.is_empty() {
        return false;
    }

    if point_in_polygon(circle_x, circle_y, polygon) {
        return true;
    }

    let radius_sq = radius * radius;
    let mut previous = polygon[polygon.len() - 1];
    for &current in polygon {
        if point_to_segment_distance_squared(circle_x, circle_y, previous.x, previous.y, current.x, current.y) <= radius_sq {
            return true;
        }
        previous = current;
    }

    false
}

fn point_in_polygon(x: f64, y: f64, polygon: &[HitPolygonPoint]) -> bool {
    let mut inside = false;
    let mut previous = polygon[polygon.len() - 1];

    for current in polygon {
        let crosses = ((current.y > y) != (previous.y > y))
            && (x < ((previous.x - current.x) * (y - current.y) / (previous.y - current.y) + current.x));
        if crosses {
            inside = !inside;
        }
        previous = *current;
    }

    inside
}

fn segment_to_segment_distance_squared(
    ax: f64,
    ay: f64,
    bx: f64,
    by: f64,
    cx: f64,
    cy: f64,
    dx: f64,
    dy: f64,
) -> f64 {
    if segments_intersect(ax, ay, bx, by, cx, cy, dx, dy) {
        return 0.0;
    }

    point_to_segment_distance_squared(ax, ay, cx, cy, dx, dy)
        .min(point_to_segment_distance_squared(bx, by, cx, cy, dx, dy))
        .min(point_to_segment_distance_squared(cx, cy, ax, ay, bx, by))
        .min(point_to_segment_distance_squared(dx, dy, ax, ay, bx, by))
}

fn segment_distance_squared_to_point(start_x: f64, start_y: f64, point_x: f64, point_y: f64) -> f64 {
    let dx = point_x - start_x;
    let dy = point_y - start_y;
    (dx * dx) + (dy * dy)
}

fn point_along_range(
    start_x: f64,
    start_y: f64,
    target_x: f64,
    target_y: f64,
    range: f64,
    width: f64,
    height: f64,
) -> Option<(f64, f64)> {
    let dx = shortest_wrapped_delta(start_x, target_x, width);
    let dy = shortest_wrapped_delta(start_y, target_y, height);
    let distance = ((dx * dx) + (dy * dy)).sqrt();
    if distance == 0.0 {
        return None;
    }

    let scale = (range / distance).min(1.0);
    Some((
        wrap_axis(start_x + (dx * scale), width),
        wrap_axis(start_y + (dy * scale), height),
    ))
}

fn point_to_segment_distance_squared(
    px: f64,
    py: f64,
    ax: f64,
    ay: f64,
    bx: f64,
    by: f64,
) -> f64 {
    let dx = bx - ax;
    let dy = by - ay;
    let length_squared = (dx * dx) + (dy * dy);

    if length_squared == 0.0 {
        let point_dx = px - ax;
        let point_dy = py - ay;
        return (point_dx * point_dx) + (point_dy * point_dy);
    }

    let t = (((px - ax) * dx) + ((py - ay) * dy)) / length_squared;
    let clamped_t = t.clamp(0.0, 1.0);
    let closest_x = ax + (dx * clamped_t);
    let closest_y = ay + (dy * clamped_t);
    let closest_dx = px - closest_x;
    let closest_dy = py - closest_y;

    (closest_dx * closest_dx) + (closest_dy * closest_dy)
}

fn segments_intersect(
    ax: f64,
    ay: f64,
    bx: f64,
    by: f64,
    cx: f64,
    cy: f64,
    dx: f64,
    dy: f64,
) -> bool {
    let ab = orientation(ax, ay, bx, by, cx, cy);
    let ac = orientation(ax, ay, bx, by, dx, dy);
    let cd = orientation(cx, cy, dx, dy, ax, ay);
    let ca = orientation(cx, cy, dx, dy, bx, by);

    (ab == 0.0 && on_segment(ax, ay, bx, by, cx, cy))
        || (ac == 0.0 && on_segment(ax, ay, bx, by, dx, dy))
        || (cd == 0.0 && on_segment(cx, cy, dx, dy, ax, ay))
        || (ca == 0.0 && on_segment(cx, cy, dx, dy, bx, by))
        || ((ab > 0.0) != (ac > 0.0) && (cd > 0.0) != (ca > 0.0))
}

fn resolve_collision_velocity(
    player_x: f64,
    player_y: f64,
    player_vx: f64,
    player_vy: f64,
    player_mass: f64,
    target_x: f64,
    target_y: f64,
    target_vx: f64,
    target_vy: f64,
    target_mass: f64,
) -> ((f64, f64), (f64, f64)) {
    let dx = target_x - player_x;
    let dy = target_y - player_y;
    let distance = (dx * dx + dy * dy).sqrt();
    let (nx, ny) = if distance <= f64::EPSILON {
        (1.0, 0.0)
    } else {
        (dx / distance, dy / distance)
    };

    let player_normal = (player_vx * nx) + (player_vy * ny);
    let target_normal = (target_vx * nx) + (target_vy * ny);
    let player_tangent_x = player_vx - (player_normal * nx);
    let player_tangent_y = player_vy - (player_normal * ny);
    let target_tangent_x = target_vx - (target_normal * nx);
    let target_tangent_y = target_vy - (target_normal * ny);

    let player_bounced_normal = (
        (player_normal * (player_mass - target_mass))
            + (2.0 * target_mass * target_normal)
    ) / (player_mass + target_mass);
    let target_bounced_normal = (
        (target_normal * (target_mass - player_mass))
            + (2.0 * player_mass * player_normal)
    ) / (player_mass + target_mass);

    (
        (
            player_tangent_x + (player_bounced_normal * nx),
            player_tangent_y + (player_bounced_normal * ny),
        ),
        (
            target_tangent_x + (target_bounced_normal * nx),
            target_tangent_y + (target_bounced_normal * ny),
        ),
    )
}

fn orientation(ax: f64, ay: f64, bx: f64, by: f64, cx: f64, cy: f64) -> f64 {
    ((by - ay) * (cx - bx)) - ((bx - ax) * (cy - by))
}

fn on_segment(ax: f64, ay: f64, bx: f64, by: f64, px: f64, py: f64) -> bool {
    px >= ax.min(bx) && px <= ax.max(bx) && py >= ay.min(by) && py <= ay.max(by)
}

#[cfg(test)]
mod tests {
    use super::{Battle, ProjectileSnapshot};
    use crate::reference_data;
    use crate::ship_input::ShipInput;
    use crate::ships::{AnyShip, HumanCruiser};
    use crate::traits::ship_trait::Ship;

    #[test]
    fn target_arilou_primary_instant_laser_aims_at_player_selected_point() {
        let mut battle = Battle::new(
            "human-cruiser",
            "arilou-skiff",
            5000.0,
            5000.0,
            7000.0,
            5000.0,
            0.0,
            0.0,
            8000.0,
            8000.0,
        ).expect("battle should build");

        battle.set_target_weapon_target_point(6500.0, 5600.0);
        battle.set_target_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: true,
            special: false,
        });
        battle.tick(1000.0 / 24.0);

        assert_eq!(battle.snapshot().lasers[0].end_y > 5000.0, true);
    }

    #[test]
    fn target_androsynth_bubble_hit_damages_player_for_two_crew() {
        let mut battle = Battle::new(
            "human-cruiser",
            "androsynth-guardian",
            5000.0,
            4300.0,
            5000.0,
            5000.0,
            500.0,
            600.0,
            10000.0,
            10000.0,
        )
        .unwrap();

        battle.set_target_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: true,
            special: false,
        });
        battle.tick(1000.0 / 24.0);

        let player_body = battle
            .matter_world
            .body_state(battle.player.body_id)
            .expect("player body");
        battle.projectiles[0].x = player_body.x;
        battle.projectiles[0].y = player_body.y;

        battle.set_target_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: false,
            special: false,
        });
        battle.tick(1000.0 / 24.0);

        assert_eq!(battle.snapshot().player.crew, 16);
    }

    #[test]
    fn arilou_primary_instant_laser_adds_audio_event() {
        let mut battle = Battle::new(
            "arilou-skiff",
            "human-cruiser",
            5000.0,
            5000.0,
            7000.0,
            5000.0,
            0.0,
            0.0,
            8000.0,
            8000.0,
        ).expect("battle should build");

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: true,
            special: false,
        });
        battle.tick(1000.0 / 24.0);

        assert_eq!(
            battle.snapshot().audio_events.iter().any(|event| event.key == "arilou-primary"),
            true,
        );
    }

    #[test]
    fn arilou_primary_instant_laser_creates_visible_beam_out_of_range() {
        let mut battle = Battle::new(
            "arilou-skiff",
            "human-cruiser",
            5000.0,
            5000.0,
            7000.0,
            5000.0,
            0.0,
            0.0,
            8000.0,
            8000.0,
        ).expect("battle should build");

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: true,
            special: false,
        });
        battle.tick(1000.0 / 24.0);

        assert_eq!(battle.snapshot().lasers.len(), 1);
    }

    #[test]
    fn arilou_primary_instant_laser_aims_at_player_selected_point() {
        let mut battle = Battle::new(
            "arilou-skiff",
            "human-cruiser",
            5000.0,
            5000.0,
            7000.0,
            5000.0,
            0.0,
            0.0,
            8000.0,
            8000.0,
        ).expect("battle should build");

        battle.set_player_weapon_target_point(4200.0, 4200.0);
        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: true,
            special: false,
        });
        battle.tick(1000.0 / 24.0);

        let laser = battle.snapshot().lasers[0];
        assert!(laser.end_x < laser.start_x && laser.end_y < laser.start_y);
    }

    #[test]
    fn arilou_primary_instant_laser_stops_at_meteor_before_target() {
        let mut battle = Battle::new(
            "arilou-skiff",
            "human-cruiser",
            5000.0,
            5000.0,
            5000.0,
            4300.0,
            0.0,
            0.0,
            8000.0,
            8000.0,
        ).expect("battle should build");
        battle.meteors[0].x = 5000.0;
        battle.meteors[0].y = 4600.0;
        battle.meteors[0].vx = 0.0;
        battle.meteors[0].vy = 0.0;
        battle.set_player_weapon_target_point(5000.0, 4200.0);
        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: true,
            special: false,
        });
        battle.tick(1000.0 / 24.0);

        assert_eq!(
            (
                battle.snapshot().lasers[0].end_x.round() as i32,
                battle.snapshot().lasers[0].end_y.round() as i32,
                battle.snapshot().target.crew,
            ),
            (5000, 4600, 18),
        );
    }

    #[test]
    fn arilou_special_teleport_adds_audio_event() {
        let mut battle = Battle::new(
            "arilou-skiff",
            "human-cruiser",
            5000.0,
            5000.0,
            6500.0,
            5000.0,
            0.0,
            0.0,
            8000.0,
            8000.0,
        ).expect("battle should build");

        battle.set_player_special_target_point(1234.0, 2345.0);
        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: false,
            special: true,
        });
        battle.tick(1000.0 / 24.0);

        assert_eq!(
            battle.snapshot().audio_events.iter().any(|event| event.key == "arilou-special"),
            true,
        );
    }

    #[test]
    fn arilou_special_teleports_to_player_selected_point() {
        let mut battle = Battle::new(
            "arilou-skiff",
            "human-cruiser",
            5000.0,
            5000.0,
            6500.0,
            5000.0,
            0.0,
            0.0,
            8000.0,
            8000.0,
        ).expect("battle should build");

        battle.set_player_special_target_point(1234.0, 2345.0);
        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: false,
            special: true,
        });
        battle.tick(1000.0 / 24.0);

        assert_eq!(
            (
                battle.snapshot().player.x.round() as i32,
                battle.snapshot().player.y.round() as i32,
            ),
            (1234, 2345),
        );
    }

    #[test]
    fn arilou_without_thrust_zeroes_existing_velocity() {
        let mut battle = Battle::new(
            "arilou-skiff",
            "human-cruiser",
            5000.0,
            5000.0,
            7000.0,
            5000.0,
            0.0,
            0.0,
            8000.0,
            8000.0,
        ).expect("battle should build");

        battle
            .matter_world
            .set_body_velocity(battle.player.body_id, 3.25, -1.5);
        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: false,
            special: false,
        });
        battle.tick(1000.0 / 24.0);

        assert_eq!((battle.snapshot().player.vx, battle.snapshot().player.vy), (0.0, 0.0));
    }

    #[test]
    fn battle_snapshot_includes_initial_meteors() {
        let battle = Battle::new(
            "human-cruiser",
            "human-cruiser",
            5000.0,
            5000.0,
            7000.0,
            5000.0,
            4000.0,
            4000.0,
            8000.0,
            8000.0,
        ).expect("battle should build");

        assert_eq!(battle.snapshot().meteors.len(), 3);
    }

    #[test]
    fn meteor_collision_damages_ship_once() {
        let mut battle = Battle::new(
            "human-cruiser",
            "human-cruiser",
            5000.0,
            5000.0,
            7000.0,
            5000.0,
            4000.0,
            4000.0,
            8000.0,
            8000.0,
        ).expect("battle should build");
        battle.meteors[0].x = 5000.0;
        battle.meteors[0].y = 5000.0;
        battle.meteors[0].vx = 0.0;
        battle.meteors[0].vy = 0.0;

        battle.tick(1000.0 / 24.0);

        assert_eq!(battle.snapshot().player.crew, 17);
    }

    #[test]
    fn mmrnmhrm_special_transforms_primary_weapon_mode() {
        let mut battle = Battle::new(
            "mmrnmhrm-xform",
            "human-cruiser",
            5000.0,
            5000.0,
            7000.0,
            5000.0,
            0.0,
            0.0,
            8000.0,
            8000.0,
        ).expect("battle should build");

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: false,
            special: true,
        });
        battle.tick(1000.0 / 24.0);
        battle.ships[battle.player.ship_id].set_energy(10);

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: true,
            special: false,
        });
        battle.tick(1000.0 / 24.0);

        assert_eq!(
            battle.snapshot().projectiles.first().map(|projectile| projectile.texture_prefix),
            Some("mmrnmhrm-torpedo"),
        );
    }

    #[test]
    fn ilwrath_cloak_prevents_enemy_projectiles_hitting() {
        let mut battle = Battle::new(
            "ilwrath-avenger",
            "human-cruiser",
            5000.0,
            5000.0,
            5300.0,
            5000.0,
            0.0,
            0.0,
            8000.0,
            8000.0,
        ).expect("battle should build");

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: false,
            special: true,
        });
        battle.tick(1000.0 / 24.0);
        battle.trigger_target_weapon();
        for _ in 0..60 {
            battle.tick(1000.0 / 24.0);
        }

        assert_eq!(battle.snapshot().player.crew, 22);
    }

    #[test]
    fn syreen_special_transfers_enemy_crew_to_player() {
        let mut battle = Battle::new(
            "syreen-penetrator",
            "human-cruiser",
            5000.0,
            5000.0,
            5150.0,
            5000.0,
            0.0,
            0.0,
            8000.0,
            8000.0,
        ).expect("battle should build");
        battle.ships[battle.player.ship_id].set_crew(4);

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: false,
            special: true,
        });
        battle.tick(1000.0 / 24.0);

        assert_eq!(
            (battle.snapshot().player.crew, battle.snapshot().target.crew),
            (12, 10),
        );
    }

    #[test]
    fn slylandro_special_harvests_energy_near_planet() {
        let mut battle = Battle::new(
            "slylandro-probe",
            "human-cruiser",
            4100.0,
            4000.0,
            7000.0,
            5000.0,
            4000.0,
            4000.0,
            8000.0,
            8000.0,
        ).expect("battle should build");
        battle.ships[battle.player.ship_id].set_energy(0);

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: false,
            special: true,
        });
        battle.tick(1000.0 / 24.0);

        assert_eq!(battle.snapshot().player.energy, 6);
    }

    #[test]
    fn druuge_special_converts_crew_into_energy() {
        let mut battle = Battle::new(
            "druuge-mauler",
            "human-cruiser",
            5000.0,
            5000.0,
            7000.0,
            5000.0,
            0.0,
            0.0,
            8000.0,
            8000.0,
        ).expect("battle should build");
        battle.ships[battle.player.ship_id].set_energy(16);

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: false,
            special: true,
        });
        battle.tick(1000.0 / 24.0);

        assert_eq!(
            (battle.snapshot().player.crew, battle.snapshot().player.energy),
            (13, 17),
        );
    }

    #[test]
    fn shofixti_special_self_destruct_kills_self_and_damages_enemy() {
        let mut battle = Battle::new(
            "shofixti-scout",
            "human-cruiser",
            5000.0,
            5000.0,
            5100.0,
            5000.0,
            0.0,
            0.0,
            8000.0,
            8000.0,
        ).expect("battle should build");

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: false,
            special: true,
        });
        battle.tick(1000.0 / 24.0);

        assert_eq!(
            (
                battle.snapshot().player.dead,
                battle.snapshot().player.crew,
                battle.snapshot().target.crew,
            ),
            (true, 0, 0),
        );
    }

    #[test]
    fn arilou_primary_instant_laser_damages_enemy_in_range() {
        let mut battle = Battle::new(
            "arilou-skiff",
            "human-cruiser",
            5000.0,
            5000.0,
            5000.0,
            4700.0,
            0.0,
            0.0,
            8000.0,
            8000.0,
        ).expect("battle should build");

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: true,
            special: false,
        });
        battle.tick(1000.0 / 24.0);

        assert_eq!(battle.snapshot().target.crew, 17);
    }

    #[test]
    fn arilou_special_teleports_the_ship() {
        let mut battle = Battle::new(
            "arilou-skiff",
            "human-cruiser",
            5000.0,
            5000.0,
            6500.0,
            5000.0,
            0.0,
            0.0,
            8000.0,
            8000.0,
        ).expect("battle should build");

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: false,
            special: true,
        });
        battle.tick(1000.0 / 24.0);

        assert_eq!(battle.snapshot().player.x.round() as i32, 7640);
    }

    #[test]
    fn yehat_shield_blocks_human_nuke_damage() {
        let mut battle = Battle::new(
            "human-cruiser",
            "yehat-terminator",
            5000.0,
            5000.0,
            5000.0,
            4700.0,
            0.0,
            0.0,
            8000.0,
            8000.0,
        ).expect("battle should build");

        battle.set_target_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: false,
            special: true,
        });
        battle.tick(1000.0 / 24.0);
        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: true,
            special: false,
        });
        for _ in 0..45 {
            battle.tick(1000.0 / 24.0);
        }

        assert_eq!(battle.snapshot().target.crew, 20);
    }

    #[test]
    fn mycon_special_regenerates_crew() {
        let mut battle = Battle::new(
            "mycon-podship",
            "human-cruiser",
            5000.0,
            5000.0,
            6500.0,
            5000.0,
            0.0,
            0.0,
            8000.0,
            8000.0,
        ).expect("battle should build");
        battle.ships[battle.player.ship_id].set_crew(12);

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: false,
            special: true,
        });
        battle.tick(1000.0 / 24.0);

        assert_eq!(battle.snapshot().player.crew, 16);
    }

    #[test]
    fn androsynth_special_first_bounce_does_not_give_the_human_full_blazer_speed() {
        let mut battle = Battle::new(
            "androsynth-guardian",
            "human-cruiser",
            5000.0,
            5000.0,
            5010.0,
            5000.0,
            0.0,
            0.0,
            8000.0,
            8000.0,
        ).expect("battle should build");

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: false,
            special: true,
        });

        battle.tick(1000.0 / 24.0);

        assert_eq!(
            (battle.snapshot().target.vx * 100.0).round() as i32,
            286,
        );
    }

    #[test]
    fn androsynth_special_second_bounce_does_not_give_the_human_full_blazer_speed() {
        let mut battle = Battle::new(
            "androsynth-guardian",
            "human-cruiser",
            5000.0,
            5000.0,
            5010.0,
            5000.0,
            0.0,
            0.0,
            8000.0,
            8000.0,
        ).expect("battle should build");

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: false,
            special: true,
        });
        battle.tick(1000.0 / 24.0);

        let target_after_first_bounce = battle
            .matter_world
            .body_state(battle.target.body_id)
            .expect("target body after first bounce");
        battle.player.special_contacting = false;
        battle.matter_world.set_body_position(
            battle.player.body_id,
            target_after_first_bounce.x - 10.0,
            target_after_first_bounce.y,
        );
        battle.matter_world.set_body_velocity(battle.player.body_id, 10.0, 0.0);

        battle.tick(1000.0 / 24.0);

        assert_eq!(
            (battle.snapshot().target.vx * 100.0).round() as i32,
            490,
        );
    }

    #[test]
    fn collision_velocity_bounces_a_light_blazer_back_harder_than_a_human_cruiser() {
        let ((player_vx, _), (target_vx, _)) = super::resolve_collision_velocity(
            5000.0, 5000.0, 10.0, 0.0, 1.0,
            5100.0, 5000.0, 0.0, 0.0, 6.0,
        );

        assert_eq!(
            ((player_vx * 100.0).round() as i32, (target_vx * 100.0).round() as i32),
            (-714, 286),
        );
    }

    #[test]
    fn androsynth_bubble_polygon_does_not_overlap_human_cruiser_at_the_logged_hit_position() {
        let ship = AnyShip::from(HumanCruiser::new());
        let projectile_collision = AnyShip::from(crate::ships::AndrosynthGuardian::new())
            .primary_projectile_spec()
            .expect("androsynth projectile spec")
            .collision;
        let projectile_polygon = super::projectile_hit_polygon(projectile_collision, 15, 4763.0, 1728.0);
        let ship_polygon = ship.hit_polygon(0, 6440.0, 1660.0);

        assert_eq!(super::polygons_intersect(&projectile_polygon, &ship_polygon), false);
    }

    #[test]
    fn androsynth_bubble_with_the_logged_target_does_not_damage_the_enemy_in_the_first_forty_five_ticks() {
        let mut battle = Battle::new(
            "androsynth-guardian",
            "human-cruiser",
            4640.0,
            2160.0,
            6440.0,
            1660.0,
            0.0,
            0.0,
            7680.0,
            4320.0,
        ).expect("battle should build");

        battle.set_player_weapon_target_point(4601.0, 1319.0);
        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: true,
            special: false,
        });
        battle.tick(1000.0 / 24.0);
        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: false,
            special: false,
        });

        for _ in 0..44 {
            battle.tick(1000.0 / 24.0);
        }

        assert_eq!(battle.snapshot().target.crew, 18);
    }

    #[test]
    fn weapon_input_adds_a_projectile_with_an_id_to_the_snapshot() {
        let mut battle = Battle::new(
            "human-cruiser",
            "human-cruiser",
            5000.0,
            5000.0,
            6200.0,
            5000.0,
            0.0,
            0.0,
            8000.0,
            8000.0,
        ).expect("battle should build");

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: true,
            special: false,
        });
        battle.tick(1000.0 / 24.0);

        assert_eq!(battle.snapshot().projectiles[0].id > 0, true);
    }

    #[test]
    fn androsynth_special_adds_audio_event_when_comet_starts() {
        let mut battle = Battle::new(
            "androsynth-guardian",
            "human-cruiser",
            5000.0,
            5000.0,
            6200.0,
            5000.0,
            0.0,
            0.0,
            8000.0,
            8000.0,
        ).expect("battle should build");

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: false,
            special: true,
        });
        battle.tick(1000.0 / 24.0);

        assert_eq!(
            battle.snapshot().audio_events.iter().any(|event| event.key == "androsynth-special"),
            true,
        );
    }

    #[test]
    fn androsynth_bubble_polygon_does_not_overlap_human_cruiser_when_still_in_front() {
        let ship = AnyShip::from(HumanCruiser::new());
        let projectile_collision = AnyShip::from(crate::ships::AndrosynthGuardian::new())
            .primary_projectile_spec()
            .expect("androsynth projectile spec")
            .collision;
        let projectile_polygon = super::projectile_hit_polygon(projectile_collision, 0, 5000.0, 4210.0);
        let ship_polygon = ship.hit_polygon(0, 5000.0, 4300.0);

        assert_eq!(super::polygons_intersect(&projectile_polygon, &ship_polygon), false);
    }

    #[test]
    fn human_nuke_polygon_can_overlap_human_cruiser_polygon() {
        let ship = AnyShip::from(HumanCruiser::new());
        let projectile_collision = AnyShip::from(HumanCruiser::new())
            .primary_projectile_spec()
            .expect("human projectile spec")
            .collision;
        let projectile_polygon = super::projectile_hit_polygon(projectile_collision, 0, 5000.0, 4300.0);
        let ship_polygon = ship.hit_polygon(0, 5000.0, 4300.0);

        assert_eq!(super::polygons_intersect(&projectile_polygon, &ship_polygon), true);
    }

    #[test]
    fn normal_ship_collision_keeps_crew_and_sets_collision_cooldowns() {
        let mut battle = Battle::new(
            "androsynth-guardian",
            "human-cruiser",
            5000.0,
            5000.0,
            5060.0,
            5000.0,
            0.0,
            0.0,
            8000.0,
            8000.0,
        ).expect("battle should build");
        battle.matter_world.set_body_velocity(battle.player.body_id, 2.0, 0.0);
        battle.matter_world.set_body_velocity(battle.target.body_id, -2.0, 0.0);

        battle.tick(1000.0 / 24.0);

        assert_eq!(
            (
                battle.ships[battle.player.ship_id].crew(),
                battle.ships[battle.target.ship_id].crew(),
                battle.ships[battle.player.ship_id].turn_counter(),
                battle.ships[battle.player.ship_id].thrust_counter(),
                battle.ships[battle.target.ship_id].turn_counter(),
                battle.ships[battle.target.ship_id].thrust_counter(),
            ),
            (20, 18, 1, 3, 1, 3),
        );
    }

    #[test]
    fn current_battle_ships_use_polygon_bodies() {
        let battle = Battle::new(
            "androsynth-guardian",
            "human-cruiser",
            5000.0,
            5000.0,
            6200.0,
            5000.0,
            0.0,
            0.0,
            8000.0,
            8000.0,
        ).expect("battle should build");

        assert_eq!(
            (
                battle.matter_world.body_uses_polygon_shape(battle.player.body_id),
                battle.matter_world.body_uses_polygon_shape(battle.target.body_id),
            ),
            (Some(true), Some(true)),
        );
    }

    #[test]
    fn androsynth_special_respects_thrust_counter() {
        let mut battle = Battle::new(
            "androsynth-guardian",
            "human-cruiser",
            5000.0,
            5000.0,
            6200.0,
            5000.0,
            0.0,
            0.0,
            8000.0,
            8000.0,
        ).expect("battle should build");
        battle.player.special_active = true;
        battle.ships[battle.player.ship_id].set_thrust_counter(2);
        battle.matter_world.set_body_velocity(battle.player.body_id, 3.0, 1.5);

        battle.tick(1000.0 / 24.0);

        assert_eq!(
            (
                (battle.snapshot().player.vx * 100.0).round() as i32,
                (battle.snapshot().player.vy * 100.0).round() as i32,
            ),
            (300, 150),
        );
    }

    #[test]
    fn androsynth_special_uses_blazer_hit_polygon() {
        let mut battle = Battle::new(
            "androsynth-guardian",
            "human-cruiser",
            5000.0,
            5000.0,
            6200.0,
            5000.0,
            0.0,
            0.0,
            8000.0,
            8000.0,
        ).expect("battle should build");

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: false,
            special: true,
        });
        battle.tick(1000.0 / 24.0);

        assert_eq!(battle.snapshot().player.texture_prefix, "androsynth-blazer");
    }

    #[test]
    fn segment_hits_human_cruiser_polygon_when_path_crosses_it() {
        let polygon = crate::ships::HumanCruiser::new().hit_polygon(0, 0.0, 0.0);
        assert_eq!(
            super::segment_hits_polygon(
                0.0,
                -120.0,
                0.0,
                120.0,
                &polygon,
                0.0,
            ),
            true,
        );
    }

    #[test]
    fn segment_hits_circle_when_path_crosses_it() {
        assert_eq!(super::segment_hits_circle(0.0, 0.0, 10.0, 0.0, 5.0, 3.0, 3.5), true);
    }

    #[test]
    fn androsynth_special_overlap_only_damages_once_per_contact() {
        let mut battle = Battle::new(
            "androsynth-guardian",
            "human-cruiser",
            5000.0,
            5000.0,
            5000.0,
            4900.0,
            0.0,
            0.0,
            8000.0,
            8000.0,
        ).expect("battle should build");

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: false,
            special: true,
        });

        battle.tick(1000.0 / 24.0);
        battle.tick(1000.0 / 24.0);

        assert_eq!(battle.snapshot().target.crew, 15);
    }

    #[test]
    fn androsynth_special_damages_the_other_ship_inside_the_visible_hit_circle() {
        let mut battle = Battle::new(
            "androsynth-guardian",
            "human-cruiser",
            5000.0,
            5000.0,
            5000.0,
            4900.0,
            0.0,
            0.0,
            8000.0,
            8000.0,
        ).expect("battle should build");

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: false,
            special: true,
        });

        battle.tick(1000.0 / 24.0);

        assert_eq!(battle.snapshot().target.crew, 15);
    }

    #[test]
    fn androsynth_special_collision_adds_impact_audio_event() {
        let mut battle = Battle::new(
            "androsynth-guardian",
            "human-cruiser",
            5000.0,
            5000.0,
            5010.0,
            5000.0,
            0.0,
            0.0,
            8000.0,
            8000.0,
        ).expect("battle should build");

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: false,
            special: true,
        });

        battle.tick(1000.0 / 24.0);

        assert_eq!(
            battle
                .snapshot()
                .audio_events
                .iter()
                .any(|event| event.key == "battle-boom-23"),
            true
        );
    }

    #[test]
    fn androsynth_special_collision_can_kill_the_other_ship() {
        let mut battle = Battle::new(
            "androsynth-guardian",
            "human-cruiser",
            5000.0,
            5000.0,
            5010.0,
            5000.0,
            0.0,
            0.0,
            8000.0,
            8000.0,
        ).expect("battle should build");
        battle.ships[battle.target.ship_id].set_crew(3);

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: false,
            special: true,
        });

        battle.tick(1000.0 / 24.0);

        assert_eq!(battle.snapshot().target.dead, true);
    }

    #[test]
    fn androsynth_special_collision_damages_the_other_ship() {
        let mut battle = Battle::new(
            "androsynth-guardian",
            "human-cruiser",
            5000.0,
            5000.0,
            5010.0,
            5000.0,
            0.0,
            0.0,
            8000.0,
            8000.0,
        ).expect("battle should build");

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: false,
            special: true,
        });

        battle.tick(1000.0 / 24.0);

        assert_eq!(battle.snapshot().target.crew, 15);
    }

    #[test]
    fn androsynth_special_switches_back_to_guardian_when_energy_runs_out() {
        let mut battle = Battle::new(
            "androsynth-guardian",
            "human-cruiser",
            5000.0,
            5000.0,
            6200.0,
            5000.0,
            0.0,
            0.0,
            8000.0,
            8000.0,
        ).expect("battle should build");

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: false,
            special: true,
        });

        for _ in 0..200 {
            battle.tick(1000.0 / 24.0);
        }

        assert_eq!(battle.snapshot().player.texture_prefix, "androsynth-guardian");
    }

    #[test]
    fn androsynth_special_decreases_player_energy_over_time() {
        let mut battle = Battle::new(
            "androsynth-guardian",
            "human-cruiser",
            5000.0,
            5000.0,
            6200.0,
            5000.0,
            0.0,
            0.0,
            8000.0,
            8000.0,
        ).expect("battle should build");

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: false,
            special: true,
        });

        for _ in 0..9 {
            battle.tick(1000.0 / 24.0);
        }

        assert_eq!(battle.snapshot().player.energy, 21);
    }

    #[test]
    fn androsynth_special_moves_player_without_thrust_input() {
        let mut battle = Battle::new(
            "androsynth-guardian",
            "human-cruiser",
            5000.0,
            5000.0,
            6200.0,
            5000.0,
            0.0,
            0.0,
            8000.0,
            8000.0,
        ).expect("battle should build");

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: false,
            special: true,
        });

        battle.tick(1000.0 / 24.0);

        assert_eq!(battle.snapshot().player.vy < 0.0, true);
    }

    #[test]
    fn androsynth_special_switches_player_snapshot_to_blazer() {
        let mut battle = Battle::new(
            "androsynth-guardian",
            "human-cruiser",
            5000.0,
            5000.0,
            6200.0,
            5000.0,
            4000.0,
            4000.0,
            8000.0,
            8000.0,
        ).expect("battle should build");

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: false,
            special: true,
        });

        battle.tick(1000.0 / 24.0);

        assert_eq!(battle.snapshot().player.texture_prefix, "androsynth-blazer");
    }

    fn distance_to_point(projectile: &ProjectileSnapshot, x: f64, y: f64) -> f64 {
        let dx = projectile.x - x;
        let dy = projectile.y - y;
        ((dx * dx) + (dy * dy)).sqrt()
    }

    #[test]
    fn androsynth_point_target_bubble_still_damages_enemy_on_hit() {
        let mut battle = Battle::new(
            "androsynth-guardian",
            "human-cruiser",
            5000.0,
            5000.0,
            5000.0,
            4300.0,
            500.0,
            600.0,
            10000.0,
            10000.0,
        )
        .unwrap();
        battle.set_player_weapon_target_point(4200.0, 4200.0);

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: true,
            special: false,
        });
        battle.tick(1000.0 / 24.0);

        let target_body = battle
            .matter_world
            .body_state(battle.target.body_id)
            .expect("target body");
        battle.projectiles[0].x = target_body.x;
        battle.projectiles[0].y = target_body.y;

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: false,
            special: false,
        });
        battle.tick(1000.0 / 24.0);

        assert_eq!(battle.snapshot().target.crew, 16);
    }

    #[test]
    fn androsynth_primary_tracks_a_close_point_without_spread() {
        let mut close_battle = Battle::new(
            "androsynth-guardian",
            "human-cruiser",
            5000.0,
            5000.0,
            7600.0,
            4500.0,
            5000.0,
            5000.0,
            10000.0,
            10000.0,
        )
        .unwrap();
        let mut far_battle = Battle::new(
            "androsynth-guardian",
            "human-cruiser",
            5000.0,
            5000.0,
            7600.0,
            4500.0,
            5000.0,
            5000.0,
            10000.0,
            10000.0,
        )
        .unwrap();
        close_battle.set_player_weapon_target_point(5080.0, 4900.0);
        far_battle.set_player_weapon_target_point(5600.0, 4100.0);

        close_battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: true,
            special: false,
        });
        far_battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: true,
            special: false,
        });
        close_battle.tick(1000.0 / 24.0);
        far_battle.tick(1000.0 / 24.0);

        close_battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: false,
            special: false,
        });
        far_battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: false,
            special: false,
        });

        for _ in 0..3 {
            close_battle.tick(1000.0 / 24.0);
            far_battle.tick(1000.0 / 24.0);
        }

        assert_eq!(
            distance_to_point(&close_battle.snapshot().projectiles[0], 5080.0, 4900.0)
                < distance_to_point(&far_battle.snapshot().projectiles[0], 5080.0, 4900.0),
            true
        );
    }

    #[test]
    fn androsynth_primary_bubble_disappears_after_its_life_runs_out() {
        let mut battle = Battle::new(
            "androsynth-guardian",
            "human-cruiser",
            5000.0,
            5000.0,
            5600.0,
            4100.0,
            5000.0,
            5000.0,
            10000.0,
            10000.0,
        )
        .unwrap();

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: true,
            special: false,
        });
        battle.tick(1000.0 / 24.0);
        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: false,
            special: false,
        });

        for _ in 0..200 {
            battle.tick(1000.0 / 24.0);
        }

        assert_eq!(battle.snapshot().projectiles.len(), 0);
    }

    #[test]
    fn androsynth_primary_bubbles_do_not_share_the_same_path() {
        let mut battle = Battle::new(
            "androsynth-guardian",
            "human-cruiser",
            5000.0,
            5000.0,
            5600.0,
            4100.0,
            5000.0,
            5000.0,
            10000.0,
            10000.0,
        )
        .unwrap();

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: true,
            special: false,
        });
        battle.tick(1000.0 / 24.0);
        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: false,
            special: false,
        });
        for _ in 0..4 {
            battle.tick(1000.0 / 24.0);
        }
        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: true,
            special: false,
        });
        battle.tick(1000.0 / 24.0);
        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: false,
            special: false,
        });
        for _ in 0..8 {
            battle.tick(1000.0 / 24.0);
        }

        assert_ne!(
            (
                battle.snapshot().projectiles[0].x.round() as i32,
                battle.snapshot().projectiles[0].y.round() as i32,
                (battle.snapshot().projectiles[0].vx * 32.0).round() as i32,
                (battle.snapshot().projectiles[0].vy * 32.0).round() as i32,
            ),
            (
                battle.snapshot().projectiles[1].x.round() as i32,
                battle.snapshot().projectiles[1].y.round() as i32,
                (battle.snapshot().projectiles[1].vx * 32.0).round() as i32,
                (battle.snapshot().projectiles[1].vy * 32.0).round() as i32,
            )
        );
    }

    #[test]
    fn androsynth_primary_uses_clicked_point_target() {
        let mut default_battle = Battle::new(
            "androsynth-guardian",
            "human-cruiser",
            5000.0,
            5000.0,
            7600.0,
            4500.0,
            5000.0,
            5000.0,
            10000.0,
            10000.0,
        )
        .unwrap();
        let mut point_battle = Battle::new(
            "androsynth-guardian",
            "human-cruiser",
            5000.0,
            5000.0,
            7600.0,
            4500.0,
            5000.0,
            5000.0,
            10000.0,
            10000.0,
        )
        .unwrap();
        point_battle.set_player_weapon_target_point(4200.0, 4100.0);

        default_battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: true,
            special: false,
        });
        point_battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: true,
            special: false,
        });
        default_battle.tick(1000.0 / 24.0);
        point_battle.tick(1000.0 / 24.0);

        default_battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: false,
            special: false,
        });
        point_battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: false,
            special: false,
        });

        for _ in 0..12 {
            default_battle.tick(1000.0 / 24.0);
            point_battle.tick(1000.0 / 24.0);
        }

        assert_eq!(
            point_battle.snapshot().projectiles[0].x < default_battle.snapshot().projectiles[0].x,
            true
        );
    }

    #[test]
    fn androsynth_primary_curves_toward_enemy_ship() {
        let mut battle = Battle::new(
            "androsynth-guardian",
            "human-cruiser",
            5000.0,
            5000.0,
            7600.0,
            4500.0,
            5000.0,
            5000.0,
            10000.0,
            10000.0,
        )
        .unwrap();

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: true,
            special: false,
        });
        battle.tick(1000.0 / 24.0);
        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: false,
            special: false,
        });

        for _ in 0..6 {
            battle.tick(1000.0 / 24.0);
        }

        assert_eq!(battle.snapshot().projectiles[0].x > 5000.0, true);
    }

    #[test]
    fn androsynth_primary_uses_bubble_texture_prefix() {
        let mut battle = Battle::new(
            "androsynth-guardian",
            "human-cruiser",
            5000.0,
            5000.0,
            7600.0,
            4500.0,
            5000.0,
            5000.0,
            10000.0,
            10000.0,
        )
        .unwrap();

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: true,
            special: false,
        });
        battle.tick(1000.0 / 24.0);

        assert_eq!(
            battle.snapshot().projectiles.first().map(|projectile| projectile.texture_prefix),
            Some("androsynth-bubble")
        );
    }

    #[test]
    fn androsynth_primary_adds_one_projectile_to_snapshot() {
        let mut battle = Battle::new(
            "androsynth-guardian",
            "human-cruiser",
            5000.0,
            5000.0,
            7600.0,
            4500.0,
            5000.0,
            5000.0,
            10000.0,
            10000.0,
        )
        .unwrap();

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: true,
            special: false,
        });
        battle.tick(1000.0 / 24.0);

        assert_eq!(battle.snapshot().projectiles.len(), 1);
    }

    #[test]
    fn destroyed_target_adds_winner_victory_audio_event_to_snapshot() {
        let mut battle = Battle::new(
            "human-cruiser",
            "human-cruiser",
            5000.0,
            5000.0,
            5000.0,
            4300.0,
            500.0,
            600.0,
            10000.0,
            10000.0,
        )
        .unwrap();
        battle.set_player_weapon_target_ship();

        for _ in 0..4 {
            battle.set_player_input(ShipInput {
                left: false,
                right: false,
                thrust: false,
                weapon: true,
                special: false,
            });
            for _ in 0..80 {
                battle.tick(1000.0 / 24.0);
                battle.set_player_input(ShipInput {
                    left: false,
                    right: false,
                    thrust: false,
                    weapon: false,
                    special: false,
                });
            }
        }

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: true,
            special: false,
        });

        for _ in 0..80 {
            battle.tick(1000.0 / 24.0);
            battle.set_player_input(ShipInput {
                left: false,
                right: false,
                thrust: false,
                weapon: false,
                special: false,
            });
            if battle.snapshot().target.dead {
                break;
            }
        }

        assert_eq!(
            battle
                .snapshot()
                .audio_events
                .iter()
                .any(|event| event.key == "human-victory"),
            true
        );
    }

    #[test]
    fn arilou_destroyed_target_adds_winner_victory_audio_event_to_snapshot() {
        let mut battle = Battle::new(
            "arilou-skiff",
            "human-cruiser",
            5000.0,
            5000.0,
            5000.0,
            4700.0,
            500.0,
            600.0,
            10000.0,
            10000.0,
        )
        .unwrap();
        battle.ships[battle.target.ship_id].set_crew(1);

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: true,
            special: false,
        });
        battle.tick(1000.0 / 24.0);

        assert_eq!(
            battle
                .snapshot()
                .audio_events
                .iter()
                .any(|event| event.key == "arilou-victory"),
            true
        );
    }

    #[test]
    fn human_special_adds_audio_event_when_destroying_enemy_projectile() {
        let mut battle = Battle::new(
            "human-cruiser",
            "human-cruiser",
            5000.0,
            5000.0,
            5500.0,
            5000.0,
            5000.0,
            5000.0,
            10000.0,
            10000.0,
        )
        .unwrap();

        for _ in 0..4 {
            battle.set_target_input(ShipInput {
                left: true,
                right: false,
                thrust: false,
                weapon: false,
                special: false,
            });
            battle.tick(1000.0 / 24.0);
        }

        battle.trigger_target_weapon();
        battle.tick(1000.0 / 24.0);
        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: false,
            special: true,
        });
        battle.tick(1000.0 / 24.0);

        assert_eq!(
            battle.snapshot().audio_events.iter().any(|event| event.key == "human-special"),
            true
        );
    }

    #[test]
    fn human_special_destroys_enemy_projectile_in_range() {
        let mut battle = Battle::new(
            "human-cruiser",
            "human-cruiser",
            5000.0,
            5000.0,
            5500.0,
            5000.0,
            5000.0,
            5000.0,
            10000.0,
            10000.0,
        )
        .unwrap();
        battle.set_player_weapon_target_ship();

        for _ in 0..4 {
            battle.set_target_input(ShipInput {
                left: true,
                right: false,
                thrust: false,
                weapon: false,
                special: false,
            });
            battle.tick(1000.0 / 24.0);
        }

        battle.trigger_target_weapon();
        battle.tick(1000.0 / 24.0);
        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: false,
            special: true,
        });
        battle.tick(1000.0 / 24.0);

        assert_eq!(battle.snapshot().projectiles.len(), 0);
    }

    #[test]
    fn queued_target_weapon_spawns_enemy_projectile() {
        let mut battle = Battle::new(
            "human-cruiser",
            "human-cruiser",
            5000.0,
            5000.0,
            7600.0,
            4500.0,
            5000.0,
            5000.0,
            10000.0,
            10000.0,
        )
        .unwrap();

        battle.trigger_target_weapon();
        battle.tick(1000.0 / 24.0);

        assert_eq!(battle.snapshot().projectiles.len(), 1);
    }

    #[test]
    fn weapon_fire_adds_primary_audio_event_to_snapshot() {
        let mut battle = Battle::new(
            "human-cruiser",
            "human-cruiser",
            5000.0,
            5000.0,
            7600.0,
            4500.0,
            5000.0,
            5000.0,
            10000.0,
            10000.0,
        )
        .unwrap();

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: true,
            special: false,
        });
        battle.tick(1000.0 / 24.0);

        assert_eq!(
            battle.snapshot().audio_events.first().map(|event| event.key),
            Some("human-primary")
        );
    }

    #[test]
    fn destroyed_target_adds_ship_death_audio_event_to_snapshot() {
        let mut battle = Battle::new(
            "human-cruiser",
            "human-cruiser",
            5000.0,
            5000.0,
            5000.0,
            4300.0,
            500.0,
            600.0,
            10000.0,
            10000.0,
        )
        .unwrap();
        battle.set_player_weapon_target_ship();

        for _ in 0..4 {
            battle.set_player_input(ShipInput {
                left: false,
                right: false,
                thrust: false,
                weapon: true,
                special: false,
            });
            for _ in 0..80 {
                battle.tick(1000.0 / 24.0);
                battle.set_player_input(ShipInput {
                    left: false,
                    right: false,
                    thrust: false,
                    weapon: false,
                    special: false,
                });
            }
        }

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: true,
            special: false,
        });

        for _ in 0..80 {
            battle.tick(1000.0 / 24.0);
            battle.set_player_input(ShipInput {
                left: false,
                right: false,
                thrust: false,
                weapon: false,
                special: false,
            });
            if battle.snapshot().target.dead {
                break;
            }
        }

        assert_eq!(
            battle
                .snapshot()
                .audio_events
                .iter()
                .any(|event| event.key == "battle-shipdies"),
            true
        );
    }

    #[test]
    fn destroyed_target_creates_ship_death_explosion_snapshot() {
        let mut battle = Battle::new(
            "human-cruiser",
            "human-cruiser",
            5000.0,
            5000.0,
            5000.0,
            4300.0,
            500.0,
            600.0,
            10000.0,
            10000.0,
        )
        .unwrap();
        battle.set_player_weapon_target_ship();

        for _ in 0..4 {
            battle.set_player_input(ShipInput {
                left: false,
                right: false,
                thrust: false,
                weapon: true,
                special: false,
            });
            for _ in 0..80 {
                battle.tick(1000.0 / 24.0);
                battle.set_player_input(ShipInput {
                    left: false,
                    right: false,
                    thrust: false,
                    weapon: false,
                    special: false,
                });
            }
        }

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: true,
            special: false,
        });

        for _ in 0..80 {
            battle.tick(1000.0 / 24.0);
            battle.set_player_input(ShipInput {
                left: false,
                right: false,
                thrust: false,
                weapon: false,
                special: false,
            });
            if battle.snapshot().target.dead {
                break;
            }
        }

        assert_eq!(
            battle
                .snapshot()
                .explosions
                .iter()
                .any(|explosion| explosion.texture_prefix == "battle-boom"),
            true
        );
    }

    #[test]
    fn destroyed_target_is_marked_dead_in_snapshot() {
        let mut battle = Battle::new(
            "human-cruiser",
            "human-cruiser",
            5000.0,
            5000.0,
            5000.0,
            4300.0,
            500.0,
            600.0,
            10000.0,
            10000.0,
        )
        .unwrap();
        battle.set_player_weapon_target_ship();

        for _ in 0..5 {
            battle.set_player_input(ShipInput {
                left: false,
                right: false,
                thrust: false,
                weapon: true,
                special: false,
            });
            for _ in 0..80 {
                battle.tick(1000.0 / 24.0);
                battle.set_player_input(ShipInput {
                    left: false,
                    right: false,
                    thrust: false,
                    weapon: false,
                    special: false,
                });
            }
        }

        assert_eq!(battle.snapshot().target.dead, true);
    }

    #[test]
    fn human_nuke_hits_target_cruiser_when_it_faces_right() {
        let mut battle = Battle::new(
            "human-cruiser",
            "human-cruiser",
            5800.0,
            5000.0,
            7600.0,
            4500.0,
            5000.0,
            5000.0,
            10000.0,
            10000.0,
        )
        .unwrap();

        for _ in 0..4 {
            battle.set_target_input(ShipInput {
                left: false,
                right: true,
                thrust: false,
                weapon: false,
                special: false,
            });
            battle.tick(1000.0 / 24.0);
        }

        battle.set_target_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: false,
            special: false,
        });
        battle.set_player_weapon_target_ship();
        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: true,
            special: false,
        });

        for _ in 0..60 {
            battle.tick(1000.0 / 24.0);
            battle.set_player_input(ShipInput {
                left: false,
                right: false,
                thrust: false,
                weapon: false,
                special: false,
            });
        }

        assert_eq!(battle.snapshot().target.crew, 14);
    }

    #[test]
    fn target_input_can_rotate_the_enemy_ship() {
        let mut battle = Battle::new(
            "human-cruiser",
            "human-cruiser",
            5000.0,
            5000.0,
            7600.0,
            4500.0,
            5000.0,
            5000.0,
            10000.0,
            10000.0,
        )
        .unwrap();

        battle.set_target_input(ShipInput {
            left: false,
            right: true,
            thrust: false,
            weapon: false,
            special: false,
        });
        battle.tick(1000.0 / 24.0);

        assert_eq!(battle.snapshot().target.facing > -std::f64::consts::FRAC_PI_2, true);
    }

    #[test]
    fn moving_androsynth_bubble_does_not_hit_on_the_spawn_tick() {
        let mut battle = Battle::new(
            "androsynth-guardian",
            "human-cruiser",
            5000.0,
            5000.0,
            7600.0,
            4500.0,
            5000.0,
            5000.0,
            10000.0,
            10000.0,
        )
        .unwrap();

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: true,
            weapon: true,
            special: false,
        });
        battle.tick(1000.0 / 24.0);

        assert_eq!(battle.snapshot().target.crew, 18);
    }

    #[test]
    fn androsynth_bubble_with_point_target_away_from_enemy_does_not_damage_enemy_early() {
        let mut battle = Battle::new(
            "androsynth-guardian",
            "human-cruiser",
            5000.0,
            5000.0,
            7600.0,
            4500.0,
            5000.0,
            5000.0,
            10000.0,
            10000.0,
        )
        .unwrap();
        battle.set_player_weapon_target_point(4200.0, 5800.0);

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: true,
            special: false,
        });
        battle.tick(1000.0 / 24.0);

        for _ in 0..12 {
            battle.set_player_input(ShipInput {
                left: false,
                right: false,
                thrust: false,
                weapon: false,
                special: false,
            });
            battle.tick(1000.0 / 24.0);
        }

        assert_eq!(battle.snapshot().target.crew, 18);
    }

    #[test]
    fn androsynth_bubble_hit_damages_target_for_two_crew() {
        let mut battle = Battle::new(
            "androsynth-guardian",
            "human-cruiser",
            5000.0,
            5000.0,
            5000.0,
            4300.0,
            500.0,
            600.0,
            10000.0,
            10000.0,
        )
        .unwrap();

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: true,
            special: false,
        });
        battle.tick(1000.0 / 24.0);

        let target_body = battle
            .matter_world
            .body_state(battle.target.body_id)
            .expect("target body");
        battle.projectiles[0].x = target_body.x;
        battle.projectiles[0].y = target_body.y;

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: false,
            special: false,
        });
        battle.tick(1000.0 / 24.0);

        assert_eq!(battle.snapshot().target.crew, 16);
    }

    #[test]
    fn androsynth_bubble_hit_uses_generic_blast_impact() {
        let mut battle = Battle::new(
            "androsynth-guardian",
            "human-cruiser",
            5000.0,
            5000.0,
            5000.0,
            4300.0,
            500.0,
            600.0,
            10000.0,
            10000.0,
        )
        .unwrap();

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: true,
            special: false,
        });
        battle.tick(1000.0 / 24.0);

        let target_body = battle
            .matter_world
            .body_state(battle.target.body_id)
            .expect("target body");
        battle.projectiles[0].x = target_body.x;
        battle.projectiles[0].y = target_body.y;

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: false,
            special: false,
        });
        battle.tick(1000.0 / 24.0);

        assert_eq!(
            battle
                .snapshot()
                .explosions
                .first()
                .map(|explosion| explosion.texture_prefix),
            Some("battle-blast")
        );
    }

    #[test]
    fn human_nuke_hit_creates_explosion_snapshot() {
        let mut battle = Battle::new(
            "human-cruiser",
            "human-cruiser",
            5000.0,
            5000.0,
            5000.0,
            4300.0,
            500.0,
            600.0,
            10000.0,
            10000.0,
        )
        .unwrap();
        battle.set_player_weapon_target_ship();

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: true,
            special: false,
        });

        for _ in 0..14 {
            battle.tick(1000.0 / 24.0);
            battle.set_player_input(ShipInput {
                left: false,
                right: false,
                thrust: false,
                weapon: false,
                special: false,
            });
        }

        assert_eq!(
            battle
                .snapshot()
                .explosions
                .first()
                .map(|explosion| (
                    explosion.texture_prefix,
                    (16..=24)
                        .contains(&explosion.frame_index),
                )),
            Some(("human-saturn", true))
        );
    }

    #[test]
    fn human_nuke_homing_hits_target_cruiser_in_battle_layout() {
        let mut battle = Battle::new(
            "human-cruiser",
            "human-cruiser",
            5800.0,
            5000.0,
            7600.0,
            4500.0,
            5000.0,
            5000.0,
            10000.0,
            10000.0,
        )
        .unwrap();
        battle.set_player_weapon_target_ship();

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: true,
            special: false,
        });

        for _ in 0..60 {
            battle.tick(1000.0 / 24.0);
            battle.set_player_input(ShipInput {
                left: false,
                right: false,
                thrust: false,
                weapon: false,
                special: false,
            });
        }

        assert_eq!(battle.snapshot().target.crew, 14);
    }

    #[test]
    fn human_nuke_hit_removes_projectile_and_damages_target() {
        let mut battle = Battle::new(
            "human-cruiser",
            "human-cruiser",
            5000.0,
            5000.0,
            5000.0,
            4300.0,
            500.0,
            600.0,
            10000.0,
            10000.0,
        )
        .unwrap();
        battle.set_player_weapon_target_ship();

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: true,
            special: false,
        });

        for _ in 0..20 {
            battle.tick(1000.0 / 24.0);
            battle.set_player_input(ShipInput {
                left: false,
                right: false,
                thrust: false,
                weapon: false,
                special: false,
            });
        }

        assert_eq!(
            (
                battle.snapshot().target.crew,
                battle.snapshot().projectiles.len(),
            ),
            (14, 0)
        );
    }

    #[test]
    fn human_nuke_homing_matches_reference_frame_three() {
        let scenario = &reference_data::load().human_nuke_homing;
        let mut battle = Battle::new(
            "human-cruiser",
            "human-cruiser",
            5000.0,
            5000.0,
            scenario.frames[0].target_x as f64,
            scenario.frames[0].target_y as f64,
            500.0,
            600.0,
            10000.0,
            10000.0,
        )
        .unwrap();
        battle.set_player_weapon_target_ship();

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: true,
            special: false,
        });

        for frame in scenario.frames.iter().take(4) {
            battle.matter_world.set_body_position(
                battle.target.body_id,
                frame.target_x as f64,
                frame.target_y as f64,
            );
            battle.tick(1000.0 / 24.0);
            battle.set_player_input(ShipInput {
                left: false,
                right: false,
                thrust: false,
                weapon: false,
                special: false,
            });
        }

        assert_eq!(
            (
                battle.snapshot().projectiles[0].x.round() as i32,
                battle.snapshot().projectiles[0].y.round() as i32,
                (battle.snapshot().projectiles[0].vx * 32.0).round() as i32,
                (battle.snapshot().projectiles[0].vy * 32.0).round() as i32,
                battle.snapshot().projectiles[0].life,
            ),
            (
                scenario.frames[3].x,
                scenario.frames[3].y,
                scenario.frames[3].vx,
                scenario.frames[3].vy,
                scenario.frames[3].life,
            )
        );
    }

    #[test]
    fn human_nuke_matches_reference_for_the_first_sixty_frames() {
        let scenario = &reference_data::load().human_nuke_straight;
        let mut battle = Battle::new(
            "human-cruiser",
            "human-cruiser",
            5000.0,
            5000.0,
            300.0,
            400.0,
            500.0,
            600.0,
            10000.0,
            10000.0,
        )
        .unwrap();

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: true,
            special: false,
        });
        battle.tick(1000.0 / 24.0);
        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: false,
            special: false,
        });

        let mut actual = vec![(
            battle.snapshot().projectiles[0].x.round() as i32,
            battle.snapshot().projectiles[0].y.round() as i32,
            (battle.snapshot().projectiles[0].vx * 32.0).round() as i32,
            (battle.snapshot().projectiles[0].vy * 32.0).round() as i32,
            battle.snapshot().projectiles[0].life,
        )];

        for _ in 1..60 {
            battle.tick(1000.0 / 24.0);
            let snapshot = battle.snapshot();
            let projectile = snapshot.projectiles.first();
            actual.push(match projectile {
                Some(projectile) => (
                    projectile.x.round() as i32,
                    projectile.y.round() as i32,
                    (projectile.vx * 32.0).round() as i32,
                    (projectile.vy * 32.0).round() as i32,
                    projectile.life,
                ),
                None => (i32::MIN, i32::MIN, i32::MIN, i32::MIN, i32::MIN),
            });
        }

        assert_eq!(
            actual,
            scenario
                .frames
                .iter()
                .take(60)
                .map(|frame| (frame.x, frame.y, frame.vx, frame.vy, frame.life))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn human_nuke_matches_reference_frame_zero_position() {
        let mut battle = Battle::new(
            "human-cruiser",
            "human-cruiser",
            5000.0,
            5000.0,
            300.0,
            400.0,
            500.0,
            600.0,
            10000.0,
            10000.0,
        )
        .unwrap();

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: true,
            special: false,
        });
        battle.tick(1000.0 / 24.0);

        assert_eq!(
            (
                battle.snapshot().projectiles[0].x,
                battle.snapshot().projectiles[0].y,
                battle.snapshot().projectiles[0].life,
            ),
            (5000.0, 4832.0, 59)
        );
    }

    #[test]
    fn human_nuke_matches_reference_frame_one_position() {
        let mut battle = Battle::new(
            "human-cruiser",
            "human-cruiser",
            5000.0,
            5000.0,
            300.0,
            400.0,
            500.0,
            600.0,
            10000.0,
            10000.0,
        )
        .unwrap();

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: true,
            special: false,
        });
        battle.tick(1000.0 / 24.0);
        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: false,
            special: false,
        });
        battle.tick(1000.0 / 24.0);

        assert_eq!(
            (
                battle.snapshot().projectiles[0].x,
                battle.snapshot().projectiles[0].y,
                battle.snapshot().projectiles[0].life,
            ),
            (5000.0, 4788.0, 58)
        );
    }

    #[test]
    fn human_nuke_matches_reference_frame_eleven_position() {
        let mut battle = Battle::new(
            "human-cruiser",
            "human-cruiser",
            5000.0,
            5000.0,
            300.0,
            400.0,
            500.0,
            600.0,
            10000.0,
            10000.0,
        )
        .unwrap();

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: true,
            special: false,
        });
        battle.tick(1000.0 / 24.0);
        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: false,
            special: false,
        });

        for _ in 0..11 {
            battle.tick(1000.0 / 24.0);
        }

        assert_eq!(
            (
                battle.snapshot().projectiles[0].x,
                battle.snapshot().projectiles[0].y,
                battle.snapshot().projectiles[0].life,
            ),
            (5000.0, 4132.0, 48)
        );
    }

    #[test]
    fn projectile_moves_after_it_is_fired() {
        let mut battle = Battle::new(
            "human-cruiser",
            "human-cruiser",
            100.0,
            200.0,
            300.0,
            400.0,
            500.0,
            600.0,
            1000.0,
            800.0,
        )
        .unwrap();

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: true,
            special: false,
        });
        battle.tick(1.0);
        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: false,
            special: false,
        });
        battle.tick(1.0);

        assert_eq!(
            (
                battle.snapshot().projectiles[0].x.round(),
                battle.snapshot().projectiles[0].y.round(),
            ),
            (100.0, -12.0)
        );
    }

    #[test]
    fn projectile_velocity_follows_the_ship_facing() {
        let mut battle = Battle::new(
            "human-cruiser",
            "human-cruiser",
            100.0,
            200.0,
            300.0,
            400.0,
            500.0,
            600.0,
            1000.0,
            800.0,
        )
        .unwrap();

        battle.set_player_input(ShipInput {
            left: false,
            right: true,
            thrust: false,
            weapon: false,
            special: false,
        });
        battle.tick(1000.0 / 60.0);
        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: true,
            special: false,
        });
        battle.tick(1000.0 / 60.0);

        let snapshot = battle.snapshot();
        let facing = snapshot.player.facing;

        assert_eq!(
            (
                snapshot.projectiles[0].vx.round(),
                snapshot.projectiles[0].vy.round(),
            ),
            ((facing.cos() * 40.0).round(), (facing.sin() * 40.0).round())
        );
    }

    #[test]
    fn projectile_gets_velocity_in_the_facing_direction() {
        let mut battle = Battle::new(
            "human-cruiser",
            "human-cruiser",
            100.0,
            200.0,
            300.0,
            400.0,
            500.0,
            600.0,
            1000.0,
            800.0,
        )
        .unwrap();

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: true,
            special: false,
        });
        battle.tick(1000.0 / 60.0);
        let facing = battle.snapshot().player.facing;

        assert_eq!(
            (
                battle.snapshot().projectiles[0].vx.round(),
                battle.snapshot().projectiles[0].vy.round(),
            ),
            ((facing.cos() * 40.0).round(), (facing.sin() * 40.0).round())
        );
    }

    #[test]
    fn projectile_spawns_at_the_player_position() {
        let mut battle = Battle::new(
            "human-cruiser",
            "human-cruiser",
            100.0,
            200.0,
            300.0,
            400.0,
            500.0,
            600.0,
            1000.0,
            800.0,
        )
        .unwrap();

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: true,
            special: false,
        });
        battle.tick(1000.0 / 60.0);

        assert_eq!(
            (
                battle.snapshot().projectiles[0].x.round(),
                battle.snapshot().projectiles[0].y.round(),
            ),
            (100.0, 32.0)
        );
    }

    #[test]
    fn weapon_input_adds_a_projectile_to_the_snapshot() {
        let mut battle = Battle::new(
            "human-cruiser",
            "human-cruiser",
            100.0,
            200.0,
            300.0,
            400.0,
            500.0,
            600.0,
            1000.0,
            800.0,
        )
        .unwrap();

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: true,
            special: false,
        });
        battle.tick(1000.0 / 60.0);

        assert_eq!(battle.snapshot().projectiles.len(), 1);
    }

    #[test]
    fn snapshot_exposes_initial_player_position() {
        let battle = Battle::new(
            "human-cruiser",
            "human-cruiser",
            100.0,
            200.0,
            300.0,
            400.0,
            500.0,
            600.0,
            1000.0,
            800.0,
        )
        .unwrap();

        assert_eq!((battle.snapshot().player.x, battle.snapshot().player.y), (100.0, 200.0));
    }
}
