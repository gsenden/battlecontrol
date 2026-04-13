use crate::define_ship_struct;
use crate::ship::Ship;
use crate::traits::ship_trait::{
    HitPolygonPoint, PrimaryProjectileSpec, ProjectileBehaviorSpec, ProjectileCollisionSpec,
    ProjectileImpactSpec, ProjectileSpawnSpec, ProjectileTargetMode, ProjectileVolleySpec,
    ShieldSpecialSpec, SpecialAbilitySpec,
};

const YEHAT_MISSILE_SPEED: f64 = 20.0;
const YEHAT_MISSILE_LIFE: i32 = 10;
const YEHAT_MISSILE_DAMAGE: i32 = 1;
const YEHAT_LAUNCH_FORWARD_OFFSET: f64 = 16.0;
const YEHAT_LAUNCH_LATERAL_OFFSET: f64 = 8.0;
const YEHAT_PROJECTILE_POLYGON: [HitPolygonPoint; 8] = [
    HitPolygonPoint { x: 0.0, y: -14.0 },
    HitPolygonPoint { x: 7.0, y: -9.0 },
    HitPolygonPoint { x: 9.0, y: 0.0 },
    HitPolygonPoint { x: 7.0, y: 9.0 },
    HitPolygonPoint { x: 0.0, y: 14.0 },
    HitPolygonPoint { x: -7.0, y: 9.0 },
    HitPolygonPoint { x: -9.0, y: 0.0 },
    HitPolygonPoint { x: -7.0, y: -9.0 },
];
const YEHAT_PRIMARY_SPAWNS: [ProjectileSpawnSpec; 2] = [
    ProjectileSpawnSpec { facing_offset: 0, forward_offset: YEHAT_LAUNCH_FORWARD_OFFSET, lateral_offset: YEHAT_LAUNCH_LATERAL_OFFSET },
    ProjectileSpawnSpec { facing_offset: 0, forward_offset: YEHAT_LAUNCH_FORWARD_OFFSET, lateral_offset: -YEHAT_LAUNCH_LATERAL_OFFSET },
];




define_ship_struct!(YehatTerminator);

impl Ship for YehatTerminator {
    const RACE_NAME: &'static str = "Yehat";
    const SHIP_CLASS: &'static str = "Terminator";
    const SPRITE_PREFIX: &'static str = "yehat-terminator";
    const CAPTAIN_NAMES: &'static [&'static str] = &["Heep-eep", "Feep-eep", "Reep-eep", "Yeep-eep", "Beep-eep", "Eeep-eep", "Meep-eep", "Teep-eep", "Jeep-eep", "Leep-eep", "Peep-eep", "Weep-eep", "Veep-eep", "Geep-eep", "Zeep-eep", "Neep-eep"];
    const COST: i32 = 23;
    const COLOR: u32 = 0xffffff;
    const SIZE: f64 = 12.0;
    const MASS: f64 = 3.0;
    const THRUST_INCREMENT: f64 = 1.2;
    const MAX_SPEED: f64 = 5.0;
    const TURN_RATE: f64 = std::f64::consts::FRAC_PI_8;
    const TURN_WAIT: i32 = 2;
    const THRUST_WAIT: i32 = 2;
    const WEAPON_WAIT: i32 = 0;
    const SPECIAL_WAIT: i32 = 2;
    const MAX_ENERGY: i32 = 10;
    const ENERGY_REGENERATION: i32 = 2;
    const ENERGY_WAIT: i32 = 6;
    const WEAPON_ENERGY_COST: i32 = 1;
    const SPECIAL_ENERGY_COST: i32 = 3;
    const MAX_CREW: i32 = 20;


    fn primary_volley_spec(&self) -> Option<ProjectileVolleySpec> {
        Some(ProjectileVolleySpec {
            projectile: PrimaryProjectileSpec {
                speed: YEHAT_MISSILE_SPEED,
                acceleration: 0.0,
                max_speed: YEHAT_MISSILE_SPEED,
                life: YEHAT_MISSILE_LIFE,
                offset: YEHAT_LAUNCH_FORWARD_OFFSET,
                turn_wait: 0,
                texture_prefix: "yehat-missile",
                sound_key: "",
                behavior: ProjectileBehaviorSpec::Tracking,
                collision: ProjectileCollisionSpec::Polygon(&YEHAT_PROJECTILE_POLYGON),
                impact: ProjectileImpactSpec {
                    damage: YEHAT_MISSILE_DAMAGE,
                    texture_prefix: "battle-blast",
                    start_frame: 0,
                    end_frame: 7,
                    sound_key: "battle-boom-23",
                },
            },
            spawns: &YEHAT_PRIMARY_SPAWNS,
            sound_key: "",
            target_mode: ProjectileTargetMode::EnemyShip,
        })
    }

    fn special_ability_spec(&self) -> SpecialAbilitySpec {
        SpecialAbilitySpec::Shield(ShieldSpecialSpec {
            active_texture_prefix: "yehat-shield",
            sound_key: "",
        })
    }

    fn active_texture_prefix(&self, special_active: bool) -> &'static str {
        if special_active {
            "yehat-shield"
        } else {
            self.sprite_prefix()
        }
    }
}
