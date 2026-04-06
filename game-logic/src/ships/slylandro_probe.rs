use crate::ship::Ship;
use crate::traits::ship_trait::{InstantLaserSpec, PlanetHarvestSpec, ProjectileTargetMode, SpecialAbilitySpec};

const SLYLANDRO_HARVEST_RANGE: f64 = 420.0;
const SLYLANDRO_HARVEST_ENERGY: i32 = 6;
const SLYLANDRO_LIGHTNING_RANGE: f64 = 72.0;

pub struct SlylandroProbe {
    crew: i32,
    energy: i32,
    facing: f64,
    turn_counter: i32,
    thrust_counter: i32,
    weapon_counter: i32,
    special_counter: i32,
    energy_counter: i32,
}

impl SlylandroProbe {
    pub fn new() -> Self {
        Self {
            crew: Self::MAX_CREW,
            energy: Self::MAX_ENERGY,
            facing: -std::f64::consts::FRAC_PI_2,
            turn_counter: 0,
            thrust_counter: 0,
            weapon_counter: 0,
            special_counter: 0,
            energy_counter: 0,
        }
    }
}

impl Ship for SlylandroProbe {
    const RACE_NAME: &'static str = "Slylandro";
    const SHIP_CLASS: &'static str = "Probe";
    const SPRITE_PREFIX: &'static str = "slylandro-probe";
    const CAPTAIN_NAMES: &'static [&'static str] = &["2418-B", "2418-B", "2418-B", "2418-B", "2418-B", "2418-B", "2418-B", "2418-B", "2418-B", "2418-B", "2418-B", "2418-B", "2418-B", "2418-B", "2418-B", "2418-B"];
    const COST: i32 = 17;
    const COLOR: u32 = 0xffffff;
    const SIZE: f64 = 12.0;
    const MASS: f64 = 1.0;
    const THRUST_INCREMENT: f64 = 12.0;
    const MAX_SPEED: f64 = 10.0;
    const TURN_RATE: f64 = std::f64::consts::FRAC_PI_8;
    const TURN_WAIT: i32 = 0;
    const THRUST_WAIT: i32 = 0;
    const WEAPON_WAIT: i32 = 17;
    const SPECIAL_WAIT: i32 = 20;
    const MAX_ENERGY: i32 = 20;
    const ENERGY_REGENERATION: i32 = 0;
    const ENERGY_WAIT: i32 = 10;
    const WEAPON_ENERGY_COST: i32 = 2;
    const SPECIAL_ENERGY_COST: i32 = 0;
    const MAX_CREW: i32 = 12;

    fn crew(&self) -> i32 { self.crew }
    fn set_crew(&mut self, value: i32) { self.crew = value }
    fn energy(&self) -> i32 { self.energy }
    fn set_energy(&mut self, value: i32) { self.energy = value }
    fn facing(&self) -> f64 { self.facing }
    fn set_facing(&mut self, value: f64) { self.facing = value }
    fn turn_counter(&self) -> i32 { self.turn_counter }
    fn set_turn_counter(&mut self, value: i32) { self.turn_counter = value }
    fn thrust_counter(&self) -> i32 { self.thrust_counter }
    fn set_thrust_counter(&mut self, value: i32) { self.thrust_counter = value }
    fn weapon_counter(&self) -> i32 { self.weapon_counter }
    fn set_weapon_counter(&mut self, value: i32) { self.weapon_counter = value }
    fn special_counter(&self) -> i32 { self.special_counter }
    fn set_special_counter(&mut self, value: i32) { self.special_counter = value }
    fn energy_counter(&self) -> i32 { self.energy_counter }
    fn set_energy_counter(&mut self, value: i32) { self.energy_counter = value }

    fn primary_instant_laser_spec(&self) -> Option<InstantLaserSpec> {
        Some(InstantLaserSpec {
            range: SLYLANDRO_LIGHTNING_RANGE,
            damage: 2,
            offset: 0.0,
            sound_key: "",
            impact_sound_key: "battle-boom-23",
            color: 0xffffff,
            width: 3.0,
            target_mode: ProjectileTargetMode::EnemyShip,
        })
    }

    fn special_ability_spec(&self) -> SpecialAbilitySpec {
        SpecialAbilitySpec::PlanetHarvest(PlanetHarvestSpec {
            range: SLYLANDRO_HARVEST_RANGE,
            energy_gain: SLYLANDRO_HARVEST_ENERGY,
            sound_key: "",
        })
    }
}
