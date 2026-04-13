use crate::define_ship_struct;
use crate::ship::Ship;
use crate::traits::ship_trait::{
    DirectionalThrustSpecialSpec, InstantLaserSpec, ProjectileTargetMode, SpecialAbilitySpec,
};

const UMGAH_CONE_RANGE: f64 = 72.0;
const UMGAH_RETRO_SPEED: f64 = 40.0;




define_ship_struct!(UmgahDrone);

impl Ship for UmgahDrone {
    const RACE_NAME: &'static str = "Umgah";
    const SHIP_CLASS: &'static str = "Drone";
    const SPRITE_PREFIX: &'static str = "umgah-drone";
    const CAPTAIN_NAMES: &'static [&'static str] = &["Julg'ka", "Gibj'o", "Baguk'i", "O'guk'e", "Gwap'he", "Chez'ef", "Znork'i", "Bob", "Kwik'ow", "Ei'Ei'o", "Brewz'k", "Pruk'u", "O'bargy", "Kterbi'a", "Chup'he", "I'buba"];
    const COST: i32 = 7;
    const COLOR: u32 = 0xffffff;
    const SIZE: f64 = 12.0;
    const MASS: f64 = 1.0;
    const THRUST_INCREMENT: f64 = 1.2;
    const MAX_SPEED: f64 = 3.0;
    const TURN_RATE: f64 = std::f64::consts::FRAC_PI_8;
    const TURN_WAIT: i32 = 4;
    const THRUST_WAIT: i32 = 3;
    const WEAPON_WAIT: i32 = 0;
    const SPECIAL_WAIT: i32 = 2;
    const MAX_ENERGY: i32 = 30;
    const ENERGY_REGENERATION: i32 = 30;
    const ENERGY_WAIT: i32 = 150;
    const WEAPON_ENERGY_COST: i32 = 0;
    const SPECIAL_ENERGY_COST: i32 = 1;
    const MAX_CREW: i32 = 10;


    fn primary_instant_laser_spec(&self) -> Option<InstantLaserSpec> {
        Some(InstantLaserSpec {
            range: UMGAH_CONE_RANGE,
            damage: 1,
            offset: 0.0,
            sound_key: "",
            impact_sound_key: "battle-boom-23",
            color: 0xffffff,
            width: 3.0,
            target_mode: ProjectileTargetMode::EnemyShip,
        })
    }

    fn special_ability_spec(&self) -> SpecialAbilitySpec {
        SpecialAbilitySpec::DirectionalThrust(DirectionalThrustSpecialSpec {
            facing_offset: std::f64::consts::PI,
            speed: UMGAH_RETRO_SPEED,
            sound_key: "",
        })
    }
}
