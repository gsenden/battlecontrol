use crate::physics_command::PhysicsCommand;
use crate::ship_input::ShipInput;
use crate::velocity_vector::VelocityVector;

const COLLISION_TURN_WAIT: i32 = 1;
const COLLISION_THRUST_WAIT: i32 = 3;
const GRAVITY_WELL_SPEED_MULTIPLIER: f64 = 1.75;
const TRAVEL_ALIGNMENT_EPSILON: f64 = 0.0001;
const GRAVITY_THRESHOLD: f64 = 420.0;
const GRAVITY_PULL: f64 = 0.12;

#[derive(Clone, Copy, PartialEq)]
pub struct HitPolygonPoint {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Copy)]
pub enum ProjectileCollisionSpec {
    None,
    Polygon(&'static [HitPolygonPoint]),
}

#[derive(Clone, Copy)]
pub struct ProjectileImpactSpec {
    pub damage: i32,
    pub texture_prefix: &'static str,
    pub start_frame: i32,
    pub end_frame: i32,
    pub sound_key: &'static str,
}

#[derive(Clone, Copy)]
pub struct PrimaryProjectileSpec {
    pub speed: f64,
    pub acceleration: f64,
    pub max_speed: f64,
    pub life: i32,
    pub offset: f64,
    pub turn_wait: i32,
    pub texture_prefix: &'static str,
    pub sound_key: &'static str,
    pub behavior: ProjectileBehaviorSpec,
    pub collision: ProjectileCollisionSpec,
    pub impact: ProjectileImpactSpec,
}

#[derive(Clone, Copy)]
pub struct ProjectileSpawnSpec {
    pub facing_offset: i32,
    pub forward_offset: f64,
    pub lateral_offset: f64,
}

#[derive(Clone, Copy)]
pub struct ProjectileVolleySpec {
    pub projectile: PrimaryProjectileSpec,
    pub spawns: &'static [ProjectileSpawnSpec],
    pub sound_key: &'static str,
    pub target_mode: ProjectileTargetMode,
}

#[derive(Clone, Copy)]
pub struct InstantLaserSpec {
    pub range: f64,
    pub damage: i32,
    pub offset: f64,
    pub sound_key: &'static str,
    pub impact_sound_key: &'static str,
    pub color: u32,
    pub width: f64,
    pub target_mode: ProjectileTargetMode,
}

#[derive(Clone, Copy)]
pub enum ProjectileBehaviorSpec {
    Tracking,
    WobbleTracking {
        direct_track_range: f64,
        spawn_rewind_divisor: f64,
    },
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ProjectileTargetMode {
    None,
    EnemyShip,
    PlayerSelectedOrEnemyShip,
    PlayerSelectedPointOrForward,
}

#[derive(Clone, Copy)]
pub struct PointDefenseSpec {
    pub range: f64,
    pub sound_key: &'static str,
}

#[derive(Clone, Copy)]
pub struct ShieldSpecialSpec {
    pub active_texture_prefix: &'static str,
    pub sound_key: &'static str,
}

#[derive(Clone, Copy)]
pub struct BlazerSpecialSpec {
    pub active_texture_prefix: &'static str,
    pub speed: f64,
    pub mass: f64,
    pub damage: i32,
    pub hit_radius: f64,
    pub activation_sound_key: &'static str,
    pub impact_sound_key: &'static str,
}

#[derive(Clone, Copy)]
pub struct TeleportSpecialSpec {
    pub effect_texture_prefix: &'static str,
    pub end_frame: i32,
    pub sound_key: &'static str,
}

#[derive(Clone, Copy)]
pub struct DirectionalThrustSpecialSpec {
    pub facing_offset: f64,
    pub speed: f64,
    pub sound_key: &'static str,
}

#[derive(Clone, Copy)]
pub struct SecondaryProjectileSpec {
    pub volley: ProjectileVolleySpec,
}

#[derive(Clone, Copy)]
pub struct CrewRegenerationSpec {
    pub amount: i32,
    pub sound_key: &'static str,
}

#[derive(Clone, Copy)]
pub struct CrewToEnergySpec {
    pub crew_cost: i32,
    pub energy_gain: i32,
    pub recoil_speed: f64,
    pub sound_key: &'static str,
}

#[derive(Clone, Copy)]
pub struct SelfDestructSpec {
    pub damage: i32,
    pub radius: f64,
    pub texture_prefix: &'static str,
    pub end_frame: i32,
    pub sound_key: &'static str,
}

#[derive(Clone, Copy)]
pub struct SoundOnlySpec {
    pub sound_key: &'static str,
}

#[derive(Clone, Copy)]
pub struct CloakSpec {
    pub sound_key: &'static str,
}

#[derive(Clone, Copy)]
pub struct TransformSpec {
    pub active_texture_prefix: &'static str,
    pub sound_key: &'static str,
}

#[derive(Clone, Copy)]
pub struct CrewDrainTransferSpec {
    pub range: f64,
    pub max_transfer: i32,
    pub sound_key: &'static str,
}

#[derive(Clone, Copy)]
pub struct PlanetHarvestSpec {
    pub range: f64,
    pub energy_gain: i32,
    pub sound_key: &'static str,
}

#[derive(Clone, Copy)]
pub enum SpecialAbilitySpec {
    None,
    PointDefense(PointDefenseSpec),
    Blazer(BlazerSpecialSpec),
    Shield(ShieldSpecialSpec),
    Teleport(TeleportSpecialSpec),
    InstantLaser(InstantLaserSpec),
    DirectionalThrust(DirectionalThrustSpecialSpec),
    Projectile(SecondaryProjectileSpec),
    CrewRegeneration(CrewRegenerationSpec),
    CrewToEnergy(CrewToEnergySpec),
    SelfDestruct(SelfDestructSpec),
    SoundOnly(SoundOnlySpec),
    Cloak(CloakSpec),
    Transform(TransformSpec),
    CrewDrainTransfer(CrewDrainTransferSpec),
    PlanetHarvest(PlanetHarvestSpec),
}

pub trait Ship {
    const RACE_NAME: &'static str;
    const SHIP_CLASS: &'static str;
    const SPRITE_PREFIX: &'static str;
    const CAPTAIN_NAMES: &'static [&'static str];
    const COST: i32;
    const COLOR: u32;
    const SIZE: f64;
    const MASS: f64;
    const THRUST_INCREMENT: f64;
    const MAX_SPEED: f64;
    const TURN_RATE: f64;
    const TURN_WAIT: i32;
    const THRUST_WAIT: i32;
    const WEAPON_WAIT: i32;
    const SPECIAL_WAIT: i32;
    const MAX_ENERGY: i32;
    const ENERGY_REGENERATION: i32;
    const ENERGY_WAIT: i32;
    const WEAPON_ENERGY_COST: i32;
    const SPECIAL_ENERGY_COST: i32;
    const MAX_CREW: i32;

