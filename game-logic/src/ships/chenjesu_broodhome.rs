use crate::define_ship_struct;
use crate::ship::Ship;
use crate::traits::ship_trait::{
    PrimaryProjectileSpec, ProjectileBehaviorSpec, ProjectileCollisionSpec,
    ProjectileImpactSpec, ProjectileSpawnSpec, ProjectileTargetMode,
    ProjectileVolleySpec, SecondaryProjectileSpec, SpecialAbilitySpec,
};

const CHENJESU_CRYSTAL_SPEED: f64 = 16.0;
const CHENJESU_CRYSTAL_LIFE: i32 = 90;
const CHENJESU_CRYSTAL_OFFSET: f64 = 32.0;
const CHENJESU_CRYSTAL_DAMAGE: i32 = 6;
const CHENJESU_DOGGY_SPEED: f64 = 8.0;
const CHENJESU_DOGGY_LIFE: i32 = 80;
const CHENJESU_DOGGY_OFFSET: f64 = 36.0;
const CHENJESU_DOGGY_DAMAGE: i32 = 3;
const CHENJESU_DOGGY_SPAWNS: [ProjectileSpawnSpec; 1] = [ProjectileSpawnSpec {
    facing_offset: 0,
    forward_offset: CHENJESU_DOGGY_OFFSET,
    lateral_offset: 0.0,
}];




define_ship_struct!(ChenjesuBroodhome);

impl Ship for ChenjesuBroodhome {
    const RACE_NAME: &'static str = "Chenjesu";
    const SHIP_CLASS: &'static str = "Broodhome";
    const SPRITE_PREFIX: &'static str = "chenjesu-broodhome";
    const CAPTAIN_NAMES: &'static [&'static str] = &["Kzzakk", "Tzrrow", "Zzmzmm", "Vziziz", "Hmmhmm", "Bzrak", "Krrtzz", "Zzzzz", "Zxzakz", "Brrzap", "Tzaprak", "Pzkrakz", "Fzzzz", "Vrroww", "Zznaz", "Zzzhmm"];
    const COST: i32 = 28;
    const COLOR: u32 = 0xffffff;
    const SIZE: f64 = 22.0;
    const MASS: f64 = 10.0;
    const THRUST_INCREMENT: f64 = 0.6;
    const MAX_SPEED: f64 = 4.5;
    const TURN_RATE: f64 = std::f64::consts::FRAC_PI_8;
    const TURN_WAIT: i32 = 6;
    const THRUST_WAIT: i32 = 4;
    const WEAPON_WAIT: i32 = 0;
    const SPECIAL_WAIT: i32 = 0;
    const MAX_ENERGY: i32 = 30;
    const ENERGY_REGENERATION: i32 = 1;
    const ENERGY_WAIT: i32 = 4;
    const WEAPON_ENERGY_COST: i32 = 5;
    const SPECIAL_ENERGY_COST: i32 = 30;
    const MAX_CREW: i32 = 36;


    fn primary_projectile_spec(&self) -> Option<PrimaryProjectileSpec> {
        Some(PrimaryProjectileSpec {
            speed: CHENJESU_CRYSTAL_SPEED,
            acceleration: 0.0,
            max_speed: CHENJESU_CRYSTAL_SPEED,
            life: CHENJESU_CRYSTAL_LIFE,
            offset: CHENJESU_CRYSTAL_OFFSET,
            turn_wait: 0,
            texture_prefix: "chenjesu-spark",
            sound_key: "",
            behavior: ProjectileBehaviorSpec::Tracking,
            collision: ProjectileCollisionSpec::None,
            impact: ProjectileImpactSpec {
                damage: CHENJESU_CRYSTAL_DAMAGE,
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
                    speed: CHENJESU_DOGGY_SPEED,
                    acceleration: 0.0,
                    max_speed: CHENJESU_DOGGY_SPEED,
                    life: CHENJESU_DOGGY_LIFE,
                    offset: CHENJESU_DOGGY_OFFSET,
                    turn_wait: 2,
                    texture_prefix: "chenjesu-doggy",
                    sound_key: "",
                    behavior: ProjectileBehaviorSpec::Tracking,
                    collision: ProjectileCollisionSpec::None,
                    impact: ProjectileImpactSpec {
                        damage: CHENJESU_DOGGY_DAMAGE,
                        texture_prefix: "battle-blast",
                        start_frame: 0,
                        end_frame: 7,
                        sound_key: "battle-boom-23",
                    },
                },
                spawns: &CHENJESU_DOGGY_SPAWNS,
                sound_key: "",
                target_mode: ProjectileTargetMode::EnemyShip,
            },
        })
    }
}
