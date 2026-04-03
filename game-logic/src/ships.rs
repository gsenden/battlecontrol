mod androsynth_guardian;
mod arilou_skiff;
mod chenjesu_broodhome;
mod chmmr_avatar;
mod druuge_mauler;
mod human_cruiser;
mod ilwrath_avenger;
mod kohrah_marauder;
mod melnorme_trader;
mod mmrnmhrm_xform;
mod mycon_podship;
mod orz_nemesis;
mod pkunk_fury;
mod shofixti_scout;
mod slylandro_probe;
mod spathi_eluder;
mod supox_blade;
mod syreen_penetrator;
mod thraddash_torch;
mod umgah_drone;
mod urquan_dreadnought;
mod utwig_jugger;
mod vux_intruder;
mod yehat_terminator;
mod zoqfotpik_stinger;

use crate::traits::ship_trait::Ship;

pub use androsynth_guardian::AndrosynthGuardian;
pub use arilou_skiff::ArilouSkiff;
pub use chenjesu_broodhome::ChenjesuBroodhome;
pub use chmmr_avatar::ChmmrAvatar;
pub use druuge_mauler::DruugeMauler;
pub use human_cruiser::HumanCruiser;
pub use ilwrath_avenger::IlwrathAvenger;
pub use kohrah_marauder::KohrahMarauder;
pub use melnorme_trader::MelnormeTrader;
pub use mmrnmhrm_xform::MmrnmhrmXform;
pub use mycon_podship::MyconPodship;
pub use orz_nemesis::OrzNemesis;
pub use pkunk_fury::PkunkFury;
pub use shofixti_scout::ShofixtiScout;
pub use slylandro_probe::SlylandroProbe;
pub use spathi_eluder::SpathiEluder;
pub use supox_blade::SupoxBlade;
pub use syreen_penetrator::SyreenPenetrator;
pub use thraddash_torch::ThraddashTorch;
pub use umgah_drone::UmgahDrone;
pub use urquan_dreadnought::UrquanDreadnought;
pub use utwig_jugger::UtwigJugger;
pub use vux_intruder::VuxIntruder;
pub use yehat_terminator::YehatTerminator;
pub use zoqfotpik_stinger::ZoqfotpikStinger;

pub enum AnyShip {
    AndrosynthGuardian(AndrosynthGuardian),
    ArilouSkiff(ArilouSkiff),
    ChenjesuBroodhome(ChenjesuBroodhome),
    ChmmrAvatar(ChmmrAvatar),
    DruugeMauler(DruugeMauler),
    HumanCruiser(HumanCruiser),
    IlwrathAvenger(IlwrathAvenger),
    KohrahMarauder(KohrahMarauder),
    MelnormeTrader(MelnormeTrader),
    MmrnmhrmXform(MmrnmhrmXform),
    MyconPodship(MyconPodship),
    OrzNemesis(OrzNemesis),
    PkunkFury(PkunkFury),
    ShofixtiScout(ShofixtiScout),
    SlylandroProbe(SlylandroProbe),
    SpathiEluder(SpathiEluder),
    SupoxBlade(SupoxBlade),
    SyreenPenetrator(SyreenPenetrator),
    ThraddashTorch(ThraddashTorch),
    UmgahDrone(UmgahDrone),
    UrquanDreadnought(UrquanDreadnought),
    UtwigJugger(UtwigJugger),
    VuxIntruder(VuxIntruder),
    YehatTerminator(YehatTerminator),
    ZoqfotpikStinger(ZoqfotpikStinger),
}

macro_rules! impl_from_ship {
    ($name:ident) => {
        impl From<$name> for AnyShip {
            fn from(ship: $name) -> Self {
                AnyShip::$name(ship)
            }
        }
    };
}

impl_from_ship!(AndrosynthGuardian);
impl_from_ship!(ArilouSkiff);
impl_from_ship!(ChenjesuBroodhome);
impl_from_ship!(ChmmrAvatar);
impl_from_ship!(DruugeMauler);
impl_from_ship!(HumanCruiser);
impl_from_ship!(IlwrathAvenger);
impl_from_ship!(KohrahMarauder);
impl_from_ship!(MelnormeTrader);
impl_from_ship!(MmrnmhrmXform);
impl_from_ship!(MyconPodship);
impl_from_ship!(OrzNemesis);
impl_from_ship!(PkunkFury);
impl_from_ship!(ShofixtiScout);
impl_from_ship!(SlylandroProbe);
impl_from_ship!(SpathiEluder);
impl_from_ship!(SupoxBlade);
impl_from_ship!(SyreenPenetrator);
impl_from_ship!(ThraddashTorch);
impl_from_ship!(UmgahDrone);
impl_from_ship!(UrquanDreadnought);
impl_from_ship!(UtwigJugger);
impl_from_ship!(VuxIntruder);
impl_from_ship!(YehatTerminator);
impl_from_ship!(ZoqfotpikStinger);

