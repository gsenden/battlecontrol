use crate::define_ship_struct;
use crate::ship::Ship;
use crate::traits::ship_trait::{
    HitPolygonPoint, PrimaryProjectileSpec, ProjectileBehaviorSpec, ProjectileCollisionSpec,
    ProjectileImpactSpec, ProjectileSpawnSpec, ProjectileTargetMode, ProjectileVolleySpec,
    ShieldSpecialSpec, SpecialAbilitySpec,
};

const YEHAT_MISSILE_SPEED: f64 = 20.0;
const YEHAT_MISSILE_LIFE: i32 = 15;
const YEHAT_MISSILE_TURN_WAIT: i32 = YEHAT_MISSILE_LIFE + 1;
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
const YEHAT_HIT_POLYGON: [HitPolygonPoint; 100] = [
    HitPolygonPoint { x: -35.0, y: -16.0 },
    HitPolygonPoint { x: -35.0, y: -15.0 },
    HitPolygonPoint { x: -35.0, y: -14.0 },
    HitPolygonPoint { x: -36.0, y: -13.0 },
    HitPolygonPoint { x: -36.0, y: -12.0 },
    HitPolygonPoint { x: -36.0, y: -11.0 },
    HitPolygonPoint { x: -36.0, y: -10.0 },
    HitPolygonPoint { x: -36.0, y: -9.0 },
    HitPolygonPoint { x: -36.0, y: -8.0 },
    HitPolygonPoint { x: -38.0, y: -7.0 },
    HitPolygonPoint { x: -38.0, y: -6.0 },
    HitPolygonPoint { x: -38.0, y: -5.0 },
    HitPolygonPoint { x: -38.0, y: -4.0 },
    HitPolygonPoint { x: -38.0, y: -3.0 },
    HitPolygonPoint { x: -38.0, y: -2.0 },
    HitPolygonPoint { x: -38.0, y: -1.0 },
    HitPolygonPoint { x: -38.0, y: 0.0 },
    HitPolygonPoint { x: -37.0, y: 1.0 },
    HitPolygonPoint { x: -37.0, y: 2.0 },
    HitPolygonPoint { x: -37.0, y: 3.0 },
    HitPolygonPoint { x: -37.0, y: 4.0 },
    HitPolygonPoint { x: -36.0, y: 5.0 },
    HitPolygonPoint { x: -36.0, y: 6.0 },
    HitPolygonPoint { x: -36.0, y: 7.0 },
    HitPolygonPoint { x: -36.0, y: 8.0 },
    HitPolygonPoint { x: -35.0, y: 9.0 },
    HitPolygonPoint { x: -35.0, y: 10.0 },
    HitPolygonPoint { x: -34.0, y: 11.0 },
    HitPolygonPoint { x: -34.0, y: 12.0 },
    HitPolygonPoint { x: -33.0, y: 13.0 },
    HitPolygonPoint { x: -33.0, y: 14.0 },
    HitPolygonPoint { x: -32.0, y: 15.0 },
    HitPolygonPoint { x: -31.0, y: 16.0 },
    HitPolygonPoint { x: -30.0, y: 17.0 },
    HitPolygonPoint { x: -29.0, y: 18.0 },
    HitPolygonPoint { x: -28.0, y: 19.0 },
    HitPolygonPoint { x: -27.0, y: 20.0 },
    HitPolygonPoint { x: -26.0, y: 21.0 },
    HitPolygonPoint { x: -25.0, y: 22.0 },
    HitPolygonPoint { x: -24.0, y: 23.0 },
    HitPolygonPoint { x: -23.0, y: 24.0 },
    HitPolygonPoint { x: -21.0, y: 25.0 },
    HitPolygonPoint { x: -20.0, y: 26.0 },
    HitPolygonPoint { x: -18.0, y: 27.0 },
    HitPolygonPoint { x: -16.0, y: 28.0 },
    HitPolygonPoint { x: -14.0, y: 29.0 },
    HitPolygonPoint { x: -12.0, y: 30.0 },
    HitPolygonPoint { x: -8.0, y: 31.0 },
    HitPolygonPoint { x: -5.0, y: 32.0 },
    HitPolygonPoint { x: -3.0, y: 33.0 },
    HitPolygonPoint { x: 3.0, y: 33.0 },
    HitPolygonPoint { x: 4.0, y: 32.0 },
    HitPolygonPoint { x: 7.0, y: 31.0 },
    HitPolygonPoint { x: 11.0, y: 30.0 },
    HitPolygonPoint { x: 13.0, y: 29.0 },
    HitPolygonPoint { x: 15.0, y: 28.0 },
    HitPolygonPoint { x: 17.0, y: 27.0 },
    HitPolygonPoint { x: 19.0, y: 26.0 },
    HitPolygonPoint { x: 20.0, y: 25.0 },
    HitPolygonPoint { x: 22.0, y: 24.0 },
    HitPolygonPoint { x: 23.0, y: 23.0 },
    HitPolygonPoint { x: 24.0, y: 22.0 },
    HitPolygonPoint { x: 25.0, y: 21.0 },
    HitPolygonPoint { x: 26.0, y: 20.0 },
    HitPolygonPoint { x: 27.0, y: 19.0 },
    HitPolygonPoint { x: 28.0, y: 18.0 },
    HitPolygonPoint { x: 29.0, y: 17.0 },
    HitPolygonPoint { x: 30.0, y: 16.0 },
    HitPolygonPoint { x: 31.0, y: 15.0 },
    HitPolygonPoint { x: 32.0, y: 14.0 },
    HitPolygonPoint { x: 32.0, y: 13.0 },
    HitPolygonPoint { x: 33.0, y: 12.0 },
    HitPolygonPoint { x: 33.0, y: 11.0 },
    HitPolygonPoint { x: 34.0, y: 10.0 },
    HitPolygonPoint { x: 34.0, y: 9.0 },
    HitPolygonPoint { x: 35.0, y: 8.0 },
    HitPolygonPoint { x: 35.0, y: 7.0 },
    HitPolygonPoint { x: 35.0, y: 6.0 },
    HitPolygonPoint { x: 35.0, y: 5.0 },
    HitPolygonPoint { x: 36.0, y: 4.0 },
    HitPolygonPoint { x: 36.0, y: 3.0 },
    HitPolygonPoint { x: 36.0, y: 2.0 },
    HitPolygonPoint { x: 37.0, y: 1.0 },
    HitPolygonPoint { x: 37.0, y: 0.0 },
    HitPolygonPoint { x: 37.0, y: -1.0 },
    HitPolygonPoint { x: 37.0, y: -2.0 },
    HitPolygonPoint { x: 37.0, y: -3.0 },
    HitPolygonPoint { x: 37.0, y: -4.0 },
    HitPolygonPoint { x: 37.0, y: -5.0 },
    HitPolygonPoint { x: 37.0, y: -6.0 },
    HitPolygonPoint { x: 37.0, y: -7.0 },
    HitPolygonPoint { x: 35.0, y: -8.0 },
    HitPolygonPoint { x: 35.0, y: -9.0 },
    HitPolygonPoint { x: 35.0, y: -10.0 },
    HitPolygonPoint { x: 35.0, y: -11.0 },
    HitPolygonPoint { x: 35.0, y: -12.0 },
    HitPolygonPoint { x: 35.0, y: -13.0 },
    HitPolygonPoint { x: 34.0, y: -14.0 },
    HitPolygonPoint { x: 34.0, y: -15.0 },
    HitPolygonPoint { x: 34.0, y: -16.0 },
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

    fn hit_polygon(&self, facing: i32, center_x: f64, center_y: f64) -> Vec<HitPolygonPoint> {
        let rotation = (facing.rem_euclid(16) as f64) * ((2.0 * std::f64::consts::PI) / 16.0);
        YEHAT_HIT_POLYGON
            .iter()
            .map(|point| HitPolygonPoint {
                x: center_x + ((point.x * rotation.cos()) - (point.y * rotation.sin())),
                y: center_y + ((point.x * rotation.sin()) + (point.y * rotation.cos())),
            })
            .collect()
    }


    fn primary_volley_spec(&self) -> Option<ProjectileVolleySpec> {
        Some(ProjectileVolleySpec {
            projectile: PrimaryProjectileSpec {
                speed: YEHAT_MISSILE_SPEED,
                acceleration: 0.0,
                max_speed: YEHAT_MISSILE_SPEED,
                life: YEHAT_MISSILE_LIFE,
                offset: YEHAT_LAUNCH_FORWARD_OFFSET,
                turn_wait: YEHAT_MISSILE_TURN_WAIT,
                texture_prefix: "yehat-missile",
                sound_key: "yehat-primary",
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
            sound_key: "yehat-primary",
            target_mode: ProjectileTargetMode::None,
        })
    }

    fn special_ability_spec(&self) -> SpecialAbilitySpec {
        SpecialAbilitySpec::Shield(ShieldSpecialSpec {
            active_texture_prefix: "yehat-shield",
            sound_key: "yehat-special",
        })
    }

    fn primary_projectile_inherits_ship_velocity(&self) -> bool {
        true
    }

    fn active_texture_prefix(&self, special_active: bool) -> &'static str {
        if special_active {
            "yehat-shield"
        } else {
            self.sprite_prefix()
        }
    }
}
