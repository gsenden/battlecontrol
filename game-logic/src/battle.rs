use crate::matter_world::{MatterBodyState, MatterWorld};
use crate::physics_command::PhysicsCommand;
use crate::ship_input::ShipInput;
use crate::ships::{apply_collision_between, build_ship, AnyShip};
use crate::velocity_vector::VelocityVector;
use crate::wrap::shortest_wrapped_delta;

#[derive(Clone, Copy)]
pub struct BattleShipSnapshot {
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub crew: i32,
    pub energy: i32,
    pub facing: f64,
    pub thrusting: bool,
}

pub struct BattleSnapshot {
    pub player: BattleShipSnapshot,
    pub target: BattleShipSnapshot,
}

struct BattleShipState {
    ship_id: usize,
    body_id: usize,
    thrusting: bool,
}

pub struct Battle {
    ships: Vec<AnyShip>,
    matter_world: MatterWorld,
    player: BattleShipState,
    target: BattleShipState,
    player_input: ShipInput,
    planet_x: f64,
    planet_y: f64,
    width: f64,
    height: f64,
}

impl Battle {
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
    ) -> Result<Self, String> {
        let mut ships = Vec::new();
        let mut matter_world = MatterWorld::new();

        let player_ship = build_ship(player_ship_type)
            .ok_or_else(|| format!("unknown ship type: {player_ship_type}"))?;
        let player_ship_id = ships.len();
        let player_body_id = matter_world.create_ship_body(
            player_x,
            player_y,
            player_ship.size(),
            player_ship.mass(),
            0.8,
        );
        ships.push(player_ship);

        let target_ship = build_ship(target_ship_type)
            .ok_or_else(|| format!("unknown ship type: {target_ship_type}"))?;
        let target_ship_id = ships.len();
        let target_body_id = matter_world.create_ship_body(
            target_x,
            target_y,
            target_ship.size(),
            target_ship.mass(),
            0.8,
        );
        ships.push(target_ship);

        Ok(Self {
            ships,
            matter_world,
            player: BattleShipState {
                ship_id: player_ship_id,
                body_id: player_body_id,
                thrusting: false,
            },
            target: BattleShipState {
                ship_id: target_ship_id,
                body_id: target_body_id,
                thrusting: false,
            },
            player_input: ShipInput {
                left: false,
                right: false,
                thrust: false,
                weapon: false,
                special: false,
            },
            planet_x,
            planet_y,
            width,
            height,
        })
    }

    pub fn set_player_input(&mut self, input: ShipInput) {
        self.player_input = input;
    }

    pub fn switch_player_ship(&mut self, ship_type: &str) -> Result<(), String> {
        let current = self
            .matter_world
            .body_state(self.player.body_id)
            .ok_or_else(|| "missing player body state".to_string())?;
        let next_ship = build_ship(ship_type).ok_or_else(|| format!("unknown ship type: {ship_type}"))?;
        let next_body_id = self.matter_world.create_ship_body(
            current.x,
            current.y,
            next_ship.size(),
            next_ship.mass(),
            0.8,
        );
        self.matter_world.set_body_velocity(next_body_id, current.vx, current.vy);
        self.matter_world.disable_body(self.player.body_id);
        self.ships[self.player.ship_id] = next_ship;
        self.player.body_id = next_body_id;
        self.player.thrusting = false;
        Ok(())
    }

    pub fn tick(&mut self, delta: f64) {
        self.step_ship(self.player.ship_id, self.player.body_id, self.player_input, true);
        self.step_ship(
            self.target.ship_id,
            self.target.body_id,
            ShipInput {
                left: false,
                right: false,
                thrust: false,
                weapon: false,
                special: false,
            },
            false,
        );

        let state = self.matter_world.step(delta);

        for collision in state.collisions {
            let ids = [collision.body_a, collision.body_b];
            if !ids.contains(&self.player.body_id) || !ids.contains(&self.target.body_id) {
                continue;
            }

            apply_collision_between(&mut self.ships, self.player.ship_id, self.target.ship_id);
        }

        let _ = self.matter_world.wrap_body(self.player.body_id, self.width, self.height);
        let _ = self.matter_world.wrap_body(self.target.body_id, self.width, self.height);
    }

    pub fn snapshot(&self) -> BattleSnapshot {
        BattleSnapshot {
            player: self.snapshot_for(&self.player),
            target: self.snapshot_for(&self.target),
        }
    }

    fn step_ship(&mut self, ship_id: usize, body_id: usize, input: ShipInput, is_player: bool) {
        let Some(body) = self.matter_world.body_state(body_id) else {
            return;
        };

        self.apply_gravity(ship_id, body_id, body);
        let current = self
            .matter_world
            .body_state(body_id)
            .unwrap_or(body);
        let in_gravity_well = self.in_gravity_well(current.x, current.y);
        let commands = self.ships[ship_id].update(
            &input,
            &VelocityVector {
                x: current.vx,
                y: current.vy,
            },
            in_gravity_well,
        );
        let thrusting = apply_commands(&mut self.matter_world, body_id, commands);

        if is_player {
            self.player.thrusting = thrusting;
        } else {
            self.target.thrusting = thrusting;
        }
    }

    fn apply_gravity(&mut self, ship_id: usize, body_id: usize, body: MatterBodyState) {
        let dx = shortest_wrapped_delta(body.x, self.planet_x, self.width);
        let dy = shortest_wrapped_delta(body.y, self.planet_y, self.height);
        if let Some(command) = self.ships[ship_id].gravity_command(dx, dy) {
            let _ = apply_commands(&mut self.matter_world, body_id, vec![command]);
        }
    }

    fn in_gravity_well(&self, x: f64, y: f64) -> bool {
        shortest_wrapped_delta(x, self.planet_x, self.width).abs() <= 420.0
            && shortest_wrapped_delta(y, self.planet_y, self.height).abs() <= 420.0
    }

    fn snapshot_for(&self, ship: &BattleShipState) -> BattleShipSnapshot {
        let body = self
            .matter_world
            .body_state(ship.body_id)
            .expect("battle body state missing");
        let logic = &self.ships[ship.ship_id];

        BattleShipSnapshot {
            x: body.x,
            y: body.y,
            vx: body.vx,
            vy: body.vy,
            crew: logic.crew(),
            energy: logic.energy(),
            facing: logic.facing(),
            thrusting: ship.thrusting,
        }
    }
}

fn apply_commands(matter_world: &mut MatterWorld, body_id: usize, commands: Vec<PhysicsCommand>) -> bool {
    let mut thrusting = false;

    for command in commands {
        match command {
            PhysicsCommand::SetVelocity { vx, vy } => {
                thrusting = true;
                matter_world.set_body_velocity(body_id, vx, vy);
            }
            PhysicsCommand::AddVelocity { vx, vy } => {
                matter_world.add_body_velocity(body_id, vx, vy);
            }
        }
    }

    thrusting
}

#[cfg(test)]
mod tests {
    use super::Battle;

    #[test]
    fn snapshot_exposes_initial_player_position() {
        let battle = Battle::new(
            "human-cruiser",
            "human-cruiser",
            100.0,
            200.0,
            300.0,
            400.0,
            500.0,
            600.0,
            1000.0,
            800.0,
        )
        .unwrap();

        assert_eq!((battle.snapshot().player.x, battle.snapshot().player.y), (100.0, 200.0));
    }
}
