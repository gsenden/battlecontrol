use crate::ship::Ship;
use crate::traits::ship_trait::HitPolygonPoint;

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
