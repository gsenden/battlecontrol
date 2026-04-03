use matter_js_rs::body::BodyHandle;
use matter_js_rs::engine::{Engine, PhysicsEvent};
use matter_js_rs::factory::Bodies;
use matter_js_rs::geometry::Vec2;

use crate::wrap::wrap_axis;

#[derive(Clone, Copy)]
pub struct MatterBodyState {
    pub id: usize,
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub angle: f64,
}

#[derive(Clone, Copy)]
pub struct MatterCollisionPair {
    pub body_a: usize,
    pub body_b: usize,
}

pub struct MatterStepResult {
    pub bodies: Vec<MatterBodyState>,
    pub collisions: Vec<MatterCollisionPair>,
}

pub struct MatterWorld {
    engine: Engine,
}

impl MatterWorld {
    pub fn new() -> Self {
        let mut engine = Engine::new();
        engine.gravity.x = 0.0;
        engine.gravity.y = 0.0;
        engine.gravity.scale = 0.0;

        Self { engine }
    }

    pub fn setup_demo(&mut self) {
        self.engine.bodies.clear();

        let mut body_a = Bodies::circle(BodyHandle(0), 100.0, 100.0, 20.0);
        body_a.set_velocity(Vec2 { x: 0.35, y: 0.0 });
        body_a.friction_air = 0.0;
        body_a.restitution = 0.8;

        let mut body_b = Bodies::circle(BodyHandle(1), 220.0, 100.0, 20.0);
        body_b.set_velocity(Vec2 { x: -0.2, y: 0.0 });
        body_b.friction_air = 0.0;
        body_b.restitution = 0.8;

        self.engine.add_body(body_a);
        self.engine.add_body(body_b);
    }

    pub fn step(&mut self, delta: f64) -> MatterStepResult {
        let events = self.engine.update(delta);

        MatterStepResult {
            bodies: self.engine.bodies.iter().map(|body| MatterBodyState {
                id: body.id,
                x: body.position.x,
                y: body.position.y,
                vx: body.velocity.x,
                vy: body.velocity.y,
                angle: body.angle,
            }).collect(),
            collisions: flatten_collisions(&events),
        }
    }

    pub fn apply_thrust(&mut self, body_id: usize, force_x: f64, force_y: f64) {
        let Some(body) = self.engine.bodies.get_mut(body_id) else {
            return;
        };

        let position = body.position;
        body.apply_force(&position, &Vec2 { x: force_x, y: force_y });
    }

    pub fn rotate_body(&mut self, body_id: usize, angle: f64) {
        let Some(body) = self.engine.bodies.get_mut(body_id) else {
            return;
        };

        body.rotate(angle, None, false);
    }

    pub fn create_ship_body(&mut self, x: f64, y: f64, radius: f64, mass: f64, restitution: f64) -> usize {
        let handle = BodyHandle(self.engine.bodies.len());
        let mut body = Bodies::circle(handle, x, y, radius);
        body.set_mass(mass);
        body.set_inertia(1e20);
        body.friction_air = 0.0;
        body.friction = 0.0;
        body.friction_static = 0.0;
        body.restitution = restitution;
        self.engine.add_body(body);
        handle.0
    }

    pub fn set_body_velocity(&mut self, body_id: usize, vx: f64, vy: f64) {
        let Some(body) = self.engine.bodies.get_mut(body_id) else {
            return;
        };

        body.set_velocity(Vec2 { x: vx, y: vy });
    }

    pub fn add_body_velocity(&mut self, body_id: usize, dvx: f64, dvy: f64) {
        let Some(body) = self.engine.bodies.get_mut(body_id) else {
            return;
        };

        body.set_velocity(Vec2 {
            x: body.velocity.x + dvx,
            y: body.velocity.y + dvy,
        });
    }

    pub fn set_body_position(&mut self, body_id: usize, x: f64, y: f64) {
        let Some(body) = self.engine.bodies.get_mut(body_id) else {
            return;
        };

        body.set_position(Vec2 { x, y }, false);
    }

    pub fn wrap_body(&mut self, body_id: usize, width: f64, height: f64) -> Option<MatterBodyState> {
        let Some(body) = self.engine.bodies.get_mut(body_id) else {
            return None;
        };

        let x = wrap_axis(body.position.x, width);
        let y = wrap_axis(body.position.y, height);

        if x != body.position.x || y != body.position.y {
            body.set_position(Vec2 { x, y }, false);
        }

        Some(MatterBodyState {
            id: body.id,
            x: body.position.x,
            y: body.position.y,
            vx: body.velocity.x,
            vy: body.velocity.y,
            angle: body.angle,
        })
    }

    pub fn body_state(&self, body_id: usize) -> Option<MatterBodyState> {
        let body = self.engine.bodies.get(body_id)?;

        Some(MatterBodyState {
            id: body.id,
            x: body.position.x,
            y: body.position.y,
            vx: body.velocity.x,
            vy: body.velocity.y,
            angle: body.angle,
        })
    }

    pub fn disable_body(&mut self, body_id: usize) {
        let Some(body) = self.engine.bodies.get_mut(body_id) else {
            return;
        };

        body.set_velocity(Vec2 { x: 0.0, y: 0.0 });
        body.set_position(Vec2 { x: -10000.0, y: -10000.0 }, false);
        body.set_static(true);
    }
}

fn flatten_collisions(events: &[PhysicsEvent]) -> Vec<MatterCollisionPair> {
    let mut pairs = Vec::new();

    for event in events {
        if let PhysicsEvent::CollisionStart { pairs: started } = event {
            for (body_a, body_b) in started {
                pairs.push(MatterCollisionPair {
                    body_a: *body_a,
                    body_b: *body_b,
                });
            }
        }
    }

    pairs
}

impl Default for MatterWorld {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::MatterWorld;

    #[test]
    fn wrap_body_wraps_past_world_edge() {
        let mut world = MatterWorld::new();
        let body_id = world.create_ship_body(1005.0, -10.0, 12.0, 10.0, 0.8);

        let wrapped = world.wrap_body(body_id, 1000.0, 800.0).unwrap();

        assert_eq!((wrapped.x, wrapped.y), (5.0, 790.0));
    }

    #[test]
    fn step_moves_body() {
        let mut world = MatterWorld::new();
        world.setup_demo();
        let before = world.step(0.0).bodies[0].x;
        let after = world.step(1000.0 / 60.0).bodies[0].x;

        assert!(after > before);
    }

    #[test]
    fn thrust_changes_velocity() {
        let mut world = MatterWorld::new();
        world.setup_demo();
        let before = world.step(0.0).bodies[0].vy;
        world.apply_thrust(0, 0.0, -0.005);
        let after = world.step(1000.0 / 60.0).bodies[0].vy;

        assert!(after < before);
    }
}
