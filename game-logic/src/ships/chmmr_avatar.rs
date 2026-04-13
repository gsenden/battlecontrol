use crate::define_ship_struct;
use crate::ship::Ship;
use crate::traits::ship_trait::{
    InstantLaserSpec, PrimaryProjectileSpec, ProjectileBehaviorSpec,
    ProjectileCollisionSpec, ProjectileImpactSpec, ProjectileSpawnSpec,
    ProjectileTargetMode, ProjectileVolleySpec, SecondaryProjectileSpec,
    SpecialAbilitySpec,
};

const CHMMR_LASER_RANGE: f64 = 480.0;
const CHMMR_LASER_OFFSET: f64 = 48.0;
const CHMMR_SATELLITE_SPEED: f64 = 10.0;
const CHMMR_SATELLITE_LIFE: i32 = 120;
const CHMMR_SATELLITE_OFFSET: f64 = 28.0;
const CHMMR_SATELLITE_SPAWNS: [ProjectileSpawnSpec; 1] = [ProjectileSpawnSpec {
    facing_offset: 0,
    forward_offset: CHMMR_SATELLITE_OFFSET,
    lateral_offset: 0.0,
}];




define_ship_struct!(ChmmrAvatar);

impl Ship for ChmmrAvatar {
    const RACE_NAME: &'static str = "Chmmr";
    const SHIP_CLASS: &'static str = "Avatar";
    const SPRITE_PREFIX: &'static str = "chmmr-avatar";
    const CAPTAIN_NAMES: &'static [&'static str] = &["Mnzgk", "Chzrmn", "Bzztrm", "Zrnzrk", "Tzzqrn", "Kzzrn", "Vzrzn", "Qrntz", "Rmnzk", "Szmrnz", "Zbzzn", "Frnkzk", "Prmtzz", "Tzrtzn", "Kztztz", "Mrnkzt"];
    const COST: i32 = 30;
    const COLOR: u32 = 0xa8fff5;
    const SIZE: f64 = 22.0;
    const MASS: f64 = 10.0;
    const THRUST_INCREMENT: f64 = 1.4;
    const MAX_SPEED: f64 = 5.8;
    const TURN_RATE: f64 = std::f64::consts::FRAC_PI_8;
    const TURN_WAIT: i32 = 3;
    const THRUST_WAIT: i32 = 5;
    const WEAPON_WAIT: i32 = 0;
    const SPECIAL_WAIT: i32 = 0;
    const MAX_ENERGY: i32 = 42;
    const ENERGY_REGENERATION: i32 = 1;
    const ENERGY_WAIT: i32 = 1;
    const WEAPON_ENERGY_COST: i32 = 2;
    const SPECIAL_ENERGY_COST: i32 = 1;
    const MAX_CREW: i32 = 42;


    fn primary_instant_laser_spec(&self) -> Option<InstantLaserSpec> {
        Some(InstantLaserSpec {
            range: CHMMR_LASER_RANGE,
            damage: 2,
            offset: CHMMR_LASER_OFFSET,
            sound_key: "",
            impact_sound_key: "battle-boom-23",
            color: 0xffffff,
            width: 3.0,
            target_mode: ProjectileTargetMode::EnemyShip,
        })
    }

    fn special_ability_spec(&self) -> SpecialAbilitySpec {
        SpecialAbilitySpec::Projectile(SecondaryProjectileSpec {
            volley: ProjectileVolleySpec {
                projectile: PrimaryProjectileSpec {
                    speed: CHMMR_SATELLITE_SPEED,
                    acceleration: 0.0,
                    max_speed: CHMMR_SATELLITE_SPEED,
                    life: CHMMR_SATELLITE_LIFE,
                    offset: CHMMR_SATELLITE_OFFSET,
                    turn_wait: 2,
                    texture_prefix: "chmmr-satellite",
                    sound_key: "",
                    behavior: ProjectileBehaviorSpec::Tracking,
                    collision: ProjectileCollisionSpec::None,
                    impact: ProjectileImpactSpec {
                        damage: 2,
                        texture_prefix: "battle-blast",
                        start_frame: 0,
                        end_frame: 7,
                        sound_key: "battle-boom-23",
                    },
                },
                spawns: &CHMMR_SATELLITE_SPAWNS,
                sound_key: "",
                target_mode: ProjectileTargetMode::EnemyShip,
            },
        })
    }
}
