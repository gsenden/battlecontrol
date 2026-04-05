use crate::matter_world::{MatterBodyState, MatterWorld};
use crate::physics_command::PhysicsCommand;
use crate::ship_input::ShipInput;
use crate::traits::game_object::GameObject;
use crate::traits::ship_trait::HitPolygonPoint;
use crate::ships::{apply_collision_between, build_ship, AnyShip};
use crate::velocity_vector::VelocityVector;
use crate::wrap::shortest_wrapped_delta;
use matter_js_rs::geometry::Vec2;

const HUMAN_NUKE_SPEED: f64 = 40.0;
const HUMAN_NUKE_ACCELERATION: f64 = 4.0;
const HUMAN_NUKE_MAX_SPEED: f64 = 80.0;
const HUMAN_NUKE_LIFE: i32 = 60;
const HUMAN_NUKE_OFFSET: f64 = 168.0;
const HUMAN_NUKE_TRACK_WAIT: i32 = 3;
const HUMAN_NUKE_DAMAGE: i32 = 4;
const HUMAN_NUKE_IMPACT_START_FRAME: i32 = 16;
const HUMAN_NUKE_IMPACT_END_FRAME: i32 = 24;
const ANDROSYNTH_BUBBLE_DAMAGE: i32 = 2;
const GENERIC_BLAST_START_FRAME: i32 = 0;
const GENERIC_BLAST_END_FRAME: i32 = 7;
const SHIP_DEATH_EXPLOSION_START_FRAME: i32 = 0;
const SHIP_DEATH_EXPLOSION_END_FRAME: i32 = 8;
const HUMAN_POINT_DEFENSE_RANGE: f64 = 400.0;
const ANDROSYNTH_BLAZER_SPEED: f64 = 10.0;
const ANDROSYNTH_BLAZER_DAMAGE: i32 = 3;
const ANDROSYNTH_BLAZER_MASS: f64 = 1.0;
const ANDROSYNTH_BLAZER_HIT_RADIUS: f64 = 24.0;
const ANDROSYNTH_BUBBLE_SPEED: f64 = 32.0;
const ANDROSYNTH_BUBBLE_LIFE: i32 = 200;
const ANDROSYNTH_BUBBLE_OFFSET: f64 = 56.0;
const ANDROSYNTH_BUBBLE_TRACK_WAIT: i32 = 2;
const ANDROSYNTH_BUBBLE_RANDOM_SEED: u32 = 0x00C0FFEE;
const ANDROSYNTH_BUBBLE_DIRECT_TRACK_RANGE: f64 = 180.0;
const PROJECTILE_FACINGS: f64 = 16.0;
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
    pub texture_prefix: &'static str,
}

#[derive(Clone, Copy)]
pub struct LaserSnapshot {
    pub id: u64,
    pub start_x: f64,
    pub start_y: f64,
    pub end_x: f64,
    pub end_y: f64,
}

#[derive(Clone, Copy)]
pub struct AudioEventSnapshot {
    pub key: &'static str,
}

pub struct BattleSnapshot {
    pub player: BattleShipSnapshot,
    pub target: BattleShipSnapshot,
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

struct ShipHitCircle {
    offset_x: f64,
    offset_y: f64,
    radius: f64,
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

