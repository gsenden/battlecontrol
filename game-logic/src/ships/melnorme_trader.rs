use crate::ship::Ship;
use crate::traits::ship_trait::{
    PrimaryProjectileSpec, ProjectileBehaviorSpec, ProjectileCollisionSpec,
    ProjectileImpactSpec, ProjectileSpawnSpec, ProjectileTargetMode,
    ProjectileVolleySpec, SecondaryProjectileSpec, SpecialAbilitySpec,
};

const MELNORME_PUMPUP_SPEED: f64 = 45.0;
const MELNORME_PUMPUP_LIFE: i32 = 10;
const MELNORME_PUMPUP_OFFSET: f64 = 24.0;
const MELNORME_PUMPUP_DAMAGE: i32 = 4;
const MELNORME_CONFUSE_SPEED: f64 = 30.0;
const MELNORME_CONFUSE_LIFE: i32 = 20;
const MELNORME_CONFUSE_OFFSET: f64 = 8.0;
const MELNORME_CONFUSE_SPAWNS: [ProjectileSpawnSpec; 1] = [ProjectileSpawnSpec {
    facing_offset: 0,
    forward_offset: MELNORME_CONFUSE_OFFSET,
    lateral_offset: 0.0,
}];

pub struct MelnormeTrader {
    crew: i32,
    energy: i32,
    facing: f64,
    turn_counter: i32,
    thrust_counter: i32,
    weapon_counter: i32,
    special_counter: i32,
    energy_counter: i32,
}

impl MelnormeTrader {
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

impl Ship for MelnormeTrader {
    const RACE_NAME: &'static str = "Melnorme";
    const SHIP_CLASS: &'static str = "Trader";
    const SPRITE_PREFIX: &'static str = "melnorme-trader";
    const CAPTAIN_NAMES: &'static [&'static str] = &["Reddish", "Orangy", "Aqua", "Crimson", "Magenta", "Cheruse", "Beige", "Fuchsia", "Umber", "Cerise", "Mauve", "Grayish", "Yellow", "Black", "Bluish", "Purple"];
    const COST: i32 = 18;
    const COLOR: u32 = 0xffffff;
    const SIZE: f64 = 18.0;
    const MASS: f64 = 7.0;
    const THRUST_INCREMENT: f64 = 1.2;
    const MAX_SPEED: f64 = 6.0;
    const TURN_RATE: f64 = std::f64::consts::FRAC_PI_8;
    const TURN_WAIT: i32 = 4;
    const THRUST_WAIT: i32 = 4;
    const WEAPON_WAIT: i32 = 1;
    const SPECIAL_WAIT: i32 = 20;
    const MAX_ENERGY: i32 = 42;
    const ENERGY_REGENERATION: i32 = 1;
    const ENERGY_WAIT: i32 = 4;
    const WEAPON_ENERGY_COST: i32 = 5;
    const SPECIAL_ENERGY_COST: i32 = 20;
    const MAX_CREW: i32 = 20;

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
            speed: MELNORME_PUMPUP_SPEED,
            acceleration: 0.0,
            max_speed: MELNORME_PUMPUP_SPEED,
            life: MELNORME_PUMPUP_LIFE,
            offset: MELNORME_PUMPUP_OFFSET,
            turn_wait: 0,
            texture_prefix: "melnorme-pumpup",
            sound_key: "",
            behavior: ProjectileBehaviorSpec::Tracking,
            collision: ProjectileCollisionSpec::None,
            impact: ProjectileImpactSpec {
                damage: MELNORME_PUMPUP_DAMAGE,
                texture_prefix: "battle-blast",
                start_frame: 0,
                end_frame: 7,
                sound_key: "battle-boom-45",
            },
        })
    }

    fn special_ability_spec(&self) -> SpecialAbilitySpec {
        SpecialAbilitySpec::Projectile(SecondaryProjectileSpec {
            volley: ProjectileVolleySpec {
                projectile: PrimaryProjectileSpec {
                    speed: MELNORME_CONFUSE_SPEED,
                    acceleration: 0.0,
                    max_speed: MELNORME_CONFUSE_SPEED,
                    life: MELNORME_CONFUSE_LIFE,
                    offset: MELNORME_CONFUSE_OFFSET,
                    turn_wait: 0,
                    texture_prefix: "melnorme-confuse",
                    sound_key: "",
                    behavior: ProjectileBehaviorSpec::Tracking,
                    collision: ProjectileCollisionSpec::None,
                    impact: ProjectileImpactSpec {
                        damage: 0,
                        texture_prefix: "battle-blast",
                        start_frame: 0,
                        end_frame: 7,
                        sound_key: "",
                    },
                },
                spawns: &MELNORME_CONFUSE_SPAWNS,
                sound_key: "",
                target_mode: ProjectileTargetMode::EnemyShip,
            },
        })
    }
}
