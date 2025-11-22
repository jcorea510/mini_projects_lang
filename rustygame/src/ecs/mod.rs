pub mod components;
pub mod world;
pub mod systems;

// Re-export commonly used types so external code can use crate::ecs::World, etc.
pub use components::{
    Entity,
    Position,
    Velocity,
    Health,
    Sprite,
    RenderLayer,
    PlayerInput,
    ChasePlayerAI,
};

pub use world::World;
