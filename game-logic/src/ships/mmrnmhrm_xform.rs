use crate::define_ship_struct;
use crate::ship::Ship;
use crate::traits::ship_trait::{
    InstantLaserSpec, PrimaryProjectileSpec, ProjectileBehaviorSpec, ProjectileCollisionSpec,
    ProjectileImpactSpec, ProjectileTargetMode, SpecialAbilitySpec, TransformSpec,
};

const MMRNMHRM_LASER_RANGE: f64 = 141.0;
const MMRNMHRM_LASER_OFFSET: f64 = 16.0;
const MMRNMHRM_YWING_SPEED: f64 = 20.0;
const MMRNMHRM_YWING_LIFE: i32 = 40;
const MMRNMHRM_YWING_OFFSET: f64 = 4.0;




define_ship_struct!(MmrnmhrmXform);

impl Ship for MmrnmhrmXform {
    const RACE_NAME: &'static str = "Mmrnmhrm";
    const SHIP_CLASS: &'static str = "X-Form";
    const SPRITE_PREFIX: &'static str = "mmrnmhrm-xform";
    const CAPTAIN_NAMES: &'static [&'static str] = &["Qir-nha", "Jhe-qir", "Qua-rhna", "Mn-quah", "Nrna-mha", "Um-hrh", "Hm-nhuh", "Rrma-hrn", "Jra-nr", "Ur-mfrs", "Qua-qir", "Mrm-na", "Jhe-mhr", "Hmr-hun", "Nhuh-na", "Hrnm-hm"];
    const COST: i32 = 19;
    const COLOR: u32 = 0xffffff;
    const SIZE: f64 = 12.0;
    const MASS: f64 = 3.0;
    const THRUST_INCREMENT: f64 = 1.0;
    const MAX_SPEED: f64 = 3.3;
    const TURN_RATE: f64 = std::f64::consts::FRAC_PI_8;
    const TURN_WAIT: i32 = 2;
    const THRUST_WAIT: i32 = 1;
    const WEAPON_WAIT: i32 = 0;
    const SPECIAL_WAIT: i32 = 0;
    const MAX_ENERGY: i32 = 10;
    const ENERGY_REGENERATION: i32 = 2;
    const ENERGY_WAIT: i32 = 6;
    const WEAPON_ENERGY_COST: i32 = 1;
    const SPECIAL_ENERGY_COST: i32 = 10;
    const MAX_CREW: i32 = 20;


    fn primary_instant_laser_spec(&self) -> Option<InstantLaserSpec> {
        Some(InstantLaserSpec {
            range: MMRNMHRM_LASER_RANGE,
            damage: 1,
            offset: MMRNMHRM_LASER_OFFSET,
            sound_key: "",
            impact_sound_key: "battle-boom-23",
            color: 0xffffff,
            width: 3.0,
            target_mode: ProjectileTargetMode::EnemyShip,
        })
    }

    fn primary_instant_laser_spec_for_state(&self, special_active: bool) -> Option<InstantLaserSpec> {
        (!special_active).then(|| self.primary_instant_laser_spec()).flatten()
    }

    fn primary_projectile_spec_for_state(&self, special_active: bool) -> Option<PrimaryProjectileSpec> {
        special_active.then_some(PrimaryProjectileSpec {
            speed: MMRNMHRM_YWING_SPEED,
            acceleration: 0.0,
            max_speed: MMRNMHRM_YWING_SPEED,
            life: MMRNMHRM_YWING_LIFE,
            offset: MMRNMHRM_YWING_OFFSET,
            turn_wait: 5,
            texture_prefix: "mmrnmhrm-torpedo",
            sound_key: "",
            behavior: ProjectileBehaviorSpec::Tracking,
            collision: ProjectileCollisionSpec::None,
            impact: ProjectileImpactSpec {
                damage: 1,
                texture_prefix: "battle-blast",
                start_frame: 0,
                end_frame: 7,
                sound_key: "battle-boom-23",
            },
        })
    }

    fn active_texture_prefix(&self, special_active: bool) -> &'static str {
        if special_active {
            "mmrnmhrm-ywing"
        } else {
            Self::SPRITE_PREFIX
        }
    }

    fn primary_projectile_target_mode_for_state(&self, special_active: bool) -> ProjectileTargetMode {
        if special_active {
            ProjectileTargetMode::EnemyShip
        } else {
            ProjectileTargetMode::None
        }
    }

    fn special_ability_spec(&self) -> SpecialAbilitySpec {
        SpecialAbilitySpec::Transform(TransformSpec {
            active_texture_prefix: "mmrnmhrm-ywing",
            sound_key: "",
        })
    }

    fn special_state_persists_after_cooldown(&self) -> bool {
        true
    }
}
