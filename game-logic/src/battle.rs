use crate::matter_world::{MatterBodyState, MatterWorld};
use crate::physics_command::PhysicsCommand;
use crate::ship_input::ShipInput;
use crate::ships::{apply_collision_between, build_ship, AnyShip};
use crate::velocity_vector::VelocityVector;
use crate::wrap::shortest_wrapped_delta;

const HUMAN_NUKE_SPEED: f64 = 40.0;
const HUMAN_NUKE_ACCELERATION: f64 = 4.0;
const HUMAN_NUKE_MAX_SPEED: f64 = 80.0;
const HUMAN_NUKE_LIFE: i32 = 60;
const HUMAN_NUKE_OFFSET: f64 = 168.0;
const HUMAN_NUKE_TRACK_WAIT: i32 = 3;
const HUMAN_NUKE_DAMAGE: i32 = 4;
const HUMAN_NUKE_IMPACT_START_FRAME: i32 = 16;
const HUMAN_NUKE_IMPACT_END_FRAME: i32 = 24;
const SHIP_DEATH_EXPLOSION_START_FRAME: i32 = 0;
const SHIP_DEATH_EXPLOSION_END_FRAME: i32 = 8;
const HUMAN_POINT_DEFENSE_RANGE: f64 = 400.0;
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
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub crew: i32,
    pub energy: i32,
    pub facing: f64,
    pub thrusting: bool,
    pub dead: bool,
}

#[derive(Clone, Copy)]
pub struct ProjectileSnapshot {
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
    owner_is_player: bool,
    target: ProjectileTarget,
}

#[derive(Clone, Copy)]
pub struct ExplosionSnapshot {
    pub x: f64,
    pub y: f64,
    pub frame_index: i32,
    pub texture_prefix: &'static str,
}

#[derive(Clone, Copy)]
pub struct LaserSnapshot {
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
    ship_id: usize,
    body_id: usize,
    thrusting: bool,
    dead: bool,
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
    planet_x: f64,
    planet_y: f64,
    width: f64,
    height: f64,
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
        let player_body_id = matter_world.create_ship_body(
            player_x,
            player_y,
            player_ship.size(),
            player_ship.mass(),
            0.8,
        );
        ships.push(player_ship);

