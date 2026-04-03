use crate::ship::Ship;

pub struct SpathiEluder {
    crew: i32,
    energy: i32,
    facing: f64,
    turn_counter: i32,
    thrust_counter: i32,
    weapon_counter: i32,
    special_counter: i32,
    energy_counter: i32,
}

impl SpathiEluder {
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
}