        Ok(Self {
            ships,
            projectiles: Vec::new(),
            explosions: Vec::new(),
            lasers: Vec::new(),
            audio_events: Vec::new(),
            matter_world,
            player: BattleShipState {
                id: 1,
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
                id: 2,
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
            bubble_rng_state: ANDROSYNTH_BUBBLE_RANDOM_SEED,
            planet_x,
            planet_y,
            width,
            height,
            next_game_object_id: 3,
        })
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
        self.explosions.retain(|explosion| {
            explosion.frame_index <= explosion_end_frame_for(explosion.texture_prefix)
        });

        let player_target_body = self.matter_world.body_state(self.player.body_id);
        let target_target_body = self.matter_world.body_state(self.target.body_id);

        for projectile in &mut self.projectiles {
            if projectile.texture_prefix == "androsynth-bubble" {
                step_androsynth_bubble(
                    projectile,
                    player_target_body,
                    target_target_body,
                );
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
                        projectile.turn_wait = HUMAN_NUKE_TRACK_WAIT;
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
                        if projectile.texture_prefix == "androsynth-bubble" {
                            projectile.facing_index = wobble_bubble_facing(projectile.facing_index, projectile.life);
                        }
                        projectile.facing = facing_index_to_radians(projectile.facing_index);
                        projectile.turn_wait = if projectile.texture_prefix == "androsynth-bubble" { 2 } else { HUMAN_NUKE_TRACK_WAIT };
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

            let damage = projectile_damage_for(projectile.texture_prefix);
            if is_player {
                let died = self.ships[self.player.ship_id].take_damage(damage);
                player_died |= died;
                target_won |= died && !projectile.owner_is_player;
            } else {
                let died = self.ships[self.target.ship_id].take_damage(damage);
                target_died |= died;
                player_won |= died && projectile.owner_is_player;
            }
            hit_explosions.push(ExplosionSnapshot {
                id: 0,
                x: projectile.x,
                y: projectile.y,
                frame_index: projectile_impact_start_frame_for(projectile.texture_prefix),
                texture_prefix: projectile_impact_texture_prefix_for(projectile.texture_prefix),
            });
            self.audio_events.push(AudioEventSnapshot {
                key: projectile_impact_sound_key_for(projectile.texture_prefix),
            });
            hit_projectile_indexes.push(index);
        }

        for index in hit_projectile_indexes.into_iter().rev() {
            self.projectiles.remove(index);
        }

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

        let player_blazer_hits = self.player.special_active && !self.target.dead && self.androsynth_blazer_hits_other_ship(true);
        if player_blazer_hits && !self.player.special_contacting {
            if let (Some(pb), Some(tb)) = (player_body_before_step, target_body_before_step) {
                self.apply_blazer_collision_velocity(
                    self.player.body_id, self.target.body_id,
                    &pb, &tb, self.ships[self.target.ship_id].mass(),
                );
            }
            apply_collision_between(&mut self.ships, self.player.ship_id, self.target.ship_id);
            let died = self.ships[self.target.ship_id].take_damage(ANDROSYNTH_BLAZER_DAMAGE);
            target_died |= died;
            player_won |= died;
            player_blazer_hit_applied = true;
            self.audio_events.push(AudioEventSnapshot {
                key: "battle-boom-23",
            });
        }
        self.player.special_contacting = player_blazer_hits;

        let target_blazer_hits = self.target.special_active && !self.player.dead && self.androsynth_blazer_hits_other_ship(false);
        if target_blazer_hits && !self.target.special_contacting {
            if let (Some(pb), Some(tb)) = (player_body_before_step, target_body_before_step) {
                self.apply_blazer_collision_velocity(
                    self.target.body_id, self.player.body_id,
                    &tb, &pb, self.ships[self.player.ship_id].mass(),
                );
            }
            apply_collision_between(&mut self.ships, self.player.ship_id, self.target.ship_id);
            let died = self.ships[self.player.ship_id].take_damage(ANDROSYNTH_BLAZER_DAMAGE);
            player_died |= died;
            target_won |= died;
            target_blazer_hit_applied = true;
            self.audio_events.push(AudioEventSnapshot {
                key: "battle-boom-23",
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
                let died = self.ships[self.target.ship_id].take_damage(ANDROSYNTH_BLAZER_DAMAGE);
                target_died |= died;
                player_won |= died;
                self.audio_events.push(AudioEventSnapshot {
                    key: "battle-boom-23",
                });
            }
            if self.target.special_active && !self.player.dead && !target_blazer_hit_applied {
                let died = self.ships[self.player.ship_id].take_damage(ANDROSYNTH_BLAZER_DAMAGE);
                player_died |= died;
                target_won |= died;
                self.audio_events.push(AudioEventSnapshot {
                    key: "battle-boom-23",
                });
            }
        }

        if player_died {
            self.mark_ship_dead(true);
        }
        if target_died {
            self.mark_ship_dead(false);
        }
        if player_won {
            self.audio_events.push(AudioEventSnapshot {
                key: victory_sound_key_for(self.ships[self.player.ship_id].sprite_prefix()),
            });
        }
        if target_won {
            self.audio_events.push(AudioEventSnapshot {
                key: victory_sound_key_for(self.ships[self.target.ship_id].sprite_prefix()),
            });
        }

        let _ = self.matter_world.wrap_body(self.player.body_id, self.width, self.height);
        let _ = self.matter_world.wrap_body(self.target.body_id, self.width, self.height);
    }

    pub fn snapshot(&self) -> BattleSnapshot {
        BattleSnapshot {
            player: self.snapshot_for(&self.player),
            target: self.snapshot_for(&self.target),
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

        if self.ship_state(is_player).special_active
            && self.ships[ship_id].sprite_prefix() == "androsynth-guardian"
        {
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
        let mut commands = self.ships[ship_id].update(
            &input,
            &VelocityVector {
                x: current.vx,
                y: current.vy,
            },
            in_gravity_well,
        );
        if self.ships[ship_id].sprite_prefix() == "androsynth-guardian"
            && input.special
            && self.ships[ship_id].energy() < energy_before
        {
            let should_play_special_sound = !self.ship_state(is_player).special_active;
            if should_play_special_sound {
                self.audio_events.push(AudioEventSnapshot {
                    key: "androsynth-special",
                });
            }
            self.matter_world.set_body_mass(body_id, ANDROSYNTH_BLAZER_MASS);
            let ship_state = self.ship_state_mut(is_player);
            ship_state.special_active = true;
            let energy_wait = self.ships[ship_id].energy_wait();
            self.ships[ship_id].set_energy_counter(energy_wait - 1);
        }
        if self.ship_state(is_player).special_active
            && self.ships[ship_id].sprite_prefix() == "androsynth-guardian"
        {
            let facing = self.ships[ship_id].facing();
            commands.push(PhysicsCommand::SetVelocity {
                vx: facing.cos() * ANDROSYNTH_BLAZER_SPEED,
                vy: facing.sin() * ANDROSYNTH_BLAZER_SPEED,
            });
        }
        if input.weapon
            && self.ships[ship_id].energy() < energy_before
        {
            let facing = self.ships[ship_id].facing();
            let facing_index = radians_to_facing_index(facing);
            let sprite_prefix = self.ships[ship_id].sprite_prefix();
            let projectile_speed = projectile_speed_for(self.ships[ship_id].sprite_prefix());
            let projectile_offset = projectile_offset_for(sprite_prefix);
            let (projectile_raw_vx, projectile_raw_vy) = projectile_velocity_for_facing(facing_index, projectile_speed as i32);
            let spawn_rewind_x = if sprite_prefix == "androsynth-guardian" {
                projectile_raw_vx as f64 / 32.0
            } else {
                0.0
            };
            let spawn_rewind_y = if sprite_prefix == "androsynth-guardian" {
                projectile_raw_vy as f64 / 32.0
            } else {
                0.0
            };
            let projectile_turn_wait = if sprite_prefix == "androsynth-guardian" {
                0
            } else {
                HUMAN_NUKE_TRACK_WAIT - 1
            };
            let projectile_id = self.next_game_object_id();
            self.projectiles.push(ProjectileSnapshot {
                id: projectile_id,
                x: current.x + (facing.cos() * projectile_offset) - spawn_rewind_x,
                y: current.y + (facing.sin() * projectile_offset) - spawn_rewind_y,
                vx: projectile_raw_vx as f64 / 32.0,
                vy: projectile_raw_vy as f64 / 32.0,
                life: projectile_life_for(sprite_prefix) - 1,
                texture_prefix: projectile_texture_prefix_for(sprite_prefix),
                facing,
                acceleration: projectile_acceleration_for(sprite_prefix),
                max_speed: projectile_max_speed_for(sprite_prefix),
                turn_wait: projectile_turn_wait,
                facing_index,
                speed: projectile_speed as i32,
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
                bubble_rng: if sprite_prefix == "androsynth-guardian" {
                    next_androsynth_random(&mut self.bubble_rng_state) as u32
                } else {
                    0
                },
                owner_is_player: is_player,
                target: self.projectile_target_for(is_player),
            });
            let projectile = self.projectiles.last_mut().expect("projectile just pushed");
            set_projectile_velocity_components(projectile, projectile_raw_vx, projectile_raw_vy);
            self.audio_events.push(AudioEventSnapshot {
                key: primary_sound_key_for(sprite_prefix),
            });
        }
        let thrusting = apply_commands(&mut self.matter_world, body_id, commands);

        if is_player {
            self.player.thrusting = thrusting;
        } else {
            self.target.thrusting = thrusting;
        }

        sync_ship_body_angle(&mut self.matter_world, body_id, &self.ships[ship_id]);
    }

    fn step_androsynth_blazer(&mut self, ship_id: usize, body_id: usize, input: ShipInput, is_player: bool) {
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
                vx: facing.cos() * ANDROSYNTH_BLAZER_SPEED,
                vy: facing.sin() * ANDROSYNTH_BLAZER_SPEED,
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
        if self.ships[ship_id].sprite_prefix() != "human-cruiser" {
            return;
        }

        let special_wait = self.ships[ship_id].special_wait();
        if self.ships[ship_id].special_counter() != special_wait {
            return;
        }

        let special_cost = self.ships[ship_id].special_energy_cost();
        let energy_before_refund = self.ships[ship_id].energy() + special_cost;
        let Some(body) = self.matter_world.body_state(ship_state.body_id) else {
            return;
        };

        let target_indexes = self.find_point_defense_targets(is_player, body.x, body.y);
        if target_indexes.is_empty() {
            self.ships[ship_id].set_special_counter(0);
            self.ships[ship_id].set_energy(energy_before_refund);
            return;
        }

        self.audio_events.push(AudioEventSnapshot {
            key: "human-special",
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
            });
        }
    }

    fn find_point_defense_targets(&self, is_player: bool, ship_x: f64, ship_y: f64) -> Vec<usize> {
        self.projectiles
            .iter()
            .enumerate()
            .filter(|(_, projectile)| projectile.owner_is_player != is_player)
            .filter_map(|(index, projectile)| {
                let dx = shortest_wrapped_delta(ship_x, projectile.x, self.width);
                let dy = shortest_wrapped_delta(ship_y, projectile.y, self.height);
                let distance = ((dx * dx) + (dy * dy)).sqrt();
                (distance <= HUMAN_POINT_DEFENSE_RANGE).then_some((index, distance))
            })
            .collect::<Vec<_>>()
            .into_iter()
            .map(|(index, _)| index)
            .collect()
    }

    fn projectile_target_for(&self, is_player: bool) -> ProjectileTarget {
        let ship_id = self.ship_state(is_player).ship_id;
        if self.ships[ship_id].sprite_prefix() == "androsynth-guardian" {
            if is_player && !matches!(self.player_weapon_target, ProjectileTarget::None) {
                return self.player_weapon_target;
            }
            return if is_player {
                ProjectileTarget::TargetShip
            } else {
                ProjectileTarget::PlayerShip
            };
        }
        if is_player {
            self.player_weapon_target
        } else {
            ProjectileTarget::PlayerShip
        }
    }

    fn projectile_hit_target(&self, projectile: &ProjectileSnapshot) -> Option<bool> {
        let (is_player, ship_state) = if projectile.owner_is_player {
            (false, &self.target)
        } else {
            (true, &self.player)
        };
        let body = self.matter_world.body_state(ship_state.body_id)?;
        let logic = &self.ships[ship_state.ship_id];
        let facing = radians_to_facing_index(logic.facing());
        let hit_polygon = logic.hit_polygon_for_state(facing, body.x, body.y, ship_state.special_active);
        if !hit_polygon.is_empty() {
            let projectile_polygon = projectile_hit_polygon(
                projectile.texture_prefix,
                projectile.facing_index,
                projectile.x,
                projectile.y,
            );
            if !projectile_polygon.is_empty() {
                return polygons_intersect(&projectile_polygon, &hit_polygon).then_some(is_player);
            }
            return point_in_polygon(projectile.x, projectile.y, &hit_polygon).then_some(is_player);
        }

        let hit_circle = ship_hit_circle_for(logic.sprite_prefix(), facing);
        let hit_center_x = body.x + hit_circle.offset_x;
        let hit_center_y = body.y + hit_circle.offset_y;
        let dx = shortest_wrapped_delta(projectile.x, hit_center_x, self.width);
        let dy = shortest_wrapped_delta(projectile.y, hit_center_y, self.height);
        ((((dx * dx) + (dy * dy)).sqrt()) <= hit_circle.radius).then_some(is_player)
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
            texture_prefix: ship_texture_prefix(ship.special_active, logic.sprite_prefix()),
        }
    }

    fn androsynth_blazer_hits_other_ship(&self, is_player: bool) -> bool {
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
                ANDROSYNTH_BLAZER_HIT_RADIUS,
            );
        }

        let defender_hit_circle = ship_hit_circle_for(defender_logic.sprite_prefix(), defender_facing);
        let defender_center_x = defender_body.x + defender_hit_circle.offset_x;
        let defender_center_y = defender_body.y + defender_hit_circle.offset_y;
        let wrapped_end_x = start_x + shortest_wrapped_delta(start_x, end_x, self.width);
        let wrapped_end_y = start_y + shortest_wrapped_delta(start_y, end_y, self.height);
        let wrapped_defender_x =
            start_x + shortest_wrapped_delta(start_x, defender_center_x, self.width);
        let wrapped_defender_y =
            start_y + shortest_wrapped_delta(start_y, defender_center_y, self.height);

        segment_hits_circle(
            start_x,
            start_y,
            wrapped_end_x,
            wrapped_end_y,
            wrapped_defender_x,
            wrapped_defender_y,
            defender_hit_circle.radius + ANDROSYNTH_BLAZER_HIT_RADIUS,
        )
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
            ANDROSYNTH_BLAZER_MASS,
            victim_before.x,
            victim_before.y,
            victim_before.vx,
            victim_before.vy,
            victim_mass,
        );
        self.matter_world.set_body_velocity(blazer_body_id, blazer_vx, blazer_vy);
        self.matter_world.set_body_velocity(victim_body_id, victim_vx, victim_vy);
    }

