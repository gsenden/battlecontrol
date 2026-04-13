use crate::define_ship_struct;
use crate::ship::Ship;
use crate::traits::ship_trait::{
    HitPolygonPoint, InstantLaserSpec, ProjectileTargetMode, ShipState, SpecialAbilitySpec,
    TeleportSpecialSpec,
};
use crate::velocity_vector::VelocityVector;

const ARILOU_LASER_RANGE: f64 = 436.0;
const ARILOU_LASER_DAMAGE: i32 = 1;
const ARILOU_LASER_OFFSET: f64 = 0.0;
const ARILOU_WARP_END_FRAME: i32 = 1;




define_ship_struct!(ArilouSkiff);

impl Ship for ArilouSkiff {
    const RACE_NAME: &'static str = "Arilou";
    const SHIP_CLASS: &'static str = "Skiff";
    const SPRITE_PREFIX: &'static str = "arilou-skiff";
    const CAPTAIN_NAMES: &'static [&'static str] = &["Fefaloo", "Bezabu", "Tiptushi", "Marypup", "Tinkafo", "Patooti", "Tifiwilo", "Loleelu", "Louifoui", "Pinywiny", "Oowbabe", "Dingdup", "Wewalia", "Yipyapi", "Ropilup", "Wolwali"];
    const COST: i32 = 16;
    const COLOR: u32 = 0xffffff;
    const SIZE: f64 = 12.0;
    const MASS: f64 = 1.0;
    const THRUST_INCREMENT: f64 = 8.0;
    const MAX_SPEED: f64 = 6.7;
    const TURN_RATE: f64 = std::f64::consts::FRAC_PI_8;
    const TURN_WAIT: i32 = 0;
    const THRUST_WAIT: i32 = 0;
    const WEAPON_WAIT: i32 = 1;
    const SPECIAL_WAIT: i32 = 2;
    const MAX_ENERGY: i32 = 20;
    const ENERGY_REGENERATION: i32 = 1;
    const ENERGY_WAIT: i32 = 6;
    const WEAPON_ENERGY_COST: i32 = 2;
    const SPECIAL_ENERGY_COST: i32 = 3;
    const MAX_CREW: i32 = 6;


    fn hit_polygon(&self, facing: i32, center_x: f64, center_y: f64) -> Vec<HitPolygonPoint> {
        const BASE_POLYGON: [HitPolygonPoint; 12] = [
            HitPolygonPoint { x: 0.0, y: -33.0 },
            HitPolygonPoint { x: 17.0, y: -29.0 },
            HitPolygonPoint { x: 29.0, y: -17.0 },
            HitPolygonPoint { x: 33.0, y: 0.0 },
            HitPolygonPoint { x: 29.0, y: 17.0 },
            HitPolygonPoint { x: 17.0, y: 29.0 },
            HitPolygonPoint { x: 0.0, y: 33.0 },
            HitPolygonPoint { x: -17.0, y: 29.0 },
            HitPolygonPoint { x: -29.0, y: 17.0 },
            HitPolygonPoint { x: -33.0, y: 0.0 },
            HitPolygonPoint { x: -29.0, y: -17.0 },
            HitPolygonPoint { x: -17.0, y: -29.0 },
        ];
        let rotation = (facing.rem_euclid(16) as f64) * ((2.0 * std::f64::consts::PI) / 16.0);
        BASE_POLYGON
            .iter()
            .map(|point| HitPolygonPoint {
                x: center_x + ((point.x * rotation.cos()) - (point.y * rotation.sin())),
                y: center_y + ((point.x * rotation.sin()) + (point.y * rotation.cos())),
            })
            .collect()
    }

    fn primary_instant_laser_spec(&self) -> Option<InstantLaserSpec> {
        Some(InstantLaserSpec {
            range: ARILOU_LASER_RANGE,
            damage: ARILOU_LASER_DAMAGE,
            offset: ARILOU_LASER_OFFSET,
            sound_key: "arilou-primary",
            impact_sound_key: "battle-boom-23",
            color: 0xffee55,
            width: 4.0,
            target_mode: ProjectileTargetMode::PlayerSelectedPointOrForward,
        })
    }

    fn special_ability_spec(&self) -> SpecialAbilitySpec {
        SpecialAbilitySpec::Teleport(TeleportSpecialSpec {
            effect_texture_prefix: "arilou-warp",
            end_frame: ARILOU_WARP_END_FRAME,
            sound_key: "arilou-special",
        })
    }

    fn victory_sound_key(&self) -> Option<&'static str> {
        Some("arilou-victory")
    }

    fn thrust_velocity(
        &self,
        _velocity: &VelocityVector,
        _allow_beyond_max_speed: bool,
        _current_speed: f64,
    ) -> Option<(f64, f64)> {
        Some((self.facing().cos() * self.max_speed(), self.facing().sin() * self.max_speed()))
    }

    fn idle_velocity(&self, _velocity: &VelocityVector) -> Option<(f64, f64)> {
        Some((0.0, 0.0))
    }
}
