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

pub struct Sprite {
    pub texture: Rc<Texture2D>,
    pub animation: Option<SpreadAnimation>,
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
