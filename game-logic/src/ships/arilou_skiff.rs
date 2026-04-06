use crate::ship::Ship;
use crate::traits::ship_trait::{
    InstantLaserSpec, ProjectileTargetMode, SpecialAbilitySpec, TeleportSpecialSpec,
};
use crate::velocity_vector::VelocityVector;

const ARILOU_LASER_RANGE: f64 = 436.0;
const ARILOU_LASER_DAMAGE: i32 = 1;
const ARILOU_LASER_OFFSET: f64 = 0.0;
const ARILOU_WARP_END_FRAME: i32 = 1;

pub struct ArilouSkiff {
    crew: i32,
    energy: i32,
    facing: f64,
    turn_counter: i32,
    thrust_counter: i32,
    weapon_counter: i32,
    special_counter: i32,
    energy_counter: i32,
}

impl ArilouSkiff {
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
