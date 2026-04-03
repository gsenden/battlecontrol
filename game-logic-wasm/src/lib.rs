use game_logic::battle::{Battle as CoreBattle, BattleSnapshot};
use game_logic::ship::{AnyShip, PhysicsCommand, ShipInput, VelocityVector};
use game_logic::matter_world::{MatterWorld as CoreMatterWorld, MatterStepResult};
use game_logic::ships::{build_ship, ALL_SHIP_TYPES, apply_collision_between};
use game_logic::wrap::shortest_wrapped_delta;
use serde::Serialize;
use wasm_bindgen::prelude::*;

const APP_VERSION: &str = include_str!("../../VERSION");

#[wasm_bindgen]
pub struct GameLogic {
    ships: Vec<AnyShip>,
}

#[wasm_bindgen]
pub struct MatterWorld {
    world: CoreMatterWorld,
}

#[wasm_bindgen]
pub struct Battle {
    battle: CoreBattle,
}

#[wasm_bindgen(js_name = "getVersion")]
pub fn get_version() -> String {
    APP_VERSION.trim().to_string()
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

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MatterBodyStateDto {
    id: usize,
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    angle: f64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MatterCollisionPairDto {
    body_a: usize,
    body_b: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MatterStepDto {
    bodies: Vec<MatterBodyStateDto>,
    collisions: Vec<MatterCollisionPairDto>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BattleShipSnapshotDto {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    crew: i32,
    energy: i32,
    facing: f64,
    thrusting: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BattleSnapshotDto {
    player: BattleShipSnapshotDto,
    target: BattleShipSnapshotDto,
}

#[wasm_bindgen]
impl GameLogic {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self { ships: Vec::new() }
    }

    #[wasm_bindgen(js_name = "getAllShipTypes")]
    pub fn get_all_ship_types(&self) -> JsValue {
        let types: Vec<&str> = ALL_SHIP_TYPES.to_vec();
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
                PhysicsCommand::AddVelocity { vx, vy } => PhysicsCommandDto {
                    command_type: "addVelocity",
                    vx,
                    vy,
                },
            })
            .collect();

        serde_wasm_bindgen::to_value(&dtos).unwrap()
    }

    #[wasm_bindgen(js_name = "applyGravity")]
    pub fn apply_gravity(&self, ship_id: usize, dx: f64, dy: f64) -> JsValue {
        let dtos: Vec<PhysicsCommandDto> = self.ships[ship_id]
            .gravity_command(dx, dy)
            .into_iter()
            .map(|cmd| match cmd {
                PhysicsCommand::SetVelocity { vx, vy } => PhysicsCommandDto {
                    command_type: "setVelocity",
                    vx,
                    vy,
                },
                PhysicsCommand::AddVelocity { vx, vy } => PhysicsCommandDto {
                    command_type: "addVelocity",
                    vx,
                    vy,
                },
            })
            .collect();

        serde_wasm_bindgen::to_value(&dtos).unwrap()
    }

    #[wasm_bindgen(js_name = "applyGravityFromPlanet")]
    pub fn apply_gravity_from_planet(
        &self,
        ship_id: usize,
        ship_x: f64,
        ship_y: f64,
        planet_x: f64,
        planet_y: f64,
        width: f64,
        height: f64,
    ) -> JsValue {
        let dx = shortest_wrapped_delta(ship_x, planet_x, width);
        let dy = shortest_wrapped_delta(ship_y, planet_y, height);
        self.apply_gravity(ship_id, dx, dy)
    }

    #[wasm_bindgen(js_name = "applyCollisionCooldowns")]
    pub fn apply_collision_cooldowns(&mut self, ship_id: usize) {
        self.ships[ship_id].apply_collision_cooldowns();
    }

    #[wasm_bindgen(js_name = "applyCollisionBetween")]
    pub fn apply_collision_between(&mut self, ship_a_id: usize, ship_b_id: usize) {
        apply_collision_between(&mut self.ships, ship_a_id, ship_b_id);
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
        build_ship(ship_type).ok_or_else(|| JsError::new(&format!("unknown ship type: {ship_type}")))
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

#[wasm_bindgen]
impl Battle {
    #[wasm_bindgen(constructor)]
    pub fn new(
        player_ship_type: &str,
        target_ship_type: &str,
        player_x: f64,
        player_y: f64,
        target_x: f64,
        target_y: f64,
        planet_x: f64,
        planet_y: f64,
        width: f64,
        height: f64,
    ) -> Result<Self, JsError> {
        Ok(Self {
            battle: CoreBattle::new(
                player_ship_type,
                target_ship_type,
                player_x,
                player_y,
                target_x,
                target_y,
                planet_x,
                planet_y,
                width,
                height,
            )
            .map_err(|err| JsError::new(&err))?,
        })
    }

    #[wasm_bindgen(js_name = "setPlayerInput")]
    pub fn set_player_input(
        &mut self,
        left: bool,
        right: bool,
        thrust: bool,
        weapon: bool,
        special: bool,
    ) {
        self.battle.set_player_input(ShipInput {
            left,
            right,
            thrust,
            weapon,
            special,
        });
    }

    #[wasm_bindgen(js_name = "switchPlayerShip")]
    pub fn switch_player_ship(&mut self, ship_type: &str) -> Result<(), JsError> {
        self.battle
            .switch_player_ship(ship_type)
            .map_err(|err| JsError::new(&err))
    }

    pub fn tick(&mut self, delta: f64) {
        self.battle.tick(delta);
    }

    #[wasm_bindgen(js_name = "getSnapshot")]
    pub fn get_snapshot(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&to_battle_snapshot_dto(self.battle.snapshot())).unwrap()
    }
}

#[wasm_bindgen]
impl MatterWorld {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            world: CoreMatterWorld::new(),
        }
    }

    #[wasm_bindgen(js_name = "setupDemo")]
    pub fn setup_demo(&mut self) {
        self.world.setup_demo();
    }

    #[wasm_bindgen(js_name = "applyThrust")]
    pub fn apply_thrust(&mut self, body_id: usize, force_x: f64, force_y: f64) {
        self.world.apply_thrust(body_id, force_x, force_y);
    }

    #[wasm_bindgen(js_name = "rotateBody")]
    pub fn rotate_body(&mut self, body_id: usize, angle: f64) {
        self.world.rotate_body(body_id, angle);
    }

    #[wasm_bindgen(js_name = "createShipBody")]
    pub fn create_ship_body(&mut self, x: f64, y: f64, radius: f64, mass: f64, restitution: f64) -> usize {
        self.world.create_ship_body(x, y, radius, mass, restitution)
    }

    #[wasm_bindgen(js_name = "setBodyVelocity")]
    pub fn set_body_velocity(&mut self, body_id: usize, vx: f64, vy: f64) {
        self.world.set_body_velocity(body_id, vx, vy);
    }

    #[wasm_bindgen(js_name = "addBodyVelocity")]
    pub fn add_body_velocity(&mut self, body_id: usize, dvx: f64, dvy: f64) {
        self.world.add_body_velocity(body_id, dvx, dvy);
    }

    #[wasm_bindgen(js_name = "setBodyPosition")]
    pub fn set_body_position(&mut self, body_id: usize, x: f64, y: f64) {
        self.world.set_body_position(body_id, x, y);
    }

    #[wasm_bindgen(js_name = "wrapBody")]
    pub fn wrap_body(&mut self, body_id: usize, width: f64, height: f64) -> JsValue {
        serde_wasm_bindgen::to_value(
            &self.world.wrap_body(body_id, width, height).map(|body| MatterBodyStateDto {
                id: body.id,
                x: body.x,
                y: body.y,
                vx: body.vx,
                vy: body.vy,
                angle: body.angle,
            }),
        )
        .unwrap()
    }

    #[wasm_bindgen(js_name = "disableBody")]
    pub fn disable_body(&mut self, body_id: usize) {
        self.world.disable_body(body_id);
    }

    pub fn step(&mut self, delta: f64) -> JsValue {
        serde_wasm_bindgen::to_value(&to_matter_step_dto(self.world.step(delta))).unwrap()
    }
}

fn to_matter_step_dto(result: MatterStepResult) -> MatterStepDto {
    MatterStepDto {
        bodies: result.bodies.into_iter().map(|body| MatterBodyStateDto {
            id: body.id,
            x: body.x,
            y: body.y,
            vx: body.vx,
            vy: body.vy,
            angle: body.angle,
        }).collect(),
        collisions: result.collisions.into_iter().map(|pair| MatterCollisionPairDto {
            body_a: pair.body_a,
            body_b: pair.body_b,
        }).collect(),
    }
}

fn to_battle_snapshot_dto(snapshot: BattleSnapshot) -> BattleSnapshotDto {
    BattleSnapshotDto {
        player: BattleShipSnapshotDto {
            x: snapshot.player.x,
            y: snapshot.player.y,
            vx: snapshot.player.vx,
            vy: snapshot.player.vy,
            crew: snapshot.player.crew,
            energy: snapshot.player.energy,
            facing: snapshot.player.facing,
            thrusting: snapshot.player.thrusting,
        },
        target: BattleShipSnapshotDto {
            x: snapshot.target.x,
            y: snapshot.target.y,
            vx: snapshot.target.vx,
            vy: snapshot.target.vy,
            crew: snapshot.target.crew,
            energy: snapshot.target.energy,
            facing: snapshot.target.facing,
            thrusting: snapshot.target.thrusting,
        },
    }
}
