use crate::ship::Ship;
use crate::traits::ship_trait::HitPolygonPoint;

pub struct HumanCruiser {
    crew: i32,
    energy: i32,
    facing: f64,
    turn_counter: i32,
    thrust_counter: i32,
    weapon_counter: i32,
    special_counter: i32,
    energy_counter: i32,
}

impl HumanCruiser {
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

impl Ship for HumanCruiser {
    const RACE_NAME: &'static str = "Earthling";
    const SHIP_CLASS: &'static str = "Cruiser";
    const SPRITE_PREFIX: &'static str = "human-cruiser";
    const CAPTAIN_NAMES: &'static [&'static str] = &["Decker", "Trent", "Adama", "Spiff", "Graeme", "Kirk", "Pike", "Halleck", "Tuf", "Pirx", "Wu", "VanRijn", "Ender", "Buck", "Solo", "Belt"];
    const COST: i32 = 11;
    const COLOR: u32 = 0x4488ff;
    const SIZE: f64 = 16.0;
    const MASS: f64 = 6.0;
    const THRUST_INCREMENT: f64 = 0.6;
    const MAX_SPEED: f64 = 4.0;
    const TURN_RATE: f64 = std::f64::consts::FRAC_PI_8;
    const TURN_WAIT: i32 = 1;
    const THRUST_WAIT: i32 = 4;
    const WEAPON_WAIT: i32 = 10;
    const SPECIAL_WAIT: i32 = 9;
    const MAX_ENERGY: i32 = 18;
    const ENERGY_REGENERATION: i32 = 1;
    const ENERGY_WAIT: i32 = 8;
    const WEAPON_ENERGY_COST: i32 = 9;
    const SPECIAL_ENERGY_COST: i32 = 4;
    const MAX_CREW: i32 = 18;

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
        const BASE_POLYGON: [HitPolygonPoint; 31] = [
            HitPolygonPoint { x: 0.0, y: -68.0 },
            HitPolygonPoint { x: 11.0, y: -65.0 },
            HitPolygonPoint { x: 17.0, y: -57.0 },
            HitPolygonPoint { x: 17.0, y: -43.0 },
            HitPolygonPoint { x: 11.0, y: -35.0 },
            HitPolygonPoint { x: 6.0, y: -27.0 },
            HitPolygonPoint { x: 6.0, y: -14.0 },
            HitPolygonPoint { x: 13.0, y: -10.0 },
            HitPolygonPoint { x: 21.0, y: 5.0 },
            HitPolygonPoint { x: 25.0, y: 39.0 },
            HitPolygonPoint { x: 24.0, y: 67.0 },
            HitPolygonPoint { x: 17.0, y: 68.0 },
            HitPolygonPoint { x: 14.0, y: 42.0 },
            HitPolygonPoint { x: 10.0, y: 8.0 },
            HitPolygonPoint { x: 6.0, y: 0.0 },
            HitPolygonPoint { x: 6.0, y: 66.0 },
            HitPolygonPoint { x: -6.0, y: 66.0 },
            HitPolygonPoint { x: -6.0, y: 0.0 },
            HitPolygonPoint { x: -10.0, y: 8.0 },
            HitPolygonPoint { x: -14.0, y: 42.0 },
            HitPolygonPoint { x: -17.0, y: 68.0 },
            HitPolygonPoint { x: -24.0, y: 67.0 },
            HitPolygonPoint { x: -25.0, y: 39.0 },
            HitPolygonPoint { x: -21.0, y: 5.0 },
            HitPolygonPoint { x: -13.0, y: -10.0 },
            HitPolygonPoint { x: -6.0, y: -14.0 },
            HitPolygonPoint { x: -6.0, y: -27.0 },
            HitPolygonPoint { x: -11.0, y: -35.0 },
            HitPolygonPoint { x: -17.0, y: -43.0 },
            HitPolygonPoint { x: -17.0, y: -57.0 },
            HitPolygonPoint { x: -11.0, y: -65.0 },
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
}