    fn crew(&self) -> i32;
    fn set_crew(&mut self, value: i32);
    fn energy(&self) -> i32;
    fn set_energy(&mut self, value: i32);
    fn facing(&self) -> f64;
    fn set_facing(&mut self, value: f64);
    fn turn_counter(&self) -> i32;
    fn set_turn_counter(&mut self, value: i32);
    fn thrust_counter(&self) -> i32;
    fn set_thrust_counter(&mut self, value: i32);
    fn weapon_counter(&self) -> i32;
    fn set_weapon_counter(&mut self, value: i32);
    fn special_counter(&self) -> i32;
    fn set_special_counter(&mut self, value: i32);
    fn energy_counter(&self) -> i32;
    fn set_energy_counter(&mut self, value: i32);
    fn hit_polygon(&self, _facing: i32, _center_x: f64, _center_y: f64) -> Vec<HitPolygonPoint> {
        Vec::new()
    }
    fn hit_polygon_for_state(
        &self,
        facing: i32,
        center_x: f64,
        center_y: f64,
        _special_active: bool,
    ) -> Vec<HitPolygonPoint> {
        self.hit_polygon(facing, center_x, center_y)
    }

    fn primary_projectile_spec(&self) -> Option<PrimaryProjectileSpec> {
        None
    }

