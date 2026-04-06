use crate::ship::Ship;
use crate::traits::ship_trait::{
    PrimaryProjectileSpec, ProjectileBehaviorSpec, ProjectileCollisionSpec,
    ProjectileImpactSpec, ProjectileSpawnSpec, ProjectileTargetMode,
    ProjectileVolleySpec, SecondaryProjectileSpec, SpecialAbilitySpec,
};

const URQUAN_FUSION_SPEED: f64 = 20.0;
const URQUAN_FUSION_LIFE: i32 = 20;
const URQUAN_FUSION_OFFSET: f64 = 32.0;
const URQUAN_FUSION_DAMAGE: i32 = 6;
const URQUAN_FIGHTER_SPEED: f64 = 8.0;
const URQUAN_FIGHTER_LIFE: i32 = 120;
const URQUAN_FIGHTER_OFFSET: f64 = 16.0;
const URQUAN_FIGHTER_SPAWNS: [ProjectileSpawnSpec; 1] = [ProjectileSpawnSpec {
    facing_offset: 0,
    forward_offset: URQUAN_FIGHTER_OFFSET,
    lateral_offset: 0.0,
}];

pub struct UrquanDreadnought {
    crew: i32,
    energy: i32,
    facing: f64,
    turn_counter: i32,
    thrust_counter: i32,
    weapon_counter: i32,
    special_counter: i32,
    energy_counter: i32,
}

impl UrquanDreadnought {
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

impl Ship for UrquanDreadnought {
    const RACE_NAME: &'static str = "Ur-Quan";
    const SHIP_CLASS: &'static str = "Dreadnought";
    const SPRITE_PREFIX: &'static str = "urquan-dreadnought";
    const CAPTAIN_NAMES: &'static [&'static str] = &["Lord 999", "Lord 342", "Lord 88", "Lord 156", "Lord 43", "Lord 412", "Lord 666", "Lord 18", "Lord 237", "Lord 89", "Lord 3", "Lord 476", "Lord 103", "Lord 783", "Lord 52", "Lord 21"];
    const COST: i32 = 30;
    const COLOR: u32 = 0xffffff;
    const SIZE: f64 = 22.0;
    const MASS: f64 = 10.0;
    const THRUST_INCREMENT: f64 = 1.2;
    const MAX_SPEED: f64 = 5.0;
    const TURN_RATE: f64 = std::f64::consts::FRAC_PI_8;
    const TURN_WAIT: i32 = 4;
    const THRUST_WAIT: i32 = 6;
    const WEAPON_WAIT: i32 = 6;
    const SPECIAL_WAIT: i32 = 9;
    const MAX_ENERGY: i32 = 42;
    const ENERGY_REGENERATION: i32 = 1;
    const ENERGY_WAIT: i32 = 4;
    const WEAPON_ENERGY_COST: i32 = 6;
    const SPECIAL_ENERGY_COST: i32 = 8;
    const MAX_CREW: i32 = 42;

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
            speed: URQUAN_FUSION_SPEED,
            acceleration: 0.0,
            max_speed: URQUAN_FUSION_SPEED,
            life: URQUAN_FUSION_LIFE,
            offset: URQUAN_FUSION_OFFSET,
            turn_wait: 0,
            texture_prefix: "urquan-fusion",
            sound_key: "",
            behavior: ProjectileBehaviorSpec::Tracking,
            collision: ProjectileCollisionSpec::None,
            impact: ProjectileImpactSpec {
                damage: URQUAN_FUSION_DAMAGE,
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
                    speed: URQUAN_FIGHTER_SPEED,
                    acceleration: 0.0,
                    max_speed: URQUAN_FIGHTER_SPEED,
                    life: URQUAN_FIGHTER_LIFE,
                    offset: URQUAN_FIGHTER_OFFSET,
                    turn_wait: 2,
                    texture_prefix: "urquan-fighter",
                    sound_key: "",
                    behavior: ProjectileBehaviorSpec::Tracking,
                    collision: ProjectileCollisionSpec::None,
                    impact: ProjectileImpactSpec {
                        damage: 1,
                        texture_prefix: "battle-blast",
                        start_frame: 0,
                        end_frame: 7,
                        sound_key: "battle-boom-23",
                    },
                },
                spawns: &URQUAN_FIGHTER_SPAWNS,
                sound_key: "",
                target_mode: ProjectileTargetMode::EnemyShip,
            },
        })
    }
}
