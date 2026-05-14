use crate::matter_world::{MatterBodyState, MatterWorld};
use crate::physics_command::PhysicsCommand;
use crate::ship_input::ShipInput;
use crate::ships::{AnyShip, apply_collision_between, build_ship};
use crate::traits::game_object::GameObject;
use crate::traits::ship_trait::{
    CrewDrainTransferSpec, CrewToEnergySpec, HitPolygonPoint, InstantLaserSpec, PlanetHarvestSpec,
    PrimaryProjectileSpec, ProjectileBehaviorSpec, ProjectileCollisionSpec, ProjectileSpawnSpec,
    ProjectileTargetMode, ProjectileVolleySpec, SecondaryProjectileSpec, SelfDestructSpec,
    SoundOnlySpec, SpecialAbilitySpec, TeleportSpecialSpec, TransformSpec,
};
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
const PROJECTILE_HIT_FALLBACK_PADDING: f64 = 0.0;
const PROJECTILE_FACINGS: f64 = 16.0;
const EXPLOSION_TEXTURE_BATTLE_BOOM: &str = "battle-boom";
const AUDIO_SHIP_DEATH: &str = "battle-shipdies";
const AUDIO_ORZ_INTRUDER: &str = "orz-intruder";
const AUDIO_ORZ_ZAP: &str = "orz-zap";
const AUDIO_ORZ_ARGH: &str = "orz-argh";
const ORZ_MARINE_WAIT_TICKS: i32 = 12;
const ORZ_MARINE_DEATH_CHANCE_MAX: i32 = 256 / 16;
const ORZ_MARINE_DAMAGE_CHANCE_MAX: i32 = (256 / 2) + (256 / 16);
const ORZ_MARINE_DAMAGE: i32 = 1;
const ORZ_MARINE_RETURN_SPEED: i32 = 10;
const ORZ_MARINE_RETURN_TURN_WAIT: i32 = 3;
const PLANET_HIT_RADIUS: f64 = 220.0;
const SC2_SINE_TABLE: [i32; 64] = [
    -16384, -16305, -16069, -15679, -15137, -14449, -13623, -12665, -11585, -10394, -9102, -7723,
    -6270, -4756, -3196, -1606, 0, 1606, 3196, 4756, 6270, 7723, 9102, 10394, 11585, 12665, 13623,
    14449, 15137, 15679, 16069, 16305, 16384, 16305, 16069, 15679, 15137, 14449, 13623, 12665,
    11585, 10394, 9102, 7723, 6270, 4756, 3196, 1606, 0, -1606, -3196, -4756, -6270, -7723, -9102,
    -10394, -11585, -12665, -13623, -14449, -15137, -15679, -16069, -16305,
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
    pub turret_facing: f64,
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
    previous_x: f64,
    previous_y: f64,
    pub vx: f64,
    pub vy: f64,
    inherited_vx: f64,
    inherited_vy: f64,
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
    owner_ship_id: u64,
    owner_is_player: bool,
    target: ProjectileTarget,
    marine_returning: bool,
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
    pub ships: Vec<BattleShipSnapshot>,
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
    Ship { id: u64 },
}

#[derive(Clone, Copy)]
enum SpecialTarget {
    None,
    Point { x: f64, y: f64 },
    Ship { id: u64 },
}

struct MarineBoarderState {
    defender_ship_id: u64,
    owner_ship_id: u64,
    ticks_until_next_roll: i32,
}

struct LaserRay {
    start_x: f64,
    start_y: f64,
    end_x: f64,
    end_y: f64,
}

struct PhysicsCollisionContext<'a> {
    player_blazer_spec: Option<crate::traits::ship_trait::BlazerSpecialSpec>,
    target_blazer_spec: Option<crate::traits::ship_trait::BlazerSpecialSpec>,
    player_body_before: Option<MatterBodyState>,
    target_body_before: Option<MatterBodyState>,
    player_blazer_hit: BlazerHitResult,
    target_blazer_hit: BlazerHitResult,
    bodies_after: &'a [MatterBodyState],
}

#[derive(Clone, Copy, Default)]
struct BlazerHitResult {
    hits: bool,
    applied: bool,
}

struct BlazerCollisionInput<'a> {
    blazer_mass: f64,
    blazer_body_id: usize,
    victim_body_id: usize,
    blazer_before: &'a MatterBodyState,
    victim_before: &'a MatterBodyState,
    victim_mass: f64,
}

#[derive(Default)]
struct BattleOutcome {
    died_ship_ids: Vec<u64>,
    winner_ship_ids: Vec<u64>,
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
    primary_mount_facing: f64,
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
    additional_active_ships: Vec<BattleShipState>,
    player_input: ShipInput,
    target_input: ShipInput,
    additional_inputs: Vec<ShipInput>,
    additional_meteor_contacts: Vec<Vec<bool>>,
    additional_weapon_targets: Vec<ProjectileTarget>,
    additional_special_targets: Vec<SpecialTarget>,
    queued_target_weapon: bool,
    player_weapon_target: ProjectileTarget,
    target_weapon_target: ProjectileTarget,
    player_special_target: SpecialTarget,
    target_special_target: SpecialTarget,
    marine_boarders: Vec<MarineBoarderState>,
    bubble_rng_state: u32,
    planet_x: f64,
    planet_y: f64,
    width: f64,
    height: f64,
    next_game_object_id: u64,
}

#[derive(Clone, Copy)]
struct Segment {
    start_x: f64,
    start_y: f64,
    end_x: f64,
    end_y: f64,
}

#[derive(Clone, Copy)]
struct CollisionBody {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    mass: f64,
}

#[derive(Clone, Copy)]
struct ActiveShipControl {
    game_object_id: u64,
    ship_id: usize,
    body_id: usize,
    input: ShipInput,
    role: Option<bool>,
}

impl Battle {
    fn active_ship_states(&self) -> Vec<&BattleShipState> {
        let mut ships = Vec::with_capacity(2 + self.additional_active_ships.len());
        ships.push(&self.player);
        ships.push(&self.target);
        ships.extend(self.additional_active_ships.iter());
        ships
    }

    pub fn active_ship_ids(&self) -> Vec<u64> {
        self.active_ship_states()
            .into_iter()
            .map(|ship| ship.game_object_id())
            .collect()
    }

    fn is_player_ship_id(&self, ship_id: u64) -> bool {
        self.player.game_object_id() == ship_id
    }

    fn opposing_active_ship_id(&self, ship_id: u64) -> Option<u64> {
        self.active_ship_states()
            .into_iter()
            .find(|state| state.game_object_id() != ship_id && !state.dead)
            .map(|state| state.game_object_id())
    }

    fn active_ship_controls(&self) -> Vec<ActiveShipControl> {
        let mut controls = Vec::with_capacity(2 + self.additional_active_ships.len());
        controls.push(ActiveShipControl {
            game_object_id: self.player.game_object_id(),
            ship_id: self.player.ship_id,
            body_id: self.player.body_id,
            input: self.player_input,
            role: Some(true),
        });
        controls.push(ActiveShipControl {
            game_object_id: self.target.game_object_id(),
            ship_id: self.target.ship_id,
            body_id: self.target.body_id,
            input: ShipInput {
                weapon: self.target_input.weapon || self.queued_target_weapon,
                ..self.target_input
            },
            role: Some(false),
        });
        controls.extend(
            self.additional_active_ships
                .iter()
                .zip(self.additional_inputs.iter())
                .map(|(ship, input)| ActiveShipControl {
                    game_object_id: ship.game_object_id(),
                    ship_id: ship.ship_id,
                    body_id: ship.body_id,
                    input: *input,
                    role: None,
                }),
        );
        controls
    }

    fn ship_state_by_game_object_id(&self, ship_id: u64) -> Option<&BattleShipState> {
        if self.player.game_object_id() == ship_id {
            Some(&self.player)
        } else if self.target.game_object_id() == ship_id {
            Some(&self.target)
        } else {
            self.additional_active_ships
                .iter()
                .find(|ship| ship.game_object_id() == ship_id)
        }
    }

    fn ship_state_by_game_object_id_mut(&mut self, ship_id: u64) -> Option<&mut BattleShipState> {
        if self.player.game_object_id() == ship_id {
            Some(&mut self.player)
        } else if self.target.game_object_id() == ship_id {
            Some(&mut self.target)
        } else {
            self.additional_active_ships
                .iter_mut()
                .find(|ship| ship.game_object_id() == ship_id)
        }
    }

    fn ship_state_by_body_id(&self, body_id: usize) -> Option<&BattleShipState> {
        self.active_ship_states()
            .into_iter()
            .find(|ship| ship.body_id == body_id)
    }

    fn active_enemy_target_for_ship(&self, ship_id: u64) -> Option<u64> {
        self.active_ship_states()
            .into_iter()
            .find(|state| state.game_object_id() != ship_id && self.ship_is_targetable_by_id(state.game_object_id()))
            .map(|state| state.game_object_id())
    }

    fn record_ship_destroyed(&self, outcome: &mut BattleOutcome, ship_id: u64) {
        if !outcome.died_ship_ids.contains(&ship_id) {
            outcome.died_ship_ids.push(ship_id);
        }
    }

    fn record_ship_victory(&self, outcome: &mut BattleOutcome, ship_id: u64) {
        if !outcome.winner_ship_ids.contains(&ship_id) {
            outcome.winner_ship_ids.push(ship_id);
        }
    }

    fn mark_ship_dead_by_id(&mut self, ship_id: u64) {
        if self.player.game_object_id() == ship_id {
            self.mark_ship_dead(true);
        } else if self.target.game_object_id() == ship_id {
            self.mark_ship_dead(false);
        } else if let Some(index) = self
            .additional_active_ships
            .iter()
            .position(|ship| ship.game_object_id() == ship_id)
        {
            self.mark_additional_ship_dead(index);
        }
    }

    fn apply_damage_to_ship_by_id(
        &mut self,
        defender_ship_id: u64,
        attacker_ship_id: u64,
        damage: i32,
        outcome: &mut BattleOutcome,
    ) -> bool {
        let Some(defender_logic_ship_id) = self
            .ship_state_by_game_object_id(defender_ship_id)
            .map(|state| state.ship_id)
        else {
            return false;
        };
        let died = self.ships[defender_logic_ship_id].take_damage(damage);
        if died {
            self.record_ship_destroyed(outcome, defender_ship_id);
            self.record_ship_victory(outcome, attacker_ship_id);
        }
        died
    }

    #[allow(clippy::too_many_arguments)]
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
        let player_body_id =
            create_ship_body_for(&mut matter_world, &player_ship, player_x, player_y, false);
        ships.push(player_ship);

