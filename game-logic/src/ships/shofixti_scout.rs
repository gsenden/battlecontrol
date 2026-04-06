use crate::ship::Ship;
use crate::traits::ship_trait::{
    PrimaryProjectileSpec, ProjectileBehaviorSpec, ProjectileCollisionSpec,
    ProjectileImpactSpec, SelfDestructSpec, SpecialAbilitySpec,
};

const SHOFIXTI_MISSILE_SPEED: f64 = 24.0;
const SHOFIXTI_MISSILE_LIFE: i32 = 10;
const SHOFIXTI_MISSILE_OFFSET: f64 = 51.0;
const SHOFIXTI_MISSILE_DAMAGE: i32 = 1;
const SHOFIXTI_DESTRUCT_DAMAGE: i32 = 18;
const SHOFIXTI_DESTRUCT_RADIUS: f64 = 180.0;
const SHOFIXTI_DESTRUCT_END_FRAME: i32 = 7;

pub struct ShofixtiScout {
    crew: i32,
    energy: i32,
    facing: f64,
    turn_counter: i32,
    thrust_counter: i32,
    weapon_counter: i32,
    special_counter: i32,
    energy_counter: i32,
}

impl ShofixtiScout {
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

impl Ship for ShofixtiScout {
    const RACE_NAME: &'static str = "Shofixti";
    const SHIP_CLASS: &'static str = "Scout";
    const SPRITE_PREFIX: &'static str = "shofixti-scout";
    const CAPTAIN_NAMES: &'static [&'static str] = &["Hiyata", "Wasabe", "Kudzu", "Ichiban", "Bonsai!", "Genjiro", "Ginzu", "Busu", "Gaijin", "Daikon", "Sushi", "Naninani", "Chimchim", "Tora-3", "Tofu", "Kimba"];
    const COST: i32 = 5;
    const COLOR: u32 = 0xffffff;
    const SIZE: f64 = 12.0;
    const MASS: f64 = 1.0;
    const THRUST_INCREMENT: f64 = 1.0;
    const MAX_SPEED: f64 = 5.8;
    const TURN_RATE: f64 = std::f64::consts::FRAC_PI_8;
    const TURN_WAIT: i32 = 1;
    const THRUST_WAIT: i32 = 0;
    const WEAPON_WAIT: i32 = 3;
    const SPECIAL_WAIT: i32 = 0;
    const MAX_ENERGY: i32 = 4;
    const ENERGY_REGENERATION: i32 = 1;
    const ENERGY_WAIT: i32 = 9;
    const WEAPON_ENERGY_COST: i32 = 1;
    const SPECIAL_ENERGY_COST: i32 = 0;
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

    fn primary_projectile_spec(&self) -> Option<PrimaryProjectileSpec> {
        Some(PrimaryProjectileSpec {
            speed: SHOFIXTI_MISSILE_SPEED,
            acceleration: 0.0,
            max_speed: SHOFIXTI_MISSILE_SPEED,
            life: SHOFIXTI_MISSILE_LIFE,
            offset: SHOFIXTI_MISSILE_OFFSET,
            turn_wait: 0,
            texture_prefix: "shofixti-missile",
            sound_key: "",
            behavior: ProjectileBehaviorSpec::Tracking,
            collision: ProjectileCollisionSpec::None,
            impact: ProjectileImpactSpec {
                damage: SHOFIXTI_MISSILE_DAMAGE,
                texture_prefix: "battle-blast",
                start_frame: 0,
                end_frame: 7,
                sound_key: "battle-boom-23",
            },
        })
    }

    fn special_ability_spec(&self) -> SpecialAbilitySpec {
        SpecialAbilitySpec::SelfDestruct(SelfDestructSpec {
            damage: SHOFIXTI_DESTRUCT_DAMAGE,
            radius: SHOFIXTI_DESTRUCT_RADIUS,
            texture_prefix: "shofixti-destruct",
            end_frame: SHOFIXTI_DESTRUCT_END_FRAME,
            sound_key: "",
        })
    }
}