    fn apply_ship_collision_velocity(&mut self) {
        let Some(player_body) = self.matter_world.body_state(self.player.body_id) else {
            return;
        };
        let Some(target_body) = self.matter_world.body_state(self.target.body_id) else {
            return;
        };

        let ((player_vx, player_vy), (target_vx, target_vy)) = resolve_collision_velocity(
            player_body.x,
            player_body.y,
            player_body.vx,
            player_body.vy,
            self.ships[self.player.ship_id].mass(),
            target_body.x,
            target_body.y,
            target_body.vx,
            target_body.vy,
            self.ships[self.target.ship_id].mass(),
        );
        self.matter_world
            .set_body_velocity(self.player.body_id, player_vx, player_vy);
        self.matter_world
            .set_body_velocity(self.target.body_id, target_vx, target_vy);
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
                texture_prefix: "battle-boom",
            });
        }
        self.audio_events.push(AudioEventSnapshot {
            key: "battle-shipdies",
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

fn ship_texture_prefix(special_active: bool, sprite_prefix: &'static str) -> &'static str {
    if special_active && sprite_prefix == "androsynth-guardian" {
        "androsynth-blazer"
    } else {
        sprite_prefix
    }
}

fn projectile_hit_polygon(
    texture_prefix: &str,
    facing: i32,
    center_x: f64,
    center_y: f64,
) -> Vec<HitPolygonPoint> {
    let base_polygon = match texture_prefix {
        "human-saturn" => &[
            HitPolygonPoint { x: 0.0, y: -34.0 },
            HitPolygonPoint { x: 8.0, y: -22.0 },
            HitPolygonPoint { x: 10.0, y: 8.0 },
            HitPolygonPoint { x: 6.0, y: 24.0 },
            HitPolygonPoint { x: 0.0, y: 34.0 },
            HitPolygonPoint { x: -6.0, y: 24.0 },
            HitPolygonPoint { x: -10.0, y: 8.0 },
            HitPolygonPoint { x: -8.0, y: -22.0 },
        ][..],
        "androsynth-bubble" => &[
            HitPolygonPoint { x: 0.0, y: -20.0 },
            HitPolygonPoint { x: 14.0, y: -14.0 },
            HitPolygonPoint { x: 20.0, y: 0.0 },
            HitPolygonPoint { x: 14.0, y: 14.0 },
            HitPolygonPoint { x: 0.0, y: 20.0 },
            HitPolygonPoint { x: -14.0, y: 14.0 },
            HitPolygonPoint { x: -20.0, y: 0.0 },
            HitPolygonPoint { x: -14.0, y: -14.0 },
        ][..],
        _ => return Vec::new(),
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

fn explosion_end_frame_for(texture_prefix: &str) -> i32 {
    match texture_prefix {
        "battle-blast" => GENERIC_BLAST_END_FRAME,
        "human-saturn" => HUMAN_NUKE_IMPACT_END_FRAME,
        "battle-boom" => SHIP_DEATH_EXPLOSION_END_FRAME,
        _ => HUMAN_NUKE_IMPACT_END_FRAME,
    }
}

fn projectile_damage_for(texture_prefix: &str) -> i32 {
    match texture_prefix {
        "androsynth-bubble" => ANDROSYNTH_BUBBLE_DAMAGE,
        "human-saturn" => HUMAN_NUKE_DAMAGE,
        _ => HUMAN_NUKE_DAMAGE,
    }
}

fn projectile_impact_texture_prefix_for(texture_prefix: &str) -> &'static str {
    match texture_prefix {
        "androsynth-bubble" => "battle-blast",
        "human-saturn" => "human-saturn",
        _ => "human-saturn",
    }
}

fn projectile_impact_start_frame_for(texture_prefix: &str) -> i32 {
    match texture_prefix {
        "androsynth-bubble" => GENERIC_BLAST_START_FRAME,
        "human-saturn" => HUMAN_NUKE_IMPACT_START_FRAME,
        _ => HUMAN_NUKE_IMPACT_START_FRAME,
    }
}

fn projectile_impact_sound_key_for(texture_prefix: &str) -> &'static str {
    match texture_prefix {
        "androsynth-bubble" => "battle-boom-23",
        "human-saturn" => "battle-boom-45",
        _ => "battle-boom-45",
    }
}

fn projectile_speed_for(ship_sprite_prefix: &str) -> f64 {
    match ship_sprite_prefix {
        "androsynth-guardian" => ANDROSYNTH_BUBBLE_SPEED,
        "human-cruiser" => HUMAN_NUKE_SPEED,
        _ => HUMAN_NUKE_SPEED,
    }
}

fn projectile_life_for(ship_sprite_prefix: &str) -> i32 {
    match ship_sprite_prefix {
        "androsynth-guardian" => ANDROSYNTH_BUBBLE_LIFE,
        "human-cruiser" => HUMAN_NUKE_LIFE,
        _ => HUMAN_NUKE_LIFE,
    }
}

fn projectile_acceleration_for(ship_sprite_prefix: &str) -> f64 {
    match ship_sprite_prefix {
        "human-cruiser" => HUMAN_NUKE_ACCELERATION,
        _ => 0.0,
    }
}

fn projectile_max_speed_for(ship_sprite_prefix: &str) -> f64 {
    match ship_sprite_prefix {
        "androsynth-guardian" => ANDROSYNTH_BUBBLE_SPEED,
        "human-cruiser" => HUMAN_NUKE_MAX_SPEED,
        _ => HUMAN_NUKE_SPEED,
    }
}

fn projectile_texture_prefix_for(ship_sprite_prefix: &str) -> &'static str {
    match ship_sprite_prefix {
        "androsynth-guardian" => "androsynth-bubble",
        "human-cruiser" => "human-saturn",
        _ => "",
    }
}

fn projectile_offset_for(ship_sprite_prefix: &str) -> f64 {
    match ship_sprite_prefix {
        "androsynth-guardian" => ANDROSYNTH_BUBBLE_OFFSET,
        "human-cruiser" => HUMAN_NUKE_OFFSET,
        _ => 0.0,
    }
}

fn primary_sound_key_for(ship_sprite_prefix: &str) -> &'static str {
    match ship_sprite_prefix {
        "androsynth-guardian" => "androsynth-primary",
        "human-cruiser" => "human-primary",
        _ => "human-primary",
    }
}

fn victory_sound_key_for(ship_sprite_prefix: &str) -> &'static str {
    match ship_sprite_prefix {
        "human-cruiser" => "human-victory",
        _ => "human-victory",
    }
}

