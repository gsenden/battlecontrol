use crate::traits::ship_trait::{Ship, ShipState};

macro_rules! define_ship_registry {
    (@inner ($d:tt) [$($variant:ident, $module:ident, $sprite:literal);+]) => {
        $(mod $module;)+
        $(pub use $module::$variant;)+

        pub const ALL_SHIP_TYPES: [&str; 25] = [$($sprite),+];

        pub enum AnyShip {
            $($variant($variant)),+
        }

        $(impl From<$variant> for AnyShip {
            fn from(ship: $variant) -> Self { AnyShip::$variant(ship) }
        })+

        pub fn build_ship(ship_type: &str) -> Option<AnyShip> {
            match ship_type {
                $($sprite => Some($variant::new().into()),)+
                _ => None,
            }
        }

        macro_rules! dispatch_ref {
            ($d self_:expr, $d method:ident()) => {
                match $d self_ {
                    $(AnyShip::$variant(ship) => ship.$d method()),+
                }
            };
            ($d self_:expr, $d method:ident($d($d arg:expr),+)) => {
                match $d self_ {
                    $(AnyShip::$variant(ship) => ship.$d method($d($d arg),+)),+
                }
            };
        }

        macro_rules! dispatch_mut {
            ($d self_:expr, $d method:ident()) => {
                match $d self_ {
                    $(AnyShip::$variant(ship) => ship.$d method()),+
                }
            };
            ($d self_:expr, $d method:ident($d($d arg:expr),+)) => {
                match $d self_ {
                    $(AnyShip::$variant(ship) => ship.$d method($d($d arg),+)),+
                }
            };
        }
    };
    ($($variant:ident, $module:ident, $sprite:literal);+ $(;)?) => {
        define_ship_registry!(@inner ($) [$($variant, $module, $sprite);+]);
    };
}

define_ship_registry! {
    AndrosynthGuardian, androsynth_guardian, "androsynth-guardian";
    ArilouSkiff, arilou_skiff, "arilou-skiff";
    ChenjesuBroodhome, chenjesu_broodhome, "chenjesu-broodhome";
    ChmmrAvatar, chmmr_avatar, "chmmr-avatar";
    DruugeMauler, druuge_mauler, "druuge-mauler";
    HumanCruiser, human_cruiser, "human-cruiser";
    IlwrathAvenger, ilwrath_avenger, "ilwrath-avenger";
    KohrahMarauder, kohrah_marauder, "kohrah-marauder";
    MelnormeTrader, melnorme_trader, "melnorme-trader";
    MmrnmhrmXform, mmrnmhrm_xform, "mmrnmhrm-xform";
    MyconPodship, mycon_podship, "mycon-podship";
    OrzNemesis, orz_nemesis, "orz-nemesis";
    PkunkFury, pkunk_fury, "pkunk-fury";
    ShofixtiScout, shofixti_scout, "shofixti-scout";
    SlylandroProbe, slylandro_probe, "slylandro-probe";
    SpathiEluder, spathi_eluder, "spathi-eluder";
    SupoxBlade, supox_blade, "supox-blade";
    SyreenPenetrator, syreen_penetrator, "syreen-penetrator";
    ThraddashTorch, thraddash_torch, "thraddash-torch";
    UmgahDrone, umgah_drone, "umgah-drone";
    UrquanDreadnought, urquan_dreadnought, "urquan-dreadnought";
    UtwigJugger, utwig_jugger, "utwig-jugger";
    VuxIntruder, vux_intruder, "vux-intruder";
    YehatTerminator, yehat_terminator, "yehat-terminator";
    ZoqfotpikStinger, zoqfotpik_stinger, "zoqfotpik-stinger";
}

impl AnyShip {
    pub fn update(
        &mut self,
        input: &crate::ship_input::ShipInput,
        velocity: &crate::velocity_vector::VelocityVector,
        allow_beyond_max_speed: bool,
    ) -> Vec<crate::physics_command::PhysicsCommand> {
        dispatch_mut!(self, update(input, velocity, allow_beyond_max_speed))
    }

