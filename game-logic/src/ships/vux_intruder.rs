use crate::define_ship_struct;
use crate::ship::Ship;
use crate::traits::ship_trait::{
    InstantLaserSpec, PrimaryProjectileSpec, ProjectileBehaviorSpec,
    ProjectileCollisionSpec, ProjectileImpactSpec, ProjectileSpawnSpec,
    ProjectileTargetMode, ProjectileVolleySpec, SecondaryProjectileSpec,
    SpecialAbilitySpec,
};

const VUX_LASER_RANGE: f64 = 162.0;
const VUX_LASER_OFFSET: f64 = 38.0;
const VUX_LIMPET_SPEED: f64 = 25.0;
const VUX_LIMPET_LIFE: i32 = 80;
const VUX_LIMPET_OFFSET: f64 = 12.0;
const VUX_LIMPET_SPAWNS: [ProjectileSpawnSpec; 1] = [ProjectileSpawnSpec {
    facing_offset: 0,
    forward_offset: VUX_LIMPET_OFFSET,
    lateral_offset: 0.0,
}];




define_ship_struct!(VuxIntruder);

impl Ship for VuxIntruder {
    const RACE_NAME: &'static str = "Vux";
    const SHIP_CLASS: &'static str = "Intruder";
    const SPRITE_PREFIX: &'static str = "vux-intruder";
    const CAPTAIN_NAMES: &'static [&'static str] = &["ZIK", "PUZ", "ZUK", "VIP", "ZIT", "YUK", "DAK", "ZRN", "PIF", "FIZ", "FUP", "ZUP", "NRF", "ZOG", "ORZ", "ZEK"];
    const COST: i32 = 12;
    const COLOR: u32 = 0xffffff;
    const SIZE: f64 = 16.0;
    const MASS: f64 = 6.0;
    const THRUST_INCREMENT: f64 = 1.4;
    const MAX_SPEED: f64 = 3.5;
    const TURN_RATE: f64 = std::f64::consts::FRAC_PI_8;
    const TURN_WAIT: i32 = 6;
    const THRUST_WAIT: i32 = 4;
    const WEAPON_WAIT: i32 = 0;
    const SPECIAL_WAIT: i32 = 7;
    const MAX_ENERGY: i32 = 40;
    const ENERGY_REGENERATION: i32 = 1;
    const ENERGY_WAIT: i32 = 8;
    const WEAPON_ENERGY_COST: i32 = 1;
    const SPECIAL_ENERGY_COST: i32 = 2;
    const MAX_CREW: i32 = 20;


    fn primary_instant_laser_spec(&self) -> Option<InstantLaserSpec> {
        Some(InstantLaserSpec {
            range: VUX_LASER_RANGE,
            damage: 1,
            offset: VUX_LASER_OFFSET,
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
                    speed: VUX_LIMPET_SPEED,
                    acceleration: 0.0,
                    max_speed: VUX_LIMPET_SPEED,
                    life: VUX_LIMPET_LIFE,
                    offset: VUX_LIMPET_OFFSET,
                    turn_wait: 2,
                    texture_prefix: "vux-limpets",
                    sound_key: "",
                    behavior: ProjectileBehaviorSpec::Tracking,
                    collision: ProjectileCollisionSpec::None,
                    impact: ProjectileImpactSpec {
                        damage: 0,
                        texture_prefix: "battle-blast",
                        start_frame: 0,
                        end_frame: 7,
                        sound_key: "",
                    },
                },
                spawns: &VUX_LIMPET_SPAWNS,
                sound_key: "",
                target_mode: ProjectileTargetMode::EnemyShip,
            },
        })
    }
}