macro_rules! dispatch_ref {
    ($self:expr, $method:ident()) => {
        match $self {
            AnyShip::AndrosynthGuardian(ship) => ship.$method(),
            AnyShip::ArilouSkiff(ship) => ship.$method(),
            AnyShip::ChenjesuBroodhome(ship) => ship.$method(),
            AnyShip::ChmmrAvatar(ship) => ship.$method(),
            AnyShip::DruugeMauler(ship) => ship.$method(),
            AnyShip::HumanCruiser(ship) => ship.$method(),
            AnyShip::IlwrathAvenger(ship) => ship.$method(),
            AnyShip::KohrahMarauder(ship) => ship.$method(),
            AnyShip::MelnormeTrader(ship) => ship.$method(),
            AnyShip::MmrnmhrmXform(ship) => ship.$method(),
            AnyShip::MyconPodship(ship) => ship.$method(),
            AnyShip::OrzNemesis(ship) => ship.$method(),
            AnyShip::PkunkFury(ship) => ship.$method(),
            AnyShip::ShofixtiScout(ship) => ship.$method(),
            AnyShip::SlylandroProbe(ship) => ship.$method(),
            AnyShip::SpathiEluder(ship) => ship.$method(),
            AnyShip::SupoxBlade(ship) => ship.$method(),
            AnyShip::SyreenPenetrator(ship) => ship.$method(),
            AnyShip::ThraddashTorch(ship) => ship.$method(),
            AnyShip::UmgahDrone(ship) => ship.$method(),
            AnyShip::UrquanDreadnought(ship) => ship.$method(),
            AnyShip::UtwigJugger(ship) => ship.$method(),
            AnyShip::VuxIntruder(ship) => ship.$method(),
            AnyShip::YehatTerminator(ship) => ship.$method(),
            AnyShip::ZoqfotpikStinger(ship) => ship.$method(),
        }
    };
}

macro_rules! dispatch_mut {
    ($self:expr, $method:ident()) => {
        match $self {
            AnyShip::AndrosynthGuardian(ship) => ship.$method(),
            AnyShip::ArilouSkiff(ship) => ship.$method(),
            AnyShip::ChenjesuBroodhome(ship) => ship.$method(),
            AnyShip::ChmmrAvatar(ship) => ship.$method(),
            AnyShip::DruugeMauler(ship) => ship.$method(),
            AnyShip::HumanCruiser(ship) => ship.$method(),
            AnyShip::IlwrathAvenger(ship) => ship.$method(),
            AnyShip::KohrahMarauder(ship) => ship.$method(),
            AnyShip::MelnormeTrader(ship) => ship.$method(),
            AnyShip::MmrnmhrmXform(ship) => ship.$method(),
            AnyShip::MyconPodship(ship) => ship.$method(),
            AnyShip::OrzNemesis(ship) => ship.$method(),
            AnyShip::PkunkFury(ship) => ship.$method(),
            AnyShip::ShofixtiScout(ship) => ship.$method(),
            AnyShip::SlylandroProbe(ship) => ship.$method(),
            AnyShip::SpathiEluder(ship) => ship.$method(),
            AnyShip::SupoxBlade(ship) => ship.$method(),
            AnyShip::SyreenPenetrator(ship) => ship.$method(),
            AnyShip::ThraddashTorch(ship) => ship.$method(),
            AnyShip::UmgahDrone(ship) => ship.$method(),
            AnyShip::UrquanDreadnought(ship) => ship.$method(),
            AnyShip::UtwigJugger(ship) => ship.$method(),
            AnyShip::VuxIntruder(ship) => ship.$method(),
            AnyShip::YehatTerminator(ship) => ship.$method(),
            AnyShip::ZoqfotpikStinger(ship) => ship.$method(),
        }
    };
    ($self:expr, $method:ident($($arg:expr),+)) => {
        match $self {
            AnyShip::AndrosynthGuardian(ship) => ship.$method($($arg),+),
            AnyShip::ArilouSkiff(ship) => ship.$method($($arg),+),
            AnyShip::ChenjesuBroodhome(ship) => ship.$method($($arg),+),
            AnyShip::ChmmrAvatar(ship) => ship.$method($($arg),+),
            AnyShip::DruugeMauler(ship) => ship.$method($($arg),+),
            AnyShip::HumanCruiser(ship) => ship.$method($($arg),+),
            AnyShip::IlwrathAvenger(ship) => ship.$method($($arg),+),
            AnyShip::KohrahMarauder(ship) => ship.$method($($arg),+),
            AnyShip::MelnormeTrader(ship) => ship.$method($($arg),+),
            AnyShip::MmrnmhrmXform(ship) => ship.$method($($arg),+),
            AnyShip::MyconPodship(ship) => ship.$method($($arg),+),
            AnyShip::OrzNemesis(ship) => ship.$method($($arg),+),
            AnyShip::PkunkFury(ship) => ship.$method($($arg),+),
            AnyShip::ShofixtiScout(ship) => ship.$method($($arg),+),
            AnyShip::SlylandroProbe(ship) => ship.$method($($arg),+),
            AnyShip::SpathiEluder(ship) => ship.$method($($arg),+),
            AnyShip::SupoxBlade(ship) => ship.$method($($arg),+),
            AnyShip::SyreenPenetrator(ship) => ship.$method($($arg),+),
            AnyShip::ThraddashTorch(ship) => ship.$method($($arg),+),
            AnyShip::UmgahDrone(ship) => ship.$method($($arg),+),
            AnyShip::UrquanDreadnought(ship) => ship.$method($($arg),+),
            AnyShip::UtwigJugger(ship) => ship.$method($($arg),+),
            AnyShip::VuxIntruder(ship) => ship.$method($($arg),+),
            AnyShip::YehatTerminator(ship) => ship.$method($($arg),+),
            AnyShip::ZoqfotpikStinger(ship) => ship.$method($($arg),+),
        }
    };
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

    pub fn take_damage(&mut self, amount: i32) -> bool {
        dispatch_mut!(self, take_damage(amount))
    }

    pub fn crew(&self) -> i32 { dispatch_ref!(self, crew()) }
    pub fn energy(&self) -> i32 { dispatch_ref!(self, energy()) }
    pub fn facing(&self) -> f64 { dispatch_ref!(self, facing()) }
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
}
