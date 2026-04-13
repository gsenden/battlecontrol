use crate::define_ship_struct;
use crate::ship::Ship;
use crate::traits::ship_trait::{
    InstantLaserSpec, PrimaryProjectileSpec, ProjectileBehaviorSpec, ProjectileTargetMode,
    ProjectileCollisionSpec, ProjectileImpactSpec, SpecialAbilitySpec,
};

const KOHRAH_BUZZSAW_SPEED: f64 = 32.0;
const KOHRAH_BUZZSAW_LIFE: i32 = 64;
const KOHRAH_BUZZSAW_OFFSET: f64 = 28.0;
const KOHRAH_BUZZSAW_DAMAGE: i32 = 4;
const KOHRAH_GAS_RANGE: f64 = 96.0;




define_ship_struct!(KohrahMarauder);

impl Ship for KohrahMarauder {
    const RACE_NAME: &'static str = "Kohr-Ah";
    const SHIP_CLASS: &'static str = "Marauder";
    const SPRITE_PREFIX: &'static str = "kohrah-marauder";
    const CAPTAIN_NAMES: &'static [&'static str] = &["Death 11", "Death 17", "Death 37", "Death 23", "Death 7", "Death 13", "Death 19", "Death 29", "Death 31", "Death 41", "Death 43", "Death 3", "Death 5", "Death 47", "Death 53", "Death 83"];
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
    const SPECIAL_ENERGY_COST: i32 = 21;
    const MAX_CREW: i32 = 42;


    fn primary_projectile_spec(&self) -> Option<PrimaryProjectileSpec> {
        Some(PrimaryProjectileSpec {
            speed: KOHRAH_BUZZSAW_SPEED,
            acceleration: 0.0,
            max_speed: KOHRAH_BUZZSAW_SPEED,
            life: KOHRAH_BUZZSAW_LIFE,
            offset: KOHRAH_BUZZSAW_OFFSET,
            turn_wait: 4,
            texture_prefix: "kohrah-buzzsaw",
            sound_key: "",
            behavior: ProjectileBehaviorSpec::Tracking,
            collision: ProjectileCollisionSpec::None,
            impact: ProjectileImpactSpec {
                damage: KOHRAH_BUZZSAW_DAMAGE,
                texture_prefix: "battle-blast",
                start_frame: 0,
                end_frame: 7,
                sound_key: "battle-boom-45",
            },
        })
    }

    fn special_ability_spec(&self) -> SpecialAbilitySpec {
        SpecialAbilitySpec::InstantLaser(InstantLaserSpec {
            range: KOHRAH_GAS_RANGE,
            damage: 3,
            offset: 2.0,
            sound_key: "",
            impact_sound_key: "battle-boom-23",
            color: 0xffffff,
            width: 3.0,
            target_mode: ProjectileTargetMode::EnemyShip,
        })
    }
}
