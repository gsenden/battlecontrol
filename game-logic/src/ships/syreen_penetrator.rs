use crate::define_ship_struct;
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




define_ship_struct!(SyreenPenetrator);

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
