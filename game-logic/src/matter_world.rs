use matter_js_rs::body::BodyHandle;
use matter_js_rs::engine::{
    CollisionHookPair, CollisionResponseHook, CollisionResponsePolicy, Engine, PhysicsEvent,
};
use matter_js_rs::factory::Bodies;
use matter_js_rs::geometry::Vec2;

use crate::wrap::wrap_axis;

const BASE_DELTA: f64 = 1000.0 / 60.0;

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
    pub normal_x: f64,
    pub normal_y: f64,
}

pub struct MatterStepResult {
    pub bodies: Vec<MatterBodyState>,
    pub collisions: Vec<MatterCollisionPair>,
}

#[derive(Default)]
struct SkipVelocityHook;

impl CollisionResponseHook for SkipVelocityHook {
    fn response_for_pair(&mut self, _pair: CollisionHookPair) -> CollisionResponsePolicy {
        CollisionResponsePolicy::SkipVelocity
    }
}

pub struct MatterWorld {
    engine: Engine<SkipVelocityHook>,
}

impl MatterWorld {
    pub fn new() -> Self {
        let mut engine = Engine::new().with_collision_response_hook(SkipVelocityHook);
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
        let step_count = (delta / BASE_DELTA).ceil().max(1.0) as usize;
        let step_delta = delta / step_count as f64;
        let mut collisions = Vec::new();

        for _ in 0..step_count {
            let events = self.engine.update(step_delta);
            for pair in flatten_collisions(&events) {
                if !collisions.iter().any(|existing: &MatterCollisionPair| {
                    existing.body_a == pair.body_a && existing.body_b == pair.body_b
                }) {
                    collisions.push(pair);
                }
            }
        }

        MatterStepResult {
            bodies: self.engine.bodies.iter().map(|body| MatterBodyState {
                id: body.id,
                x: body.position.x,
                y: body.position.y,
                vx: body.velocity.x,
                vy: body.velocity.y,
                angle: body.angle,
            }).collect(),
            collisions,
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

    pub fn create_ship_polygon_body(
        &mut self,
        x: f64,
        y: f64,
        vertices: &[Vec2],
        mass: f64,
        restitution: f64,
        angle: f64,
    ) -> usize {
        let handle = BodyHandle(self.engine.bodies.len());
        let mut body = Bodies::from_vertices(handle, Vec2 { x, y }, vertices.to_vec());
        body.set_mass(mass);
        body.set_inertia(1e20);
        body.friction_air = 0.0;
        body.friction = 0.0;
        body.friction_static = 0.0;
        body.restitution = restitution;
        body.set_angle(angle, false);
        self.engine.add_body(body);
        handle.0
    }

    pub fn set_body_mass(&mut self, body_id: usize, mass: f64) {
        let Some(body) = self.engine.bodies.get_mut(body_id) else {
            return;
        };

        body.set_mass(mass);
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

    pub fn set_body_angle(&mut self, body_id: usize, angle: f64) {
        let Some(body) = self.engine.bodies.get_mut(body_id) else {
            return;
        };

        body.set_angle(angle, false);
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

    pub fn body_uses_polygon_shape(&self, body_id: usize) -> Option<bool> {
        let body = self.engine.bodies.get(body_id)?;
        Some(body.circle_radius == 0.0)
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
        let collisions = match event {
            PhysicsEvent::CollisionStart { pairs }
            | PhysicsEvent::CollisionActive { pairs } => pairs,
            PhysicsEvent::CollisionEnd { .. } => continue,
        };

        for pair in collisions {
            pairs.push(MatterCollisionPair {
                body_a: pair.body_a,
                body_b: pair.body_b,
                normal_x: pair.normal.x,
                normal_y: pair.normal.y,
            });
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
    use matter_js_rs::geometry::Vec2;

    #[test]
    fn step_reports_collision_normal_for_polygon_bodies() {
        let mut world = MatterWorld::new();
        let body_a = world.create_ship_polygon_body(
            100.0,
            100.0,
            &[
                Vec2 { x: -30.0, y: -8.0 },
                Vec2 { x: 30.0, y: -8.0 },
                Vec2 { x: 30.0, y: 8.0 },
                Vec2 { x: -30.0, y: 8.0 },
            ],
            10.0,
            0.8,
            0.0,
        );
        let body_b = world.create_ship_polygon_body(
            155.0,
            100.0,
            &[
                Vec2 { x: -30.0, y: -8.0 },
                Vec2 { x: 30.0, y: -8.0 },
                Vec2 { x: 30.0, y: 8.0 },
                Vec2 { x: -30.0, y: 8.0 },
            ],
            10.0,
            0.8,
            0.0,
        );
        world.set_body_velocity(body_a, 10.0, 0.0);
        world.set_body_velocity(body_b, -10.0, 0.0);

        let result = world.step(1000.0 / 24.0);
        let pair = result.collisions[0];

        assert_eq!(
            ((pair.normal_x.abs() + pair.normal_y.abs()) * 100.0).round() as i32 > 0,
            true,
        );
    }

    #[test]
    fn step_detects_collision_between_polygon_bodies() {
        let mut world = MatterWorld::new();
        let body_a = world.create_ship_polygon_body(
            100.0,
            100.0,
            &[
                Vec2 { x: -30.0, y: -8.0 },
                Vec2 { x: 30.0, y: -8.0 },
                Vec2 { x: 30.0, y: 8.0 },
                Vec2 { x: -30.0, y: 8.0 },
            ],
            10.0,
            0.8,
            0.0,
        );
        let body_b = world.create_ship_polygon_body(
            155.0,
            100.0,
            &[
                Vec2 { x: -30.0, y: -8.0 },
                Vec2 { x: 30.0, y: -8.0 },
                Vec2 { x: 30.0, y: 8.0 },
                Vec2 { x: -30.0, y: 8.0 },
            ],
            10.0,
            0.8,
            0.0,
        );
        world.set_body_velocity(body_a, 10.0, 0.0);
        world.set_body_velocity(body_b, -10.0, 0.0);

        let result = world.step(1000.0 / 24.0);

        assert_eq!(result.collisions.is_empty(), false);
    }

    #[test]
    fn step_detects_fast_collision_across_a_battle_tick() {
        let mut world = MatterWorld::new();
        let body_a = world.create_ship_body(100.0, 100.0, 12.0, 10.0, 0.8);
        let body_b = world.create_ship_body(170.0, 100.0, 12.0, 10.0, 0.8);
        world.set_body_velocity(body_a, 30.0, 0.0);
        world.set_body_velocity(body_b, -30.0, 0.0);

        let result = world.step(1000.0 / 24.0);

        assert_eq!(result.collisions.is_empty(), false);
    }

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
