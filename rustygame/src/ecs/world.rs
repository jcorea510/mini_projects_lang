use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use macroquad::math::Vec2;
use macroquad::texture::Texture2D;

use crate::animations::animations::SpreadAnimation;
use crate::ecs::components::{
    Entity,
    Position,
    Velocity,
    Health,
    Sprite,
    RenderLayer,
    PlayerInput,
    ChasePlayerAI,
};

pub struct World {
    pub(crate) next_entity: u32,

    pub(crate) positions: HashMap<Entity, Position>,
    pub(crate) velocities: HashMap<Entity, Velocity>,
    pub(crate) healths: HashMap<Entity, Health>,
    pub(crate) sprites: HashMap<Entity, Sprite>,

    pub(crate) player_inputs: HashSet<Entity>,
    pub(crate) chase_ais: HashMap<Entity, ChasePlayerAI>,
}

impl World {
    pub fn new() -> Self {
        Self {
            next_entity: 0,
            positions: HashMap::new(),
            velocities: HashMap::new(),
            healths: HashMap::new(),
            sprites: HashMap::new(),
            player_inputs: HashSet::new(),
            chase_ais: HashMap::new(),
        }
    }

    pub(crate) fn spawn_empty(&mut self) -> Entity {
        let id = self.next_entity;
        self.next_entity += 1;
        Entity(id)
    }

    pub fn spawn_player(&mut self, sprite_texture: Rc<Texture2D>) -> Entity {
        // Match previous parameters from Player::new
        let position = Vec2::new(10.0, 10.0);
        let health = 100;
        let fps = 1.0 / 2.0;
        let num_cols = 6;
        let num_rows = 4;
        let animation = SpreadAnimation::new(&sprite_texture, num_cols, num_rows, fps);

        let e = self.spawn_empty();

        self.positions.insert(e, Position(position));
        self.healths.insert(e, Health { current: health });
        self.velocities.insert(e, Velocity(Vec2::new(0.0, 0.0)));
        self.sprites.insert(
            e,
            Sprite {
                texture: sprite_texture,
                animation: Some(animation),
                layer: RenderLayer::Entity,
            },
        );
        self.player_inputs.insert(e);

        e
    }

    pub fn spawn_chaser_slime(&mut self, sprite_texture: Rc<Texture2D>, target: Entity) -> Entity {
        // Match previous parameters from SlimeSimple::new
        let position = Vec2::new(10.0, 10.0);
        let health = 100;
        let fps = 1.0 / 2.0;
        let num_cols = 6;
        let num_rows = 4;
        let animation = SpreadAnimation::new(&sprite_texture, num_cols, num_rows, fps);

        let e = self.spawn_empty();

        self.positions.insert(e, Position(position));
        self.healths.insert(e, Health { current: health });
        self.velocities.insert(e, Velocity(Vec2::new(0.0, 0.0)));
        self.sprites.insert(
            e,
            Sprite {
                texture: sprite_texture,
                animation: Some(animation),
                layer: RenderLayer::Entity,
            },
        );
        self.chase_ais.insert(e, ChasePlayerAI { target });

        e
    }
}