    fn primary_projectile_spec_for_state(&self, _special_active: bool) -> Option<PrimaryProjectileSpec> {
        self.primary_projectile_spec()
    }

    fn primary_volley_spec(&self) -> Option<ProjectileVolleySpec> {
        None
    }

    fn primary_volley_spec_for_state(&self, _special_active: bool) -> Option<ProjectileVolleySpec> {
        self.primary_volley_spec()
    }

    fn primary_instant_laser_spec(&self) -> Option<InstantLaserSpec> {
        None
    }

    fn primary_instant_laser_spec_for_state(&self, _special_active: bool) -> Option<InstantLaserSpec> {
        self.primary_instant_laser_spec()
    }

    fn victory_sound_key(&self) -> Option<&'static str> {
        None
    }

    fn active_texture_prefix(&self, _special_active: bool) -> &'static str {
        self.sprite_prefix()
    }

    fn special_ability_spec(&self) -> SpecialAbilitySpec {
        SpecialAbilitySpec::None
    }

    fn primary_projectile_target_mode(&self) -> ProjectileTargetMode {
        ProjectileTargetMode::None
    }

    fn thrust_velocity(
        &self,
        velocity: &VelocityVector,
        allow_beyond_max_speed: bool,
        current_speed: f64,
    ) -> Option<(f64, f64)> {
        let facing = self.facing();
        let thrust_increment = self.thrust_increment();
        let max_speed = self.max_speed();
        let delta_x = facing.cos() * thrust_increment;
        let delta_y = facing.sin() * thrust_increment;

        Some(get_thrust_velocity(
            facing,
            thrust_increment,
            max_speed,
            velocity,
            delta_x,
            delta_y,
            current_speed,
            allow_beyond_max_speed,
        ))
    }

    fn idle_velocity(&self, _velocity: &VelocityVector) -> Option<(f64, f64)> {
        None
    }

    fn primary_projectile_target_mode_for_state(&self, _special_active: bool) -> ProjectileTargetMode {
        self.primary_projectile_target_mode()
    }

    fn special_state_persists_after_cooldown(&self) -> bool {
        false
    }

    fn is_targetable(&self, _special_active: bool) -> bool {
        true
    }

    fn is_cloaked(&self, _special_active: bool) -> bool {
        false
    }

    fn increase_crew(&mut self, amount: i32) {
        self.set_crew(self.crew() + amount);
    }

    fn decrease_crew(&mut self, amount: i32) {
        self.set_crew(self.crew() - amount);
    }

    fn increase_energy(&mut self, amount: i32) {
        self.set_energy(self.energy() + amount);
    }

    fn decrease_energy(&mut self, amount: i32) {
        self.set_energy(self.energy() - amount);
    }

    fn increase_facing(&mut self, amount: f64) {
        self.set_facing(self.facing() + amount);
    }

    fn decrease_facing(&mut self, amount: f64) {
        self.set_facing(self.facing() - amount);
    }

    fn increase_turn_counter(&mut self, amount: i32) {
        self.set_turn_counter(self.turn_counter() + amount);
    }

    fn decrease_turn_counter(&mut self, amount: i32) {
        self.set_turn_counter(self.turn_counter() - amount);
    }

    fn increase_thrust_counter(&mut self, amount: i32) {
        self.set_thrust_counter(self.thrust_counter() + amount);
    }

    fn decrease_thrust_counter(&mut self, amount: i32) {
        self.set_thrust_counter(self.thrust_counter() - amount);
    }

    fn increase_weapon_counter(&mut self, amount: i32) {
        self.set_weapon_counter(self.weapon_counter() + amount);
    }

    fn decrease_weapon_counter(&mut self, amount: i32) {
        self.set_weapon_counter(self.weapon_counter() - amount);
    }

    fn increase_special_counter(&mut self, amount: i32) {
        self.set_special_counter(self.special_counter() + amount);
    }

    fn decrease_special_counter(&mut self, amount: i32) {
        self.set_special_counter(self.special_counter() - amount);
    }

    fn increase_energy_counter(&mut self, amount: i32) {
        self.set_energy_counter(self.energy_counter() + amount);
    }

    fn decrease_energy_counter(&mut self, amount: i32) {
        self.set_energy_counter(self.energy_counter() - amount);
    }

    fn race_name(&self) -> &'static str { Self::RACE_NAME }
    fn ship_class(&self) -> &'static str { Self::SHIP_CLASS }
    fn sprite_prefix(&self) -> &'static str { Self::SPRITE_PREFIX }
    fn captain_names(&self) -> &'static [&'static str] { Self::CAPTAIN_NAMES }
    fn cost(&self) -> i32 { Self::COST }
    fn color(&self) -> u32 { Self::COLOR }
    fn size(&self) -> f64 { Self::SIZE }
    fn mass(&self) -> f64 { Self::MASS }
    fn thrust_increment(&self) -> f64 { Self::THRUST_INCREMENT }
    fn max_speed(&self) -> f64 { Self::MAX_SPEED }
    fn turn_rate(&self) -> f64 { Self::TURN_RATE }
    fn turn_wait(&self) -> i32 { Self::TURN_WAIT }
    fn thrust_wait(&self) -> i32 { Self::THRUST_WAIT }
    fn weapon_wait(&self) -> i32 { Self::WEAPON_WAIT }
    fn special_wait(&self) -> i32 { Self::SPECIAL_WAIT }
    fn max_energy(&self) -> i32 { Self::MAX_ENERGY }
    fn energy_regeneration(&self) -> i32 { Self::ENERGY_REGENERATION }
    fn energy_wait(&self) -> i32 { Self::ENERGY_WAIT }
    fn weapon_energy_cost(&self) -> i32 { Self::WEAPON_ENERGY_COST }
    fn special_energy_cost(&self) -> i32 { Self::SPECIAL_ENERGY_COST }
    fn max_crew(&self) -> i32 { Self::MAX_CREW }

    fn update(
        &mut self,
        input: &ShipInput,
        velocity: &VelocityVector,
        allow_beyond_max_speed: bool,
    ) -> Vec<PhysicsCommand> {
        let mut commands = Vec::new();
        let current_speed = (velocity.x.powi(2) + velocity.y.powi(2)).sqrt();

        // Energy regeneration
        let (max_energy, regen, e_wait) =
            (self.max_energy(), self.energy_regeneration(), self.energy_wait());
        if self.energy_counter() > 0 {
            self.decrease_energy_counter(1);
        } else if self.energy() < max_energy {
            self.set_energy((self.energy() + regen).min(max_energy));
            self.set_energy_counter(e_wait);
        }

        // Turning
        let (rate, t_wait) = (self.turn_rate(), self.turn_wait());
        if self.turn_counter() > 0 {
            self.decrease_turn_counter(1);
        } else if input.left || input.right {
            if input.left {
                self.decrease_facing(rate);
            } else {
                self.increase_facing(rate);
            }
            self.set_turn_counter(t_wait);
        }

        // Thrust
        let (th_wait, max_spd) = (self.thrust_wait(), self.max_speed());
        if self.thrust_counter() > 0 {
            self.decrease_thrust_counter(1);
        } else if input.thrust {
            let (vx, vy) = self
                .thrust_velocity(velocity, allow_beyond_max_speed, current_speed)
                .unwrap_or((self.facing().cos() * max_spd, self.facing().sin() * max_spd));
            commands.push(PhysicsCommand::SetVelocity { vx, vy });
            self.set_thrust_counter(th_wait);
        } else if let Some((vx, vy)) = self.idle_velocity(velocity) {
            commands.push(PhysicsCommand::SetVelocity { vx, vy });
        }

        // Weapon
        let (w_wait, w_cost) = (self.weapon_wait(), self.weapon_energy_cost());
        if self.weapon_counter() > 0 {
            self.decrease_weapon_counter(1);
        } else if input.weapon && self.energy() >= w_cost {
            self.decrease_energy(w_cost);
            self.set_weapon_counter(w_wait);
        }

        // Special
        let (sp_wait, sp_cost) = (self.special_wait(), self.special_energy_cost());
        if self.special_counter() > 0 {
            self.decrease_special_counter(1);
        } else if input.special && self.energy() >= sp_cost {
            self.decrease_energy(sp_cost);
            self.set_special_counter(sp_wait);
        }

        commands
    }

    fn take_damage(&mut self, amount: i32) -> bool {
        self.set_crew((self.crew() - amount).max(0));
        self.crew() <= 0
    }

    fn apply_collision_cooldowns(&mut self) {
        if self.turn_counter() < COLLISION_TURN_WAIT {
            self.increase_turn_counter(COLLISION_TURN_WAIT);
        }
        if self.thrust_counter() < COLLISION_THRUST_WAIT {
            self.increase_thrust_counter(COLLISION_THRUST_WAIT);
        }
    }

    fn gravity_command(&self, dx: f64, dy: f64) -> Option<PhysicsCommand> {
        let abs_dx = dx.abs();
        let abs_dy = dy.abs();

        if abs_dx > GRAVITY_THRESHOLD || abs_dy > GRAVITY_THRESHOLD {
            return None;
        }

        let dist_squared = (abs_dx * abs_dx) + (abs_dy * abs_dy);
        if dist_squared == 0.0 || dist_squared > (GRAVITY_THRESHOLD * GRAVITY_THRESHOLD) {
            return None;
        }

        let dist = dist_squared.sqrt();
        Some(PhysicsCommand::AddVelocity {
            vx: (dx / dist) * GRAVITY_PULL,
            vy: (dy / dist) * GRAVITY_PULL,
        })
    }
}