        let target_ship = build_ship(target_ship_type)
            .ok_or_else(|| format!("unknown ship type: {target_ship_type}"))?;
        let target_ship_id = ships.len();
        let target_body_id = matter_world.create_ship_body(
            target_x,
            target_y,
            target_ship.size(),
            target_ship.mass(),
            0.8,
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
                ship_id: player_ship_id,
                body_id: player_body_id,
                thrusting: false,
                dead: false,
            },
            target: BattleShipState {
                ship_id: target_ship_id,
                body_id: target_body_id,
                thrusting: false,
                dead: false,
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
            planet_x,
            planet_y,
            width,
            height,
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
        let next_body_id = self.matter_world.create_ship_body(
            current.x,
            current.y,
            next_ship.size(),
            next_ship.mass(),
            0.8,
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
                        projectile.facing = facing_index_to_radians(projectile.facing_index);
                        projectile.turn_wait = HUMAN_NUKE_TRACK_WAIT;
                    }
                }
            }
            projectile.speed = (projectile.speed + projectile.acceleration as i32).min(projectile.max_speed as i32);
            let (raw_vx, raw_vy) = projectile_velocity_for_facing(projectile.facing_index, projectile.speed);
            projectile.raw_vx = raw_vx;
            projectile.raw_vy = raw_vy;
            projectile.vx = raw_vx as f64 / 32.0;
            projectile.vy = raw_vy as f64 / 32.0;
            projectile.x += (raw_vx / 32) as f64;
            projectile.y += (raw_vy / 32) as f64;
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
        let mut player_died = false;
        let mut target_died = false;
        let mut player_won = false;
        let mut target_won = false;

        for (index, projectile) in self.projectiles.iter().enumerate() {
            let Some((is_player, hit_center_x, hit_center_y, hit_radius)) =
                self.projectile_hit_target(projectile.target)
            else {
                continue;
            };

            let dx = shortest_wrapped_delta(projectile.x, hit_center_x, self.width);
            let dy = shortest_wrapped_delta(projectile.y, hit_center_y, self.height);
            if ((dx * dx) + (dy * dy)).sqrt() <= hit_radius {
                if is_player {
                    let died = self.ships[self.player.ship_id].take_damage(HUMAN_NUKE_DAMAGE);
                    player_died |= died;
                    target_won |= died && !projectile.owner_is_player;
                } else {
                    let died = self.ships[self.target.ship_id].take_damage(HUMAN_NUKE_DAMAGE);
                    target_died |= died;
                    player_won |= died && projectile.owner_is_player;
                }
                self.explosions.push(ExplosionSnapshot {
                    x: projectile.x,
                    y: projectile.y,
                    frame_index: HUMAN_NUKE_IMPACT_START_FRAME,
                    texture_prefix: "human-saturn",
                });
                self.audio_events.push(AudioEventSnapshot {
                    key: "battle-boom-45",
                });
                hit_projectile_indexes.push(index);
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

        for index in hit_projectile_indexes.into_iter().rev() {
            self.projectiles.remove(index);
        }

        self.projectiles.retain(|projectile| projectile.life >= 0);

        let state = self.matter_world.step(delta);

        for collision in state.collisions {
            let ids = [collision.body_a, collision.body_b];
            if !ids.contains(&self.player.body_id) || !ids.contains(&self.target.body_id) {
                continue;
            }

            apply_collision_between(&mut self.ships, self.player.ship_id, self.target.ship_id);
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

        let Some(body) = self.matter_world.body_state(body_id) else {
            return;
        };

        self.apply_gravity(ship_id, body_id, body);
        let current = self
            .matter_world
            .body_state(body_id)
            .unwrap_or(body);
        let in_gravity_well = self.in_gravity_well(current.x, current.y);
        let weapon_counter_before = self.ships[ship_id].weapon_counter();
        let commands = self.ships[ship_id].update(
            &input,
            &VelocityVector {
                x: current.vx,
                y: current.vy,
            },
            in_gravity_well,
        );
        if input.weapon
            && weapon_counter_before == 0
            && self.ships[ship_id].weapon_counter() > 0
        {
            let facing = self.ships[ship_id].facing();
            let facing_index = radians_to_facing_index(facing);
            let projectile_speed = projectile_speed_for(self.ships[ship_id].sprite_prefix());
            let projectile_offset = projectile_offset_for(self.ships[ship_id].sprite_prefix());
            let (projectile_raw_vx, projectile_raw_vy) = projectile_velocity_for_facing(facing_index, projectile_speed as i32);
            self.projectiles.push(ProjectileSnapshot {
                x: current.x + (facing.cos() * projectile_offset),
                y: current.y + (facing.sin() * projectile_offset),
                vx: projectile_raw_vx as f64 / 32.0,
                vy: projectile_raw_vy as f64 / 32.0,
                life: projectile_life_for(self.ships[ship_id].sprite_prefix()) - 1,
                texture_prefix: projectile_texture_prefix_for(self.ships[ship_id].sprite_prefix()),
                facing,
                acceleration: projectile_acceleration_for(self.ships[ship_id].sprite_prefix()),
                max_speed: projectile_max_speed_for(self.ships[ship_id].sprite_prefix()),
                turn_wait: HUMAN_NUKE_TRACK_WAIT - 1,
                facing_index,
                speed: projectile_speed as i32,
                raw_vx: projectile_raw_vx,
                raw_vy: projectile_raw_vy,
                owner_is_player: is_player,
                target: self.projectile_target_for(is_player),
            });
            self.audio_events.push(AudioEventSnapshot {
                key: primary_sound_key_for(self.ships[ship_id].sprite_prefix()),
            });
        }
        let thrusting = apply_commands(&mut self.matter_world, body_id, commands);

        if is_player {
            self.player.thrusting = thrusting;
        } else {
            self.target.thrusting = thrusting;
        }
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
            self.lasers.push(LaserSnapshot {
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
        if is_player {
            self.player_weapon_target
        } else {
            ProjectileTarget::PlayerShip
        }
    }

    fn projectile_hit_target(&self, target: ProjectileTarget) -> Option<(bool, f64, f64, f64)> {
        let (is_player, ship_state) = match target {
            ProjectileTarget::PlayerShip => (true, &self.player),
            ProjectileTarget::TargetShip => (false, &self.target),
            ProjectileTarget::None | ProjectileTarget::Point { .. } => return None,
        };
        let body = self.matter_world.body_state(ship_state.body_id)?;
        let logic = &self.ships[ship_state.ship_id];
        let facing = radians_to_facing_index(logic.facing());
        let hit_circle = ship_hit_circle_for(logic.sprite_prefix(), facing);
        Some((
            is_player,
            body.x + hit_circle.offset_x,
            body.y + hit_circle.offset_y,
            hit_circle.radius,
        ))
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
            x: body.x,
            y: body.y,
            vx: body.vx,
            vy: body.vy,
            crew: logic.crew(),
            energy: logic.energy(),
            facing: logic.facing(),
            thrusting: ship.thrusting,
            dead: ship.dead,
        }
    }

    fn ship_state(&self, is_player: bool) -> &BattleShipState {
        if is_player {
            &self.player
        } else {
            &self.target
        }
    }

    fn mark_ship_dead(&mut self, is_player: bool) {
        let ship = if is_player {
            &mut self.player
        } else {
            &mut self.target
        };

        if ship.dead {
            return;
        }

        if let Some(body) = self.matter_world.body_state(ship.body_id) {
            self.explosions.push(ExplosionSnapshot {
                x: body.x,
                y: body.y,
                frame_index: SHIP_DEATH_EXPLOSION_START_FRAME,
                texture_prefix: "battle-boom",
            });
        }
        self.audio_events.push(AudioEventSnapshot {
            key: "battle-shipdies",
        });

        ship.dead = true;
        ship.thrusting = false;
        self.matter_world.disable_body(ship.body_id);
    }
}

fn explosion_end_frame_for(texture_prefix: &str) -> i32 {
    match texture_prefix {
        "human-saturn" => HUMAN_NUKE_IMPACT_END_FRAME,
        "battle-boom" => SHIP_DEATH_EXPLOSION_END_FRAME,
        _ => HUMAN_NUKE_IMPACT_END_FRAME,
    }
}

fn projectile_speed_for(ship_sprite_prefix: &str) -> f64 {
    match ship_sprite_prefix {
        "human-cruiser" => HUMAN_NUKE_SPEED,
        _ => HUMAN_NUKE_SPEED,
    }
}

fn projectile_life_for(ship_sprite_prefix: &str) -> i32 {
    match ship_sprite_prefix {
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
        "human-cruiser" => HUMAN_NUKE_MAX_SPEED,
        _ => HUMAN_NUKE_SPEED,
    }
}

fn projectile_texture_prefix_for(ship_sprite_prefix: &str) -> &'static str {
    match ship_sprite_prefix {
        "human-cruiser" => "human-saturn",
        _ => "",
    }
}

fn projectile_offset_for(ship_sprite_prefix: &str) -> f64 {
    match ship_sprite_prefix {
        "human-cruiser" => HUMAN_NUKE_OFFSET,
        _ => 0.0,
    }
}

fn primary_sound_key_for(ship_sprite_prefix: &str) -> &'static str {
    match ship_sprite_prefix {
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

#[cfg(test)]
mod tests {
    use super::{Battle, HUMAN_NUKE_IMPACT_END_FRAME, HUMAN_NUKE_IMPACT_START_FRAME, HUMAN_NUKE_SPEED};
    use crate::reference_data;
    use crate::ship_input::ShipInput;

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
