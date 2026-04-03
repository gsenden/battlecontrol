use game_logic::ship::{
    AnyShip, PhysicsCommand, ShipInput, VelocityVector,
    AndrosynthGuardian, ArilouSkiff, ChenjesuBroodhome, ChmmrAvatar,
    DruugeMauler, HumanCruiser, IlwrathAvenger, KohrahMarauder,
    MelnormeTrader, MmrnmhrmXform, MyconPodship, OrzNemesis,
    PkunkFury, ShofixtiScout, SlylandroProbe, SpathiEluder,
    SupoxBlade, SyreenPenetrator, ThraddashTorch, UmgahDrone,
    UrquanDreadnought, UtwigJugger, VuxIntruder, YehatTerminator,
    ZoqfotpikStinger,
};
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct GameLogic {
    ships: Vec<AnyShip>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PhysicsCommandDto {
    #[serde(rename = "type")]
    command_type: &'static str,
    vx: f64,
    vy: f64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ShipStateDto {
    crew: i32,
    energy: i32,
    facing: f64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ShipStatsDto {
    race_name: &'static str,
    ship_class: &'static str,
    sprite_prefix: &'static str,
    captain_names: &'static [&'static str],
    cost: i32,
    color: u32,
    size: f64,
    mass: f64,
    thrust_increment: f64,
    max_speed: f64,
    turn_rate: f64,
    turn_wait: i32,
    thrust_wait: i32,
    weapon_wait: i32,
    special_wait: i32,
    max_energy: i32,
    energy_regeneration: i32,
    energy_wait: i32,
    weapon_energy_cost: i32,
    special_energy_cost: i32,
    max_crew: i32,
}

#[wasm_bindgen]
impl GameLogic {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self { ships: Vec::new() }
    }

    #[wasm_bindgen(js_name = "getAllShipTypes")]
    pub fn get_all_ship_types(&self) -> JsValue {
        let types: Vec<&str> = vec![
            "androsynth-guardian",
            "arilou-skiff",
            "chenjesu-broodhome",
            "chmmr-avatar",
            "druuge-mauler",
            "human-cruiser",
            "ilwrath-avenger",
            "kohrah-marauder",
            "melnorme-trader",
            "mmrnmhrm-xform",
            "mycon-podship",
            "orz-nemesis",
            "pkunk-fury",
            "shofixti-scout",
            "slylandro-probe",
            "spathi-eluder",
            "supox-blade",
            "syreen-penetrator",
            "thraddash-torch",
            "umgah-drone",
            "urquan-dreadnought",
            "utwig-jugger",
            "vux-intruder",
            "yehat-terminator",
            "zoqfotpik-stinger",
        ];
        serde_wasm_bindgen::to_value(&types).unwrap()
    }

    #[wasm_bindgen(js_name = "getStatsByType")]
    pub fn get_stats_by_type(&self, ship_type: &str) -> Result<JsValue, JsError> {
        let ship = Self::build_ship(ship_type)?;
        Ok(Self::stats_to_js(&ship))
    }

    #[wasm_bindgen(js_name = "createShip")]
    pub fn create_ship(&mut self, ship_type: &str) -> Result<usize, JsError> {
        let ship = Self::build_ship(ship_type)?;
        let id = self.ships.len();
        self.ships.push(ship);
        Ok(id)
    }

    #[wasm_bindgen(js_name = "updateShip")]
    pub fn update_ship(
        &mut self,
        ship_id: usize,
        left: bool,
        right: bool,
        thrust: bool,
        weapon: bool,
        special: bool,
        vel_x: f64,
        vel_y: f64,
        allow_beyond_max_speed: bool,
    ) -> JsValue {
        let input = ShipInput { left, right, thrust, weapon, special };
        let velocity = VelocityVector { x: vel_x, y: vel_y };
        let commands = self.ships[ship_id].update(&input, &velocity, allow_beyond_max_speed);

        let dtos: Vec<PhysicsCommandDto> = commands
            .into_iter()
            .map(|cmd| match cmd {
                PhysicsCommand::SetVelocity { vx, vy } => PhysicsCommandDto {
                    command_type: "setVelocity",
                    vx,
                    vy,
                },
            })
            .collect();

        serde_wasm_bindgen::to_value(&dtos).unwrap()
    }

    #[wasm_bindgen(js_name = "applyCollisionCooldowns")]
    pub fn apply_collision_cooldowns(&mut self, ship_id: usize) {
        self.ships[ship_id].apply_collision_cooldowns();
    }

    #[wasm_bindgen(js_name = "takeDamage")]
    pub fn take_damage(&mut self, ship_id: usize, amount: i32) -> bool {
        self.ships[ship_id].take_damage(amount)
    }

    #[wasm_bindgen(js_name = "getShipState")]
    pub fn get_ship_state(&self, ship_id: usize) -> JsValue {
        let ship = &self.ships[ship_id];
        let dto = ShipStateDto {
            crew: ship.crew(),
            energy: ship.energy(),
            facing: ship.facing(),
        };
        serde_wasm_bindgen::to_value(&dto).unwrap()
    }

    #[wasm_bindgen(js_name = "getShipStats")]
    pub fn get_ship_stats(&self, ship_id: usize) -> JsValue {
        Self::stats_to_js(&self.ships[ship_id])
    }

    fn build_ship(ship_type: &str) -> Result<AnyShip, JsError> {
        match ship_type {
            "androsynth-guardian" => Ok(AndrosynthGuardian::new().into()),
            "arilou-skiff" => Ok(ArilouSkiff::new().into()),
            "chenjesu-broodhome" => Ok(ChenjesuBroodhome::new().into()),
            "chmmr-avatar" => Ok(ChmmrAvatar::new().into()),
            "druuge-mauler" => Ok(DruugeMauler::new().into()),
            "human-cruiser" => Ok(HumanCruiser::new().into()),
            "ilwrath-avenger" => Ok(IlwrathAvenger::new().into()),
            "kohrah-marauder" => Ok(KohrahMarauder::new().into()),
            "melnorme-trader" => Ok(MelnormeTrader::new().into()),
            "mmrnmhrm-xform" => Ok(MmrnmhrmXform::new().into()),
            "mycon-podship" => Ok(MyconPodship::new().into()),
            "orz-nemesis" => Ok(OrzNemesis::new().into()),
            "pkunk-fury" => Ok(PkunkFury::new().into()),
            "shofixti-scout" => Ok(ShofixtiScout::new().into()),
            "slylandro-probe" => Ok(SlylandroProbe::new().into()),
            "spathi-eluder" => Ok(SpathiEluder::new().into()),
            "supox-blade" => Ok(SupoxBlade::new().into()),
            "syreen-penetrator" => Ok(SyreenPenetrator::new().into()),
            "thraddash-torch" => Ok(ThraddashTorch::new().into()),
            "umgah-drone" => Ok(UmgahDrone::new().into()),
            "urquan-dreadnought" => Ok(UrquanDreadnought::new().into()),
            "utwig-jugger" => Ok(UtwigJugger::new().into()),
            "vux-intruder" => Ok(VuxIntruder::new().into()),
            "yehat-terminator" => Ok(YehatTerminator::new().into()),
            "zoqfotpik-stinger" => Ok(ZoqfotpikStinger::new().into()),
            _ => Err(JsError::new(&format!("unknown ship type: {ship_type}"))),
        }
    }

    fn stats_to_js(ship: &AnyShip) -> JsValue {
        let dto = ShipStatsDto {
            race_name: ship.race_name(),
            ship_class: ship.ship_class(),
            sprite_prefix: ship.sprite_prefix(),
            captain_names: ship.captain_names(),
            cost: ship.cost(),
            color: ship.color(),
            size: ship.size(),
            mass: ship.mass(),
            thrust_increment: ship.thrust_increment(),
            max_speed: ship.max_speed(),
            turn_rate: ship.turn_rate(),
            turn_wait: ship.turn_wait(),
            thrust_wait: ship.thrust_wait(),
            weapon_wait: ship.weapon_wait(),
            special_wait: ship.special_wait(),
            max_energy: ship.max_energy(),
            energy_regeneration: ship.energy_regeneration(),
            energy_wait: ship.energy_wait(),
            weapon_energy_cost: ship.weapon_energy_cost(),
            special_energy_cost: ship.special_energy_cost(),
            max_crew: ship.max_crew(),
        };
        serde_wasm_bindgen::to_value(&dto).unwrap()
    }
}