fn get_thrust_velocity(
    facing: f64,
    thrust_increment: f64,
    max_speed: f64,
    current_velocity: &VelocityVector,
    dvx: f64,
    dvy: f64,
    current_speed: f64,
    allow_beyond_max_speed: bool,
) -> (f64, f64) {
    let gravity_max = max_speed * GRAVITY_WELL_SPEED_MULTIPLIER;
    let travel_aligned =
        current_speed <= TRAVEL_ALIGNMENT_EPSILON || is_travel_aligned(facing, current_velocity);

    if !allow_beyond_max_speed && travel_aligned && current_speed > max_speed {
        return (current_velocity.x, current_velocity.y);
    }

    let desired_x = current_velocity.x + dvx;
    let desired_y = current_velocity.y + dvy;
    let desired_speed = (desired_x.powi(2) + desired_y.powi(2)).sqrt();

    if desired_speed <= max_speed {
        return (desired_x, desired_y);
    }

    if !travel_aligned && current_speed >= max_speed {
        let travel_angle = current_velocity.y.atan2(current_velocity.x);
        let rotated_x =
            current_velocity.x + (dvx * 0.5) - (travel_angle.cos() * thrust_increment);
        let rotated_y =
            current_velocity.y + (dvy * 0.5) - (travel_angle.sin() * thrust_increment);
        let rotated_speed = (rotated_x.powi(2) + rotated_y.powi(2)).sqrt();

        if rotated_speed <= gravity_max || rotated_speed < current_speed {
            return (rotated_x, rotated_y);
        }
    }

    if (allow_beyond_max_speed && desired_speed <= gravity_max) || desired_speed < current_speed {
        return (desired_x, desired_y);
    }

    if travel_aligned {
        let limited_speed = if allow_beyond_max_speed {
            gravity_max
        } else {
            max_speed
        }
        .min(desired_speed);
        return (facing.cos() * limited_speed, facing.sin() * limited_speed);
    }

    (current_velocity.x, current_velocity.y)
}