fn ship_hit_circle_for(ship_sprite_prefix: &str, facing: i32) -> ShipHitCircle {
    match ship_sprite_prefix {
        "human-cruiser" => {
            const HUMAN_CRUISER_HIT_CIRCLE_OFFSETS: [(f64, f64); 16] = [
                (0.0, -6.0),
                (2.0, -6.0),
                (4.0, -4.0),
                (6.0, -2.0),
                (6.0, 0.0),
                (6.0, 2.0),
                (4.0, 4.0),
                (2.0, 6.0),
                (0.0, 6.0),
                (-2.0, 6.0),
                (-4.0, 4.0),
                (-6.0, 2.0),
                (-6.0, 0.0),
                (-6.0, -2.0),
                (-4.0, -4.0),
                (-2.0, -6.0),
            ];
            let (offset_x, offset_y) =
                HUMAN_CRUISER_HIT_CIRCLE_OFFSETS[facing.rem_euclid(16) as usize];
            ShipHitCircle {
                offset_x,
                offset_y,
                radius: 72.0,
            }
        }
        _ => ShipHitCircle {
            offset_x: 0.0,
            offset_y: 0.0,
            radius: 16.0,
        },
    }
}

fn apply_commands(matter_world: &mut MatterWorld, body_id: usize, commands: Vec<PhysicsCommand>) -> bool {
    let mut thrusting = false;

    for command in commands {
        match command {
            PhysicsCommand::SetVelocity { vx, vy } => {
                thrusting = true;
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

fn step_androsynth_bubble(
    projectile: &mut ProjectileSnapshot,
    player_target_body: Option<MatterBodyState>,
    target_target_body: Option<MatterBodyState>,
) {
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

        projectile.facing_index = if target_distance <= ANDROSYNTH_BUBBLE_DIRECT_TRACK_RANGE {
            desired_facing
        } else if delta_facing <= 8 {
            (current_facing + random_turn).rem_euclid(PROJECTILE_FACINGS as i32)
        } else {
            (current_facing - random_turn).rem_euclid(PROJECTILE_FACINGS as i32)
        };
        projectile.facing = facing_index_to_radians(projectile.facing_index);
        turn_wait = ANDROSYNTH_BUBBLE_TRACK_WAIT;
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

fn wobble_bubble_facing(facing: i32, life: i32) -> i32 {
    if (life / 2) % 2 == 0 {
        (facing + 1).rem_euclid(PROJECTILE_FACINGS as i32)
    } else {
        (facing - 1).rem_euclid(PROJECTILE_FACINGS as i32)
    }
}

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
    use super::{Battle, HUMAN_NUKE_IMPACT_END_FRAME, HUMAN_NUKE_IMPACT_START_FRAME, HUMAN_NUKE_SPEED, ProjectileSnapshot};
    use crate::reference_data;
    use crate::ship_input::ShipInput;
    use crate::ships::{AnyShip, HumanCruiser};
    use crate::traits::ship_trait::Ship;

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
        let projectile_polygon = super::projectile_hit_polygon("androsynth-bubble", 15, 4763.0, 1728.0);
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
        let projectile_polygon = super::projectile_hit_polygon("androsynth-bubble", 0, 5000.0, 4210.0);
        let ship_polygon = ship.hit_polygon(0, 5000.0, 4300.0);

        assert_eq!(super::polygons_intersect(&projectile_polygon, &ship_polygon), false);
    }

    #[test]
    fn human_nuke_polygon_can_overlap_human_cruiser_polygon() {
        let ship = AnyShip::from(HumanCruiser::new());
        let projectile_polygon = super::projectile_hit_polygon("human-saturn", 0, 5000.0, 4300.0);
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
                    (HUMAN_NUKE_IMPACT_START_FRAME..=HUMAN_NUKE_IMPACT_END_FRAME)
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
            ((facing.cos() * HUMAN_NUKE_SPEED).round(), (facing.sin() * HUMAN_NUKE_SPEED).round())
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
            ((facing.cos() * HUMAN_NUKE_SPEED).round(), (facing.sin() * HUMAN_NUKE_SPEED).round())
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
