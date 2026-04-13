use crate::define_ship_struct;
use crate::ship::Ship;
use crate::traits::ship_trait::{
    CloakSpec,
    PrimaryProjectileSpec, ProjectileBehaviorSpec, ProjectileCollisionSpec,
    ProjectileImpactSpec, SpecialAbilitySpec,
};

const ILWRATH_FIRE_SPEED: f64 = 18.0;
const ILWRATH_FIRE_LIFE: i32 = 12;
const ILWRATH_FIRE_OFFSET: f64 = 24.0;
const ILWRATH_FIRE_DAMAGE: i32 = 1;




define_ship_struct!(IlwrathAvenger);

impl Ship for IlwrathAvenger {
    const RACE_NAME: &'static str = "Ilwrath";
    const SHIP_CLASS: &'static str = "Avenger";
    const SPRITE_PREFIX: &'static str = "ilwrath-avenger";
    const CAPTAIN_NAMES: &'static [&'static str] = &["Gorgon", "Taragon", "Kalgon", "Borgo", "Dirga", "Slygor", "Rogash", "Argarak", "Kayzar", "Baylor", "Zoggak", "Targa", "Vogar", "Lurgo", "Regorjo", "Manglor"];
    const COST: i32 = 10;
    const COLOR: u32 = 0xffffff;
    const SIZE: f64 = 18.0;
    const MASS: f64 = 7.0;
    const THRUST_INCREMENT: f64 = 1.0;
    const MAX_SPEED: f64 = 4.2;
    const TURN_RATE: f64 = std::f64::consts::FRAC_PI_8;
    const TURN_WAIT: i32 = 2;
    const THRUST_WAIT: i32 = 0;
    const WEAPON_WAIT: i32 = 0;
    const SPECIAL_WAIT: i32 = 13;
    const MAX_ENERGY: i32 = 16;
    const ENERGY_REGENERATION: i32 = 4;
    const ENERGY_WAIT: i32 = 4;
    const WEAPON_ENERGY_COST: i32 = 1;
    const SPECIAL_ENERGY_COST: i32 = 3;
    const MAX_CREW: i32 = 22;


    fn primary_projectile_spec(&self) -> Option<PrimaryProjectileSpec> {
        Some(PrimaryProjectileSpec {
            speed: ILWRATH_FIRE_SPEED,
            acceleration: 0.0,
            max_speed: ILWRATH_FIRE_SPEED,
            life: ILWRATH_FIRE_LIFE,
            offset: ILWRATH_FIRE_OFFSET,
            turn_wait: 0,
            texture_prefix: "ilwrath-fire",
            sound_key: "",
            behavior: ProjectileBehaviorSpec::Tracking,
            collision: ProjectileCollisionSpec::None,
            impact: ProjectileImpactSpec {
                damage: ILWRATH_FIRE_DAMAGE,
                texture_prefix: "battle-blast",
                start_frame: 0,
                end_frame: 7,
                sound_key: "battle-boom-23",
            },
        })
    }

    fn special_ability_spec(&self) -> SpecialAbilitySpec {
        SpecialAbilitySpec::Cloak(CloakSpec {
            sound_key: "",
        })
    }

    fn special_state_persists_after_cooldown(&self) -> bool {
        true
    }

    fn is_targetable(&self, special_active: bool) -> bool {
        !special_active
    }

    fn is_cloaked(&self, special_active: bool) -> bool {
        special_active
    }
}