    pub fn apply_collision_cooldowns(&mut self) {
        dispatch_mut!(self, apply_collision_cooldowns())
    }

    pub fn gravity_command(&self, dx: f64, dy: f64) -> Option<crate::physics_command::PhysicsCommand> {
        dispatch_ref!(self, gravity_command(dx, dy))
    }

    pub fn take_damage(&mut self, amount: i32) -> bool {
        dispatch_mut!(self, take_damage(amount))
    }

    pub fn crew(&self) -> i32 { dispatch_ref!(self, crew()) }
    pub fn energy(&self) -> i32 { dispatch_ref!(self, energy()) }
    pub fn facing(&self) -> f64 { dispatch_ref!(self, facing()) }
    pub fn turn_counter(&self) -> i32 { dispatch_ref!(self, turn_counter()) }
    pub fn thrust_counter(&self) -> i32 { dispatch_ref!(self, thrust_counter()) }
    pub fn weapon_counter(&self) -> i32 { dispatch_ref!(self, weapon_counter()) }
    pub fn special_counter(&self) -> i32 { dispatch_ref!(self, special_counter()) }
    pub fn energy_counter(&self) -> i32 { dispatch_ref!(self, energy_counter()) }
    pub fn hit_polygon(&self, facing: i32, center_x: f64, center_y: f64) -> Vec<crate::traits::ship_trait::HitPolygonPoint> {
        dispatch_ref!(self, hit_polygon(facing, center_x, center_y))
    }
    pub fn hit_polygon_for_state(
        &self,
        facing: i32,
        center_x: f64,
        center_y: f64,
        special_active: bool,
    ) -> Vec<crate::traits::ship_trait::HitPolygonPoint> {
        dispatch_ref!(self, hit_polygon_for_state(facing, center_x, center_y, special_active))
    }
    pub fn set_crew(&mut self, value: i32) { dispatch_mut!(self, set_crew(value)) }
    pub fn set_energy(&mut self, value: i32) { dispatch_mut!(self, set_energy(value)) }
    pub fn set_turn_counter(&mut self, value: i32) { dispatch_mut!(self, set_turn_counter(value)) }
    pub fn set_thrust_counter(&mut self, value: i32) { dispatch_mut!(self, set_thrust_counter(value)) }
    pub fn set_special_counter(&mut self, value: i32) { dispatch_mut!(self, set_special_counter(value)) }
    pub fn set_energy_counter(&mut self, value: i32) { dispatch_mut!(self, set_energy_counter(value)) }
    pub fn decrease_energy(&mut self, amount: i32) { dispatch_mut!(self, decrease_energy(amount)) }
    pub fn decrease_facing(&mut self, amount: f64) { dispatch_mut!(self, decrease_facing(amount)) }
    pub fn increase_facing(&mut self, amount: f64) { dispatch_mut!(self, increase_facing(amount)) }
    pub fn decrease_turn_counter(&mut self, amount: i32) { dispatch_mut!(self, decrease_turn_counter(amount)) }
    pub fn decrease_thrust_counter(&mut self, amount: i32) { dispatch_mut!(self, decrease_thrust_counter(amount)) }
    pub fn decrease_energy_counter(&mut self, amount: i32) { dispatch_mut!(self, decrease_energy_counter(amount)) }
    pub fn race_name(&self) -> &'static str { dispatch_ref!(self, race_name()) }
    pub fn ship_class(&self) -> &'static str { dispatch_ref!(self, ship_class()) }
    pub fn sprite_prefix(&self) -> &'static str { dispatch_ref!(self, sprite_prefix()) }
    pub fn captain_names(&self) -> &'static [&'static str] { dispatch_ref!(self, captain_names()) }
    pub fn cost(&self) -> i32 { dispatch_ref!(self, cost()) }
    pub fn color(&self) -> u32 { dispatch_ref!(self, color()) }
    pub fn size(&self) -> f64 { dispatch_ref!(self, size()) }
    pub fn mass(&self) -> f64 { dispatch_ref!(self, mass()) }
    pub fn thrust_increment(&self) -> f64 { dispatch_ref!(self, thrust_increment()) }
    pub fn max_speed(&self) -> f64 { dispatch_ref!(self, max_speed()) }
    pub fn turn_rate(&self) -> f64 { dispatch_ref!(self, turn_rate()) }
    pub fn turn_wait(&self) -> i32 { dispatch_ref!(self, turn_wait()) }
    pub fn thrust_wait(&self) -> i32 { dispatch_ref!(self, thrust_wait()) }
    pub fn weapon_wait(&self) -> i32 { dispatch_ref!(self, weapon_wait()) }
    pub fn special_wait(&self) -> i32 { dispatch_ref!(self, special_wait()) }
    pub fn max_energy(&self) -> i32 { dispatch_ref!(self, max_energy()) }
    pub fn energy_regeneration(&self) -> i32 { dispatch_ref!(self, energy_regeneration()) }
    pub fn energy_wait(&self) -> i32 { dispatch_ref!(self, energy_wait()) }
    pub fn weapon_energy_cost(&self) -> i32 { dispatch_ref!(self, weapon_energy_cost()) }
    pub fn special_energy_cost(&self) -> i32 { dispatch_ref!(self, special_energy_cost()) }
    pub fn max_crew(&self) -> i32 { dispatch_ref!(self, max_crew()) }
    pub fn primary_projectile_spec(&self) -> Option<crate::traits::ship_trait::PrimaryProjectileSpec> {
        dispatch_ref!(self, primary_projectile_spec())
    }
    pub fn primary_projectile_spec_for_state(
        &self,
        special_active: bool,
    ) -> Option<crate::traits::ship_trait::PrimaryProjectileSpec> {
        dispatch_ref!(self, primary_projectile_spec_for_state(special_active))
    }
    pub fn primary_volley_spec(&self) -> Option<crate::traits::ship_trait::ProjectileVolleySpec> {
        dispatch_ref!(self, primary_volley_spec())
    }
    pub fn primary_volley_spec_for_state(
        &self,
        special_active: bool,
    ) -> Option<crate::traits::ship_trait::ProjectileVolleySpec> {
        dispatch_ref!(self, primary_volley_spec_for_state(special_active))
    }
    pub fn primary_instant_laser_spec(&self) -> Option<crate::traits::ship_trait::InstantLaserSpec> {
        dispatch_ref!(self, primary_instant_laser_spec())
    }
    pub fn primary_instant_laser_spec_for_state(
        &self,
        special_active: bool,
    ) -> Option<crate::traits::ship_trait::InstantLaserSpec> {
        dispatch_ref!(self, primary_instant_laser_spec_for_state(special_active))
    }
    pub fn victory_sound_key(&self) -> Option<&'static str> {
        dispatch_ref!(self, victory_sound_key())
    }
    pub fn active_texture_prefix(&self, special_active: bool) -> &'static str {
        dispatch_ref!(self, active_texture_prefix(special_active))
    }
    pub fn special_ability_spec(&self) -> crate::traits::ship_trait::SpecialAbilitySpec {
        dispatch_ref!(self, special_ability_spec())
    }
    pub fn primary_projectile_target_mode(&self) -> crate::traits::ship_trait::ProjectileTargetMode {
        dispatch_ref!(self, primary_projectile_target_mode())
    }
    pub fn primary_projectile_target_mode_for_state(
        &self,
        special_active: bool,
    ) -> crate::traits::ship_trait::ProjectileTargetMode {
        dispatch_ref!(self, primary_projectile_target_mode_for_state(special_active))
    }
    pub fn primary_projectile_inherits_ship_velocity_for_state(&self, special_active: bool) -> bool {
        dispatch_ref!(self, primary_projectile_inherits_ship_velocity_for_state(special_active))
    }
    pub fn special_state_persists_after_cooldown(&self) -> bool {
        dispatch_ref!(self, special_state_persists_after_cooldown())
    }
    pub fn is_targetable(&self, special_active: bool) -> bool {
        dispatch_ref!(self, is_targetable(special_active))
    }
    pub fn is_cloaked(&self, special_active: bool) -> bool {
        dispatch_ref!(self, is_cloaked(special_active))
    }
}

