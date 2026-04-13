use crate::define_ship_struct;
use crate::ship::Ship;
use crate::traits::ship_trait::{
    HitPolygonPoint, PrimaryProjectileSpec, ProjectileBehaviorSpec, ProjectileCollisionSpec,
    ProjectileImpactSpec, ProjectileSpawnSpec, ProjectileTargetMode, ProjectileVolleySpec,
    SecondaryProjectileSpec, SpecialAbilitySpec,
};

const SPATHI_MISSILE_SPEED: f64 = 30.0;
const SPATHI_MISSILE_LIFE: i32 = 10;
const SPATHI_MISSILE_OFFSET: f64 = 59.0;
const SPATHI_MISSILE_DAMAGE: i32 = 1;
const SPATHI_DISCRIMINATOR_SPEED: f64 = 8.0;
const SPATHI_DISCRIMINATOR_LIFE: i32 = 30;
const SPATHI_DISCRIMINATOR_OFFSET: f64 = 71.0;
const SPATHI_DISCRIMINATOR_DAMAGE: i32 = 2;
const SPATHI_DISCRIMINATOR_TRACK_WAIT: i32 = 1;
const SPATHI_PROJECTILE_POLYGON: [HitPolygonPoint; 8] = [
    HitPolygonPoint { x: 0.0, y: -16.0 },
    HitPolygonPoint { x: 8.0, y: -10.0 },
    HitPolygonPoint { x: 10.0, y: 0.0 },
    HitPolygonPoint { x: 8.0, y: 10.0 },
    HitPolygonPoint { x: 0.0, y: 16.0 },
    HitPolygonPoint { x: -8.0, y: 10.0 },
    HitPolygonPoint { x: -10.0, y: 0.0 },
    HitPolygonPoint { x: -8.0, y: -10.0 },
];
const SPATHI_SPECIAL_SPAWNS: [ProjectileSpawnSpec; 1] = [ProjectileSpawnSpec {
    facing_offset: 8,
    forward_offset: SPATHI_DISCRIMINATOR_OFFSET,
    lateral_offset: 0.0,
}];




define_ship_struct!(SpathiEluder);

impl Ship for SpathiEluder {
    const RACE_NAME: &'static str = "Spathi";
    const SHIP_CLASS: &'static str = "Eluder";
    const SPRITE_PREFIX: &'static str = "spathi-eluder";
    const CAPTAIN_NAMES: &'static [&'static str] = &["Thwil", "Pwappy", "Phwiff", "Wiffy", "Plibnik", "Snurfel", "Kwimp", "Pkunky", "Jinkeze", "Thintho", "Rupatup", "Nargle", "Phlendo", "Snelopy", "Bwinkin", "Whuff"];
    const COST: i32 = 18;
    const COLOR: u32 = 0xffffff;
    const SIZE: f64 = 15.0;
    const MASS: f64 = 5.0;
    const THRUST_INCREMENT: f64 = 2.4;
    const MAX_SPEED: f64 = 8.0;
    const TURN_RATE: f64 = std::f64::consts::FRAC_PI_8;
    const TURN_WAIT: i32 = 1;
    const THRUST_WAIT: i32 = 1;
    const WEAPON_WAIT: i32 = 0;
    const SPECIAL_WAIT: i32 = 7;
    const MAX_ENERGY: i32 = 10;
    const ENERGY_REGENERATION: i32 = 1;
    const ENERGY_WAIT: i32 = 10;
    const WEAPON_ENERGY_COST: i32 = 2;
    const SPECIAL_ENERGY_COST: i32 = 3;
    const MAX_CREW: i32 = 30;


    fn primary_projectile_spec(&self) -> Option<PrimaryProjectileSpec> {
        Some(PrimaryProjectileSpec {
            speed: SPATHI_MISSILE_SPEED,
            acceleration: 0.0,
            max_speed: SPATHI_MISSILE_SPEED,
            life: SPATHI_MISSILE_LIFE,
            offset: SPATHI_MISSILE_OFFSET,
            turn_wait: 0,
            texture_prefix: "spathi-missile",
            sound_key: "",
            behavior: ProjectileBehaviorSpec::Tracking,
            collision: ProjectileCollisionSpec::Polygon(&SPATHI_PROJECTILE_POLYGON),
            impact: ProjectileImpactSpec {
                damage: SPATHI_MISSILE_DAMAGE,
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
                    speed: SPATHI_DISCRIMINATOR_SPEED,
                    acceleration: 0.0,
                    max_speed: SPATHI_DISCRIMINATOR_SPEED,
                    life: SPATHI_DISCRIMINATOR_LIFE,
                    offset: SPATHI_DISCRIMINATOR_OFFSET,
                    turn_wait: SPATHI_DISCRIMINATOR_TRACK_WAIT,
                    texture_prefix: "spathi-missile",
                    sound_key: "",
                    behavior: ProjectileBehaviorSpec::Tracking,
                    collision: ProjectileCollisionSpec::Polygon(&SPATHI_PROJECTILE_POLYGON),
                    impact: ProjectileImpactSpec {
                        damage: SPATHI_DISCRIMINATOR_DAMAGE,
                        texture_prefix: "battle-blast",
                        start_frame: 0,
                        end_frame: 7,
                        sound_key: "battle-boom-23",
                    },
                },
                spawns: &SPATHI_SPECIAL_SPAWNS,
                sound_key: "",
                target_mode: ProjectileTargetMode::EnemyShip,
            },
        })
    }

    fn primary_projectile_target_mode(&self) -> ProjectileTargetMode {
        ProjectileTargetMode::EnemyShip
    }
}
