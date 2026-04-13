use crate::define_ship_struct;
use crate::ship::Ship;
use crate::traits::ship_trait::{
    DirectionalThrustSpecialSpec, HitPolygonPoint, PrimaryProjectileSpec, ProjectileBehaviorSpec,
    ProjectileCollisionSpec, ProjectileImpactSpec, ProjectileTargetMode, SpecialAbilitySpec,
};

const THRADDASH_HORN_SPEED: f64 = 30.0;
const THRADDASH_HORN_LIFE: i32 = 15;
const THRADDASH_HORN_OFFSET: f64 = 12.0;
const THRADDASH_HORN_DAMAGE: i32 = 1;
const THRADDASH_AFTERBURNER_SPEED: f64 = 12.0;
const THRADDASH_HORN_POLYGON: [HitPolygonPoint; 8] = [
    HitPolygonPoint { x: 0.0, y: -14.0 },
    HitPolygonPoint { x: 6.0, y: -9.0 },
    HitPolygonPoint { x: 8.0, y: 0.0 },
    HitPolygonPoint { x: 6.0, y: 9.0 },
    HitPolygonPoint { x: 0.0, y: 14.0 },
    HitPolygonPoint { x: -6.0, y: 9.0 },
    HitPolygonPoint { x: -8.0, y: 0.0 },
    HitPolygonPoint { x: -6.0, y: -9.0 },
];




define_ship_struct!(ThraddashTorch);

impl Ship for ThraddashTorch {
    const RACE_NAME: &'static str = "Thraddash";
    const SHIP_CLASS: &'static str = "Torch";
    const SPRITE_PREFIX: &'static str = "thraddash-torch";
    const CAPTAIN_NAMES: &'static [&'static str] = &["Dthunk", "Bardat", "Znonk", "Mnump", "Bronk", "Smup", "Grulk", "Hornk", "Knarg", "Drulg", "Dgako", "Znork", "Kwamp", "Fkank", "Pdump", "Whumps"];
    const COST: i32 = 10;
    const COLOR: u32 = 0xffffff;
    const SIZE: f64 = 18.0;
    const MASS: f64 = 7.0;
    const THRUST_INCREMENT: f64 = 1.4;
    const MAX_SPEED: f64 = 4.7;
    const TURN_RATE: f64 = std::f64::consts::FRAC_PI_8;
    const TURN_WAIT: i32 = 1;
    const THRUST_WAIT: i32 = 0;
    const WEAPON_WAIT: i32 = 12;
    const SPECIAL_WAIT: i32 = 0;
    const MAX_ENERGY: i32 = 24;
    const ENERGY_REGENERATION: i32 = 1;
    const ENERGY_WAIT: i32 = 6;
    const WEAPON_ENERGY_COST: i32 = 2;
    const SPECIAL_ENERGY_COST: i32 = 1;
    const MAX_CREW: i32 = 8;


    fn primary_projectile_spec(&self) -> Option<PrimaryProjectileSpec> {
        Some(PrimaryProjectileSpec {
            speed: THRADDASH_HORN_SPEED,
            acceleration: 0.0,
            max_speed: THRADDASH_HORN_SPEED,
            life: THRADDASH_HORN_LIFE,
            offset: THRADDASH_HORN_OFFSET,
            turn_wait: 0,
            texture_prefix: "thraddash-horn",
            sound_key: "",
            behavior: ProjectileBehaviorSpec::Tracking,
            collision: ProjectileCollisionSpec::Polygon(&THRADDASH_HORN_POLYGON),
            impact: ProjectileImpactSpec {
                damage: THRADDASH_HORN_DAMAGE,
                texture_prefix: "battle-blast",
                start_frame: 0,
                end_frame: 7,
                sound_key: "battle-boom-23",
            },
        })
    }

    fn special_ability_spec(&self) -> SpecialAbilitySpec {
        SpecialAbilitySpec::DirectionalThrust(DirectionalThrustSpecialSpec {
            facing_offset: std::f64::consts::PI,
            speed: THRADDASH_AFTERBURNER_SPEED,
            sound_key: "",
        })
    }

    fn primary_projectile_target_mode(&self) -> ProjectileTargetMode {
        ProjectileTargetMode::EnemyShip
    }
}
