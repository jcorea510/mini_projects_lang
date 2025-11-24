use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use macroquad::color::{BLUE, RED};
use macroquad::math::{Circle, Vec2};
use macroquad::window::{screen_height, screen_width};
use rand::Rng;
use macroquad::shapes::{draw_circle};

use crate::animations::animations::SpreadAnimation;
use crate::ecs::components::{
    ChasePlayerAI,
    Entity,
    Health,
    // PlayerInput,
    Position,
    RenderLayer,
    Sprite,
    State,
    StateMachine,
    Velocity,
};

pub struct World {
    pub(crate) next_entity: u32,

    pub(crate) positions: HashMap<Entity, Position>,
    pub(crate) velocities: HashMap<Entity, Velocity>,
    pub(crate) healths: HashMap<Entity, Health>,
    pub(crate) sprites: HashMap<Entity, Sprite>,
    pub(crate) state_machines: HashMap<Entity, StateMachine>,

    pub(crate) player_inputs: HashSet<Entity>,
    pub(crate) chase_ais: HashMap<Entity, ChasePlayerAI>,
    pub(crate) _game_name: String,
}

impl World {
    pub fn new(name: String) -> Self {
        Self {
            next_entity: 0,
            positions: HashMap::new(),
            velocities: HashMap::new(),
            healths: HashMap::new(),
            sprites: HashMap::new(),
            state_machines: HashMap::new(),
            player_inputs: HashSet::new(),
            chase_ais: HashMap::new(),
            _game_name: name,
        }
    }

    pub(crate) fn spawn_empty(&mut self) -> Entity {
        let id = self.next_entity;
        self.next_entity += 1;
        Entity(id)
    }

    /// Spawn the player with all of its state-specific textures.
    pub fn spawn_player(
        &mut self,
        textures: Rc<HashMap<String, Rc<macroquad::texture::Texture2D>>>,
    ) -> Entity {
        // Match previous parameters from Player::new
        let (position_x, position_y) = (screen_width() / 2.0, screen_height() / 2.0);
        let position = Vec2::new(position_x, position_y);
        let health = 100;

        // Configure per-state animations. You can tweak these numbers per sprite sheet.
        let mut animations = HashMap::new();
        let idle_tex = textures
            .get(State::Idle.as_str())
            .expect("player idle texture should exist");
        animations.insert(
            State::Idle,
            SpreadAnimation::new(idle_tex, 12, 4, 1.0 / 2.0),
        );
        if let Some(tex) = textures.get(State::Walk.as_str()) {
            animations.insert(State::Walk, SpreadAnimation::new(tex, 6, 4, 1.0 / 2.0));
        }
        if let Some(tex) = textures.get(State::Run.as_str()) {
            animations.insert(State::Run, SpreadAnimation::new(tex, 8, 4, 1.0 / 2.0));
        }
        if let Some(tex) = textures.get(State::Attack.as_str()) {
            animations.insert(State::Attack, SpreadAnimation::new(tex, 8, 4, 1.0 / 2.0));
        }
        if let Some(tex) = textures.get(State::Hurt.as_str()) {
            animations.insert(State::Hurt, SpreadAnimation::new(tex, 5, 4, 1.0 / 2.0));
        }
        if let Some(tex) = textures.get(State::Death.as_str()) {
            animations.insert(State::Death, SpreadAnimation::new(tex, 7, 4, 1.0 / 2.0));
        }

        let state = State::Idle;
        let texture = idle_tex.clone();

        let e = self.spawn_empty();

        self.positions.insert(e, Position(position));
        self.healths.insert(e, Health { current: health });
        self.velocities.insert(e, Velocity(Vec2::new(0.0, 0.0)));
        self.sprites.insert(
            e,
            Sprite {
                textures: textures.clone(),
                state,
                texture,
                animations,
                layer: RenderLayer::Entity,
            },
        );
        // State machine: start in Idle, auto-return after a short delay.
        self.state_machines.insert(
            e,
            StateMachine {
                state,
                time_in_state: 0.0,
                time_to_idle: 0.25,
            },
        );
        self.player_inputs.insert(e);

        e
    }
    
