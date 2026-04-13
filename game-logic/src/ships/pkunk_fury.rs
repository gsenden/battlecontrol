use crate::define_ship_struct;
use crate::ship::Ship;
use crate::traits::ship_trait::{
    PrimaryProjectileSpec, ProjectileBehaviorSpec, ProjectileCollisionSpec,
    ProjectileImpactSpec, ProjectileSpawnSpec, ProjectileTargetMode,
    ProjectileVolleySpec, SoundOnlySpec, SpecialAbilitySpec,
};

const PKUNK_BUG_SPEED: f64 = 24.0;
const PKUNK_BUG_LIFE: i32 = 5;
const PKUNK_BUG_OFFSET: f64 = 15.0;
const PKUNK_BUG_DAMAGE: i32 = 1;
const PKUNK_BUG_SPAWNS: [ProjectileSpawnSpec; 3] = [
    ProjectileSpawnSpec { facing_offset: -4, forward_offset: PKUNK_BUG_OFFSET, lateral_offset: 0.0 },
    ProjectileSpawnSpec { facing_offset: 0, forward_offset: PKUNK_BUG_OFFSET, lateral_offset: 0.0 },
    ProjectileSpawnSpec { facing_offset: 4, forward_offset: PKUNK_BUG_OFFSET, lateral_offset: 0.0 },
];




define_ship_struct!(PkunkFury);

impl Ship for PkunkFury {
    const RACE_NAME: &'static str = "Pkunk";
    const SHIP_CLASS: &'static str = "Fury";
    const SPRITE_PREFIX: &'static str = "pkunk-fury";
    const CAPTAIN_NAMES: &'static [&'static str] = &["Awwky", "Tweety", "WudStok", "Poppy", "Brakky", "Hooter", "Buzzard", "Polly", "Ernie", "Yompin", "Fuzzy", "Raven", "Crow", "Jay", "Screech", "Twitter"];
    const COST: i32 = 20;
    const COLOR: u32 = 0xffffff;
    const SIZE: f64 = 12.0;
    const MASS: f64 = 1.0;
    const THRUST_INCREMENT: f64 = 3.2;
    const MAX_SPEED: f64 = 10.7;
    const TURN_RATE: f64 = std::f64::consts::FRAC_PI_8;
    const TURN_WAIT: i32 = 0;
    const THRUST_WAIT: i32 = 0;
    const WEAPON_WAIT: i32 = 0;
    const SPECIAL_WAIT: i32 = 16;
    const MAX_ENERGY: i32 = 12;
    const ENERGY_REGENERATION: i32 = 0;
    const ENERGY_WAIT: i32 = 0;
    const WEAPON_ENERGY_COST: i32 = 1;
    const SPECIAL_ENERGY_COST: i32 = 2;
    const MAX_CREW: i32 = 8;


    fn primary_volley_spec(&self) -> Option<ProjectileVolleySpec> {
        Some(ProjectileVolleySpec {
            projectile: PrimaryProjectileSpec {
                speed: PKUNK_BUG_SPEED,
                acceleration: 0.0,
                max_speed: PKUNK_BUG_SPEED,
                life: PKUNK_BUG_LIFE,
                offset: PKUNK_BUG_OFFSET,
                turn_wait: 0,
                texture_prefix: "pkunk-bug",
                sound_key: "",
                behavior: ProjectileBehaviorSpec::Tracking,
                collision: ProjectileCollisionSpec::None,
                impact: ProjectileImpactSpec {
                    damage: PKUNK_BUG_DAMAGE,
                    texture_prefix: "battle-blast",
                    start_frame: 0,
                    end_frame: 7,
                    sound_key: "battle-boom-23",
                },
            },
            spawns: &PKUNK_BUG_SPAWNS,
            sound_key: "",
            target_mode: ProjectileTargetMode::None,
        })
    }

    fn special_ability_spec(&self) -> SpecialAbilitySpec {
        SpecialAbilitySpec::SoundOnly(SoundOnlySpec {
            sound_key: "",
        })
    }
}
