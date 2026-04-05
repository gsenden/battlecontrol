use crate::ship::Ship;
use crate::traits::ship_trait::{
    BlazerSpecialSpec, HitPolygonPoint, PrimaryProjectileSpec, ProjectileBehaviorSpec,
    ProjectileCollisionSpec,
    ProjectileImpactSpec,
    ProjectileTargetMode, SpecialAbilitySpec,
};

const ANDROSYNTH_BUBBLE_SPEED: f64 = 32.0;
const ANDROSYNTH_BUBBLE_LIFE: i32 = 200;
const ANDROSYNTH_BUBBLE_OFFSET: f64 = 56.0;
const ANDROSYNTH_BUBBLE_DAMAGE: i32 = 2;
const GENERIC_BLAST_START_FRAME: i32 = 0;
const GENERIC_BLAST_END_FRAME: i32 = 7;
const ANDROSYNTH_BLAZER_SPEED: f64 = 10.0;
const ANDROSYNTH_BLAZER_MASS: f64 = 1.0;
const ANDROSYNTH_BLAZER_DAMAGE: i32 = 3;
const ANDROSYNTH_BLAZER_HIT_RADIUS: f64 = 24.0;
const ANDROSYNTH_BUBBLE_DIRECT_TRACK_RANGE: f64 = 180.0;
const ANDROSYNTH_BUBBLE_SPAWN_REWIND_DIVISOR: f64 = 32.0;
const ANDROSYNTH_BUBBLE_POLYGON: [HitPolygonPoint; 8] = [
    HitPolygonPoint { x: 0.0, y: -20.0 },
    HitPolygonPoint { x: 14.0, y: -14.0 },
    HitPolygonPoint { x: 20.0, y: 0.0 },
    HitPolygonPoint { x: 14.0, y: 14.0 },
    HitPolygonPoint { x: 0.0, y: 20.0 },
    HitPolygonPoint { x: -14.0, y: 14.0 },
    HitPolygonPoint { x: -20.0, y: 0.0 },
    HitPolygonPoint { x: -14.0, y: -14.0 },
];

pub struct AndrosynthGuardian {
    crew: i32,
    energy: i32,
    facing: f64,
    turn_counter: i32,
    thrust_counter: i32,
    weapon_counter: i32,
    special_counter: i32,
    energy_counter: i32,
}

impl AndrosynthGuardian {
    pub fn new() -> Self {
        Self {
            crew: Self::MAX_CREW,
            energy: Self::MAX_ENERGY,
            facing: -std::f64::consts::FRAC_PI_2,
            turn_counter: 0,
            thrust_counter: 0,
            weapon_counter: 0,
            special_counter: 0,
            energy_counter: 0,
        }
    }
}

