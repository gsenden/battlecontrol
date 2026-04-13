use crate::define_ship_struct;
use crate::ship::Ship;
use crate::traits::ship_trait::{
    PrimaryProjectileSpec, ProjectileBehaviorSpec, ProjectileCollisionSpec,
    ProjectileImpactSpec, ProjectileSpawnSpec, ProjectileTargetMode, ProjectileVolleySpec,
    SecondaryProjectileSpec, SpecialAbilitySpec,
};

const ORZ_HOWITZER_SPEED: f64 = 24.0;
const ORZ_HOWITZER_LIFE: i32 = 18;
const ORZ_HOWITZER_OFFSET: f64 = 20.0;
const ORZ_HOWITZER_DAMAGE: i32 = 3;
const ORZ_MARINE_SPEED: f64 = 10.0;
const ORZ_MARINE_LIFE: i32 = 90;
const ORZ_MARINE_OFFSET: f64 = 14.0;
const ORZ_MARINE_SPAWNS: [ProjectileSpawnSpec; 1] = [ProjectileSpawnSpec {
    facing_offset: 0,
    forward_offset: ORZ_MARINE_OFFSET,
    lateral_offset: 0.0,
}];




define_ship_struct!(OrzNemesis);

impl Ship for OrzNemesis {
    const RACE_NAME: &'static str = "Orz";
    const SHIP_CLASS: &'static str = "Nemesis";
    const SPRITE_PREFIX: &'static str = "orz-nemesis";
    const CAPTAIN_NAMES: &'static [&'static str] = &["*Wet*", "*Happy*", "*Frumple*", "*Camper*", "*Loner*", "*Dancer*", "*Singer*", "*Heavy*", "*NewBoy*", "*FatFun*", "*Pepper*", "*Hungry*", "*Deep*", "*Smell*", "*Juice*", "*Squirt*"];
    const COST: i32 = 23;
    const COLOR: u32 = 0xffffff;
    const SIZE: f64 = 13.0;
    const MASS: f64 = 4.0;
    const THRUST_INCREMENT: f64 = 1.0;
    const MAX_SPEED: f64 = 5.8;
    const TURN_RATE: f64 = std::f64::consts::FRAC_PI_8;
    const TURN_WAIT: i32 = 1;
    const THRUST_WAIT: i32 = 0;
    const WEAPON_WAIT: i32 = 4;
    const SPECIAL_WAIT: i32 = 12;
    const MAX_ENERGY: i32 = 20;
    const ENERGY_REGENERATION: i32 = 1;
    const ENERGY_WAIT: i32 = 6;
    const WEAPON_ENERGY_COST: i32 = 6;
    const SPECIAL_ENERGY_COST: i32 = 0;
    const MAX_CREW: i32 = 16;


    fn primary_projectile_spec(&self) -> Option<PrimaryProjectileSpec> {
        Some(PrimaryProjectileSpec {
            speed: ORZ_HOWITZER_SPEED,
            acceleration: 0.0,
            max_speed: ORZ_HOWITZER_SPEED,
            life: ORZ_HOWITZER_LIFE,
            offset: ORZ_HOWITZER_OFFSET,
            turn_wait: 0,
            texture_prefix: "orz-howitzer",
            sound_key: "",
            behavior: ProjectileBehaviorSpec::Tracking,
            collision: ProjectileCollisionSpec::None,
            impact: ProjectileImpactSpec {
                damage: ORZ_HOWITZER_DAMAGE,
                texture_prefix: "battle-blast",
                start_frame: 0,
                end_frame: 7,
                sound_key: "battle-boom-23",
            },
        })
    }

    fn special_ability_spec(&self) -> SpecialAbilitySpec {
        SpecialAbilitySpec::Projectile(SecondaryProjectileSpec {
            volley: ProjectileVolleySpec {
                projectile: PrimaryProjectileSpec {
                    speed: ORZ_MARINE_SPEED,
                    acceleration: 0.0,
                    max_speed: ORZ_MARINE_SPEED,
                    life: ORZ_MARINE_LIFE,
                    offset: ORZ_MARINE_OFFSET,
                    turn_wait: 3,
                    texture_prefix: "orz-turret",
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
                spawns: &ORZ_MARINE_SPAWNS,
                sound_key: "",
                target_mode: ProjectileTargetMode::EnemyShip,
            },
        })
    }
}
