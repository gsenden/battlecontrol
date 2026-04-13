use crate::define_ship_struct;
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




define_ship_struct!(UrquanDreadnought);

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
