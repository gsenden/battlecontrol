use crate::ship::Ship;
use crate::traits::ship_trait::{
    HitPolygonPoint, PrimaryProjectileSpec, ProjectileBehaviorSpec, ProjectileCollisionSpec,
    ProjectileImpactSpec, ProjectileSpawnSpec, ProjectileTargetMode, ProjectileVolleySpec,
    ShieldSpecialSpec, SpecialAbilitySpec,
};

const UTWIG_LANCE_SPEED: f64 = 30.0;
const UTWIG_LANCE_LIFE: i32 = 10;
const UTWIG_LANCE_DAMAGE: i32 = 1;
const UTWIG_PROJECTILE_POLYGON: [HitPolygonPoint; 8] = [
    HitPolygonPoint { x: 0.0, y: -14.0 },
    HitPolygonPoint { x: 6.0, y: -9.0 },
    HitPolygonPoint { x: 8.0, y: 0.0 },
    HitPolygonPoint { x: 6.0, y: 9.0 },
    HitPolygonPoint { x: 0.0, y: 14.0 },
    HitPolygonPoint { x: -6.0, y: 9.0 },
    HitPolygonPoint { x: -8.0, y: 0.0 },
    HitPolygonPoint { x: -6.0, y: -9.0 },
];
const UTWIG_PRIMARY_SPAWNS: [ProjectileSpawnSpec; 6] = [
    ProjectileSpawnSpec { facing_offset: 0, forward_offset: 63.0, lateral_offset: 12.0 },
    ProjectileSpawnSpec { facing_offset: 0, forward_offset: 63.0, lateral_offset: -12.0 },
    ProjectileSpawnSpec { facing_offset: 1, forward_offset: 54.0, lateral_offset: 0.0 },
    ProjectileSpawnSpec { facing_offset: -1, forward_offset: 54.0, lateral_offset: 0.0 },
    ProjectileSpawnSpec { facing_offset: 2, forward_offset: 46.0, lateral_offset: 0.0 },
    ProjectileSpawnSpec { facing_offset: -2, forward_offset: 46.0, lateral_offset: 0.0 },
];

pub struct UtwigJugger {
    crew: i32,
    energy: i32,
    facing: f64,
    turn_counter: i32,
    thrust_counter: i32,
    weapon_counter: i32,
    special_counter: i32,
    energy_counter: i32,
}

impl UtwigJugger {
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

impl Ship for UtwigJugger {
    const RACE_NAME: &'static str = "Utwig";
    const SHIP_CLASS: &'static str = "Jugger";
    const SPRITE_PREFIX: &'static str = "utwig-jugger";
    const CAPTAIN_NAMES: &'static [&'static str] = &["Endo", "Vermi", "Manny", "Uuter", "Nergo", "Sami", "Duna", "Frann", "Krisk", "Lololo", "Snoon", "Nestor", "Lurg", "Thory", "Jujuby", "Erog"];
    const COST: i32 = 22;
    const COLOR: u32 = 0xffffff;
    const SIZE: f64 = 19.0;
    const MASS: f64 = 8.0;
    const THRUST_INCREMENT: f64 = 1.2;
    const MAX_SPEED: f64 = 6.0;
    const TURN_RATE: f64 = std::f64::consts::FRAC_PI_8;
    const TURN_WAIT: i32 = 1;
    const THRUST_WAIT: i32 = 6;
    const WEAPON_WAIT: i32 = 7;
    const SPECIAL_WAIT: i32 = 12;
    const MAX_ENERGY: i32 = 20;
    const ENERGY_REGENERATION: i32 = 0;
    const ENERGY_WAIT: i32 = 255;
    const WEAPON_ENERGY_COST: i32 = 0;
    const SPECIAL_ENERGY_COST: i32 = 1;
    const MAX_CREW: i32 = 20;

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

    fn primary_volley_spec(&self) -> Option<ProjectileVolleySpec> {
        Some(ProjectileVolleySpec {
            projectile: PrimaryProjectileSpec {
                speed: UTWIG_LANCE_SPEED,
                acceleration: 0.0,
                max_speed: UTWIG_LANCE_SPEED,
                life: UTWIG_LANCE_LIFE,
                offset: 0.0,
                turn_wait: 0,
                texture_prefix: "utwig-lance",
                sound_key: "",
                behavior: ProjectileBehaviorSpec::Tracking,
                collision: ProjectileCollisionSpec::Polygon(&UTWIG_PROJECTILE_POLYGON),
                impact: ProjectileImpactSpec {
                    damage: UTWIG_LANCE_DAMAGE,
                    texture_prefix: "battle-blast",
                    start_frame: 0,
                    end_frame: 7,
                    sound_key: "battle-boom-23",
                },
            },
            spawns: &UTWIG_PRIMARY_SPAWNS,
            sound_key: "",
            target_mode: ProjectileTargetMode::EnemyShip,
        })
    }

    fn special_ability_spec(&self) -> SpecialAbilitySpec {
        SpecialAbilitySpec::Shield(ShieldSpecialSpec {
            active_texture_prefix: "utwig-jugger",
            sound_key: "",
        })
    }
}