    // this can be simplified by type of slime
    // given them the same logic to spawn 
    // but changing the number of columns/frames of each state
    // It should be possible also to add some debuf as component
    // per enemy so that each enemy produce a type of damage
    //
    /// Spawn a simple chaser slime that shares its textures with other slimes.
    pub fn spawn_chaser_slime(
        &mut self,
        textures: Rc<HashMap<String, Rc<macroquad::texture::Texture2D>>>,
        target: Entity,
    ) -> Entity {
        // Match previous parameters from SlimeSimple::new
        let (position_x, position_y) = generate_offscreen_position();
        let position = Vec2::new(position_x, position_y);
        let health = 100;

        // Configure per-state animations for slimes.
        let mut animations = HashMap::new();
        let idle_tex = textures
            .get(State::Idle.as_str())
            .expect("slime idle texture should exist");
        animations.insert(
            State::Idle,
            SpreadAnimation::new(idle_tex, 6, 4, 1.0 / 2.0),
        );
        if let Some(tex) = textures.get(State::Walk.as_str()) {
            animations.insert(State::Walk, SpreadAnimation::new(tex, 8, 4, 1.0 / 2.0));
        }
        if let Some(tex) = textures.get(State::Run.as_str()) {
            animations.insert(State::Run, SpreadAnimation::new(tex, 8, 4, 1.0 / 2.0));
        }
        if let Some(tex) = textures.get(State::Attack.as_str()) {
            animations.insert(State::Attack, SpreadAnimation::new(tex, 10, 4, 1.0 / 2.0));
        }
        if let Some(tex) = textures.get(State::Hurt.as_str()) {
            animations.insert(State::Hurt, SpreadAnimation::new(tex, 5, 4, 1.0 / 2.0));
        }
        if let Some(tex) = textures.get(State::Death.as_str()) {
            animations.insert(State::Death, SpreadAnimation::new(tex, 10, 4, 1.0 / 2.0));
        }

        let state = State::Idle;
        let texture = idle_tex.clone();

        let e = self.spawn_empty();

        self.positions.insert(e, Position(position));
        self.healths.insert(e, Health { current: health });
        self.velocities.insert(e, Velocity(Vec2::new(0.0, 0.0)));
        self.sprites.insert(
            e,
            Sprite {
                textures: textures.clone(),
                state,
                texture,
                animations,
                layer: RenderLayer::Entity,
            },
        );
        self.state_machines.insert(
            e,
            StateMachine {
                state,
                time_in_state: 0.0,
                time_to_idle: 0.25,
            },
        );
        self.chase_ais.insert(e, ChasePlayerAI { target });

        e
    }