impl Ship for AndrosynthGuardian {
    const RACE_NAME: &'static str = "Androsynth";
    const SHIP_CLASS: &'static str = "Guardian";
    const SPRITE_PREFIX: &'static str = "androsynth-guardian";
    const CAPTAIN_NAMES: &'static [&'static str] = &["BOOJI-1", "DORN-3", "BIM-XT", "JOR-15", "976-KILL", "KORB-7B", "XR4-TI", "CRC-16", "BHS-79", "DOS-1.0", "ME-262", "AK-47", "1040-EZ", "NECRO-99", "HAL-2001", "SR-71"];
    const COST: i32 = 15;
    const COLOR: u32 = 0xffffff;
    const SIZE: f64 = 16.0;
    const MASS: f64 = 6.0;
    const THRUST_INCREMENT: f64 = 0.6;
    const MAX_SPEED: f64 = 4.0;
    const TURN_RATE: f64 = std::f64::consts::FRAC_PI_8;
    const TURN_WAIT: i32 = 4;
    const THRUST_WAIT: i32 = 0;
    const WEAPON_WAIT: i32 = 0;
    const SPECIAL_WAIT: i32 = 0;
    const MAX_ENERGY: i32 = 24;
    const ENERGY_REGENERATION: i32 = 1;
    const ENERGY_WAIT: i32 = 8;
    const WEAPON_ENERGY_COST: i32 = 3;
    const SPECIAL_ENERGY_COST: i32 = 2;
    const MAX_CREW: i32 = 20;

    fn crew(&self) -> i32 { self.crew }
    fn set_crew(&mut self, value: i32) { self.crew = value }
    fn energy(&self) -> i32 { self.energy }
    fn set_energy(&mut self, value: i32) { self.energy = value }
    fn facing(&self) -> f64 { self.facing }
    fn set_facing(&mut self, value: f64) { self.facing = value }
    fn turn_counter(&self) -> i32 { self.turn_counter }
    fn set_turn_counter(&mut self, value: i32) { self.turn_counter = value }
    fn thrust_counter(&self) -> i32 { self.thrust_counter }
    fn set_thrust_counter(&mut self, value: i32) { self.thrust_counter = value }
    fn weapon_counter(&self) -> i32 { self.weapon_counter }
    fn set_weapon_counter(&mut self, value: i32) { self.weapon_counter = value }
    fn special_counter(&self) -> i32 { self.special_counter }
    fn set_special_counter(&mut self, value: i32) { self.special_counter = value }
    fn energy_counter(&self) -> i32 { self.energy_counter }
    fn set_energy_counter(&mut self, value: i32) { self.energy_counter = value }
    fn hit_polygon(&self, facing: i32, center_x: f64, center_y: f64) -> Vec<HitPolygonPoint> {
        const GUARDIAN_POLYGON: [HitPolygonPoint; 16] = [
            HitPolygonPoint { x: 0.0, y: -54.0 },
            HitPolygonPoint { x: 18.0, y: -44.0 },
            HitPolygonPoint { x: 24.0, y: -24.0 },
            HitPolygonPoint { x: 20.0, y: -8.0 },
            HitPolygonPoint { x: 30.0, y: 8.0 },
            HitPolygonPoint { x: 24.0, y: 28.0 },
            HitPolygonPoint { x: 10.0, y: 48.0 },
            HitPolygonPoint { x: 0.0, y: 56.0 },
            HitPolygonPoint { x: -10.0, y: 48.0 },
            HitPolygonPoint { x: -24.0, y: 28.0 },
            HitPolygonPoint { x: -30.0, y: 8.0 },
            HitPolygonPoint { x: -20.0, y: -8.0 },
            HitPolygonPoint { x: -24.0, y: -24.0 },
            HitPolygonPoint { x: -18.0, y: -44.0 },
            HitPolygonPoint { x: -8.0, y: -52.0 },
            HitPolygonPoint { x: 8.0, y: -52.0 },
        ];
        rotate_polygon(&GUARDIAN_POLYGON, facing, center_x, center_y)
    }

    fn hit_polygon_for_state(
        &self,
        facing: i32,
        center_x: f64,
        center_y: f64,
        special_active: bool,
    ) -> Vec<HitPolygonPoint> {
        if !special_active {
            return self.hit_polygon(facing, center_x, center_y);
        }

        const BLAZER_POLYGON: [HitPolygonPoint; 12] = [
            HitPolygonPoint { x: 0.0, y: -60.0 },
            HitPolygonPoint { x: 14.0, y: -46.0 },
            HitPolygonPoint { x: 18.0, y: -22.0 },
            HitPolygonPoint { x: 12.0, y: 6.0 },
            HitPolygonPoint { x: 6.0, y: 36.0 },
            HitPolygonPoint { x: 0.0, y: 58.0 },
            HitPolygonPoint { x: -6.0, y: 36.0 },
            HitPolygonPoint { x: -12.0, y: 6.0 },
            HitPolygonPoint { x: -18.0, y: -22.0 },
            HitPolygonPoint { x: -14.0, y: -46.0 },
            HitPolygonPoint { x: -6.0, y: -58.0 },
            HitPolygonPoint { x: 6.0, y: -58.0 },
        ];
        rotate_polygon(&BLAZER_POLYGON, facing, center_x, center_y)
    }

    fn primary_projectile_spec(&self) -> Option<PrimaryProjectileSpec> {
        Some(PrimaryProjectileSpec {
            speed: ANDROSYNTH_BUBBLE_SPEED,
            acceleration: 0.0,
            max_speed: ANDROSYNTH_BUBBLE_SPEED,
            life: ANDROSYNTH_BUBBLE_LIFE,
            offset: ANDROSYNTH_BUBBLE_OFFSET,
            turn_wait: 0,
            texture_prefix: "androsynth-bubble",
            sound_key: "androsynth-primary",
            behavior: ProjectileBehaviorSpec::WobbleTracking {
                direct_track_range: ANDROSYNTH_BUBBLE_DIRECT_TRACK_RANGE,
                spawn_rewind_divisor: ANDROSYNTH_BUBBLE_SPAWN_REWIND_DIVISOR,
            },
            collision: ProjectileCollisionSpec::Polygon(&ANDROSYNTH_BUBBLE_POLYGON),
            impact: ProjectileImpactSpec {
                damage: ANDROSYNTH_BUBBLE_DAMAGE,
                texture_prefix: "battle-blast",
                start_frame: GENERIC_BLAST_START_FRAME,
                end_frame: GENERIC_BLAST_END_FRAME,
                sound_key: "battle-boom-23",
            },
        })
    }

    fn active_texture_prefix(&self, special_active: bool) -> &'static str {
        if special_active {
            "androsynth-blazer"
        } else {
            self.sprite_prefix()
        }
    }

    fn special_ability_spec(&self) -> SpecialAbilitySpec {
        SpecialAbilitySpec::Blazer(BlazerSpecialSpec {
            active_texture_prefix: "androsynth-blazer",
            speed: ANDROSYNTH_BLAZER_SPEED,
            mass: ANDROSYNTH_BLAZER_MASS,
            damage: ANDROSYNTH_BLAZER_DAMAGE,
            hit_radius: ANDROSYNTH_BLAZER_HIT_RADIUS,
            activation_sound_key: "androsynth-special",
            impact_sound_key: "battle-boom-23",
        })
    }

    fn primary_projectile_target_mode(&self) -> ProjectileTargetMode {
        ProjectileTargetMode::PlayerSelectedOrEnemyShip
    }
}

fn rotate_polygon(
    base_polygon: &[HitPolygonPoint],
    facing: i32,
    center_x: f64,
    center_y: f64,
) -> Vec<HitPolygonPoint> {
    let rotation = (facing.rem_euclid(16) as f64) * ((2.0 * std::f64::consts::PI) / 16.0);
    base_polygon
        .iter()
        .map(|point| HitPolygonPoint {
            x: center_x + ((point.x * rotation.cos()) - (point.y * rotation.sin())),
            y: center_y + ((point.x * rotation.sin()) + (point.y * rotation.cos())),
        })
        .collect()
}
