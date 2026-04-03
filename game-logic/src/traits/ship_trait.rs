use crate::physics_command::PhysicsCommand;
use crate::ship_input::ShipInput;
use crate::velocity_vector::VelocityVector;

const COLLISION_TURN_WAIT: i32 = 1;
const COLLISION_THRUST_WAIT: i32 = 3;
const GRAVITY_WELL_SPEED_MULTIPLIER: f64 = 1.75;
const TRAVEL_ALIGNMENT_EPSILON: f64 = 0.0001;
const GRAVITY_THRESHOLD: f64 = 420.0;
const GRAVITY_PULL: f64 = 0.12;

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
        let (ti, th_wait, max_spd) =
            (self.thrust_increment(), self.thrust_wait(), self.max_speed());
        if self.thrust_counter() > 0 {
            self.decrease_thrust_counter(1);
        } else if input.thrust {
            let facing = self.facing();
            let dvx = facing.cos() * ti;
            let dvy = facing.sin() * ti;
            let (vx, vy) = get_thrust_velocity(
                facing, ti, max_spd, velocity, dvx, dvy, current_speed, allow_beyond_max_speed,
            );
            commands.push(PhysicsCommand::SetVelocity { vx, vy });
            self.set_thrust_counter(th_wait);
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