        let target_ship = build_ship(target_ship_type)
            .ok_or_else(|| format!("unknown ship type: {target_ship_type}"))?;
        let target_ship_id = ships.len();
        let target_body_id =
            create_ship_body_for(&mut matter_world, &target_ship, target_x, target_y, false);
        ships.push(target_ship);
        let player_initial_mount_facing = ships[player_ship_id].facing();
        let target_initial_mount_facing = ships[target_ship_id].facing();

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
                primary_mount_facing: player_initial_mount_facing,
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
                primary_mount_facing: target_initial_mount_facing,
            },
            additional_active_ships: Vec::new(),
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
            additional_inputs: Vec::new(),
            additional_meteor_contacts: Vec::new(),
            additional_weapon_targets: Vec::new(),
            additional_special_targets: Vec::new(),
            queued_target_weapon: false,
            player_weapon_target: ProjectileTarget::None,
            target_weapon_target: ProjectileTarget::None,
            player_special_target: SpecialTarget::None,
            target_special_target: SpecialTarget::None,
            marine_boarders: Vec::new(),
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

    pub fn set_input_for_ship(&mut self, ship_id: u64, input: ShipInput) -> Result<(), String> {
        if self.player.game_object_id() == ship_id {
            self.player_input = input;
            Ok(())
        } else if self.target.game_object_id() == ship_id {
            self.target_input = input;
            Ok(())
        } else if let Some(index) = self
            .additional_active_ships
            .iter()
            .position(|ship| ship.game_object_id() == ship_id)
        {
            self.additional_inputs[index] = input;
            Ok(())
        } else {
            Err(format!("unknown ship id: {ship_id}"))
        }
    }

    pub fn add_active_ship(&mut self, ship_type: &str, x: f64, y: f64) -> Result<u64, String> {
        let ship = build_ship(ship_type).ok_or_else(|| format!("unknown ship type: {ship_type}"))?;
        let ship_id = self.ships.len();
        let body_id = create_ship_body_for(&mut self.matter_world, &ship, x, y, false);
        let primary_mount_facing = ship.facing();
        self.ships.push(ship);
        let game_object_id = self.next_game_object_id();
        self.additional_active_ships.push(BattleShipState {
            id: game_object_id,
            ship_id,
            body_id,
            thrusting: false,
            dead: false,
            special_active: false,
            special_contacting: false,
            previous_x: x,
            previous_y: y,
            primary_mount_facing,
        });
        self.additional_inputs.push(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: false,
            special: false,
        });
        self.additional_meteor_contacts
            .push(vec![false; self.meteors.len()]);
        self.additional_weapon_targets.push(ProjectileTarget::None);
        self.additional_special_targets.push(SpecialTarget::None);
        Ok(game_object_id)
    }

    pub fn trigger_target_weapon(&mut self) {
        self.queued_target_weapon = true;
    }

    pub fn set_player_weapon_target_point(&mut self, x: f64, y: f64) {
        self.player_weapon_target = ProjectileTarget::Point { x, y };
    }

    pub fn set_player_weapon_target_ship(&mut self) {
        self.player_weapon_target = ProjectileTarget::Ship { id: self.target.id };
    }

    pub fn clear_player_weapon_target(&mut self) {
        self.player_weapon_target = ProjectileTarget::None;
    }

    pub fn set_target_weapon_target_point(&mut self, x: f64, y: f64) {
        self.target_weapon_target = ProjectileTarget::Point { x, y };
    }

    pub fn set_target_weapon_target_ship(&mut self) {
        self.target_weapon_target = ProjectileTarget::Ship { id: self.player.id };
    }

    pub fn clear_target_weapon_target(&mut self) {
        self.target_weapon_target = ProjectileTarget::None;
    }

    pub fn set_player_special_target_point(&mut self, x: f64, y: f64) {
        self.player_special_target = SpecialTarget::Point { x, y };
    }

    pub fn set_player_special_target_ship(&mut self) {
        self.player_special_target = SpecialTarget::Ship { id: self.target.id };
    }

    pub fn clear_player_special_target(&mut self) {
        self.player_special_target = SpecialTarget::None;
    }

    pub fn set_target_special_target_point(&mut self, x: f64, y: f64) {
        self.target_special_target = SpecialTarget::Point { x, y };
    }

    pub fn set_target_special_target_ship(&mut self) {
        self.target_special_target = SpecialTarget::Ship { id: self.player.id };
    }

    pub fn clear_target_special_target(&mut self) {
        self.target_special_target = SpecialTarget::None;
    }

    pub fn set_weapon_target_ship_for(
        &mut self,
        ship_id: u64,
        target_ship_id: u64,
    ) -> Result<(), String> {
        if self.player.game_object_id() == ship_id {
            self.player_weapon_target = ProjectileTarget::Ship { id: target_ship_id };
            Ok(())
        } else if self.target.game_object_id() == ship_id {
            self.target_weapon_target = ProjectileTarget::Ship { id: target_ship_id };
            Ok(())
        } else if let Some(index) = self
            .additional_active_ships
            .iter()
            .position(|ship| ship.game_object_id() == ship_id)
        {
            self.additional_weapon_targets[index] = ProjectileTarget::Ship { id: target_ship_id };
            Ok(())
        } else {
            Err(format!("unknown ship id: {ship_id}"))
        }
    }

    pub fn set_weapon_target_point_for(
        &mut self,
        ship_id: u64,
        x: f64,
        y: f64,
    ) -> Result<(), String> {
        if self.player.game_object_id() == ship_id {
            self.player_weapon_target = ProjectileTarget::Point { x, y };
            Ok(())
        } else if self.target.game_object_id() == ship_id {
            self.target_weapon_target = ProjectileTarget::Point { x, y };
            Ok(())
        } else if let Some(index) = self
            .additional_active_ships
            .iter()
            .position(|ship| ship.game_object_id() == ship_id)
        {
            self.additional_weapon_targets[index] = ProjectileTarget::Point { x, y };
            Ok(())
        } else {
            Err(format!("unknown ship id: {ship_id}"))
        }
    }

    pub fn clear_weapon_target_for(&mut self, ship_id: u64) -> Result<(), String> {
        if self.player.game_object_id() == ship_id {
            self.player_weapon_target = ProjectileTarget::None;
            Ok(())
        } else if self.target.game_object_id() == ship_id {
            self.target_weapon_target = ProjectileTarget::None;
            Ok(())
        } else if let Some(index) = self
            .additional_active_ships
            .iter()
            .position(|ship| ship.game_object_id() == ship_id)
        {
            self.additional_weapon_targets[index] = ProjectileTarget::None;
            Ok(())
        } else {
            Err(format!("unknown ship id: {ship_id}"))
        }
    }

    pub fn set_special_target_ship_for(
        &mut self,
        ship_id: u64,
        target_ship_id: u64,
    ) -> Result<(), String> {
        if self.player.game_object_id() == ship_id {
            self.player_special_target = SpecialTarget::Ship { id: target_ship_id };
            Ok(())
        } else if self.target.game_object_id() == ship_id {
            self.target_special_target = SpecialTarget::Ship { id: target_ship_id };
            Ok(())
        } else if let Some(index) = self
            .additional_active_ships
            .iter()
            .position(|ship| ship.game_object_id() == ship_id)
        {
            self.additional_special_targets[index] = SpecialTarget::Ship { id: target_ship_id };
            Ok(())
        } else {
            Err(format!("unknown ship id: {ship_id}"))
        }
    }

    pub fn clear_special_target_for(&mut self, ship_id: u64) -> Result<(), String> {
        if self.player.game_object_id() == ship_id {
            self.player_special_target = SpecialTarget::None;
            Ok(())
        } else if self.target.game_object_id() == ship_id {
            self.target_special_target = SpecialTarget::None;
            Ok(())
        } else if let Some(index) = self
            .additional_active_ships
            .iter()
            .position(|ship| ship.game_object_id() == ship_id)
        {
            self.additional_special_targets[index] = SpecialTarget::None;
            Ok(())
        } else {
            Err(format!("unknown ship id: {ship_id}"))
        }
    }

    pub fn set_special_target_point_for(
        &mut self,
        ship_id: u64,
        x: f64,
        y: f64,
    ) -> Result<(), String> {
        if self.player.game_object_id() == ship_id {
            self.player_special_target = SpecialTarget::Point { x, y };
            Ok(())
        } else if self.target.game_object_id() == ship_id {
            self.target_special_target = SpecialTarget::Point { x, y };
            Ok(())
        } else if let Some(index) = self
            .additional_active_ships
            .iter()
            .position(|ship| ship.game_object_id() == ship_id)
        {
            self.additional_special_targets[index] = SpecialTarget::Point { x, y };
            Ok(())
        } else {
            Err(format!("unknown ship id: {ship_id}"))
        }
    }

    pub fn switch_player_ship(&mut self, ship_type: &str) -> Result<(), String> {
        let current = self
            .matter_world
            .body_state(self.player.body_id)
            .ok_or_else(|| "missing player body state".to_string())?;
        let next_ship =
            build_ship(ship_type).ok_or_else(|| format!("unknown ship type: {ship_type}"))?;
        let next_body_id = create_ship_body_for(
            &mut self.matter_world,
            &next_ship,
            current.x,
            current.y,
            self.player.special_active,
        );
        self.matter_world
            .set_body_velocity(next_body_id, current.vx, current.vy);
        self.matter_world.disable_body(self.player.body_id);
        self.ships[self.player.ship_id] = next_ship;
        self.player.body_id = next_body_id;
        self.player.thrusting = false;
        self.player.dead = false;
        self.player.special_active = false;
        self.player.special_contacting = false;
        self.player.primary_mount_facing = self.ships[self.player.ship_id].facing();
        self.player_input = ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: false,
            special: false,
        };
        self.player_weapon_target = ProjectileTarget::None;
        self.player_special_target = SpecialTarget::None;
        Ok(())
    }

    pub fn switch_target_ship(&mut self, ship_type: &str) -> Result<(), String> {
        let current = self
            .matter_world
            .body_state(self.target.body_id)
            .ok_or_else(|| "missing target body state".to_string())?;
        let next_ship =
            build_ship(ship_type).ok_or_else(|| format!("unknown ship type: {ship_type}"))?;
        let next_body_id = create_ship_body_for(
            &mut self.matter_world,
            &next_ship,
            current.x,
            current.y,
            self.target.special_active,
        );
        self.matter_world
            .set_body_velocity(next_body_id, current.vx, current.vy);
        self.matter_world.disable_body(self.target.body_id);
        self.ships[self.target.ship_id] = next_ship;
        self.target.body_id = next_body_id;
        self.target.thrusting = false;
        self.target.dead = false;
        self.target.special_active = false;
        self.target.special_contacting = false;
        self.target.primary_mount_facing = self.ships[self.target.ship_id].facing();
        self.target_input = ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: false,
            special: false,
        };
        self.target_weapon_target = ProjectileTarget::None;
        self.target_special_target = SpecialTarget::None;
        Ok(())
    }

    pub fn switch_ship_for(&mut self, ship_id: u64, ship_type: &str) -> Result<(), String> {
        if self.player.game_object_id() == ship_id {
            self.switch_player_ship(ship_type)
        } else if self.target.game_object_id() == ship_id {
            self.switch_target_ship(ship_type)
        } else {
            Err(format!("unknown ship id: {ship_id}"))
        }
    }

    pub fn tick(&mut self, delta: f64) {
        self.audio_events.clear();
        self.lasers.clear();

        self.tick_explosions();
        self.tick_projectiles();
        self.step_meteors();
        let active_ship_controls = self.step_ships();
        self.handle_point_defense(active_ship_controls);

        let mut outcome = BattleOutcome::default();
        self.resolve_projectile_hits(&mut outcome);
        self.tick_marine_boarders(&mut outcome);
        self.handle_meteor_collisions();
        self.resolve_ship_collisions(delta, &mut outcome);
        self.resolve_death_and_victory(&outcome);

        self.wrap_bodies();
    }

    fn tick_explosions(&mut self) {
        for explosion in &mut self.explosions {
            explosion.frame_index += 1;
        }
        self.explosions
            .retain(|explosion| explosion.frame_index <= explosion.end_frame);
    }

    fn tick_projectiles(&mut self) {
        let player_body = self.matter_world.body_state(self.player.body_id);
        let target_body = self.matter_world.body_state(self.target.body_id);
        let active_ship_target_bodies: Vec<(u64, Option<MatterBodyState>)> = self
            .active_ship_states()
            .into_iter()
            .map(|ship| (ship.game_object_id(), self.matter_world.body_state(ship.body_id)))
            .collect();
        let mut returning_arrivals = Vec::new();

        for projectile in &mut self.projectiles {
            projectile.previous_x = projectile.x;
            projectile.previous_y = projectile.y;
            let target_position =
                projectile_target_position_from_states(projectile.target, &active_ship_target_bodies);
            match projectile.behavior {
                ProjectileBehaviorSpec::WobbleTracking { .. } => {
                    step_wobble_tracking_projectile(projectile, target_position);
                }
                ProjectileBehaviorSpec::Tracking => {
                    track_projectile(projectile, target_position);
                    projectile.speed = (projectile.speed + projectile.acceleration as i32)
                        .min(projectile.max_speed as i32);
                    let (raw_vx, raw_vy) =
                        projectile_velocity_for_facing(projectile.facing_index, projectile.speed);
                    set_projectile_velocity_components(projectile, raw_vx, raw_vy);
                    advance_projectile_position(projectile);
                    if projectile.texture_prefix != "orz-turret" {
                        projectile.life -= 1;
                    }
                }
                ProjectileBehaviorSpec::Straight => {
                    projectile.speed = (projectile.speed + projectile.acceleration as i32)
                        .min(projectile.max_speed as i32);
                    let (raw_vx, raw_vy) =
                        projectile_velocity_for_facing(projectile.facing_index, projectile.speed);
                    set_projectile_velocity_components(projectile, raw_vx, raw_vy);
                    advance_projectile_position(projectile);
                    if projectile.texture_prefix != "orz-turret" {
                        projectile.life -= 1;
                    }
                }
            }
        }

        for projectile in &mut self.projectiles {
            if projectile.texture_prefix == "orz-turret"
                && projectile_hits_planet(
                    projectile.x,
                    projectile.y,
                    self.planet_x,
                    self.planet_y,
                    self.width,
                    self.height,
                    PLANET_HIT_RADIUS,
                )
            {
                projectile.life = -1;
                self.audio_events.push(AudioEventSnapshot {
                    key: AUDIO_ORZ_ARGH,
                });
            }

            if projectile.marine_returning {
                let owner_body = if projectile.owner_is_player {
                    player_body
                } else {
                    target_body
                };
                if let Some(owner) = owner_body {
                    let dx = shortest_wrapped_delta(projectile.x, owner.x, self.width);
                    let dy = shortest_wrapped_delta(projectile.y, owner.y, self.height);
                    if ((dx * dx) + (dy * dy)).sqrt() <= 40.0 {
                        projectile.life = -1;
                        returning_arrivals.push(projectile.owner_ship_id);
                    }
                }
            }
        }

        for owner_ship_id in returning_arrivals {
            self.increment_ship_crew_by_ship_id(owner_ship_id, 1);
        }
    }

    fn step_ships(&mut self) -> Vec<ActiveShipControl> {
        let active_ship_controls = self.active_ship_controls();
        for control in active_ship_controls.iter().copied() {
            self.step_ship(control);
        }
        self.queued_target_weapon = false;
        active_ship_controls
    }

    fn handle_point_defense(&mut self, active_ship_controls: Vec<ActiveShipControl>) {
        for control in active_ship_controls {
            if let Some(is_player) = control.role {
                self.handle_human_point_defense(is_player, control.input);
            }
        }
    }

    fn wrap_bodies(&mut self) {
        let body_ids: Vec<usize> = self
            .active_ship_states()
            .into_iter()
            .map(|ship| ship.body_id)
            .collect();
        for body_id in body_ids {
            let _ = self.matter_world.wrap_body(body_id, self.width, self.height);
        }
    }

    fn resolve_projectile_hits(&mut self, outcome: &mut BattleOutcome) {
        struct ProjectileHit {
            index: usize,
            defender_ship_id: u64,
            owner_ship_id: u64,
            owner_is_player: bool,
            is_orz_marine: bool,
            marine_returning: bool,
            damage: i32,
            x: f64,
            y: f64,
            impact_start_frame: i32,
            impact_end_frame: i32,
            impact_texture_prefix: &'static str,
            impact_sound_key: &'static str,
        }

        let hits: Vec<ProjectileHit> = self
            .projectiles
            .iter()
            .enumerate()
            .filter_map(|(index, projectile)| {
                let defender_ship_id = if projectile.marine_returning {
                    if self.projectile_hits_ship(projectile, projectile.owner_is_player) {
                        projectile.owner_ship_id
                    } else {
                        return None;
                    }
                } else {
                    self.projectile_hit_target(projectile)?
                };
                Some(ProjectileHit {
                    index,
                    defender_ship_id,
                    owner_ship_id: projectile.owner_ship_id,
                    owner_is_player: projectile.owner_is_player,
                    is_orz_marine: projectile.texture_prefix == "orz-turret",
                    marine_returning: projectile.marine_returning,
                    damage: projectile.damage,
                    x: projectile.x,
                    y: projectile.y,
                    impact_start_frame: projectile.impact_start_frame,
                    impact_end_frame: projectile.impact_end_frame,
                    impact_texture_prefix: projectile.impact_texture_prefix,
                    impact_sound_key: projectile.impact_sound_key,
                })
            })
            .collect();

        let mut hit_projectile_indexes = Vec::new();
        let mut hit_explosions = Vec::new();
        for hit in hits {
            if hit.marine_returning {
                self.increment_ship_crew_by_ship_id(hit.owner_ship_id, 1);
                hit_projectile_indexes.push(hit.index);
                continue;
            }

            if hit.is_orz_marine {
                if !self.ship_blocks_damage_by_ship_id(hit.defender_ship_id) {
                    self.apply_marine_boarding_hit(
                        self.is_player_ship_id(hit.defender_ship_id),
                        hit.owner_is_player,
                        outcome,
                    );
                }
                hit_projectile_indexes.push(hit.index);
                continue;
            }

            if !self.ship_blocks_damage_by_ship_id(hit.defender_ship_id) {
                self.apply_projectile_damage(
                    hit.defender_ship_id,
                    hit.owner_ship_id,
                    hit.damage,
                    outcome,
                );
            }
            hit_explosions.push(ExplosionSnapshot {
                id: 0,
                x: hit.x,
                y: hit.y,
                frame_index: hit.impact_start_frame,
                end_frame: hit.impact_end_frame,
                texture_prefix: hit.impact_texture_prefix,
            });
            self.audio_events.push(AudioEventSnapshot {
                key: hit.impact_sound_key,
            });
            hit_projectile_indexes.push(hit.index);
        }

        for index in hit_projectile_indexes.into_iter().rev() {
            self.projectiles.remove(index);
        }

        for mut explosion in hit_explosions {
            explosion.id = self.next_game_object_id();
            self.explosions.push(explosion);
        }

        self.projectiles.retain(|projectile| projectile.life >= 0);
    }

    fn increment_ship_crew(&mut self, is_player: bool, amount: i32) {
        if self.ship_state(is_player).dead {
            return;
        }
        let ship_id = self.ship_state(is_player).ship_id;
        let next_crew = (self.ships[ship_id].crew() + amount).min(self.ships[ship_id].max_crew());
        self.ships[ship_id].set_crew(next_crew);
    }

    fn increment_ship_crew_by_ship_id(&mut self, ship_id: u64, amount: i32) {
        if self.player.game_object_id() == ship_id {
            self.increment_ship_crew(true, amount);
        } else if self.target.game_object_id() == ship_id {
            self.increment_ship_crew(false, amount);
        }
    }

    fn ship_blocks_damage_by_ship_id(&self, ship_id: u64) -> bool {
        if self.player.game_object_id() == ship_id {
            self.ship_blocks_damage(true)
        } else if self.target.game_object_id() == ship_id {
            self.ship_blocks_damage(false)
        } else {
            false
        }
    }

    fn apply_projectile_damage(
        &mut self,
        defender_ship_id: u64,
        attacker_ship_id: u64,
        damage: i32,
        outcome: &mut BattleOutcome,
    ) {
        let _ = self.apply_damage_to_ship_by_id(defender_ship_id, attacker_ship_id, damage, outcome);
    }

    fn tick_marine_boarders(&mut self, outcome: &mut BattleOutcome) {
        let mut next_boarders = Vec::with_capacity(self.marine_boarders.len());
        let mut active_boarders = std::mem::take(&mut self.marine_boarders);

        for mut boarder in active_boarders.drain(..) {
            let Some(ship_state) = self.ship_state_by_game_object_id(boarder.defender_ship_id) else {
                continue;
            };
            if ship_state.dead {
                self.launch_returning_marine(boarder.defender_ship_id, boarder.owner_ship_id);
                continue;
            }

            if boarder.ticks_until_next_roll > 0 {
                boarder.ticks_until_next_roll -= 1;
                next_boarders.push(boarder);
                continue;
            }

            boarder.ticks_until_next_roll = ORZ_MARINE_WAIT_TICKS;
            let roll = next_androsynth_random(&mut self.bubble_rng_state) & 0xff;

            if roll < ORZ_MARINE_DEATH_CHANCE_MAX {
                self.audio_events.push(AudioEventSnapshot {
                    key: AUDIO_ORZ_ARGH,
                });
                continue;
            }

            if roll < ORZ_MARINE_DAMAGE_CHANCE_MAX {
                self.apply_marine_periodic_damage(boarder.defender_ship_id, boarder.owner_ship_id, outcome);
            }

            next_boarders.push(boarder);
        }

        self.marine_boarders = next_boarders;
    }

    fn apply_marine_boarding_hit(
        &mut self,
        defender_is_player: bool,
        attacker_is_player: bool,
        outcome: &mut BattleOutcome,
    ) {
        self.apply_orz_marine_damage(defender_is_player, attacker_is_player, outcome);
        self.marine_boarders.push(MarineBoarderState {
            defender_ship_id: if defender_is_player {
                self.player.game_object_id()
            } else {
                self.target.game_object_id()
            },
            owner_ship_id: if attacker_is_player {
                self.player.game_object_id()
            } else {
                self.target.game_object_id()
            },
            ticks_until_next_roll: ORZ_MARINE_WAIT_TICKS,
        });
        self.audio_events.push(AudioEventSnapshot {
            key: AUDIO_ORZ_INTRUDER,
        });
    }

    fn apply_marine_periodic_damage(
        &mut self,
        defender_ship_id: u64,
        attacker_ship_id: u64,
        outcome: &mut BattleOutcome,
    ) {
        self.apply_orz_marine_damage_by_ship_id(defender_ship_id, attacker_ship_id, outcome);
        self.audio_events.push(AudioEventSnapshot { key: AUDIO_ORZ_ZAP });
    }

    fn apply_orz_marine_damage(
        &mut self,
        defender_is_player: bool,
        attacker_is_player: bool,
        outcome: &mut BattleOutcome,
    ) {
        let defender_ship_id = if defender_is_player {
            self.player.ship_id
        } else {
            self.target.ship_id
        };
        let died = self.ships[defender_ship_id].take_damage(ORZ_MARINE_DAMAGE);
        if died {
            let defender_game_object_id = if defender_is_player {
                self.player.game_object_id()
            } else {
                self.target.game_object_id()
            };
            let attacker_game_object_id = if attacker_is_player {
                self.player.game_object_id()
            } else {
                self.target.game_object_id()
            };
            self.record_ship_destroyed(outcome, defender_game_object_id);
            self.record_ship_victory(outcome, attacker_game_object_id);
        }
    }

    fn apply_orz_marine_damage_by_ship_id(
        &mut self,
        defender_ship_id: u64,
        attacker_ship_id: u64,
        outcome: &mut BattleOutcome,
    ) {
        let _ = self.apply_damage_to_ship_by_id(
            defender_ship_id,
            attacker_ship_id,
            ORZ_MARINE_DAMAGE,
            outcome,
        );
    }

    fn launch_returning_marine(&mut self, defender_ship_id: u64, owner_ship_id: u64) {
        let Some(owner_state) = self.ship_state_by_game_object_id(owner_ship_id) else {
            return;
        };
        if owner_state.dead {
            return;
        }
        let defender_state = self
            .ship_state_by_game_object_id(defender_ship_id)
            .expect("checked active defender ship");
        let defender_body_id = defender_state.body_id;
        let owner_body_id = owner_state.body_id;
        let (start_x, start_y) = self
            .matter_world
            .body_state(defender_body_id)
            .map(|body| (body.x, body.y))
            .unwrap_or((defender_state.previous_x, defender_state.previous_y));
        let Some(owner_body) = self.matter_world.body_state(owner_body_id) else {
            return;
        };
        let dx = shortest_wrapped_delta(start_x, owner_body.x, self.width);
        let dy = shortest_wrapped_delta(start_y, owner_body.y, self.height);
        let facing_index = vector_to_facing_index(dx, dy);
        let facing = facing_index_to_radians(facing_index);
        let (raw_vx, raw_vy) = projectile_velocity_for_facing(facing_index, ORZ_MARINE_RETURN_SPEED);
        let mut projectile = ProjectileSnapshot {
            id: self.next_game_object_id(),
            x: start_x,
            y: start_y,
            previous_x: start_x,
            previous_y: start_y,
            vx: raw_vx as f64 / 32.0,
            vy: raw_vy as f64 / 32.0,
            inherited_vx: 0.0,
            inherited_vy: 0.0,
            life: 90,
            texture_prefix: "orz-turret",
            damage: 0,
            impact_texture_prefix: "",
            impact_start_frame: 0,
            impact_end_frame: 0,
            impact_sound_key: "",
            track_wait: ORZ_MARINE_RETURN_TURN_WAIT,
            behavior: ProjectileBehaviorSpec::Tracking,
            collision: ProjectileCollisionSpec::None,
            facing,
            acceleration: 0.0,
            max_speed: ORZ_MARINE_RETURN_SPEED as f64,
            turn_wait: ORZ_MARINE_RETURN_TURN_WAIT,
            facing_index,
            speed: ORZ_MARINE_RETURN_SPEED,
            raw_vx,
            raw_vy,
            velocity_width: 0,
            velocity_height: 0,
            velocity_fract_width: 0,
            velocity_fract_height: 0,
            velocity_error_width: 0,
            velocity_error_height: 0,
            velocity_sign_width: 1,
            velocity_sign_height: 1,
            bubble_rng: 0,
            owner_ship_id,
            owner_is_player: self.is_player_ship_id(owner_ship_id),
            target: ProjectileTarget::Ship { id: owner_ship_id },
            marine_returning: true,
        };
        set_projectile_velocity_components(&mut projectile, raw_vx, raw_vy);
        self.projectiles.push(projectile);
    }

    fn resolve_ship_collisions(&mut self, delta: f64, outcome: &mut BattleOutcome) {
        let player_body_before = self.matter_world.body_state(self.player.body_id);
        let target_body_before = self.matter_world.body_state(self.target.body_id);
        let state = self.matter_world.step(delta);

        let player_blazer_spec = self.blazer_spec_for(true);
        let target_blazer_spec = self.blazer_spec_for(false);

        let player_blazer_hit = self.resolve_blazer_hit(
            true,
            player_blazer_spec,
            player_body_before,
            target_body_before,
            outcome,
        );
        self.player.special_contacting = player_blazer_hit.hits;

        let target_blazer_hit = self.resolve_blazer_hit(
            false,
            target_blazer_spec,
            player_body_before,
            target_body_before,
            outcome,
        );
        self.target.special_contacting = target_blazer_hit.hits;

        for collision in state.collisions {
            let ids = [collision.body_a, collision.body_b];
            if !ids.contains(&self.player.body_id) || !ids.contains(&self.target.body_id) {
                let Some(ship_a_id) = self
                    .ship_state_by_body_id(collision.body_a)
                    .map(|ship| ship.ship_id)
                else {
                    continue;
                };
                let Some(ship_b_id) = self
                    .ship_state_by_body_id(collision.body_b)
                    .map(|ship| ship.ship_id)
                else {
                    continue;
                };
                apply_collision_between(&mut self.ships, ship_a_id, ship_b_id);
                continue;
            }

            apply_collision_between(&mut self.ships, self.player.ship_id, self.target.ship_id);
            self.resolve_physics_collision_velocity(&PhysicsCollisionContext {
                player_blazer_spec,
                target_blazer_spec,
                player_body_before,
                target_body_before,
                player_blazer_hit,
                target_blazer_hit,
                bodies_after: &state.bodies,
            });
            self.resolve_physics_collision_blazer_damage(
                player_blazer_spec,
                player_blazer_hit.applied,
                false,
                outcome,
            );
            self.resolve_physics_collision_blazer_damage(
                target_blazer_spec,
                target_blazer_hit.applied,
                true,
                outcome,
            );
        }
    }

    fn resolve_blazer_hit(
        &mut self,
        is_player: bool,
        blazer_spec: Option<crate::traits::ship_trait::BlazerSpecialSpec>,
        player_body_before: Option<MatterBodyState>,
        target_body_before: Option<MatterBodyState>,
        outcome: &mut BattleOutcome,
    ) -> BlazerHitResult {
        let Some(spec) = blazer_spec else {
            return BlazerHitResult::default();
        };
        let (attacker, defender, attacker_before, defender_before) = if is_player {
            (
                &self.player,
                &self.target,
                player_body_before,
                target_body_before,
            )
        } else {
            (
                &self.target,
                &self.player,
                target_body_before,
                player_body_before,
            )
        };
        let hits = attacker.special_active
            && !defender.dead
            && self.androsynth_blazer_hits_other_ship(is_player);

        if !hits || attacker.special_contacting {
            return BlazerHitResult {
                hits,
                applied: false,
            };
        }

        if let (Some(ab), Some(db)) = (attacker_before, defender_before) {
            let (attacker_body_id, defender_body_id) = if is_player {
                (self.player.body_id, self.target.body_id)
            } else {
                (self.target.body_id, self.player.body_id)
            };
            let defender_ship_id = if is_player {
                self.target.ship_id
            } else {
                self.player.ship_id
            };
            self.apply_blazer_collision_velocity(&BlazerCollisionInput {
                blazer_mass: spec.mass,
                blazer_body_id: attacker_body_id,
                victim_body_id: defender_body_id,
                blazer_before: &ab,
                victim_before: &db,
                victim_mass: self.ships[defender_ship_id].mass(),
            });
        }
        apply_collision_between(&mut self.ships, self.player.ship_id, self.target.ship_id);
        let defender_game_object_id = if is_player {
            self.target.game_object_id()
        } else {
            self.player.game_object_id()
        };
        if !self.ship_blocks_damage_by_ship_id(defender_game_object_id) {
            let attacker_game_object_id = if is_player {
                self.player.game_object_id()
            } else {
                self.target.game_object_id()
            };
            let _ = self.apply_damage_to_ship_by_id(
                defender_game_object_id,
                attacker_game_object_id,
                spec.damage,
                outcome,
            );
        }
        self.audio_events.push(AudioEventSnapshot {
            key: spec.impact_sound_key,
        });
        BlazerHitResult {
            hits,
            applied: true,
        }
    }

    fn resolve_physics_collision_velocity(&mut self, ctx: &PhysicsCollisionContext) {
        if self.player.special_active && !self.target.dead && !ctx.player_blazer_hit.applied {
            if let (Some(spec), Some(pb), Some(tb)) = (
                ctx.player_blazer_spec,
                ctx.player_body_before,
                ctx.target_body_before,
            ) {
                self.apply_blazer_collision_velocity(&BlazerCollisionInput {
                    blazer_mass: spec.mass,
                    blazer_body_id: self.player.body_id,
                    victim_body_id: self.target.body_id,
                    blazer_before: &pb,
                    victim_before: &tb,
                    victim_mass: self.ships[self.target.ship_id].mass(),
                });
            }
        } else if self.target.special_active && !self.player.dead && !ctx.target_blazer_hit.applied
        {
            if let (Some(spec), Some(pb), Some(tb)) = (
                ctx.target_blazer_spec,
                ctx.player_body_before,
                ctx.target_body_before,
            ) {
                self.apply_blazer_collision_velocity(&BlazerCollisionInput {
                    blazer_mass: spec.mass,
                    blazer_body_id: self.target.body_id,
                    victim_body_id: self.player.body_id,
                    blazer_before: &tb,
                    victim_before: &pb,
                    victim_mass: self.ships[self.player.ship_id].mass(),
                });
            }
        } else if !ctx.player_blazer_hit.hits
            && !ctx.target_blazer_hit.hits
            && let (Some(player_before), Some(target_before)) =
                (ctx.player_body_before, ctx.target_body_before)
        {
            let player_after = ctx
                .bodies_after
                .iter()
                .find(|b| b.id == self.player.body_id)
                .copied()
                .unwrap_or(player_before);
            let target_after = ctx
                .bodies_after
                .iter()
                .find(|b| b.id == self.target.body_id)
                .copied()
                .unwrap_or(target_before);
            let ((pvx, pvy), (tvx, tvy)) = resolve_collision_velocity(
                CollisionBody {
                    x: player_after.x,
                    y: player_after.y,
                    vx: player_before.vx,
                    vy: player_before.vy,
                    mass: self.ships[self.player.ship_id].mass(),
                },
                CollisionBody {
                    x: target_after.x,
                    y: target_after.y,
                    vx: target_before.vx,
                    vy: target_before.vy,
                    mass: self.ships[self.target.ship_id].mass(),
                },
            );
            self.matter_world
                .set_body_velocity(self.player.body_id, pvx, pvy);
            self.matter_world
                .set_body_velocity(self.target.body_id, tvx, tvy);
        }
    }

    fn resolve_physics_collision_blazer_damage(
        &mut self,
        blazer_spec: Option<crate::traits::ship_trait::BlazerSpecialSpec>,
        already_applied: bool,
        defender_is_player: bool,
        outcome: &mut BattleOutcome,
    ) {
        if already_applied {
            return;
        }
        let (attacker, defender) = if defender_is_player {
            (&self.target, &self.player)
        } else {
            (&self.player, &self.target)
        };
        if !attacker.special_active || defender.dead {
            return;
        }
        let Some(spec) = blazer_spec else { return };
        let defender_game_object_id = if defender_is_player {
            self.player.game_object_id()
        } else {
            self.target.game_object_id()
        };
        if !self.ship_blocks_damage_by_ship_id(defender_game_object_id) {
            let attacker_game_object_id = if defender_is_player {
                self.target.game_object_id()
            } else {
                self.player.game_object_id()
            };
            let _ = self.apply_damage_to_ship_by_id(
                defender_game_object_id,
                attacker_game_object_id,
                spec.damage,
                outcome,
            );
        }
        self.audio_events.push(AudioEventSnapshot {
            key: spec.impact_sound_key,
        });
    }

    fn resolve_death_and_victory(&mut self, outcome: &BattleOutcome) {
        for ship_id in &outcome.died_ship_ids {
            self.mark_ship_dead_by_id(*ship_id);
        }
        for ship_id in &outcome.winner_ship_ids {
            if self.player.game_object_id() == *ship_id
                && let Some(key) = self.ships[self.player.ship_id].victory_sound_key()
            {
                self.audio_events.push(AudioEventSnapshot { key });
            } else if self.target.game_object_id() == *ship_id
                && let Some(key) = self.ships[self.target.ship_id].victory_sound_key()
            {
                self.audio_events.push(AudioEventSnapshot { key });
            } else if let Some(state) = self.ship_state_by_game_object_id(*ship_id)
                && let Some(key) = self.ships[state.ship_id].victory_sound_key()
            {
                self.audio_events.push(AudioEventSnapshot { key });
            }
        }
    }

    pub fn snapshot(&self) -> BattleSnapshot {
        let player = self.snapshot_for(&self.player);
        let target = self.snapshot_for(&self.target);
        let ships = self
            .active_ship_states()
            .into_iter()
            .map(|ship| self.snapshot_for(ship))
            .collect();
        BattleSnapshot {
            ships,
            player,
            target,
            meteors: self.meteors.clone(),
            projectiles: self.projectiles.clone(),
            explosions: self.explosions.clone(),
            lasers: self.lasers.clone(),
            audio_events: self.audio_events.clone(),
        }
    }

    fn step_ship(&mut self, control: ActiveShipControl) {
        if self
            .ship_state_by_game_object_id(control.game_object_id)
            .is_none_or(|ship| ship.dead)
        {
            return;
        }

        if let Some(is_player) = control.role
            && self.ship_state(is_player).special_active
            && self.blazer_spec_for(is_player).is_some()
        {
            self.step_androsynth_blazer(control.ship_id, control.body_id, control.input, is_player);
            return;
        }

        let Some(body) = self.matter_world.body_state(control.body_id) else {
            return;
        };
        let ship_state = self
            .ship_state_by_game_object_id_mut(control.game_object_id)
            .expect("active ship state should exist while stepping");
        ship_state.previous_x = body.x;
        ship_state.previous_y = body.y;

        self.apply_gravity(control.ship_id, control.body_id, body);
        let current = self.matter_world.body_state(control.body_id).unwrap_or(body);
        let in_gravity_well = self.in_gravity_well(current.x, current.y);
        let energy_before = self.ships[control.ship_id].energy();
        let energy_counter_before = self.ships[control.ship_id].energy_counter();
        let weapon_counter_before = self.ships[control.ship_id].weapon_counter();
        let special_counter_before = self.ships[control.ship_id].special_counter();
        let mut commands = self.ships[control.ship_id].update(
            &control.input,
            &VelocityVector {
                x: current.vx,
                y: current.vy,
            },
            in_gravity_well,
        );
        if let Some(is_player) = control.role {
            self.update_primary_mount_facing(is_player, current);
            self.handle_blazer_activation(
                control.ship_id,
                control.body_id,
                &control.input,
                energy_before,
                is_player,
                &mut commands,
            );

            if is_weapon_triggered(
                &self.ships[control.ship_id],
                &control.input,
                weapon_counter_before,
                energy_before,
                energy_counter_before,
            ) && !self.ship_blocks_damage(is_player)
            {
                self.handle_weapon_fire(control.ship_id, current, is_player, &mut commands);
            }

            if is_special_triggered(
                &self.ships[control.ship_id],
                &control.input,
                special_counter_before,
                energy_before,
                energy_counter_before,
            ) {
                self.handle_special_activation(
                    control.ship_id,
                    control.body_id,
                    current,
                    is_player,
                    &mut commands,
                );
            }
        } else {
            self.ship_state_by_game_object_id_mut(control.game_object_id)
                .expect("active ship state should exist while stepping")
                .primary_mount_facing = self.ships[control.ship_id].facing();
            if is_weapon_triggered(
                &self.ships[control.ship_id],
                &control.input,
                weapon_counter_before,
                energy_before,
                energy_counter_before,
            ) {
                self.handle_weapon_fire_for_ship(
                    control.game_object_id,
                    control.ship_id,
                    current,
                    false,
                    &mut commands,
                );
            }
            if is_special_triggered(
                &self.ships[control.ship_id],
                &control.input,
                special_counter_before,
                energy_before,
                energy_counter_before,
            ) {
                self.handle_special_activation_for_ship(
                    control.game_object_id,
                    control.ship_id,
                    control.body_id,
                    current,
                    false,
                    &mut commands,
                );
            }
        }

        let thrusting = apply_commands(&mut self.matter_world, control.body_id, commands);
        if let Some(is_player) = control.role {
            self.expire_special_cooldown(control.ship_id, is_player, control.input);
            self.emit_active_special_audio(control.ship_id, is_player);
        }
        self.ship_state_by_game_object_id_mut(control.game_object_id)
            .expect("active ship state should exist while stepping")
            .thrusting = thrusting;
        sync_ship_body_angle(
            &mut self.matter_world,
            control.body_id,
            &self.ships[control.ship_id],
        );
    }

    fn emit_active_special_audio(&mut self, ship_id: usize, is_player: bool) {
        if !self.ship_state(is_player).special_active {
            return;
        }
        let SpecialAbilitySpec::Shield(spec) = self.ships[ship_id].special_ability_spec() else {
            return;
        };
        if self.ships[ship_id].special_counter() > 0 && !spec.sound_key.is_empty() {
            self.audio_events.push(AudioEventSnapshot {
                key: spec.sound_key,
            });
        }
    }

    fn update_primary_mount_facing(&mut self, is_player: bool, current: MatterBodyState) {
        let ship_id = self.ship_state(is_player).ship_id;
        let Some(turn_rate) = self.ships[ship_id].primary_mount_turn_rate() else {
            self.ship_state_mut(is_player).primary_mount_facing = self.ships[ship_id].facing();
            return;
        };
        let desired = self.primary_mount_target_facing(is_player, current);
        let current_facing = self.ship_state(is_player).primary_mount_facing;
        self.ship_state_mut(is_player).primary_mount_facing =
            rotate_toward_angle(current_facing, desired, turn_rate);
    }

    fn primary_mount_target_facing(&self, is_player: bool, current: MatterBodyState) -> f64 {
        let target = self.projectile_target_for(is_player);
        match self.projectile_target_position(target) {
            Some((target_x, target_y)) => {
                let dx = shortest_wrapped_delta(current.x, target_x, self.width);
                let dy = shortest_wrapped_delta(current.y, target_y, self.height);
                dy.atan2(dx)
            }
            None => self.ships[self.ship_state(is_player).ship_id].facing(),
        }
    }

    fn handle_blazer_activation(
        &mut self,
        ship_id: usize,
        body_id: usize,
        input: &ShipInput,
        energy_before: i32,
        is_player: bool,
        commands: &mut Vec<PhysicsCommand>,
    ) {
        let SpecialAbilitySpec::Blazer(blazer_spec) = self.ships[ship_id].special_ability_spec()
        else {
            return;
        };
        if input.special && self.ships[ship_id].energy() < energy_before {
            if !self.ship_state(is_player).special_active {
                self.audio_events.push(AudioEventSnapshot {
                    key: blazer_spec.activation_sound_key,
                });
            }
            self.matter_world.set_body_mass(body_id, blazer_spec.mass);
            self.ship_state_mut(is_player).special_active = true;
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

    fn handle_weapon_fire(
        &mut self,
        ship_id: usize,
        current: MatterBodyState,
        is_player: bool,
        commands: &mut Vec<PhysicsCommand>,
    ) {
        let owner_ship_id = self.ship_state(is_player).game_object_id();
        self.handle_weapon_fire_for_ship(owner_ship_id, ship_id, current, is_player, commands);
    }

    fn handle_weapon_fire_for_ship(
        &mut self,
        owner_ship_id: u64,
        ship_id: usize,
        current: MatterBodyState,
        _is_player: bool,
        commands: &mut Vec<PhysicsCommand>,
    ) {
        let special_active = self
            .ship_state_by_game_object_id(owner_ship_id)
            .map(|ship| ship.special_active)
            .unwrap_or(false);
        if self.ships[ship_id].is_cloaked(special_active)
            && let Some(ship_state) = self.ship_state_by_game_object_id_mut(owner_ship_id)
        {
            ship_state.special_active = false;
        }
        let inherit_ship_velocity =
            self.ships[ship_id].primary_projectile_inherits_ship_velocity_for_state(special_active);
        let projectile_target = self.projectile_target_for_ship(owner_ship_id);
        let primary_mount_facing = self
            .ship_state_by_game_object_id(owner_ship_id)
            .map(|ship| ship.primary_mount_facing)
            .unwrap_or(self.ships[ship_id].facing());

        if let Some(volley_spec) = self.ships[ship_id].primary_volley_spec_for_state(special_active)
        {
            self.spawn_projectile_volley_for_ship(
                current,
                owner_ship_id,
                volley_spec,
                inherit_ship_velocity,
                primary_mount_facing,
            );
        } else if let Some(projectile_spec) =
            self.ships[ship_id].primary_projectile_spec_for_state(special_active)
        {
            self.spawn_projectile_from_spec_for_ship(
                current,
                owner_ship_id,
                projectile_spec,
                ProjectileSpawnSpec {
                    facing_offset: 0,
                    forward_offset: projectile_spec.offset,
                    lateral_offset: 0.0,
                },
                projectile_target,
                inherit_ship_velocity,
                primary_mount_facing,
            );
            if !projectile_spec.sound_key.is_empty() {
                self.audio_events.push(AudioEventSnapshot {
                    key: projectile_spec.sound_key,
                });
            }
        } else if let Some(laser_spec) =
            self.ships[ship_id].primary_instant_laser_spec_for_state(special_active)
        {
            self.fire_instant_laser_for_ship(current, owner_ship_id, ship_id, laser_spec);
        }
        let _ = commands;
    }

    fn handle_special_activation(
        &mut self,
        ship_id: usize,
        body_id: usize,
        current: MatterBodyState,
        is_player: bool,
        commands: &mut Vec<PhysicsCommand>,
    ) {
        let owner_ship_id = self.ship_state(is_player).game_object_id();
        self.handle_special_activation_for_ship(
            owner_ship_id,
            ship_id,
            body_id,
            current,
            is_player,
            commands,
        );
    }

    fn handle_special_activation_for_ship(
        &mut self,
        owner_ship_id: u64,
        ship_id: usize,
        body_id: usize,
        current: MatterBodyState,
        is_player: bool,
        commands: &mut Vec<PhysicsCommand>,
    ) {
        match self.ships[ship_id].special_ability_spec() {
            SpecialAbilitySpec::Teleport(spec) => {
                self.activate_teleport_special_for_ship(owner_ship_id, body_id, spec);
                if let Some(ship_state) = self.ship_state_by_game_object_id_mut(owner_ship_id) {
                    ship_state.special_active = true;
                }
            }
            SpecialAbilitySpec::InstantLaser(spec) => {
                self.fire_instant_laser_for_ship(current, owner_ship_id, ship_id, spec);
            }
            SpecialAbilitySpec::Shield(spec) => {
                if !spec.sound_key.is_empty() {
                    self.audio_events.push(AudioEventSnapshot {
                        key: spec.sound_key,
                    });
                }
                if let Some(ship_state) = self.ship_state_by_game_object_id_mut(owner_ship_id) {
                    ship_state.special_active = true;
                }
            }
            SpecialAbilitySpec::DirectionalThrust(spec) => {
                if !spec.sound_key.is_empty() {
                    self.audio_events.push(AudioEventSnapshot {
                        key: spec.sound_key,
                    });
                }
                let facing = self.ships[ship_id].facing() + spec.facing_offset;
                commands.push(PhysicsCommand::SetVelocity {
                    vx: facing.cos() * spec.speed,
                    vy: facing.sin() * spec.speed,
                });
            }
            SpecialAbilitySpec::Projectile(SecondaryProjectileSpec { volley }) => {
                let is_orz_marine = volley.projectile.texture_prefix == "orz-turret";
                let target = if is_orz_marine {
                    match self.special_target_to_projectile_target(is_player) {
                        ProjectileTarget::Ship { id }
                            if is_player
                                && id == self.target.game_object_id()
                                && self.ship_is_targetable(false) =>
                        {
                            ProjectileTarget::Ship { id }
                        }
                        ProjectileTarget::Ship { id }
                            if !is_player
                                && id == self.player.game_object_id()
                                && self.ship_is_targetable(true) =>
                        {
                            ProjectileTarget::Ship { id }
                        }
                        _ => return,
                    }
                } else {
                    self.special_projectile_target_for_mode(is_player, volley.target_mode)
                };
                if is_orz_marine {
                    if self.ships[ship_id].crew() <= 1 {
                        return;
                    }
                    let next_crew = self.ships[ship_id].crew() - 1;
                    self.ships[ship_id].set_crew(next_crew);
                }
                for spawn in volley.spawns {
                    self.spawn_projectile_from_spec(
                        current,
                        is_player,
                        volley.projectile,
                        *spawn,
                        target,
                        false,
                        self.ships[ship_id].facing(),
                    );
                }
                if !volley.sound_key.is_empty() {
                    self.audio_events.push(AudioEventSnapshot {
                        key: volley.sound_key,
                    });
                }
            }
            SpecialAbilitySpec::CrewRegeneration(spec) => {
                if !spec.sound_key.is_empty() {
                    self.audio_events.push(AudioEventSnapshot {
                        key: spec.sound_key,
                    });
                }
                let max_crew = self.ships[ship_id].max_crew();
                let next_crew = (self.ships[ship_id].crew() + spec.amount).min(max_crew);
                self.ships[ship_id].set_crew(next_crew);
            }
            SpecialAbilitySpec::CrewToEnergy(spec) => {
                self.activate_crew_to_energy_special(current, ship_id, spec, commands);
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
                self.activate_crew_drain_special_for_ship(owner_ship_id, spec);
            }
            SpecialAbilitySpec::PlanetHarvest(spec) => {
                self.activate_planet_harvest_special(current, ship_id, spec);
            }
            SpecialAbilitySpec::None
            | SpecialAbilitySpec::PointDefense(_)
            | SpecialAbilitySpec::Blazer(_) => {}
        }
    }

    fn expire_special_cooldown(&mut self, ship_id: usize, is_player: bool, input: ShipInput) {
        if matches!(
            self.ships[ship_id].special_ability_spec(),
            SpecialAbilitySpec::Shield(_)
        ) && input.special
        {
            return;
        }
        if self.ships[ship_id].special_state_persists_after_cooldown()
            || matches!(
                self.ships[ship_id].special_ability_spec(),
                SpecialAbilitySpec::Blazer(_)
            )
        {
            return;
        }
        if self.ship_state(is_player).special_active && self.ships[ship_id].special_counter() == 0 {
            self.ship_state_mut(is_player).special_active = false;
        }
    }

    fn step_androsynth_blazer(
        &mut self,
        ship_id: usize,
        body_id: usize,
        input: ShipInput,
        is_player: bool,
    ) {
        let SpecialAbilitySpec::Blazer(blazer_spec) = self.ships[ship_id].special_ability_spec()
        else {
            return;
        };
        if self.ships[ship_id].energy() <= 0 {
            self.matter_world
                .set_body_mass(body_id, self.ships[ship_id].mass());
            self.ship_state_mut(is_player).special_active = false;
            let _ = apply_commands(
                &mut self.matter_world,
                body_id,
                vec![PhysicsCommand::SetVelocity { vx: 0.0, vy: 0.0 }],
            );
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
            apply_commands(
                &mut self.matter_world,
                body_id,
                vec![PhysicsCommand::SetVelocity {
                    vx: facing.cos() * blazer_spec.speed,
                    vy: facing.sin() * blazer_spec.speed,
                }],
            )
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

        let ship_state = if is_player {
            &self.player
        } else {
            &self.target
        };
        if ship_state.dead {
            return;
        }

        let ship_id = ship_state.ship_id;
        let SpecialAbilitySpec::PointDefense(point_defense_spec) =
            self.ships[ship_id].special_ability_spec()
        else {
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

        let target_indexes =
            self.find_point_defense_targets(is_player, body.x, body.y, point_defense_spec.range);
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

    fn find_point_defense_targets(
        &self,
        is_player: bool,
        ship_x: f64,
        ship_y: f64,
        range: f64,
    ) -> Vec<usize> {
        self.projectiles
            .iter()
            .enumerate()
            .filter(|(_, projectile)| projectile.owner_ship_id != self.ship_state(is_player).game_object_id())
            .filter_map(|(index, projectile)| {
                let dx = shortest_wrapped_delta(ship_x, projectile.x, self.width);
                let dy = shortest_wrapped_delta(ship_y, projectile.y, self.height);
                let distance = ((dx * dx) + (dy * dy)).sqrt();
                (distance <= range).then_some(index)
            })
            .collect()
    }

    fn projectile_target_for(&self, is_player: bool) -> ProjectileTarget {
        let ship_id = self.ship_state(is_player).ship_id;
        let special_active = self.ship_state(is_player).special_active;
        self.projectile_target_for_mode(
            is_player,
            self.ships[ship_id].primary_projectile_target_mode_for_state(special_active),
        )
    }

    fn projectile_target_for_ship(&self, ship_game_object_id: u64) -> ProjectileTarget {
        let Some(ship_state) = self.ship_state_by_game_object_id(ship_game_object_id) else {
            return ProjectileTarget::None;
        };
        let special_active = ship_state.special_active;
        self.projectile_target_for_mode_for_ship(
            ship_game_object_id,
            self.ships[ship_state.ship_id].primary_projectile_target_mode_for_state(special_active),
        )
    }

    fn projectile_target_for_mode(
        &self,
        is_player: bool,
        mode: ProjectileTargetMode,
    ) -> ProjectileTarget {
        let selected_target = self.selected_weapon_target(is_player);
        match mode {
            ProjectileTargetMode::None => {
                if !is_player && matches!(selected_target, ProjectileTarget::None) {
                    self.default_enemy_target_for(is_player)
                } else {
                    self.normalized_selected_weapon_target(is_player, selected_target)
                }
            }
            ProjectileTargetMode::EnemyShip => self.default_enemy_target_for(is_player),
            ProjectileTargetMode::PlayerSelectedOrEnemyShip => {
                if matches!(selected_target, ProjectileTarget::None) {
                    self.default_enemy_target_for(is_player)
                } else {
                    self.normalized_selected_weapon_target(is_player, selected_target)
                }
            }
            ProjectileTargetMode::PlayerSelectedPointOrForward => {
                if matches!(selected_target, ProjectileTarget::Point { .. }) {
                    selected_target
                } else {
                    ProjectileTarget::None
                }
            }
        }
    }

    fn projectile_target_for_mode_for_ship(
        &self,
        ship_game_object_id: u64,
        mode: ProjectileTargetMode,
    ) -> ProjectileTarget {
        let selected_target = self.selected_weapon_target_for_ship(ship_game_object_id);
        match mode {
            ProjectileTargetMode::None => selected_target,
            ProjectileTargetMode::EnemyShip => self.default_enemy_target_for_ship(ship_game_object_id),
            ProjectileTargetMode::PlayerSelectedOrEnemyShip => {
                if matches!(selected_target, ProjectileTarget::None) {
                    self.default_enemy_target_for_ship(ship_game_object_id)
                } else {
                    self.normalized_selected_weapon_target_for_ship(ship_game_object_id, selected_target)
                }
            }
            ProjectileTargetMode::PlayerSelectedPointOrForward => {
                if matches!(selected_target, ProjectileTarget::Point { .. }) {
                    selected_target
                } else {
                    ProjectileTarget::None
                }
            }
        }
    }

    fn selected_weapon_target(&self, is_player: bool) -> ProjectileTarget {
        if is_player {
            self.player_weapon_target
        } else {
            self.target_weapon_target
        }
    }

    fn selected_weapon_target_for_ship(&self, ship_game_object_id: u64) -> ProjectileTarget {
        if self.player.game_object_id() == ship_game_object_id {
            self.player_weapon_target
        } else if self.target.game_object_id() == ship_game_object_id {
            self.target_weapon_target
        } else if let Some(index) = self
            .additional_active_ships
            .iter()
            .position(|ship| ship.game_object_id() == ship_game_object_id)
        {
            self.additional_weapon_targets[index]
        } else {
            ProjectileTarget::None
        }
    }

    fn normalized_selected_weapon_target(
        &self,
        is_player: bool,
        target: ProjectileTarget,
    ) -> ProjectileTarget {
        match target {
            ProjectileTarget::Ship { id } => {
                let current_target_id = if is_player {
                    self.target.game_object_id()
                } else {
                    self.player.game_object_id()
                };
                if id == current_target_id && !self.ship_is_targetable(!is_player) {
                    ProjectileTarget::None
                } else {
                    target
                }
            }
            _ => target,
        }
    }

    fn normalized_selected_weapon_target_for_ship(
        &self,
        ship_game_object_id: u64,
        target: ProjectileTarget,
    ) -> ProjectileTarget {
        match target {
            ProjectileTarget::Ship { id } if self.ship_is_targetable_by_id(id) => target,
            ProjectileTarget::Ship { .. } => self.default_enemy_target_for_ship(ship_game_object_id),
            _ => target,
        }
    }

    fn default_enemy_target_for(&self, is_player: bool) -> ProjectileTarget {
        if !self.ship_is_targetable(!is_player) {
            ProjectileTarget::None
        } else if is_player {
            ProjectileTarget::Ship {
                id: self.target.game_object_id(),
            }
        } else {
            ProjectileTarget::Ship {
                id: self.player.game_object_id(),
            }
        }
    }

    fn default_enemy_target_for_ship(&self, ship_game_object_id: u64) -> ProjectileTarget {
        self.active_enemy_target_for_ship(ship_game_object_id)
            .map(|id| ProjectileTarget::Ship { id })
            .unwrap_or(ProjectileTarget::None)
    }

    fn special_projectile_target_for_mode(
        &self,
        is_player: bool,
        mode: ProjectileTargetMode,
    ) -> ProjectileTarget {
        match mode {
            ProjectileTargetMode::None => self.default_enemy_target_for(is_player),
            ProjectileTargetMode::EnemyShip => self.default_enemy_target_for(is_player),
            ProjectileTargetMode::PlayerSelectedOrEnemyShip => {
                let selected_target = self.special_target_to_projectile_target(is_player);
                if matches!(selected_target, ProjectileTarget::None) {
                    self.default_enemy_target_for(is_player)
                } else {
                    self.normalized_selected_weapon_target(is_player, selected_target)
                }
            }
            ProjectileTargetMode::PlayerSelectedPointOrForward => {
                let selected_target = self.special_target_to_projectile_target(is_player);
                if matches!(selected_target, ProjectileTarget::Point { .. }) {
                    selected_target
                } else {
                    ProjectileTarget::None
                }
            }
        }
    }

    fn special_target_to_projectile_target(&self, is_player: bool) -> ProjectileTarget {
        match self.special_target_for(is_player) {
            SpecialTarget::None => ProjectileTarget::None,
            SpecialTarget::Point { x, y } => ProjectileTarget::Point { x, y },
            SpecialTarget::Ship { id } => ProjectileTarget::Ship { id },
        }
    }

    fn spawn_projectile_volley_for_ship(
        &mut self,
        current: MatterBodyState,
        owner_ship_id: u64,
        volley_spec: ProjectileVolleySpec,
        inherit_ship_velocity: bool,
        base_facing: f64,
    ) {
        for spawn in volley_spec.spawns {
            self.spawn_projectile_from_spec_for_ship(
                current,
                owner_ship_id,
                volley_spec.projectile,
                *spawn,
                self.projectile_target_for_mode_for_ship(owner_ship_id, volley_spec.target_mode),
                inherit_ship_velocity,
                base_facing,
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
        inherit_ship_velocity: bool,
        base_facing: f64,
    ) {
        let owner_ship_id = self.ship_state(is_player).game_object_id();
        self.spawn_projectile_from_spec_for_ship(
            current,
            owner_ship_id,
            projectile_spec,
            spawn,
            target,
            inherit_ship_velocity,
            base_facing,
        );
    }

    fn spawn_projectile_from_spec_for_ship(
        &mut self,
        current: MatterBodyState,
        owner_ship_id: u64,
        projectile_spec: PrimaryProjectileSpec,
        spawn: ProjectileSpawnSpec,
        target: ProjectileTarget,
        inherit_ship_velocity: bool,
        base_facing: f64,
    ) {
        let base_facing_index = radians_to_facing_index(base_facing);
        let facing_index =
            (base_facing_index + spawn.facing_offset).rem_euclid(PROJECTILE_FACINGS as i32);
        let facing = facing_index_to_radians(facing_index);
        let (projectile_raw_vx, projectile_raw_vy) =
            projectile_velocity_for_facing(facing_index, projectile_spec.speed as i32);
        let (spawn_rewind_x, spawn_rewind_y) = match projectile_spec.behavior {
            ProjectileBehaviorSpec::WobbleTracking {
                spawn_rewind_divisor,
                ..
            } => (
                projectile_raw_vx as f64 / spawn_rewind_divisor,
                projectile_raw_vy as f64 / spawn_rewind_divisor,
            ),
            ProjectileBehaviorSpec::Tracking | ProjectileBehaviorSpec::Straight => (0.0, 0.0),
        };
        let lateral_facing = facing + std::f64::consts::FRAC_PI_2;
        let projectile_id = self.next_game_object_id();
        let mut projectile = ProjectileSnapshot {
            id: projectile_id,
            x: current.x
                + (facing.cos() * spawn.forward_offset)
                + (lateral_facing.cos() * spawn.lateral_offset)
                - spawn_rewind_x,
            y: current.y
                + (facing.sin() * spawn.forward_offset)
                + (lateral_facing.sin() * spawn.lateral_offset)
                - spawn_rewind_y,
            previous_x: current.x
                + (facing.cos() * spawn.forward_offset)
                + (lateral_facing.cos() * spawn.lateral_offset)
                - spawn_rewind_x,
            previous_y: current.y
                + (facing.sin() * spawn.forward_offset)
                + (lateral_facing.sin() * spawn.lateral_offset)
                - spawn_rewind_y,
            vx: projectile_raw_vx as f64 / 32.0,
            vy: projectile_raw_vy as f64 / 32.0,
            inherited_vx: if inherit_ship_velocity {
                current.vx
            } else {
                0.0
            },
            inherited_vy: if inherit_ship_velocity {
                current.vy
            } else {
                0.0
            },
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
            bubble_rng: if matches!(
                projectile_spec.behavior,
                ProjectileBehaviorSpec::WobbleTracking { .. }
            ) {
                next_androsynth_random(&mut self.bubble_rng_state) as u32
            } else {
                0
            },
            owner_ship_id,
            owner_is_player: self.is_player_ship_id(owner_ship_id),
            target,
            marine_returning: false,
        };
        set_projectile_velocity_components(&mut projectile, projectile_raw_vx, projectile_raw_vy);
        self.projectiles.push(projectile);
    }

    fn step_meteors(&mut self) {
        for meteor in &mut self.meteors {
            meteor.x = wrap_axis(meteor.x + meteor.vx, self.width);
            meteor.y = wrap_axis(meteor.y + meteor.vy, self.height);
            meteor.frame_index =
                (meteor.frame_index + meteor.spin_step).rem_euclid(meteor.frame_count);
        }
    }

    fn handle_meteor_collisions(&mut self) {
        let player_hits: Vec<bool> = self
            .meteors
            .iter()
            .map(|meteor| self.meteor_hits_ship(meteor, true))
            .collect();
        let target_hits: Vec<bool> = self
            .meteors
            .iter()
            .map(|meteor| self.meteor_hits_ship(meteor, false))
            .collect();
        let additional_hits: Vec<Vec<bool>> = self
            .additional_active_ships
            .iter()
            .map(|ship| {
                self.meteors
                    .iter()
                    .map(|meteor| self.meteor_hits_ship_by_id(meteor, ship.game_object_id()))
                    .collect()
            })
            .collect();

        let (player_died, target_died, additional_died_ship_ids) =
            self.resolve_meteor_ship_hits(player_hits, target_hits, additional_hits);
        self.resolve_meteor_projectile_hits();

        if player_died {
            self.mark_ship_dead(true);
        }
        if target_died {
            self.mark_ship_dead(false);
        }
        for ship_id in additional_died_ship_ids {
            self.mark_ship_dead_by_id(ship_id);
        }
    }

    fn resolve_meteor_ship_hits(
        &mut self,
        player_hits: Vec<bool>,
        target_hits: Vec<bool>,
        additional_hits: Vec<Vec<bool>>,
    ) -> (bool, bool, Vec<u64>) {
        let mut player_died = false;
        let mut target_died = false;
        let mut additional_died_ship_ids = Vec::new();

        for index in 0..self.meteors.len() {
            let player_hit = player_hits[index];
            let target_hit = target_hits[index];
            let player_contacting = self.meteors[index].player_contacting;
            let target_contacting = self.meteors[index].target_contacting;

            player_died |= self.apply_meteor_ship_hit(index, true, player_hit, player_contacting);
            target_died |= self.apply_meteor_ship_hit(index, false, target_hit, target_contacting);

            self.meteors[index].player_contacting = player_hit;
            self.meteors[index].target_contacting = target_hit;

            for (ship_index, ship_hits) in additional_hits.iter().enumerate() {
                let ship_hit = ship_hits[index];
                let was_contacting = self.additional_meteor_contacts[ship_index][index];
                let ship_id = self.additional_active_ships[ship_index].game_object_id();
                if self.apply_meteor_ship_hit_by_id(index, ship_id, ship_hit, was_contacting) {
                    additional_died_ship_ids.push(ship_id);
                }
                self.additional_meteor_contacts[ship_index][index] = ship_hit;
            }
        }

        (player_died, target_died, additional_died_ship_ids)
    }

    fn apply_meteor_ship_hit(
        &mut self,
        index: usize,
        is_player: bool,
        ship_hit: bool,
        was_contacting: bool,
    ) -> bool {
        if !ship_hit || was_contacting {
            return false;
        }

        let meteor_x = self.meteors[index].x;
        let meteor_y = self.meteors[index].y;

        let defender_game_object_id = self.ship_state(is_player).game_object_id();
        let attacker_game_object_id = self
            .opposing_active_ship_id(defender_game_object_id)
            .unwrap_or(defender_game_object_id);
        let died = if self.ship_blocks_damage_by_ship_id(defender_game_object_id) {
            false
        } else {
            let died = self.apply_damage_to_ship_by_id(
                defender_game_object_id,
                attacker_game_object_id,
                METEOR_DAMAGE,
                &mut BattleOutcome::default(),
            );
            self.spawn_meteor_impact_effect(meteor_x, meteor_y);
            self.push_ship_from_meteor(is_player, meteor_x, meteor_y);
            died
        };

        self.meteors[index].vx = -self.meteors[index].vx;
        self.meteors[index].vy = -self.meteors[index].vy;
        died
    }

    fn apply_meteor_ship_hit_by_id(
        &mut self,
        index: usize,
        defender_game_object_id: u64,
        ship_hit: bool,
        was_contacting: bool,
    ) -> bool {
        if !ship_hit || was_contacting {
            return false;
        }

        let meteor_x = self.meteors[index].x;
        let meteor_y = self.meteors[index].y;
        let attacker_game_object_id = self
            .opposing_active_ship_id(defender_game_object_id)
            .unwrap_or(defender_game_object_id);
        let died = if self.ship_blocks_damage_by_ship_id(defender_game_object_id) {
            false
        } else {
            let died = self.apply_damage_to_ship_by_id(
                defender_game_object_id,
                attacker_game_object_id,
                METEOR_DAMAGE,
                &mut BattleOutcome::default(),
            );
            self.spawn_meteor_impact_effect(meteor_x, meteor_y);
            self.push_ship_from_meteor_by_id(defender_game_object_id, meteor_x, meteor_y);
            died
        };

        self.meteors[index].vx = -self.meteors[index].vx;
        self.meteors[index].vy = -self.meteors[index].vy;
        died
    }

    fn spawn_meteor_impact_effect(&mut self, x: f64, y: f64) {
        let explosion_id = self.next_game_object_id();
        self.explosions.push(ExplosionSnapshot {
            id: explosion_id,
            x,
            y,
            frame_index: 0,
            end_frame: 7,
            texture_prefix: "battle-blast",
        });
        self.audio_events.push(AudioEventSnapshot {
            key: "battle-boom-23",
        });
    }

    fn resolve_meteor_projectile_hits(&mut self) {
        let mut hit_projectile_indexes = Vec::new();
        let projectile_hits: Vec<(usize, f64, f64, i32, i32, &'static str, &'static str)> = self
            .projectiles
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

        for (
            index,
            x,
            y,
            impact_start_frame,
            impact_end_frame,
            impact_texture_prefix,
            impact_sound_key,
        ) in projectile_hits
        {
            let projectile = self.projectiles[index];
            if projectile.texture_prefix == "orz-turret" {
                self.audio_events.push(AudioEventSnapshot {
                    key: AUDIO_ORZ_ARGH,
                });
                hit_projectile_indexes.push(index);
                continue;
            }
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
    }

    fn push_ship_from_meteor(&mut self, is_player: bool, meteor_x: f64, meteor_y: f64) {
        let ship = self.ship_state(is_player);
        self.push_ship_from_meteor_by_id(ship.game_object_id(), meteor_x, meteor_y);
    }

    fn push_ship_from_meteor_by_id(&mut self, ship_id: u64, meteor_x: f64, meteor_y: f64) {
        let Some(ship) = self.ship_state_by_game_object_id(ship_id) else {
            return;
        };
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

    fn fire_instant_laser_for_ship(
        &mut self,
        current: MatterBodyState,
        attacker_game_object_id: u64,
        attacker_ship_id: usize,
        laser_spec: InstantLaserSpec,
    ) {
        let facing = self.ships[attacker_ship_id].facing();
        let start_x = current.x + (facing.cos() * laser_spec.offset);
        let start_y = current.y + (facing.sin() * laser_spec.offset);
        let (aim_end_x, aim_end_y) = self.instant_laser_end_for_mode_for_ship(
            start_x,
            start_y,
            facing,
            attacker_game_object_id,
            laser_spec,
        );
        let ray = LaserRay {
            start_x,
            start_y,
            end_x: aim_end_x,
            end_y: aim_end_y,
        };
        let (end_x, end_y, hit_ship_id) =
            self.instant_laser_hit_result_for_ship(&ray, attacker_game_object_id);

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

        let Some(defender_game_object_id) = hit_ship_id else {
            return;
        };

        if self.ship_blocks_damage_by_ship_id(defender_game_object_id) {
            return;
        }

        self.apply_instant_laser_ship_hit_by_id(
            attacker_ship_id,
            defender_game_object_id,
            end_x,
            end_y,
            laser_spec,
        );
    }

    fn instant_laser_hit_result_for_ship(
        &self,
        ray: &LaserRay,
        attacker_game_object_id: u64,
    ) -> (f64, f64, Option<u64>) {
        let ship_candidate =
            self.instant_laser_ship_candidate_for_ship(ray, attacker_game_object_id);
        let meteor_candidate = self.instant_laser_meteor_candidate(ray);

        match (ship_candidate, meteor_candidate) {
            (Some((ship_id, x, y, ship_distance)), Some(meteor))
                if ship_distance <= meteor.2 =>
            {
                (x, y, Some(ship_id))
            }
            (Some(_), Some(meteor)) => (meteor.0, meteor.1, None),
            (Some((ship_id, x, y, _)), None) => (x, y, Some(ship_id)),
            (None, Some(meteor)) => (meteor.0, meteor.1, None),
            (None, None) => (ray.end_x, ray.end_y, None),
        }
    }

    fn instant_laser_ship_candidate_for_ship(
        &self,
        ray: &LaserRay,
        attacker_game_object_id: u64,
    ) -> Option<(u64, f64, f64, f64)> {
        self.active_ship_states()
            .into_iter()
            .filter(|ship| ship.game_object_id() != attacker_game_object_id)
            .filter(|ship| self.ship_is_targetable_by_id(ship.game_object_id()))
            .filter_map(|ship| {
                self.matter_world.body_state(ship.body_id).and_then(|body| {
                    let defender_logic = &self.ships[ship.ship_id];
                    let defender_facing = radians_to_facing_index(defender_logic.facing());
                    let defender_hit_polygon = defender_logic.hit_polygon_for_state(
                        defender_facing,
                        body.x,
                        body.y,
                        ship.special_active,
                    );
                    segment_hits_polygon(
                        ray.start_x,
                        ray.start_y,
                        ray.end_x,
                        ray.end_y,
                        &defender_hit_polygon,
                        0.0,
                    )
                    .then_some((
                        ship.game_object_id(),
                        body.x,
                        body.y,
                        segment_distance_squared_to_point(
                            ray.start_x,
                            ray.start_y,
                            body.x,
                            body.y,
                        ),
                    ))
                })
            })
            .min_by(|a, b| a.3.partial_cmp(&b.3).unwrap_or(std::cmp::Ordering::Equal))
    }

    fn instant_laser_meteor_candidate(&self, ray: &LaserRay) -> Option<(f64, f64, f64)> {
        self.meteors
            .iter()
            .filter(|meteor| {
                point_to_segment_distance_squared(
                    meteor.x,
                    meteor.y,
                    ray.start_x,
                    ray.start_y,
                    ray.end_x,
                    ray.end_y,
                ) <= meteor.radius * meteor.radius
            })
            .map(|meteor| {
                (
                    meteor.x,
                    meteor.y,
                    segment_distance_squared_to_point(ray.start_x, ray.start_y, meteor.x, meteor.y),
                )
            })
            .min_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal))
    }

    fn apply_instant_laser_ship_hit_by_id(
        &mut self,
        attacker_ship_id: usize,
        defender_game_object_id: u64,
        impact_x: f64,
        impact_y: f64,
        laser_spec: InstantLaserSpec,
    ) {
        let Some(defender_ship_id) = self
            .ship_state_by_game_object_id(defender_game_object_id)
            .map(|ship| ship.ship_id)
        else {
            return;
        };
        let died = self.ships[defender_ship_id].take_damage(laser_spec.damage);
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
            self.mark_ship_dead_by_id(defender_game_object_id);
            if let Some(key) = self.ships[attacker_ship_id].victory_sound_key() {
                self.audio_events.push(AudioEventSnapshot { key });
            }
        }
    }

    fn instant_laser_end_for_mode_for_ship(
        &self,
        start_x: f64,
        start_y: f64,
        facing: f64,
        attacker_game_object_id: u64,
        laser_spec: InstantLaserSpec,
    ) -> (f64, f64) {
        let default_end = (
            wrap_axis(start_x + (facing.cos() * laser_spec.range), self.width),
            wrap_axis(start_y + (facing.sin() * laser_spec.range), self.height),
        );

        match laser_spec.target_mode {
            ProjectileTargetMode::None => default_end,
            ProjectileTargetMode::EnemyShip => {
                let target = self.default_enemy_target_for_ship(attacker_game_object_id);
                self.projectile_target_position(target)
                .and_then(|(x, y)| {
                    point_along_range(
                        start_x,
                        start_y,
                        x,
                        y,
                        laser_spec.range,
                        self.width,
                        self.height,
                    )
                })
                .unwrap_or(default_end)
            }
            ProjectileTargetMode::PlayerSelectedOrEnemyShip => {
                let target = self.projectile_target_for_mode_for_ship(
                    attacker_game_object_id,
                    ProjectileTargetMode::PlayerSelectedOrEnemyShip,
                );
                self.projectile_target_position(target)
                .and_then(|(x, y)| {
                    point_along_range(
                        start_x,
                        start_y,
                        x,
                        y,
                        laser_spec.range,
                        self.width,
                        self.height,
                    )
                })
                .unwrap_or(default_end)
            }
            ProjectileTargetMode::PlayerSelectedPointOrForward => {
                self.projectile_target_position(
                    self.selected_weapon_target_for_ship(attacker_game_object_id),
                )
                .and_then(|(x, y)| {
                    point_along_range(
                        start_x,
                        start_y,
                        x,
                        y,
                        laser_spec.range,
                        self.width,
                        self.height,
                    )
                })
                .unwrap_or(default_end)
            }
        }
    }

    fn activate_teleport_special_for_ship(
        &mut self,
        ship_game_object_id: u64,
        body_id: usize,
        spec: TeleportSpecialSpec,
    ) {
        let Some(body) = self.matter_world.body_state(body_id) else {
            return;
        };
        let old_x = body.x;
        let old_y = body.y;
        let (new_x, new_y) = match self.special_target_for_ship(ship_game_object_id) {
            SpecialTarget::Point { x, y } => (wrap_axis(x, self.width), wrap_axis(y, self.height)),
            SpecialTarget::Ship { id } => self
                .ship_body_state_by_game_object_id(id)
                .map(|body| (body.x, body.y))
                .unwrap_or((
                    wrap_axis(body.x + (self.width * 0.33), self.width),
                    wrap_axis(body.y + (self.height * 0.27), self.height),
                )),
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

    fn special_target_for_ship(&self, ship_game_object_id: u64) -> SpecialTarget {
        if self.player.game_object_id() == ship_game_object_id {
            self.player_special_target
        } else if self.target.game_object_id() == ship_game_object_id {
            self.target_special_target
        } else if let Some(index) = self
            .additional_active_ships
            .iter()
            .position(|ship| ship.game_object_id() == ship_game_object_id)
        {
            self.additional_special_targets[index]
        } else {
            SpecialTarget::None
        }
    }

    fn activate_sound_only_special(&mut self, spec: SoundOnlySpec) {
        if !spec.sound_key.is_empty() {
            self.audio_events.push(AudioEventSnapshot {
                key: spec.sound_key,
            });
        }
    }

    fn activate_cloak_special(
        &mut self,
        is_player: bool,
        spec: crate::traits::ship_trait::CloakSpec,
    ) {
        if !spec.sound_key.is_empty() {
            self.audio_events.push(AudioEventSnapshot {
                key: spec.sound_key,
            });
        }
        let ship_state = self.ship_state_mut(is_player);
        ship_state.special_active = !ship_state.special_active;
    }

    fn activate_transform_special(&mut self, is_player: bool, spec: TransformSpec) {
        if !spec.sound_key.is_empty() {
            self.audio_events.push(AudioEventSnapshot {
                key: spec.sound_key,
            });
        }
        let ship_state = self.ship_state_mut(is_player);
        ship_state.special_active = !ship_state.special_active;
    }

    fn activate_crew_drain_special_for_ship(
        &mut self,
        attacker_game_object_id: u64,
        spec: CrewDrainTransferSpec,
    ) {
        let Some(attacker_ship) = self.ship_state_by_game_object_id(attacker_game_object_id) else {
            return;
        };
        let attacker_game_object_id = attacker_ship.game_object_id();
        let defender_game_object_id = match self.opposing_active_ship_id(attacker_game_object_id) {
            Some(id) => id,
            None => return,
        };
        let Some(defender_state) = self.ship_state_by_game_object_id(defender_game_object_id) else {
            return;
        };
        if defender_state.dead {
            return;
        }
        let attacker_body = match self.matter_world.body_state(attacker_ship.body_id) {
            Some(body) => body,
            None => return,
        };
        let defender_body = match self.matter_world.body_state(defender_state.body_id) {
            Some(body) => body,
            None => return,
        };
        let dx = shortest_wrapped_delta(attacker_body.x, defender_body.x, self.width);
        let dy = shortest_wrapped_delta(attacker_body.y, defender_body.y, self.height);
        let distance = ((dx * dx) + (dy * dy)).sqrt();
        if distance > spec.range {
            return;
        }

        let attacker_ship_id = attacker_ship.ship_id;
        let defender_ship_id = defender_state.ship_id;
        let available = (self.ships[defender_ship_id].crew() - 1).max(0);
        let capacity =
            self.ships[attacker_ship_id].max_crew() - self.ships[attacker_ship_id].crew();
        let transfer = spec.max_transfer.min(available).min(capacity);
        if transfer <= 0 {
            return;
        }

        let defender_next_crew = self.ships[defender_ship_id].crew() - transfer;
        let attacker_next_crew = self.ships[attacker_ship_id].crew() + transfer;
        self.ships[defender_ship_id].set_crew(defender_next_crew);
        self.ships[attacker_ship_id].set_crew(attacker_next_crew);
        if !spec.sound_key.is_empty() {
            self.audio_events.push(AudioEventSnapshot {
                key: spec.sound_key,
            });
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

        let next_energy =
            (self.ships[ship_id].energy() + spec.energy_gain).min(self.ships[ship_id].max_energy());
        if next_energy == self.ships[ship_id].energy() {
            return;
        }

        self.ships[ship_id].set_energy(next_energy);
        if !spec.sound_key.is_empty() {
            self.audio_events.push(AudioEventSnapshot {
                key: spec.sound_key,
            });
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
            self.audio_events.push(AudioEventSnapshot {
                key: spec.sound_key,
            });
        }

        let next_crew = self.ships[ship_id].crew() - spec.crew_cost;
        let next_energy =
            (self.ships[ship_id].energy() + spec.energy_gain).min(self.ships[ship_id].max_energy());
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
            self.audio_events.push(AudioEventSnapshot {
                key: spec.sound_key,
            });
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

        let attacker_game_object_id = self.ship_state(is_player).game_object_id();
        let Some(defender_game_object_id) = self.opposing_active_ship_id(attacker_game_object_id) else {
            self.remove_ship_without_death_effect(is_player);
            return;
        };
        if !self.ship_blocks_damage_by_ship_id(defender_game_object_id) {
            let defender_state = self
                .ship_state_by_game_object_id(defender_game_object_id)
                .expect("checked active defender ship");
            if !defender_state.dead
                && let Some(defender_body) = self.matter_world.body_state(defender_state.body_id)
            {
                let dx = shortest_wrapped_delta(defender_body.x, current.x, self.width);
                let dy = shortest_wrapped_delta(defender_body.y, current.y, self.height);
                let distance = ((dx * dx) + (dy * dy)).sqrt();
                if distance <= spec.radius {
                    let mut outcome = BattleOutcome::default();
                    if self.apply_damage_to_ship_by_id(
                        defender_game_object_id,
                        attacker_game_object_id,
                        spec.damage,
                        &mut outcome,
                    ) {
                        if self.is_player_ship_id(defender_game_object_id) {
                            self.mark_ship_dead(true);
                        } else {
                            self.mark_ship_dead(false);
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

        matches!(
            self.ships[ship_state.ship_id].special_ability_spec(),
            SpecialAbilitySpec::Shield(_)
        )
    }

    fn projectile_hit_target(&self, projectile: &ProjectileSnapshot) -> Option<u64> {
        let mut candidate_ship_ids = Vec::new();
        if let ProjectileTarget::Ship { id } = projectile.target
            && id != projectile.owner_ship_id
        {
            candidate_ship_ids.push(id);
        }
        for ship_id in self
            .active_ship_states()
            .into_iter()
            .map(|ship| ship.game_object_id())
        {
            if ship_id != projectile.owner_ship_id && !candidate_ship_ids.contains(&ship_id) {
                candidate_ship_ids.push(ship_id);
            }
        }

        for ship_id in candidate_ship_ids {
            if self.projectile_hits_ship_by_id(projectile, ship_id) {
                return Some(ship_id);
            }
        }

        None
    }

    fn projectile_hits_ship_by_id(&self, projectile: &ProjectileSnapshot, ship_id: u64) -> bool {
        let Some(ship_state) = self.ship_state_by_game_object_id(ship_id) else {
            return false;
        };
        if ship_state.dead {
            return false;
        }
        let Some(body) = self.matter_world.body_state(ship_state.body_id) else {
            return false;
        };
        let logic = &self.ships[ship_state.ship_id];
        if !logic.is_targetable(ship_state.special_active) {
            return false;
        }
        let facing = radians_to_facing_index(logic.facing());
        let hit_polygon =
            logic.hit_polygon_for_state(facing, body.x, body.y, ship_state.special_active);
        if hit_polygon.is_empty() {
            return false;
        }
        let projectile_polygon = projectile_hit_polygon(
            projectile.collision,
            projectile.facing_index,
            projectile.x,
            projectile.y,
        );
        if !projectile_polygon.is_empty() {
            return polygons_intersect(&projectile_polygon, &hit_polygon);
        }
        let start_x = body.x + shortest_wrapped_delta(body.x, projectile.previous_x, self.width);
        let start_y = body.y + shortest_wrapped_delta(body.y, projectile.previous_y, self.height);
        let end_x = body.x + shortest_wrapped_delta(body.x, projectile.x, self.width);
        let end_y = body.y + shortest_wrapped_delta(body.y, projectile.y, self.height);
        segment_hits_polygon(start_x, start_y, end_x, end_y, &hit_polygon, 0.0)
    }

    fn projectile_hits_ship(&self, projectile: &ProjectileSnapshot, is_player: bool) -> bool {
        self.projectile_hits_ship_by_id(
            projectile,
            if is_player {
                self.player.game_object_id()
            } else {
                self.target.game_object_id()
            },
        )
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
            turret_facing: ship.primary_mount_facing,
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

    fn ship_is_targetable_by_id(&self, ship_id: u64) -> bool {
        let Some(ship) = self.ship_state_by_game_object_id(ship_id) else {
            return false;
        };
        !ship.dead && self.ships[ship.ship_id].is_targetable(ship.special_active)
    }

    fn meteor_hits_ship(&self, meteor: &MeteorSnapshot, is_player: bool) -> bool {
        self.meteor_hits_ship_by_id(
            meteor,
            if is_player {
                self.player.game_object_id()
            } else {
                self.target.game_object_id()
            },
        )
    }

    fn meteor_hits_ship_by_id(&self, meteor: &MeteorSnapshot, ship_id: u64) -> bool {
        let Some(ship) = self.ship_state_by_game_object_id(ship_id) else {
            return false;
        };
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

    fn projectile_hits_meteor(
        &self,
        projectile: &ProjectileSnapshot,
        meteor: &MeteorSnapshot,
    ) -> bool {
        let projectile_polygon = projectile_hit_polygon(
            projectile.collision,
            projectile.facing_index,
            projectile.x,
            projectile.y,
        );
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

    fn ship_body_state_by_game_object_id(&self, ship_id: u64) -> Option<MatterBodyState> {
        if self.player.game_object_id() == ship_id {
            self.matter_world.body_state(self.player.body_id)
        } else if self.target.game_object_id() == ship_id {
            self.matter_world.body_state(self.target.body_id)
        } else {
            self.additional_active_ships
                .iter()
                .find(|ship| ship.game_object_id() == ship_id)
                .and_then(|ship| self.matter_world.body_state(ship.body_id))
        }
    }

    fn projectile_target_position(&self, target: ProjectileTarget) -> Option<(f64, f64)> {
        match target {
            ProjectileTarget::None => None,
            ProjectileTarget::Point { x, y } => Some((x, y)),
            ProjectileTarget::Ship { id } => self
                .ship_body_state_by_game_object_id(id)
                .map(|body| (body.x, body.y)),
        }
    }

    fn apply_blazer_collision_velocity(&mut self, input: &BlazerCollisionInput) {
        let dx = input.victim_before.x - input.blazer_before.x;
        let dy = input.victim_before.y - input.blazer_before.y;
        let distance = (dx * dx + dy * dy).sqrt();
        let (nx, ny) = if distance <= f64::EPSILON {
            (1.0, 0.0)
        } else {
            (dx / distance, dy / distance)
        };
        let blazer_speed = (input.blazer_before.vx.powi(2) + input.blazer_before.vy.powi(2)).sqrt();
        let ((blazer_vx, blazer_vy), (victim_vx, victim_vy)) = resolve_collision_velocity(
            CollisionBody {
                x: input.blazer_before.x,
                y: input.blazer_before.y,
                vx: nx * blazer_speed,
                vy: ny * blazer_speed,
                mass: input.blazer_mass,
            },
            CollisionBody {
                x: input.victim_before.x,
                y: input.victim_before.y,
                vx: input.victim_before.vx,
                vy: input.victim_before.vy,
                mass: input.victim_mass,
            },
        );
        self.matter_world
            .set_body_velocity(input.blazer_body_id, blazer_vx, blazer_vy);
        self.matter_world
            .set_body_velocity(input.victim_body_id, victim_vx, victim_vy);
    }

    fn blazer_spec_for(
        &self,
        is_player: bool,
    ) -> Option<crate::traits::ship_trait::BlazerSpecialSpec> {
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

        let dead_ship_id = if is_player {
            self.player.game_object_id()
        } else {
            self.target.game_object_id()
        };
        let mut remaining_boarders = Vec::with_capacity(self.marine_boarders.len());
        let mut active_boarders = std::mem::take(&mut self.marine_boarders);
        for boarder in active_boarders.drain(..) {
            if boarder.defender_ship_id == dead_ship_id {
                self.launch_returning_marine(boarder.defender_ship_id, boarder.owner_ship_id);
            } else {
                remaining_boarders.push(boarder);
            }
        }
        self.marine_boarders = remaining_boarders;

        let ship = if is_player {
            &mut self.player
        } else {
            &mut self.target
        };
        ship.dead = true;
        ship.thrusting = false;
        self.matter_world.disable_body(body_id);
    }

    fn mark_additional_ship_dead(&mut self, index: usize) {
        let Some(ship) = self.additional_active_ships.get(index) else {
            return;
        };
        if ship.dead {
            return;
        }
        let body_id = ship.body_id;
        let dead_ship_id = ship.game_object_id();

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

        let mut remaining_boarders = Vec::with_capacity(self.marine_boarders.len());
        let mut active_boarders = std::mem::take(&mut self.marine_boarders);
        for boarder in active_boarders.drain(..) {
            if boarder.defender_ship_id == dead_ship_id {
                self.launch_returning_marine(boarder.defender_ship_id, boarder.owner_ship_id);
            } else {
                remaining_boarders.push(boarder);
            }
        }
        self.marine_boarders = remaining_boarders;

        if let Some(ship) = self.additional_active_ships.get_mut(index) {
            ship.dead = true;
            ship.thrusting = false;
        }
        self.matter_world.disable_body(body_id);
    }
}

fn track_projectile(projectile: &mut ProjectileSnapshot, target_pos: Option<(f64, f64)>) {
    let Some((tx, ty)) = target_pos else {
        return;
    };
    if projectile.turn_wait > 0 {
        projectile.turn_wait -= 1;
        return;
    }
    projectile.facing_index = turn_projectile_toward_target(
        projectile.facing_index,
        projectile.x,
        projectile.y,
        tx,
        ty,
    );
    projectile.facing = facing_index_to_radians(projectile.facing_index);
    projectile.turn_wait = projectile.track_wait;
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

fn projectile_hit_radius(collision: ProjectileCollisionSpec) -> f64 {
    match collision {
        ProjectileCollisionSpec::None => 0.0,
        ProjectileCollisionSpec::Polygon(points) => points
            .iter()
            .map(|point| ((point.x * point.x) + (point.y * point.y)).sqrt())
            .fold(0.0, f64::max),
    }
}

fn ship_hit_radius(polygon: &[HitPolygonPoint], center_x: f64, center_y: f64) -> f64 {
    polygon
        .iter()
        .map(|point| {
            let dx = point.x - center_x;
            let dy = point.y - center_y;
            ((dx * dx) + (dy * dy)).sqrt()
        })
        .fold(0.0, f64::max)
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
        .map(|point| Vec2 {
            x: point.x,
            y: point.y,
        })
        .collect::<Vec<_>>();
    matter_world.create_ship_polygon_body(x, y, &vertices, ship.mass(), 0.8, ship_body_angle(ship))
}

fn sync_ship_body_angle(matter_world: &mut MatterWorld, body_id: usize, ship: &AnyShip) {
    matter_world.set_body_angle(body_id, ship_body_angle(ship));
}

fn ship_body_angle(ship: &AnyShip) -> f64 {
    ship.facing() + std::f64::consts::FRAC_PI_2
}

fn is_weapon_triggered(
    ship: &AnyShip,
    input: &ShipInput,
    weapon_counter_before: i32,
    energy_before: i32,
    energy_counter_before: i32,
) -> bool {
    input.weapon
        && weapon_counter_before == 0
        && effective_energy_for_input(ship, energy_before, energy_counter_before)
            >= ship.weapon_energy_cost()
}

fn is_special_triggered(
    ship: &AnyShip,
    input: &ShipInput,
    special_counter_before: i32,
    energy_before: i32,
    energy_counter_before: i32,
) -> bool {
    input.special
        && special_counter_before == 0
        && effective_energy_for_input(ship, energy_before, energy_counter_before)
            >= ship.special_energy_cost()
}

fn effective_energy_for_input(
    ship: &AnyShip,
    energy_before: i32,
    energy_counter_before: i32,
) -> i32 {
    if energy_counter_before == 0 && energy_before < ship.max_energy() {
        (energy_before + ship.energy_regeneration()).min(ship.max_energy())
    } else {
        energy_before
    }
}

fn apply_commands(
    matter_world: &mut MatterWorld,
    body_id: usize,
    commands: Vec<PhysicsCommand>,
) -> bool {
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

fn rotate_toward_angle(current: f64, desired: f64, max_step: f64) -> f64 {
    let delta = (desired - current + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU)
        - std::f64::consts::PI;
    if delta.abs() <= max_step {
        desired
    } else {
        current + (max_step * delta.signum())
    }
}

fn radians_to_facing_index(facing: f64) -> i32 {
    let angle = (facing + std::f64::consts::FRAC_PI_2).rem_euclid(std::f64::consts::TAU);
    (angle / (std::f64::consts::TAU / PROJECTILE_FACINGS)).round() as i32
        % PROJECTILE_FACINGS as i32
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

fn set_projectile_velocity_components(
    projectile: &mut ProjectileSnapshot,
    raw_vx: i32,
    raw_vy: i32,
) {
    projectile.raw_vx = raw_vx;
    projectile.raw_vy = raw_vy;
    projectile.vx = (raw_vx as f64 / 32.0) + projectile.inherited_vx;
    projectile.vy = (raw_vy as f64 / 32.0) + projectile.inherited_vy;

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
    projectile.x += dx as f64 + projectile.inherited_vx;
    projectile.y += dy as f64 + projectile.inherited_vy;
}

fn next_sc2_velocity_step(vector: i32, fract: i32, error: &mut i32, sign: i32) -> i32 {
    let e = *error + fract;
    let step = vector + (sign * (e >> 5));
    *error = e & 31;
    step
}

fn step_wobble_tracking_projectile(
    projectile: &mut ProjectileSnapshot,
    target_position: Option<(f64, f64)>,
) {
    let ProjectileBehaviorSpec::WobbleTracking {
        direct_track_range, ..
    } = projectile.behavior
    else {
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
    } else if let Some((target_x, target_y)) = target_position {
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
    let (raw_vx, raw_vy) =
        projectile_velocity_for_facing(projectile.facing_index, projectile.speed);
    set_projectile_velocity_components(projectile, raw_vx, raw_vy);
    advance_projectile_position(projectile);
    projectile.life -= 1;
}

fn projectile_target_position_from_states(
    target: ProjectileTarget,
    active_ship_target_bodies: &[(u64, Option<MatterBodyState>)],
) -> Option<(f64, f64)> {
    match target {
        ProjectileTarget::None => None,
        ProjectileTarget::Point { x, y } => Some((x, y)),
        ProjectileTarget::Ship { id } => active_ship_target_bodies
            .iter()
            .find(|(ship_id, _)| *ship_id == id)
            .and_then(|(_, body)| body.map(|body| (body.x, body.y))),
    }
}

fn projectile_hits_planet(
    projectile_x: f64,
    projectile_y: f64,
    planet_x: f64,
    planet_y: f64,
    width: f64,
    height: f64,
    radius: f64,
) -> bool {
    let dx = shortest_wrapped_delta(projectile_x, planet_x, width);
    let dy = shortest_wrapped_delta(projectile_y, planet_y, height);
    ((dx * dx) + (dy * dy)).sqrt() <= radius
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
            Segment {
                start_x,
                start_y,
                end_x,
                end_y,
            },
            Segment {
                start_x: edge_start.x,
                start_y: edge_start.y,
                end_x: edge_end.x,
                end_y: edge_end.y,
            },
        ) <= radius * radius
        {
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
            if segments_intersect(
                Segment {
                    start_x: a_start.x,
                    start_y: a_start.y,
                    end_x: a_end.x,
                    end_y: a_end.y,
                },
                Segment {
                    start_x: b_start.x,
                    start_y: b_start.y,
                    end_x: b_end.x,
                    end_y: b_end.y,
                },
            ) {
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
        if point_to_segment_distance_squared(
            circle_x, circle_y, previous.x, previous.y, current.x, current.y,
        ) <= radius_sq
        {
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
            && (x
                < ((previous.x - current.x) * (y - current.y) / (previous.y - current.y)
                    + current.x));
        if crosses {
            inside = !inside;
        }
        previous = *current;
    }

    inside
}

fn segment_to_segment_distance_squared(a: Segment, b: Segment) -> f64 {
    if segments_intersect(a, b) {
        return 0.0;
    }

    point_to_segment_distance_squared(a.start_x, a.start_y, b.start_x, b.start_y, b.end_x, b.end_y)
        .min(point_to_segment_distance_squared(
            a.end_x, a.end_y, b.start_x, b.start_y, b.end_x, b.end_y,
        ))
        .min(point_to_segment_distance_squared(
            b.start_x, b.start_y, a.start_x, a.start_y, a.end_x, a.end_y,
        ))
        .min(point_to_segment_distance_squared(
            b.end_x, b.end_y, a.start_x, a.start_y, a.end_x, a.end_y,
        ))
}

fn segment_distance_squared_to_point(
    start_x: f64,
    start_y: f64,
    point_x: f64,
    point_y: f64,
) -> f64 {
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

fn point_to_segment_distance_squared(px: f64, py: f64, ax: f64, ay: f64, bx: f64, by: f64) -> f64 {
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

fn segments_intersect(a: Segment, b: Segment) -> bool {
    let ab = orientation(a.start_x, a.start_y, a.end_x, a.end_y, b.start_x, b.start_y);
    let ac = orientation(a.start_x, a.start_y, a.end_x, a.end_y, b.end_x, b.end_y);
    let cd = orientation(b.start_x, b.start_y, b.end_x, b.end_y, a.start_x, a.start_y);
    let ca = orientation(b.start_x, b.start_y, b.end_x, b.end_y, a.end_x, a.end_y);

    (ab == 0.0 && on_segment(a.start_x, a.start_y, a.end_x, a.end_y, b.start_x, b.start_y))
        || (ac == 0.0 && on_segment(a.start_x, a.start_y, a.end_x, a.end_y, b.end_x, b.end_y))
        || (cd == 0.0 && on_segment(b.start_x, b.start_y, b.end_x, b.end_y, a.start_x, a.start_y))
        || (ca == 0.0 && on_segment(b.start_x, b.start_y, b.end_x, b.end_y, a.end_x, a.end_y))
        || ((ab > 0.0) != (ac > 0.0) && (cd > 0.0) != (ca > 0.0))
}

fn resolve_collision_velocity(
    player: CollisionBody,
    target: CollisionBody,
) -> ((f64, f64), (f64, f64)) {
    let dx = target.x - player.x;
    let dy = target.y - player.y;
    let distance = (dx * dx + dy * dy).sqrt();
    let (nx, ny) = if distance <= f64::EPSILON {
        (1.0, 0.0)
    } else {
        (dx / distance, dy / distance)
    };

    let player_normal = (player.vx * nx) + (player.vy * ny);
    let target_normal = (target.vx * nx) + (target.vy * ny);
    let player_tangent_x = player.vx - (player_normal * nx);
    let player_tangent_y = player.vy - (player_normal * ny);
    let target_tangent_x = target.vx - (target_normal * nx);
    let target_tangent_y = target.vy - (target_normal * ny);

    let player_bounced_normal = ((player_normal * (player.mass - target.mass))
        + (2.0 * target.mass * target_normal))
        / (player.mass + target.mass);
    let target_bounced_normal = ((target_normal * (target.mass - player.mass))
        + (2.0 * player.mass * player_normal))
        / (player.mass + target.mass);

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
    use super::{
        Battle, BattleOutcome, ProjectileSnapshot, ProjectileTarget, INITIAL_PLAYER_ID,
        INITIAL_TARGET_ID,
    };
    use crate::reference_data;
    use crate::ship_input::ShipInput;
    use crate::ships::{AnyShip, HumanCruiser};
    use crate::traits::ship_trait::Ship;

    #[test]
    fn added_active_ship_can_fire_primary_weapon() {
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
        let ship_id = battle
            .add_active_ship("human-cruiser", 700.0, 500.0)
            .unwrap();

        battle
            .set_input_for_ship(
                ship_id,
                ShipInput {
                    left: false,
                    right: false,
                    thrust: false,
                    weapon: true,
                    special: false,
                },
            )
            .unwrap();
        battle.tick(1000.0 / 60.0);

        assert_eq!(battle.snapshot().projectiles.len(), 1);
    }

    #[test]
    fn meteor_collision_damages_added_active_ship_once() {
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
        )
        .expect("battle should build");
        let ship_id = battle
            .add_active_ship("human-cruiser", 6200.0, 5600.0)
            .unwrap();
        battle.meteors[0].x = 6200.0;
        battle.meteors[0].y = 5600.0;
        battle.meteors[0].vx = 0.0;
        battle.meteors[0].vy = 0.0;

        battle.tick(1000.0 / 24.0);

        assert_eq!(
            battle
                .snapshot()
                .ships
                .into_iter()
                .find(|ship| ship.id == ship_id)
                .unwrap()
                .crew,
            17
        );
    }

    #[test]
    fn normal_ship_collision_with_added_active_ship_sets_collision_cooldowns() {
        let mut battle = Battle::new(
            "androsynth-guardian",
            "human-cruiser",
            5000.0,
            5000.0,
            7000.0,
            5000.0,
            0.0,
            0.0,
            8000.0,
            8000.0,
        )
        .expect("battle should build");
        let ship_id = battle
            .add_active_ship("human-cruiser", 5060.0, 5000.0)
            .unwrap();
        let added_ship_index = battle
            .additional_active_ships
            .iter()
            .find(|ship| ship.id == ship_id)
            .unwrap()
            .ship_id;
        let added_body_id = battle
            .additional_active_ships
            .iter()
            .find(|ship| ship.id == ship_id)
            .unwrap()
            .body_id;
        battle
            .matter_world
            .set_body_velocity(battle.player.body_id, 2.0, 0.0);
        battle
            .matter_world
            .set_body_velocity(added_body_id, -2.0, 0.0);

        battle.tick(1000.0 / 24.0);

        assert_eq!(
            (
                battle.ships[battle.player.ship_id].turn_counter(),
                battle.ships[battle.player.ship_id].thrust_counter(),
                battle.ships[added_ship_index].turn_counter(),
                battle.ships[added_ship_index].thrust_counter(),
            ),
            (1, 3, 1, 3),
        );
    }

    #[test]
    fn added_active_ship_can_be_targeted_by_player_selected_laser() {
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
        )
        .unwrap();
        let player_id = battle.snapshot().player.id;
        let target_ship_id = battle.add_active_ship("human-cruiser", 6200.0, 5600.0).unwrap();

        battle
            .set_weapon_target_ship_for(player_id, target_ship_id)
            .unwrap();
        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: true,
            special: false,
        });
        battle.tick(1000.0 / 24.0);

        assert!(battle.snapshot().lasers[0].end_y > battle.snapshot().lasers[0].start_y);
    }

    #[test]
    fn added_active_ship_instant_laser_damages_default_enemy_in_range() {
        let mut battle = Battle::new(
            "human-cruiser",
            "human-cruiser",
            5000.0,
            4928.0,
            7000.0,
            7000.0,
            0.0,
            0.0,
            8000.0,
            8000.0,
        )
        .expect("battle should build");
        let ship_id = battle
            .add_active_ship("slylandro-probe", 5000.0, 5000.0)
            .unwrap();
        battle
            .set_input_for_ship(
                ship_id,
                ShipInput {
                    left: false,
                    right: false,
                    thrust: false,
                    weapon: true,
                    special: false,
                },
            )
            .unwrap();
        battle.tick(1000.0 / 24.0);

        assert_eq!(battle.snapshot().player.crew, 16);
    }

    #[test]
    fn added_active_ship_teleports_to_selected_point() {
        let mut battle = Battle::new(
            "human-cruiser",
            "human-cruiser",
            5000.0,
            5000.0,
            6500.0,
            5000.0,
            0.0,
            0.0,
            8000.0,
            8000.0,
        )
        .expect("battle should build");
        let ship_id = battle.add_active_ship("arilou-skiff", 6000.0, 6000.0).unwrap();

        battle
            .set_special_target_point_for(ship_id, 1234.0, 2345.0)
            .unwrap();
        battle
            .set_input_for_ship(
                ship_id,
                ShipInput {
                    left: false,
                    right: false,
                    thrust: false,
                    weapon: false,
                    special: true,
                },
            )
            .unwrap();
        battle.tick(1000.0 / 24.0);

        let ship = battle
            .snapshot()
            .ships
            .into_iter()
            .find(|ship| ship.id == ship_id)
            .unwrap();
        assert_eq!((ship.x.round() as i32, ship.y.round() as i32), (1234, 2345));
    }

    #[test]
    fn added_active_ship_syreen_special_transfers_enemy_crew() {
        let mut battle = Battle::new(
            "human-cruiser",
            "human-cruiser",
            5150.0,
            5000.0,
            7000.0,
            5000.0,
            0.0,
            0.0,
            8000.0,
            8000.0,
        )
        .expect("battle should build");
        let ship_id = battle
            .add_active_ship("syreen-penetrator", 5000.0, 5000.0)
            .unwrap();
        let added_ship_index = battle
            .additional_active_ships
            .iter()
            .find(|ship| ship.id == ship_id)
            .unwrap()
            .ship_id;
        battle.ships[added_ship_index].set_crew(4);

        battle
            .set_input_for_ship(
                ship_id,
                ShipInput {
                    left: false,
                    right: false,
                    thrust: false,
                    weapon: false,
                    special: true,
                },
            )
            .unwrap();
        battle.tick(1000.0 / 24.0);

        let ship = battle
            .snapshot()
            .ships
            .into_iter()
            .find(|ship| ship.id == ship_id)
            .unwrap();
        assert_eq!((ship.crew, battle.snapshot().player.crew), (12, 10));
    }

    #[test]
    fn target_slot_can_explicitly_target_added_active_ship() {
        let mut battle = Battle::new(
            "human-cruiser",
            "human-cruiser",
            4200.0,
            4200.0,
            5200.0,
            4200.0,
            5000.0,
            5000.0,
            10000.0,
            10000.0,
        )
        .unwrap();
        let additional_ship_id = battle.add_active_ship("human-cruiser", 5200.0, 5200.0).unwrap();
        let target_ship_id = battle.snapshot().target.id;
        battle
            .set_weapon_target_ship_for(target_ship_id, additional_ship_id)
            .unwrap();
        battle.set_target_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: true,
            special: false,
        });

        battle.tick(1000.0 / 24.0);

        assert!(matches!(
            battle.snapshot().projectiles[0].target,
            ProjectileTarget::Ship { id } if id == additional_ship_id
        ));
    }

    #[test]
    fn added_active_ship_can_be_marked_dead_from_outcome() {
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
        let ship_id = battle
            .add_active_ship("human-cruiser", 700.0, 500.0)
            .unwrap();
        let mut outcome = BattleOutcome::default();
        outcome.died_ship_ids.push(ship_id);

        battle.resolve_death_and_victory(&outcome);

        assert!(
            battle
                .snapshot()
                .ships
                .into_iter()
                .find(|ship| ship.id == ship_id)
                .unwrap()
                .dead
        );
    }

    #[test]
    fn added_active_ship_appears_in_snapshot() {
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

        battle
            .add_active_ship("human-cruiser", 700.0, 500.0)
            .unwrap();

        assert_eq!(battle.snapshot().ships.len(), 3);
    }

    #[test]
    fn added_active_ship_moves_when_input_is_set() {
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
        let ship_id = battle
            .add_active_ship("human-cruiser", 700.0, 500.0)
            .unwrap();
        let before = battle
            .snapshot()
            .ships
            .into_iter()
            .find(|ship| ship.id == ship_id)
            .unwrap()
            .x;

        battle
            .set_input_for_ship(
                ship_id,
                ShipInput {
                    left: false,
                    right: false,
                    thrust: true,
                    weapon: false,
                    special: false,
                },
            )
            .unwrap();
        battle.tick(1000.0 / 60.0);

        assert_ne!(
            battle
                .snapshot()
                .ships
                .into_iter()
                .find(|ship| ship.id == ship_id)
                .unwrap()
                .x,
            before
        );
    }

    #[test]
    fn player_projectile_uses_player_ship_id_as_owner() {
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
        let player_id = battle.snapshot().player.id;

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: true,
            special: false,
        });
        battle.tick(1000.0 / 60.0);

        assert_eq!(battle.projectiles[0].owner_ship_id, player_id);
    }

    #[test]
    fn active_ship_ids_expose_both_active_slots() {
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

        assert_eq!(battle.active_ship_ids(), vec![INITIAL_PLAYER_ID, INITIAL_TARGET_ID]);
    }

    #[test]
    fn player_weapon_target_ship_uses_target_ship_id() {
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
        let target_id = battle.snapshot().target.id;

        battle.set_player_weapon_target_ship();
        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: true,
            special: false,
        });
        battle.tick(1000.0 / 60.0);

        assert!(matches!(
            battle.projectiles[0].target,
            ProjectileTarget::Ship { id } if id == target_id
        ));
    }

    #[test]
    fn switch_ship_for_replaces_the_matching_ship() {
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
        let target_id = battle.snapshot().target.id;
        let original_texture = battle.snapshot().target.texture_prefix;

        battle.switch_ship_for(target_id, "yehat-terminator").unwrap();

        assert_ne!(battle.snapshot().target.texture_prefix, original_texture);
    }

    #[test]
    fn set_input_for_ship_controls_the_matching_ship() {
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
        let target_id = battle.snapshot().target.id;

        battle
            .set_input_for_ship(
                target_id,
                ShipInput {
                    left: false,
                    right: false,
                    thrust: true,
                    weapon: false,
                    special: false,
                },
            )
            .unwrap();
        battle.tick(1000.0 / 60.0);

        assert_ne!(
            (battle.snapshot().target.x, battle.snapshot().target.y),
            (300.0, 400.0)
        );
    }

    #[test]
    fn snapshot_exposes_all_ships_in_world_coordinates() {
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

        assert_eq!(
            battle
                .snapshot()
                .ships
                .iter()
                .map(|ship| (ship.x, ship.y))
                .collect::<Vec<_>>(),
            vec![(100.0, 200.0), (300.0, 400.0)]
        );
    }

    #[test]
    fn orz_special_launch_costs_one_crew() {
        let mut battle = Battle::new(
            "orz-nemesis",
            "human-cruiser",
            5000.0,
            5000.0,
            7000.0,
            5000.0,
            0.0,
            0.0,
            8000.0,
            8000.0,
        )
        .expect("battle should build");
        let player_ship_id = battle.player.ship_id;
        let crew_before = battle.ships[player_ship_id].crew();

        battle.set_player_special_target_ship();
        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: false,
            special: true,
        });
        battle.tick(1000.0 / 24.0);

        assert_eq!(battle.ships[player_ship_id].crew(), crew_before - 1);
    }

    #[test]
    fn orz_marine_lifetime_does_not_count_down() {
        let mut battle = Battle::new(
            "orz-nemesis",
            "human-cruiser",
            5000.0,
            5000.0,
            7000.0,
            5000.0,
            0.0,
            0.0,
            8000.0,
            8000.0,
        )
        .expect("battle should build");

        battle.set_player_special_target_ship();
        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: false,
            special: true,
        });
        battle.tick(1000.0 / 24.0);
        let life_before = battle
            .snapshot()
            .projectiles
            .iter()
            .find(|projectile| projectile.texture_prefix == "orz-turret")
            .map(|projectile| projectile.life)
            .expect("expected orz marine projectile");

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: false,
            special: false,
        });
        battle.tick(1000.0 / 24.0);
        let life_after = battle
            .snapshot()
            .projectiles
            .iter()
            .find(|projectile| projectile.texture_prefix == "orz-turret")
            .map(|projectile| projectile.life)
            .expect("expected orz marine projectile");

        assert_eq!(life_after, life_before);
    }

    #[test]
    fn boarded_marine_returns_to_orz_when_target_ship_dies() {
        let mut battle = Battle::new(
            "orz-nemesis",
            "human-cruiser",
            5000.0,
            5000.0,
            5050.0,
            5000.0,
            0.0,
            0.0,
            8000.0,
            8000.0,
        )
        .expect("battle should build");
        battle.meteors.clear();

        let player_ship_id = battle.player.ship_id;
        battle.ships[player_ship_id].set_crew(5);
        let mut outcome = BattleOutcome::default();
        battle.apply_marine_boarding_hit(false, true, &mut outcome);
        battle.mark_ship_dead(false);
        for _ in 0..240 {
            battle.tick(1000.0 / 24.0);
            if battle.ships[player_ship_id].crew() > 5 {
                break;
            }
        }

        assert_eq!(battle.ships[player_ship_id].crew(), 6);
    }

    #[test]
    fn orz_special_requires_explicit_enemy_target_to_launch() {
        let mut battle = Battle::new(
            "orz-nemesis",
            "human-cruiser",
            5000.0,
            5000.0,
            7000.0,
            5000.0,
            0.0,
            0.0,
            8000.0,
            8000.0,
        )
        .expect("battle should build");
        let player_ship_id = battle.player.ship_id;
        let crew_before = battle.ships[player_ship_id].crew();

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: false,
            special: true,
        });
        battle.tick(1000.0 / 24.0);

        assert_eq!(battle.ships[player_ship_id].crew(), crew_before);
    }

    #[test]
    fn orz_special_emits_orz_secondary_audio_event() {
        let mut battle = Battle::new(
            "orz-nemesis",
            "human-cruiser",
            5000.0,
            5000.0,
            7000.0,
            5000.0,
            0.0,
            0.0,
            8000.0,
            8000.0,
        )
        .expect("battle should build");

        battle.set_player_special_target_ship();
        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: false,
            special: true,
        });
        battle.tick(1000.0 / 24.0);

        assert!(
            battle
                .snapshot()
                .audio_events
                .iter()
                .any(|event| event.key == "orz-secondary")
        );
    }

    #[test]
    fn orz_marine_boarding_emits_intruder_audio_event() {
        let mut battle = Battle::new(
            "orz-nemesis",
            "human-cruiser",
            5000.0,
            5000.0,
            5200.0,
            5000.0,
            0.0,
            0.0,
            8000.0,
            8000.0,
        )
        .expect("battle should build");
        battle.set_player_special_target_ship();

        let mut intruder_audio_seen = false;
        for _ in 0..40 {
            battle.set_player_input(ShipInput {
                left: false,
                right: false,
                thrust: false,
                weapon: false,
                special: true,
            });
            battle.tick(1000.0 / 24.0);
            if battle
                .snapshot()
                .audio_events
                .iter()
                .any(|event| event.key == "orz-intruder")
            {
                intruder_audio_seen = true;
                break;
            }
        }

        assert!(intruder_audio_seen);
    }

    #[test]
    fn orz_primary_emits_orz_primary_audio_event() {
        let mut battle = Battle::new(
            "orz-nemesis",
            "human-cruiser",
            5000.0,
            5000.0,
            7000.0,
            5000.0,
            0.0,
            0.0,
            8000.0,
            8000.0,
        )
        .expect("battle should build");

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: true,
            special: false,
        });
        battle.tick(1000.0 / 24.0);

        assert!(
            battle
                .snapshot()
                .audio_events
                .iter()
                .any(|event| event.key == "orz-primary")
        );
    }

    #[test]
    fn orz_primary_projectile_does_not_home_after_spawn() {
        let mut battle = Battle::new(
            "orz-nemesis",
            "human-cruiser",
            5000.0,
            5000.0,
            7000.0,
            5000.0,
            0.0,
            0.0,
            8000.0,
            8000.0,
        )
        .expect("battle should build");

        battle.set_player_weapon_target_point(7000.0, 6500.0);
        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: true,
            special: false,
        });
        battle.tick(1000.0 / 24.0);
        let first_vx = battle.snapshot().projectiles[0].vx;

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: false,
            special: false,
        });
        battle.tick(1000.0 / 24.0);

        assert_eq!(battle.snapshot().projectiles[0].vx, first_vx);
    }

    #[test]
    fn orz_primary_turret_turn_rate_limits_first_shot_direction() {
        let mut battle = Battle::new(
            "orz-nemesis",
            "human-cruiser",
            5000.0,
            5000.0,
            7000.0,
            5000.0,
            0.0,
            0.0,
            8000.0,
            8000.0,
        )
        .expect("battle should build");

        battle.set_player_weapon_target_point(7000.0, 5000.0);
        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: true,
            special: false,
        });
        battle.tick(1000.0 / 24.0);

        assert!(battle.snapshot().projectiles[0].vy < -1.0);
    }

    #[test]
    fn yehat_primary_inherits_player_ship_velocity() {
        let mut battle = Battle::new(
            "yehat-terminator",
            "human-cruiser",
            5000.0,
            5000.0,
            7000.0,
            5000.0,
            0.0,
            0.0,
            8000.0,
            8000.0,
        )
        .expect("battle should build");

        battle
            .matter_world
            .set_body_velocity(battle.player.body_id, 0.0, -5.0);
        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: true,
            special: false,
        });
        battle.tick(1000.0 / 24.0);

        assert!(battle.snapshot().projectiles[0].vy < -20.5);
    }

    #[test]
    fn human_weapon_does_not_fire_without_enough_energy() {
        let mut battle = Battle::new(
            "human-cruiser",
            "human-cruiser",
            5000.0,
            5000.0,
            7000.0,
            5000.0,
            0.0,
            0.0,
            8000.0,
            8000.0,
        )
        .expect("battle should build");
        let ship_id = battle.player.ship_id;
        battle.ships[ship_id].set_energy(0);
        battle.ships[ship_id].set_energy_counter(5);

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: true,
            special: false,
        });
        battle.tick(1000.0 / 24.0);

        assert_eq!(battle.snapshot().projectiles.len(), 0);
    }

    #[test]
    fn yehat_cannot_fire_primary_while_shield_is_active() {
        let mut battle = Battle::new(
            "yehat-terminator",
            "human-cruiser",
            5000.0,
            5000.0,
            7000.0,
            5000.0,
            0.0,
            0.0,
            8000.0,
            8000.0,
        )
        .expect("battle should build");

        battle.set_player_input(ShipInput {
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
            special: true,
        });
        battle.tick(1000.0 / 24.0);

        assert_eq!(battle.snapshot().projectiles.len(), 0);
    }

    #[test]
    fn yehat_shield_stays_active_while_holding_special() {
        let mut battle = Battle::new(
            "yehat-terminator",
            "human-cruiser",
            5000.0,
            5000.0,
            7000.0,
            5000.0,
            0.0,
            0.0,
            8000.0,
            8000.0,
        )
        .expect("battle should build");

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: false,
            special: true,
        });
        battle.tick(1000.0 / 24.0);
        battle.tick(1000.0 / 24.0);
        battle.tick(1000.0 / 24.0);

        assert_eq!(battle.snapshot().player.texture_prefix, "yehat-shield");
    }

    #[test]
    fn yehat_special_shield_keeps_audio_while_active() {
        let mut battle = Battle::new(
            "yehat-terminator",
            "human-cruiser",
            5000.0,
            5000.0,
            7000.0,
            5000.0,
            0.0,
            0.0,
            8000.0,
            8000.0,
        )
        .expect("battle should build");

        battle.set_player_input(ShipInput {
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
            weapon: false,
            special: false,
        });
        battle.tick(1000.0 / 24.0);

        assert!(
            battle
                .snapshot()
                .audio_events
                .iter()
                .any(|event| event.key == "yehat-special")
        );
    }

    #[test]
    fn yehat_primary_adds_audio_event() {
        let mut battle = Battle::new(
            "yehat-terminator",
            "human-cruiser",
            5000.0,
            5000.0,
            7000.0,
            5000.0,
            0.0,
            0.0,
            8000.0,
            8000.0,
        )
        .expect("battle should build");

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: true,
            special: false,
        });
        battle.tick(1000.0 / 24.0);

        assert!(
            battle
                .snapshot()
                .audio_events
                .iter()
                .any(|event| event.key == "yehat-primary")
        );
    }

    #[test]
    fn yehat_special_shield_adds_audio_event() {
        let mut battle = Battle::new(
            "yehat-terminator",
            "human-cruiser",
            5000.0,
            5000.0,
            7000.0,
            5000.0,
            0.0,
            0.0,
            8000.0,
            8000.0,
        )
        .expect("battle should build");

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: false,
            special: true,
        });
        battle.tick(1000.0 / 24.0);

        assert!(
            battle
                .snapshot()
                .audio_events
                .iter()
                .any(|event| event.key == "yehat-special")
        );
    }

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
        )
        .expect("battle should build");

        battle.set_target_weapon_target_point(6500.0, 5600.0);
        battle.set_target_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: true,
            special: false,
        });
        battle.tick(1000.0 / 24.0);

        assert!(battle.snapshot().lasers[0].end_y > 5000.0);
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
        )
        .expect("battle should build");

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: true,
            special: false,
        });
        battle.tick(1000.0 / 24.0);

        assert!(
            battle
                .snapshot()
                .audio_events
                .iter()
                .any(|event| event.key == "arilou-primary")
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
        )
        .expect("battle should build");

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
        )
        .expect("battle should build");

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
        )
        .expect("battle should build");
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
        )
        .expect("battle should build");

        battle.set_player_special_target_point(1234.0, 2345.0);
        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: false,
            special: true,
        });
        battle.tick(1000.0 / 24.0);

        assert!(
            battle
                .snapshot()
                .audio_events
                .iter()
                .any(|event| event.key == "arilou-special")
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
        )
        .expect("battle should build");

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
        )
        .expect("battle should build");

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

        assert_eq!(
            (battle.snapshot().player.vx, battle.snapshot().player.vy),
            (0.0, 0.0)
        );
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
        )
        .expect("battle should build");

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
        )
        .expect("battle should build");
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
        )
        .expect("battle should build");

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
            battle
                .snapshot()
                .projectiles
                .first()
                .map(|projectile| projectile.texture_prefix),
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
        )
        .expect("battle should build");

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
        )
        .expect("battle should build");
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
        )
        .expect("battle should build");
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
        )
        .expect("battle should build");
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
            (
                battle.snapshot().player.crew,
                battle.snapshot().player.energy
            ),
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
        )
        .expect("battle should build");

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
        )
        .expect("battle should build");

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
        )
        .expect("battle should build");

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
        )
        .expect("battle should build");

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
        )
        .expect("battle should build");
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
        )
        .expect("battle should build");

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: false,
            special: true,
        });

        battle.tick(1000.0 / 24.0);

        assert_eq!((battle.snapshot().target.vx * 100.0).round() as i32, 286,);
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
        )
        .expect("battle should build");

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
        battle
            .matter_world
            .set_body_velocity(battle.player.body_id, 10.0, 0.0);

        battle.tick(1000.0 / 24.0);

        assert_eq!((battle.snapshot().target.vx * 100.0).round() as i32, 490,);
    }

    #[test]
    fn collision_velocity_bounces_a_light_blazer_back_harder_than_a_human_cruiser() {
        let ((player_vx, _), (target_vx, _)) = super::resolve_collision_velocity(
            super::CollisionBody {
                x: 5000.0,
                y: 5000.0,
                vx: 10.0,
                vy: 0.0,
                mass: 1.0,
            },
            super::CollisionBody {
                x: 5100.0,
                y: 5000.0,
                vx: 0.0,
                vy: 0.0,
                mass: 6.0,
            },
        );

        assert_eq!(
            (
                (player_vx * 100.0).round() as i32,
                (target_vx * 100.0).round() as i32
            ),
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
        let projectile_polygon =
            super::projectile_hit_polygon(projectile_collision, 15, 4763.0, 1728.0);
        let ship_polygon = ship.hit_polygon(0, 6440.0, 1660.0);

        assert!(!super::polygons_intersect(
            &projectile_polygon,
            &ship_polygon
        ));
    }

    #[test]
    fn androsynth_bubble_with_the_logged_target_does_not_damage_the_enemy_in_the_first_forty_five_ticks()
     {
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
        )
        .expect("battle should build");

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
        )
        .expect("battle should build");

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: true,
            special: false,
        });
        battle.tick(1000.0 / 24.0);

        assert!(battle.snapshot().projectiles[0].id > 0);
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
        )
        .expect("battle should build");

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: false,
            special: true,
        });
        battle.tick(1000.0 / 24.0);

        assert!(
            battle
                .snapshot()
                .audio_events
                .iter()
                .any(|event| event.key == "androsynth-special")
        );
    }

    #[test]
    fn androsynth_bubble_polygon_does_not_overlap_human_cruiser_when_still_in_front() {
        let ship = AnyShip::from(HumanCruiser::new());
        let projectile_collision = AnyShip::from(crate::ships::AndrosynthGuardian::new())
            .primary_projectile_spec()
            .expect("androsynth projectile spec")
            .collision;
        let projectile_polygon =
            super::projectile_hit_polygon(projectile_collision, 0, 5000.0, 4210.0);
        let ship_polygon = ship.hit_polygon(0, 5000.0, 4300.0);

        assert!(!super::polygons_intersect(
            &projectile_polygon,
            &ship_polygon
        ));
    }

    #[test]
    fn human_nuke_polygon_can_overlap_human_cruiser_polygon() {
        let ship = AnyShip::from(HumanCruiser::new());
        let projectile_collision = AnyShip::from(HumanCruiser::new())
            .primary_projectile_spec()
            .expect("human projectile spec")
            .collision;
        let projectile_polygon =
            super::projectile_hit_polygon(projectile_collision, 0, 5000.0, 4300.0);
        let ship_polygon = ship.hit_polygon(0, 5000.0, 4300.0);

        assert!(super::polygons_intersect(
            &projectile_polygon,
            &ship_polygon
        ));
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
        )
        .expect("battle should build");
        battle
            .matter_world
            .set_body_velocity(battle.player.body_id, 2.0, 0.0);
        battle
            .matter_world
            .set_body_velocity(battle.target.body_id, -2.0, 0.0);

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
        )
        .expect("battle should build");

        assert_eq!(
            (
                battle
                    .matter_world
                    .body_uses_polygon_shape(battle.player.body_id),
                battle
                    .matter_world
                    .body_uses_polygon_shape(battle.target.body_id),
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
        )
        .expect("battle should build");
        battle.player.special_active = true;
        battle.ships[battle.player.ship_id].set_thrust_counter(2);
        battle
            .matter_world
            .set_body_velocity(battle.player.body_id, 3.0, 1.5);

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
        )
        .expect("battle should build");

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
        assert!(super::segment_hits_polygon(
            0.0, -120.0, 0.0, 120.0, &polygon, 0.0,
        ));
    }

    #[test]
    fn segment_hits_circle_when_path_crosses_it() {
        assert!(super::segment_hits_circle(
            0.0, 0.0, 10.0, 0.0, 5.0, 3.0, 3.5
        ));
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
        )
        .expect("battle should build");

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
        )
        .expect("battle should build");

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
        )
        .expect("battle should build");

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: false,
            special: true,
        });

        battle.tick(1000.0 / 24.0);

        assert!(
            battle
                .snapshot()
                .audio_events
                .iter()
                .any(|event| event.key == "battle-boom-23")
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
        )
        .expect("battle should build");
        battle.ships[battle.target.ship_id].set_crew(3);

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: false,
            special: true,
        });

        battle.tick(1000.0 / 24.0);

        assert!(battle.snapshot().target.dead);
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
        )
        .expect("battle should build");

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
        )
        .expect("battle should build");

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

        assert_eq!(
            battle.snapshot().player.texture_prefix,
            "androsynth-guardian"
        );
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
        )
        .expect("battle should build");

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
        )
        .expect("battle should build");

        battle.set_player_input(ShipInput {
            left: false,
            right: false,
            thrust: false,
            weapon: false,
            special: true,
        });

        battle.tick(1000.0 / 24.0);

        assert!(battle.snapshot().player.vy < 0.0);
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
        )
        .expect("battle should build");

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

        assert!(
            distance_to_point(&close_battle.snapshot().projectiles[0], 5080.0, 4900.0)
                < distance_to_point(&far_battle.snapshot().projectiles[0], 5080.0, 4900.0)
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

        assert!(
            point_battle.snapshot().projectiles[0].x < default_battle.snapshot().projectiles[0].x
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

        assert!(battle.snapshot().projectiles[0].x > 5000.0);
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
            battle
                .snapshot()
                .projectiles
                .first()
                .map(|projectile| projectile.texture_prefix),
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

        assert!(
            battle
                .snapshot()
                .audio_events
                .iter()
                .any(|event| event.key == "human-victory")
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

        assert!(
            battle
                .snapshot()
                .audio_events
                .iter()
                .any(|event| event.key == "arilou-victory")
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

        assert!(
            battle
                .snapshot()
                .audio_events
                .iter()
                .any(|event| event.key == "human-special")
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
            battle
                .snapshot()
                .audio_events
                .first()
                .map(|event| event.key),
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

        assert!(
            battle
                .snapshot()
                .audio_events
                .iter()
                .any(|event| event.key == "battle-shipdies")
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

        assert!(
            battle
                .snapshot()
                .explosions
                .iter()
                .any(|explosion| explosion.texture_prefix == "battle-boom")
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

        assert!(battle.snapshot().target.dead);
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

        assert!(battle.snapshot().target.facing > -std::f64::consts::FRAC_PI_2);
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
            battle.snapshot().explosions.first().map(|explosion| (
                explosion.texture_prefix,
                (16..=24).contains(&explosion.frame_index),
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

        assert_eq!(
            (battle.snapshot().player.x, battle.snapshot().player.y),
            (100.0, 200.0)
        );
    }
}