pub fn apply_collision_between(ships: &mut [AnyShip], ship_a_id: usize, ship_b_id: usize) {
    if ship_a_id == ship_b_id || ship_a_id >= ships.len() || ship_b_id >= ships.len() {
        return;
    }

    if ship_a_id < ship_b_id {
        let (left, right) = ships.split_at_mut(ship_b_id);
        left[ship_a_id].apply_collision_cooldowns();
        right[0].apply_collision_cooldowns();
    } else {
        let (left, right) = ships.split_at_mut(ship_a_id);
        right[0].apply_collision_cooldowns();
        left[ship_b_id].apply_collision_cooldowns();
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AndrosynthGuardian, AnyShip, ArilouSkiff, ChenjesuBroodhome, ChmmrAvatar,
        DruugeMauler, HumanCruiser, KohrahMarauder, MelnormeTrader, MmrnmhrmXform,
        MyconPodship, OrzNemesis, PkunkFury, ShofixtiScout, SpathiEluder,
        IlwrathAvenger, SlylandroProbe, SyreenPenetrator, UmgahDrone, UrquanDreadnought,
        UtwigJugger, VuxIntruder, YehatTerminator, ZoqfotpikStinger, apply_collision_between,
    };
    use crate::traits::ship_trait::{ProjectileTargetMode, ShipState};

    #[test]
    fn yehat_terminator_exposes_hit_polygon() {
        assert!(!AnyShip::from(YehatTerminator::new()).hit_polygon(0, 0.0, 0.0).is_empty());
    }

    #[test]
    fn yehat_terminator_primary_volley_does_not_auto_target_enemy_ship() {
        assert!(matches!(
            AnyShip::from(YehatTerminator::new())
                .primary_volley_spec()
                .map(|spec| spec.target_mode),
            Some(ProjectileTargetMode::None),
        ));
    }

    #[test]
    fn yehat_terminator_primary_volley_turn_wait_exceeds_projectile_life() {
        assert!(AnyShip::from(YehatTerminator::new())
            .primary_volley_spec()
            .is_some_and(|spec| spec.projectile.turn_wait > spec.projectile.life));
    }

    #[test]
    fn yehat_terminator_primary_volley_emits_yehat_primary_sound_key() {
        assert!(AnyShip::from(YehatTerminator::new())
            .primary_volley_spec()
            .is_some_and(|spec| spec.projectile.sound_key == "yehat-primary"));
    }

    #[test]
    fn ilwrath_avenger_exposes_cloak_special() {
        assert!(matches!(
            AnyShip::from(IlwrathAvenger::new()).special_ability_spec(),
            crate::traits::ship_trait::SpecialAbilitySpec::Cloak(_)
        ));
    }

    #[test]
    fn mmrnmhrm_xform_exposes_transform_special() {
        assert!(matches!(
            AnyShip::from(MmrnmhrmXform::new()).special_ability_spec(),
            crate::traits::ship_trait::SpecialAbilitySpec::Transform(_)
        ));
    }

    #[test]
    fn syreen_penetrator_exposes_crew_drain_special() {
        assert!(matches!(
            AnyShip::from(SyreenPenetrator::new()).special_ability_spec(),
            crate::traits::ship_trait::SpecialAbilitySpec::CrewDrainTransfer(_)
        ));
    }

    #[test]
    fn slylandro_probe_exposes_planet_harvest_special() {
        assert!(matches!(
            AnyShip::from(SlylandroProbe::new()).special_ability_spec(),
            crate::traits::ship_trait::SpecialAbilitySpec::PlanetHarvest(_)
        ));
    }

    #[test]
    fn pkunk_fury_exposes_sound_only_special() {
        assert!(matches!(
            AnyShip::from(PkunkFury::new()).special_ability_spec(),
            crate::traits::ship_trait::SpecialAbilitySpec::SoundOnly(_)
        ));
    }

    #[test]
    fn orz_nemesis_exposes_secondary_projectile_special() {
        assert!(matches!(
            AnyShip::from(OrzNemesis::new()).special_ability_spec(),
            crate::traits::ship_trait::SpecialAbilitySpec::Projectile(_)
        ));
    }

    #[test]
    fn chenjesu_broodhome_exposes_secondary_projectile_special() {
        assert!(matches!(
            AnyShip::from(ChenjesuBroodhome::new()).special_ability_spec(),
            crate::traits::ship_trait::SpecialAbilitySpec::Projectile(_)
        ));
    }

    #[test]
    fn druuge_mauler_exposes_crew_to_energy_special() {
        assert!(matches!(
            AnyShip::from(DruugeMauler::new()).special_ability_spec(),
            crate::traits::ship_trait::SpecialAbilitySpec::CrewToEnergy(_)
        ));
    }

    #[test]
    fn vux_intruder_exposes_secondary_projectile_special() {
        assert!(matches!(
            AnyShip::from(VuxIntruder::new()).special_ability_spec(),
            crate::traits::ship_trait::SpecialAbilitySpec::Projectile(_)
        ));
    }

    #[test]
    fn zoqfotpik_stinger_exposes_instant_laser_special() {
        assert!(matches!(
            AnyShip::from(ZoqfotpikStinger::new()).special_ability_spec(),
            crate::traits::ship_trait::SpecialAbilitySpec::InstantLaser(_)
        ));
    }

    #[test]
    fn urquan_dreadnought_exposes_secondary_projectile_special() {
        assert!(matches!(
            AnyShip::from(UrquanDreadnought::new()).special_ability_spec(),
            crate::traits::ship_trait::SpecialAbilitySpec::Projectile(_)
        ));
    }

    #[test]
    fn shofixti_scout_exposes_self_destruct_special() {
        assert!(matches!(
            AnyShip::from(ShofixtiScout::new()).special_ability_spec(),
            crate::traits::ship_trait::SpecialAbilitySpec::SelfDestruct(_)
        ));
    }

    #[test]
    fn slylandro_probe_exposes_instant_laser_primary() {
        assert_eq!(
            AnyShip::from(SlylandroProbe::new())
                .primary_instant_laser_spec()
                .map(|spec| spec.damage),
            Some(2),
        );
    }

    #[test]
    fn chmmr_avatar_exposes_secondary_projectile_special() {
        assert!(matches!(
            AnyShip::from(ChmmrAvatar::new()).special_ability_spec(),
            crate::traits::ship_trait::SpecialAbilitySpec::Projectile(_)
        ));
    }

    #[test]
    fn pkunk_fury_exposes_primary_volley_spec() {
        assert_eq!(
            AnyShip::from(PkunkFury::new())
                .primary_volley_spec()
                .map(|spec| spec.spawns.len()),
            Some(3),
        );
    }

    #[test]
    fn umgah_drone_exposes_directional_thrust_special() {
        assert!(matches!(
            AnyShip::from(UmgahDrone::new()).special_ability_spec(),
            crate::traits::ship_trait::SpecialAbilitySpec::DirectionalThrust(_)
        ));
    }

    #[test]
    fn melnorme_trader_exposes_secondary_projectile_special() {
        assert!(matches!(
            AnyShip::from(MelnormeTrader::new()).special_ability_spec(),
            crate::traits::ship_trait::SpecialAbilitySpec::Projectile(_)
        ));
    }

    #[test]
    fn orz_nemesis_exposes_primary_projectile_spec() {
        assert_eq!(
            AnyShip::from(OrzNemesis::new())
                .primary_projectile_spec()
                .map(|spec| spec.texture_prefix),
            Some("orz-howitzer"),
        );
    }

    #[test]
    fn mmrnmhrm_xform_exposes_instant_laser_primary() {
        assert_eq!(
            AnyShip::from(MmrnmhrmXform::new())
                .primary_instant_laser_spec()
                .map(|spec| spec.range as i32),
            Some(141),
        );
    }

    #[test]
    fn kohrah_marauder_exposes_primary_projectile_spec() {
        assert_eq!(
            AnyShip::from(KohrahMarauder::new())
                .primary_projectile_spec()
                .map(|spec| spec.texture_prefix),
            Some("kohrah-buzzsaw"),
        );
    }

    #[test]
    fn androsynth_guardian_exposes_primary_projectile_spec() {
        assert_eq!(
            AnyShip::from(AndrosynthGuardian::new())
                .primary_projectile_spec()
                .map(|spec| spec.texture_prefix),
            Some("androsynth-bubble"),
        );
    }

    #[test]
    fn human_cruiser_exposes_primary_projectile_spec() {
        assert_eq!(
            AnyShip::from(HumanCruiser::new())
                .primary_projectile_spec()
                .map(|spec| spec.texture_prefix),
            Some("human-saturn"),
        );
    }

    #[test]
    fn androsynth_guardian_exposes_blazer_special_spec() {
        assert!(matches!(
            AnyShip::from(AndrosynthGuardian::new()).special_ability_spec(),
            crate::traits::ship_trait::SpecialAbilitySpec::Blazer(_)
        ));
    }

    #[test]
    fn human_cruiser_exposes_point_defense_special_spec() {
        assert!(matches!(
            AnyShip::from(HumanCruiser::new()).special_ability_spec(),
            crate::traits::ship_trait::SpecialAbilitySpec::PointDefense(_)
        ));
    }

    #[test]
    fn arilou_skiff_exposes_instant_laser_primary() {
        assert_eq!(
            AnyShip::from(ArilouSkiff::new())
                .primary_instant_laser_spec()
                .map(|spec| spec.damage),
            Some(1),
        );
    }

    #[test]
    fn spathi_eluder_exposes_secondary_projectile_special() {
        assert!(matches!(
            AnyShip::from(SpathiEluder::new()).special_ability_spec(),
            crate::traits::ship_trait::SpecialAbilitySpec::Projectile(_)
        ));
    }

    #[test]
    fn yehat_terminator_exposes_primary_volley_spec() {
        assert_eq!(
            AnyShip::from(YehatTerminator::new())
                .primary_volley_spec()
                .map(|spec| spec.spawns.len()),
            Some(2),
        );
    }

    #[test]
    fn utwig_jugger_exposes_shield_special_spec() {
        assert!(matches!(
            AnyShip::from(UtwigJugger::new()).special_ability_spec(),
            crate::traits::ship_trait::SpecialAbilitySpec::Shield(_)
        ));
    }

    #[test]
    fn mycon_podship_exposes_crew_regeneration_special_spec() {
        assert!(matches!(
            AnyShip::from(MyconPodship::new()).special_ability_spec(),
            crate::traits::ship_trait::SpecialAbilitySpec::CrewRegeneration(_)
        ));
    }

    #[test]
    fn androsynth_guardian_blazer_polygon_differs_from_guardian_polygon() {
        let ship = AnyShip::from(AndrosynthGuardian::new());
        assert!(
            ship.hit_polygon_for_state(0, 0.0, 0.0, false)
                != ship.hit_polygon_for_state(0, 0.0, 0.0, true)
        );
    }

    #[test]
    fn androsynth_guardian_exposes_hit_polygon() {
        assert!(!AnyShip::from(AndrosynthGuardian::new()).hit_polygon(0, 0.0, 0.0).is_empty());
    }

    #[test]
    fn human_cruiser_exposes_hit_polygon() {
        assert!(!AnyShip::from(HumanCruiser::new()).hit_polygon(0, 0.0, 0.0).is_empty());
    }

    #[test]
    fn apply_collision_between_sets_both_ship_cooldowns() {
        let mut ships = vec![AnyShip::from(HumanCruiser::new()), AnyShip::from(HumanCruiser::new())];

        apply_collision_between(&mut ships, 0, 1);

        assert!(matches!(
            (&ships[0], &ships[1]),
            (AnyShip::HumanCruiser(a), AnyShip::HumanCruiser(b))
                if (a.turn_counter(), a.thrust_counter(), b.turn_counter(), b.thrust_counter()) == (1, 3, 1, 3)
        ));
    }
}
