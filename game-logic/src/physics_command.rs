pub enum PhysicsCommand {
    SetVelocity { vx: f64, vy: f64 },
    AddVelocity { vx: f64, vy: f64 },
}
