use crate::define_ship_struct;
use crate::ship::Ship;
use crate::traits::ship_trait::{
    CrewToEnergySpec, PrimaryProjectileSpec, ProjectileBehaviorSpec,
    ProjectileCollisionSpec, ProjectileImpactSpec, SpecialAbilitySpec,
};

const DRUUGE_CANNON_SPEED: f64 = 30.0;
const DRUUGE_CANNON_LIFE: i32 = 20;
const DRUUGE_CANNON_OFFSET: f64 = 24.0;
const DRUUGE_CANNON_DAMAGE: i32 = 6;
const DRUUGE_RECOIL_SPEED: f64 = 6.0;
const DRUUGE_SPECIAL_CREW_COST: i32 = 1;
const DRUUGE_SPECIAL_ENERGY_GAIN: i32 = 16;




define_ship_struct!(DruugeMauler);

impl Ship for DruugeMauler {
    const RACE_NAME: &'static str = "Druuge";
    const SHIP_CLASS: &'static str = "Mauler";
    const SPRITE_PREFIX: &'static str = "druuge-mauler";
    const CAPTAIN_NAMES: &'static [&'static str] = &["Tuuga", "Siinur", "Kaapo", "Juugl", "Paato", "Feezo", "Maad", "Moola", "Kooli", "Faazur", "Zooto", "Biitur", "Duulard", "Piini", "Soopi", "Peeru"];
    const COST: i32 = 17;
    const COLOR: u32 = 0xffffff;
    const SIZE: f64 = 15.0;
    const MASS: f64 = 5.0;
    const THRUST_INCREMENT: f64 = 0.4;
    const MAX_SPEED: f64 = 3.3;
    const TURN_RATE: f64 = std::f64::consts::FRAC_PI_8;
    const TURN_WAIT: i32 = 4;
    const THRUST_WAIT: i32 = 1;
    const WEAPON_WAIT: i32 = 10;
    const SPECIAL_WAIT: i32 = 30;
    const MAX_ENERGY: i32 = 32;
    const ENERGY_REGENERATION: i32 = 1;
    const ENERGY_WAIT: i32 = 50;
    const WEAPON_ENERGY_COST: i32 = 4;
    const SPECIAL_ENERGY_COST: i32 = 16;
    const MAX_CREW: i32 = 14;


    fn primary_projectile_spec(&self) -> Option<PrimaryProjectileSpec> {
        Some(PrimaryProjectileSpec {
            speed: DRUUGE_CANNON_SPEED,
            acceleration: 0.0,
            max_speed: DRUUGE_CANNON_SPEED,
            life: DRUUGE_CANNON_LIFE,
            offset: DRUUGE_CANNON_OFFSET,
            turn_wait: 0,
            texture_prefix: "druuge-cannon",
            sound_key: "",
            behavior: ProjectileBehaviorSpec::Tracking,
            collision: ProjectileCollisionSpec::None,
            impact: ProjectileImpactSpec {
                damage: DRUUGE_CANNON_DAMAGE,
                texture_prefix: "battle-blast",
                start_frame: 0,
                end_frame: 7,
                sound_key: "battle-boom-45",
            },
        })
    }

    fn special_ability_spec(&self) -> SpecialAbilitySpec {
        SpecialAbilitySpec::CrewToEnergy(CrewToEnergySpec {
            crew_cost: DRUUGE_SPECIAL_CREW_COST,
            energy_gain: DRUUGE_SPECIAL_ENERGY_GAIN,
            recoil_speed: DRUUGE_RECOIL_SPEED,
            sound_key: "",
        })
    }
}