fn is_travel_aligned(facing: f64, current_velocity: &VelocityVector) -> bool {
    let travel_angle = current_velocity.y.atan2(current_velocity.x);
    let facing_delta = (facing - travel_angle).sin().atan2((facing - travel_angle).cos());
    facing_delta.abs() <= TRAVEL_ALIGNMENT_EPSILON
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reference_data;
    use crate::ships::{AnyShip, ArilouSkiff, ChmmrAvatar, HumanCruiser};

    fn no_input() -> ShipInput {
        ShipInput { left: false, right: false, thrust: false, weapon: false, special: false }
    }

    fn zero_velocity() -> VelocityVector {
        VelocityVector { x: 0.0, y: 0.0 }
    }

    #[test]
    fn arilou_thrust_replaces_existing_diagonal_drift_with_facing_velocity() {
        let mut ship = ArilouSkiff::new();
        let facing = -std::f64::consts::FRAC_PI_4;
        ship.set_facing(facing);

        let commands = ship.update(
            &ShipInput {
                left: false,
                right: false,
                thrust: true,
                weapon: false,
                special: false,
            },
            &VelocityVector { x: -3.0, y: 2.0 },
            false,
        );

        assert!(matches!(
            commands.as_slice(),
            [PhysicsCommand::SetVelocity { vx, vy }]
                if (*vx - facing.cos() * ship.max_speed()).abs() < 1e-9
                    && (*vy - facing.sin() * ship.max_speed()).abs() < 1e-9
        ));
    }

    #[test]
    fn energy_regenerates_after_weapon_fire() {
        let ref_data = reference_data::load();
        let scenario = &ref_data.energy;

        let mut ship = HumanCruiser::new();

        for (i, frame) in scenario.frames.iter().enumerate() {
            let input = if i == 0 || i == 15 {
                ShipInput { weapon: true, ..no_input() }
            } else {
                no_input()
            };
            ship.update(&input, &zero_velocity(), false);
            assert_eq!(ship.energy(), frame.energy, "energy mismatch at frame {i}");
            assert_eq!(ship.energy_counter(), frame.energy_counter, "energy_counter mismatch at frame {i}");
        }
    }

    #[test]
    fn weapon_sets_cooldown_matching_reference() {
        let ref_data = reference_data::load();
        let frame0 = &ref_data.energy.frames[0];

        let mut ship = HumanCruiser::new();
        let input = ShipInput { weapon: true, ..no_input() };
        ship.update(&input, &zero_velocity(), false);

        assert_eq!(ship.weapon_counter(), frame0.weapon_counter);
    }

    #[test]
    fn weapon_drains_energy_matching_reference() {
        let ref_data = reference_data::load();
        let frame0 = &ref_data.energy.frames[0];

        let mut ship = HumanCruiser::new();
        let input = ShipInput { weapon: true, ..no_input() };
        ship.update(&input, &zero_velocity(), false);

        assert_eq!(ship.energy(), frame0.energy);
    }

    #[test]
    fn collision_applies_cooldowns() {
        let ref_data = reference_data::load();
        let scenario = &ref_data.collision_cooldowns;

        let mut ship = HumanCruiser::new();
        ship.apply_collision_cooldowns();

        assert_eq!(
            (ship.turn_counter(), ship.thrust_counter()),
            (scenario.turn_wait, scenario.thrust_wait)
        );
    }

    #[test]
    fn collision_keeps_higher_existing_cooldowns() {
        let ref_data = reference_data::load();
        let scenario = &ref_data.collision_existing_cooldowns;

        let mut ship = HumanCruiser::new();
        ship.set_turn_counter(2);
        ship.set_thrust_counter(4);
        ship.apply_collision_cooldowns();

        assert_eq!(
            (ship.turn_counter(), ship.thrust_counter()),
            (scenario.turn_wait, scenario.thrust_wait)
        );
    }

    #[test]
    fn gravity_command_pulls_toward_planet() {
        let ship = HumanCruiser::new();
        let command = ship.gravity_command(-400.0, 0.0);

        assert!(matches!(
            command,
            Some(PhysicsCommand::AddVelocity { vx, vy }) if (vx + 0.12).abs() < f64::EPSILON && vy.abs() < f64::EPSILON
        ));
    }

    #[test]
    fn has_human_cruiser_max_crew() {
        let ship = HumanCruiser::new();
        assert_eq!(ship.max_crew(), 18);
    }

    #[test]
    fn any_ship_vec_holds_mixed_types() {
        let ships: Vec<AnyShip> = vec![
            HumanCruiser::new().into(),
            ChmmrAvatar::new().into(),
            ArilouSkiff::new().into(),
        ];

        assert_eq!(ships[0].max_crew(), 18);
        assert_eq!(ships[1].max_crew(), 42);
        assert_eq!(ships[2].max_crew(), 6);
    }
}
