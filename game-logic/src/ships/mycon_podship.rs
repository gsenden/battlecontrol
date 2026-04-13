use crate::define_ship_struct;
use crate::ship::Ship;
use crate::traits::ship_trait::{
    CrewRegenerationSpec, HitPolygonPoint, PrimaryProjectileSpec, ProjectileBehaviorSpec,
    ProjectileCollisionSpec, ProjectileImpactSpec, ProjectileTargetMode, SpecialAbilitySpec,
};

const MYCON_PLASMA_SPEED: f64 = 8.0;
const MYCON_PLASMA_LIFE: i32 = 143;
const MYCON_PLASMA_OFFSET: f64 = 24.0;
const MYCON_PLASMA_DAMAGE: i32 = 10;
const MYCON_PLASMA_TRACK_WAIT: i32 = 1;
const MYCON_REGEN_AMOUNT: i32 = 4;
const MYCON_PLASMA_POLYGON: [HitPolygonPoint; 8] = [
    HitPolygonPoint { x: 0.0, y: -22.0 },
    HitPolygonPoint { x: 12.0, y: -16.0 },
    HitPolygonPoint { x: 16.0, y: 0.0 },
    HitPolygonPoint { x: 12.0, y: 16.0 },
    HitPolygonPoint { x: 0.0, y: 22.0 },
    HitPolygonPoint { x: -12.0, y: 16.0 },
    HitPolygonPoint { x: -16.0, y: 0.0 },
    HitPolygonPoint { x: -12.0, y: -16.0 },
];




define_ship_struct!(MyconPodship);

impl Ship for MyconPodship {
    const RACE_NAME: &'static str = "Mycon";
    const SHIP_CLASS: &'static str = "Podship";
    const SPRITE_PREFIX: &'static str = "mycon-podship";
    const CAPTAIN_NAMES: &'static [&'static str] = &["Blort", "Chupp", "Floos", "Glish", "Glob", "Glush", "Plork", "Shlish", "Shlupp", "Slingy", "Sploozo", "Spork", "Uffo", "Yush", "Zaloop", "Znuff"];
    const COST: i32 = 21;
    const COLOR: u32 = 0xffffff;
    const SIZE: f64 = 18.0;
    const MASS: f64 = 7.0;
    const THRUST_INCREMENT: f64 = 1.8;
    const MAX_SPEED: f64 = 4.5;
    const TURN_RATE: f64 = std::f64::consts::FRAC_PI_8;
    const TURN_WAIT: i32 = 6;
    const THRUST_WAIT: i32 = 6;
    const WEAPON_WAIT: i32 = 5;
    const SPECIAL_WAIT: i32 = 0;
    const MAX_ENERGY: i32 = 40;
    const ENERGY_REGENERATION: i32 = 1;
    const ENERGY_WAIT: i32 = 4;
    const WEAPON_ENERGY_COST: i32 = 20;
    const SPECIAL_ENERGY_COST: i32 = 40;
    const MAX_CREW: i32 = 20;


    fn primary_projectile_spec(&self) -> Option<PrimaryProjectileSpec> {
        Some(PrimaryProjectileSpec {
            speed: MYCON_PLASMA_SPEED,
            acceleration: 0.0,
            max_speed: MYCON_PLASMA_SPEED,
            life: MYCON_PLASMA_LIFE,
            offset: MYCON_PLASMA_OFFSET,
            turn_wait: MYCON_PLASMA_TRACK_WAIT,
            texture_prefix: "mycon-plasma",
            sound_key: "",
            behavior: ProjectileBehaviorSpec::Tracking,
            collision: ProjectileCollisionSpec::Polygon(&MYCON_PLASMA_POLYGON),
            impact: ProjectileImpactSpec {
                damage: MYCON_PLASMA_DAMAGE,
                texture_prefix: "battle-blast",
                start_frame: 0,
                end_frame: 7,
                sound_key: "battle-boom-45",
            },
        })
    }

    fn special_ability_spec(&self) -> SpecialAbilitySpec {
        SpecialAbilitySpec::CrewRegeneration(CrewRegenerationSpec {
            amount: MYCON_REGEN_AMOUNT,
            sound_key: "",
        })
    }

    fn primary_projectile_target_mode(&self) -> ProjectileTargetMode {
        ProjectileTargetMode::EnemyShip
    }
}
