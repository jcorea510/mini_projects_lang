use std::collections::HashMap;
use std::rc::Rc;

use macroquad::math::Vec2;
use macroquad::texture::Texture2D;

use crate::animations::animations::SpreadAnimation;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Entity(pub u32);

#[derive(Clone, Debug)]
pub struct Position(pub Vec2);

#[derive(Clone, Debug)]
pub struct Velocity(pub Vec2);

#[derive(Clone, Debug)]
pub struct Health {
    pub current: i32,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum RenderLayer {
    /// Big background image / tilemap behind everything.
    Background,
    /// Ground or floor surface.
    Surface,
    /// Objects that sit on top of the surface (traps, mud, ice, fire, etc.).
    SurfaceObject,
    /// Dynamic entities such as player, enemies, projectiles.
    Entity,
}

/// High-level state of an entity, used to choose which sprite sheet to render.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum State {
    Idle,
    Attack,
    Walk,
    Run,
    Hurt,
    Death,
}

impl State {
    /// Map logical state to the string keys used in the texture maps.
    pub fn as_str(&self) -> &'static str {
        match self {
            State::Idle => "Idle",
            State::Attack => "Attack",
            State::Walk => "Walk",
            State::Run => "Run",
            State::Hurt => "Hurt",
            State::Death => "Die",
        }
    }
}

/// Simple per-entity state machine driving action/animation state.
///
/// `state` is the logical action (Idle, Walk, Attack, ...).
/// `time_in_state` accumulates time since last change.
/// `time_to_idle` is the delay after which we automatically return to Idle
/// for non-terminal states (e.g. Attack, Hurt).
pub struct StateMachine {
    pub state: State,
    pub time_in_state: f32,
    pub time_to_idle: f32,
}

pub struct Sprite {
    /// All textures for this entity (by state). Shared between all entities of the same type.
    pub textures: Rc<HashMap<String, Rc<Texture2D>>>,
    /// Currently active state.
    pub state: State,
    /// Convenience handle to the currently active texture.
    pub texture: Rc<Texture2D>,
    /// Independent animation for each logical state. This allows each state
    /// to use a different number of columns / timing without glitches.
    pub animations: HashMap<State, SpreadAnimation>,
    pub layer: RenderLayer,
}

/// Marker component for the controllable player.
#[derive(Clone, Debug)]
pub struct PlayerInput;

/// Simple AI: chase a target entity.
#[derive(Clone, Debug)]
pub struct ChasePlayerAI {
    pub target: Entity,
}
