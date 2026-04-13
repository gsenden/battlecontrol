use crate::define_ship_struct;
use crate::ship::Ship;
use crate::traits::ship_trait::{
    InstantLaserSpec, PrimaryProjectileSpec, ProjectileBehaviorSpec, ProjectileTargetMode,
    ProjectileCollisionSpec, ProjectileImpactSpec, SpecialAbilitySpec,
};

const ZOQ_SPIT_SPEED: f64 = 10.0;
const ZOQ_SPIT_LIFE: i32 = 10;
const ZOQ_SPIT_OFFSET: f64 = 13.0;
const ZOQ_SPIT_DAMAGE: i32 = 1;
const ZOQ_TONGUE_RANGE: f64 = 56.0;
const ZOQ_TONGUE_OFFSET: f64 = 17.0;
const ZOQ_TONGUE_DAMAGE: i32 = 12;




define_ship_struct!(ZoqfotpikStinger);

impl Ship for ZoqfotpikStinger {
    const RACE_NAME: &'static str = "Zoq-Fot-Pik";
    const SHIP_CLASS: &'static str = "Stinger";
    const SPRITE_PREFIX: &'static str = "zoqfotpik-stinger";
    const CAPTAIN_NAMES: &'static [&'static str] = &["NikNak", "FipPat", "DipPak", "FatPot", "ZikFat", "PukYor", "TopNik", "PorKoo", "TikTak", "RinTin", "FitFap", "TotToe", "ZipZak", "TikTok", "MikMok", "SikSok"];
    const COST: i32 = 6;
    const COLOR: u32 = 0xffffff;
    const SIZE: f64 = 15.0;
    const MASS: f64 = 5.0;
    const THRUST_INCREMENT: f64 = 2.0;
    const MAX_SPEED: f64 = 6.7;
    const TURN_RATE: f64 = std::f64::consts::FRAC_PI_8;
    const TURN_WAIT: i32 = 1;
    const THRUST_WAIT: i32 = 0;
    const WEAPON_WAIT: i32 = 0;
    const SPECIAL_WAIT: i32 = 6;
    const MAX_ENERGY: i32 = 10;
    const ENERGY_REGENERATION: i32 = 1;
    const ENERGY_WAIT: i32 = 4;
    const WEAPON_ENERGY_COST: i32 = 1;
    const SPECIAL_ENERGY_COST: i32 = 7;
    const MAX_CREW: i32 = 10;


    fn primary_projectile_spec(&self) -> Option<PrimaryProjectileSpec> {
        Some(PrimaryProjectileSpec {
            speed: ZOQ_SPIT_SPEED,
            acceleration: 0.0,
            max_speed: ZOQ_SPIT_SPEED,
            life: ZOQ_SPIT_LIFE,
            offset: ZOQ_SPIT_OFFSET,
            turn_wait: 0,
            texture_prefix: "zoqfotpik-spit",
            sound_key: "",
            behavior: ProjectileBehaviorSpec::Tracking,
            collision: ProjectileCollisionSpec::None,
            impact: ProjectileImpactSpec {
                damage: ZOQ_SPIT_DAMAGE,
                texture_prefix: "battle-blast",
                start_frame: 0,
                end_frame: 7,
                sound_key: "battle-boom-23",
            },
        })
    }

    fn special_ability_spec(&self) -> SpecialAbilitySpec {
        SpecialAbilitySpec::InstantLaser(InstantLaserSpec {
            range: ZOQ_TONGUE_RANGE,
            damage: ZOQ_TONGUE_DAMAGE,
            offset: ZOQ_TONGUE_OFFSET,
            sound_key: "",
            impact_sound_key: "battle-boom-45",
            color: 0xffffff,
            width: 3.0,
            target_mode: ProjectileTargetMode::EnemyShip,
        })
    }
}
