use crate::ship::Ship;
use crate::traits::ship_trait::{
    CrewDrainTransferSpec,
    PrimaryProjectileSpec, ProjectileBehaviorSpec, ProjectileCollisionSpec,
    ProjectileImpactSpec, SpecialAbilitySpec,
};

const SYREEN_SONG_RANGE: f64 = 208.0;
const SYREEN_MAX_TRANSFER: i32 = 8;
const SYREEN_DAGGER_SPEED: f64 = 24.0;
const SYREEN_DAGGER_LIFE: i32 = 16;
const SYREEN_DAGGER_OFFSET: f64 = 28.0;
const SYREEN_DAGGER_DAMAGE: i32 = 2;

pub struct SyreenPenetrator {
    crew: i32,
    energy: i32,
    facing: f64,
    turn_counter: i32,
    thrust_counter: i32,
    weapon_counter: i32,
    special_counter: i32,
    energy_counter: i32,
}

impl SyreenPenetrator {
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

impl Ship for SyreenPenetrator {
    const RACE_NAME: &'static str = "Syreen";
    const SHIP_CLASS: &'static str = "Penetrator";
    const SPRITE_PREFIX: &'static str = "syreen-penetrator";
    const CAPTAIN_NAMES: &'static [&'static str] = &["Teela", "Dejah", "Penny", "Alia", "Be'lit", "Ripley", "Yarr", "Ardala", "Sparta", "Munro", "Danning", "Brawne", "Maya", "Aelita", "Alura", "Dale"];
    const COST: i32 = 13;
    const COLOR: u32 = 0xffffff;
    const SIZE: f64 = 12.0;
    const MASS: f64 = 2.0;
    const THRUST_INCREMENT: f64 = 1.8;
    const MAX_SPEED: f64 = 6.0;
    const TURN_RATE: f64 = std::f64::consts::FRAC_PI_8;
    const TURN_WAIT: i32 = 1;
    const THRUST_WAIT: i32 = 1;
    const WEAPON_WAIT: i32 = 8;
    const SPECIAL_WAIT: i32 = 20;
    const MAX_ENERGY: i32 = 16;
    const ENERGY_REGENERATION: i32 = 1;
    const ENERGY_WAIT: i32 = 6;
    const WEAPON_ENERGY_COST: i32 = 1;
    const SPECIAL_ENERGY_COST: i32 = 5;
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

    fn primary_projectile_spec(&self) -> Option<PrimaryProjectileSpec> {
        Some(PrimaryProjectileSpec {
            speed: SYREEN_DAGGER_SPEED,
            acceleration: 0.0,
            max_speed: SYREEN_DAGGER_SPEED,
            life: SYREEN_DAGGER_LIFE,
            offset: SYREEN_DAGGER_OFFSET,
            turn_wait: 0,
            texture_prefix: "syreen-dagger",
            sound_key: "",
            behavior: ProjectileBehaviorSpec::Tracking,
            collision: ProjectileCollisionSpec::None,
            impact: ProjectileImpactSpec {
                damage: SYREEN_DAGGER_DAMAGE,
                texture_prefix: "battle-blast",
                start_frame: 0,
                end_frame: 7,
                sound_key: "battle-boom-23",
            },
        })
    }

    fn special_ability_spec(&self) -> SpecialAbilitySpec {
        SpecialAbilitySpec::CrewDrainTransfer(CrewDrainTransferSpec {
            range: SYREEN_SONG_RANGE,
            max_transfer: SYREEN_MAX_TRANSFER,
            sound_key: "",
        })
    }
}
