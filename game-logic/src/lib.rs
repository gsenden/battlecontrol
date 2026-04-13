pub mod battle;
pub mod matter_world;
pub mod physics_command;
pub mod ship_input;
pub mod ships;
pub mod traits;
pub mod velocity_vector;
pub mod wrap;

pub mod ship {
    pub use crate::physics_command::PhysicsCommand;
    pub use crate::ship_input::ShipInput;
    pub use crate::ships::*;
    pub use crate::traits::game_object::GameObject;
    pub use crate::traits::ship_trait::Ship;
    pub use crate::velocity_vector::VelocityVector;
}

#[cfg(test)]
pub(crate) mod reference_data;