    pub fn spawn_chaser_slime_cold(
        &mut self,
        textures: Rc<HashMap<String, Rc<macroquad::texture::Texture2D>>>,
        target: Entity,
    ) -> Entity { 
        let (pos_x, pos_y) = generate_offscreen_position();
        let position = Vec2::new(pos_x, pos_y);
        let health = 150;

        let mut animations = HashMap::new();
        let idle_tex = textures
            .get(State::Idle.as_str())
            .expect("slime cold idle texture should exist");
        animations.insert(
            State::Idle,
            SpreadAnimation::new(idle_tex, 6, 4, 1.0/2.0));
        if let Some(tex) = textures.get(State::Walk.as_str()) {
            animations.insert(State::Walk, SpreadAnimation::new(tex, 8, 4, 1.0/2.0));
        }
        if let Some(tex) = textures.get(State::Run.as_str()) {
            animations.insert(State::Run, SpreadAnimation::new(tex, 8, 4, 1.0 / 2.0));
        }
        if let Some(tex) = textures.get(State::Attack.as_str()) {
            animations.insert(State::Attack, SpreadAnimation::new(tex, 11, 4, 1.0 / 2.0));
        }
        if let Some(tex) = textures.get(State::Hurt.as_str()) {
            animations.insert(State::Hurt, SpreadAnimation::new(tex, 5, 4, 1.0 / 2.0));
        }
        if let Some(tex) = textures.get(State::Death.as_str()) {
            animations.insert(State::Death, SpreadAnimation::new(tex, 10, 4, 1.0 / 2.0));
        }
        
        let state = State::Idle;
        let texture = idle_tex.clone();

        let e = self.spawn_empty();
        self.positions.insert(e, Position(position));
        self.healths.insert(e, Health { current: health });
        self.velocities.insert(e, Velocity(Vec2::new(0.0, 0.0)));
        self.sprites.insert(
            e,
            Sprite { 
                textures: textures.clone(),
                state,
                texture,
                animations,
                layer: RenderLayer::Entity,
            },
        );
        self.state_machines.insert(
            e,
            StateMachine { 
                state,
                time_in_state: 0.0,
                time_to_idle: 0.25, 
            },
        );
        self.chase_ais.insert(e, ChasePlayerAI { target });

        e
    }

    pub fn spawn_chaser_slime_fire(
        &mut self,
        textures: Rc<HashMap<String, Rc<macroquad::texture::Texture2D>>>,
        target: Entity,
    ) -> Entity { 
        let (pos_x, pos_y) = generate_offscreen_position();
        let position = Vec2::new(pos_x, pos_y);
        let health = 150;

        let mut animations = HashMap::new();
        let idle_tex = textures
            .get(State::Idle.as_str())
            .expect("slime cold idle texture should exist");
        animations.insert(
            State::Idle,
            SpreadAnimation::new(idle_tex, 5, 4, 1.0/2.0));
        if let Some(tex) = textures.get(State::Walk.as_str()) {
            animations.insert(State::Walk, SpreadAnimation::new(tex, 8, 4, 1.0/2.0));
        }
        if let Some(tex) = textures.get(State::Run.as_str()) {
            animations.insert(State::Run, SpreadAnimation::new(tex, 8, 4, 1.0 / 2.0));
        }
        if let Some(tex) = textures.get(State::Attack.as_str()) {
            animations.insert(State::Attack, SpreadAnimation::new(tex, 9, 4, 1.0 / 2.0));
        }
        if let Some(tex) = textures.get(State::Hurt.as_str()) {
            animations.insert(State::Hurt, SpreadAnimation::new(tex, 5, 4, 1.0 / 2.0));
        }
        if let Some(tex) = textures.get(State::Death.as_str()) {
            animations.insert(State::Death, SpreadAnimation::new(tex, 10, 4, 1.0 / 2.0));
        }
        
        let state = State::Idle;
        let texture = idle_tex.clone();

        let e = self.spawn_empty();
        self.positions.insert(e, Position(position));
        self.healths.insert(e, Health { current: health });
        self.velocities.insert(e, Velocity(Vec2::new(0.0, 0.0)));
        self.sprites.insert(
            e,
            Sprite { 
                textures: textures.clone(),
                state,
                texture,
                animations,
                layer: RenderLayer::Entity,
            },
        );
        self.state_machines.insert(
            e,
            StateMachine { 
                state,
                time_in_state: 0.0,
                time_to_idle: 0.25, 
            },
        );
        self.chase_ais.insert(e, ChasePlayerAI { target });

        e
    }

    /// Change an entity's sprite state and swap its texture if that state exists.
    pub fn set_sprite_state(&mut self, entity: &Entity, state: State) {
        if let Some(sprite) = self.sprites.get_mut(entity) {
            if sprite.state == state {
                return;
            }
            if let Some(tex) = sprite.textures.get(state.as_str()) {
                sprite.texture = tex.clone();
                sprite.state = state;
            }
        }
    }

    /// Change the logical action state (and, through it, the sprite) for an entity.
    pub fn set_entity_state(&mut self, entity: &Entity, state: State) {
        if let Some(sm) = self.state_machines.get_mut(entity) {
            if sm.state == state {
                return;
            }
            sm.state = state;
            sm.time_in_state = 0.0;
            self.set_sprite_state(entity, state);
        }
    }

    /// Convenience: set the same state for all player-controlled entities.
    pub fn set_players_state(&mut self, state: State) {
        let players: Vec<Entity> = self.player_inputs.iter().copied().collect();
        for e in players {
            self.set_entity_state(&e, state);
        }
    }

    /// Compute a collision region centered on the entity's logical position.
    ///
    /// We treat `Position` as the *center* of the sprite, not the top-left.
    /// This keeps collisions (and debug circles) stable even when different
    /// states use different frame sizes.
    pub fn get_collision_region(&self, entity: &Entity) -> Option<Circle> {
        let sprite = self.sprites.get(entity)?;
        let anim = sprite.animations.get(&sprite.state)?;
        let rect = anim.get_draw_rect();

        let pos = self.positions.get(entity)?;
        let center = pos.0;
        // Radius based on current frame height; tweak factor as desired.
        Some(Circle { x: center.x, y: center.y, r: rect.h / 6.0 })
    }

    /// Like `get_collision_region` but using the entity's position after applying `delta`.
    pub fn get_collision_region_with_delta(&self, entity: &Entity, delta: Vec2) -> Option<Circle> {
        let mut circle = self.get_collision_region(entity)?;
        circle.x += delta.x;
        circle.y += delta.y;
        Some(circle)
    }

    /// Check if `entity`, when moved by `delta`, would collide with `target` (which is static).
    pub fn entity_collision_with_delta(&self, entity: &Entity, target: &Entity, delta: Vec2) -> bool {
        let entity_region_opt = self.get_collision_region_with_delta(entity, delta);
        let target_region_opt = self.get_collision_region(target);

        if entity_region_opt.is_none() || target_region_opt.is_none() {
            return false;
        }
        let entity_region = entity_region_opt.unwrap();
        let target_region = target_region_opt.unwrap();
        entity_region.overlaps(&target_region)
    }

    pub fn entity_collision(&self, entity: &Entity, target: &Entity) -> bool {
        let entity_colition_region_opt = self.get_collision_region(entity);
        let target_colition_region_opt = self.get_collision_region(target);

        if entity_colition_region_opt.is_none() || target_colition_region_opt.is_none() {
            return false;
        }
        let entity_colition_region = entity_colition_region_opt.unwrap();
        let target_colition_region = target_colition_region_opt.unwrap();
        draw_circle(
            target_colition_region.x, 
            target_colition_region.y, 
            target_colition_region.r,
            RED);
        draw_circle(
            entity_colition_region.x, 
            entity_colition_region.y, 
            entity_colition_region.r,
            BLUE);
        entity_colition_region.overlaps(&target_colition_region)
    }
}

fn generate_offscreen_position() -> (f32, f32) {
    let screen_w = screen_width();
    let screen_h = screen_height();

    let min_x = -1.0;
    let max_x = screen_w + 1.0;
    let min_y = -1.0;
    let max_y = screen_h + 1.0;

    // Randomly choose an edge to spawn on (0: top, 1: bottom, 2: left, 3: right)
    let mut rng = rand::rng();
    let edge = rng.random_range(0..4);
    match edge {
        0 | 1 => {
            let x = rng.random_range(0.0..screen_w);
            let y = if edge == 0 {min_y} else {max_y};
            (x, y)
        }
        _ => {
            let y = rng.random_range(0.0..screen_h);
            let x = if edge == 2 {min_x} else {max_x};
            (x, y)
        }
    }
}
