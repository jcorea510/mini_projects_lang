use std::default::Default;
use std::rc::Rc;
use std::cell::RefCell;

// use macroquad::math::Rect;
use macroquad::math::Vec2;
use macroquad::texture;
use macroquad::color;
use macroquad::input;
use crate::entities::entity::Entity;
use crate::entities::entity::EntityLogic;
use crate::entities::identifier::EntityID;
use crate::resources::textures::TextureManager;
use crate::animations::animations::SpreadAnimation;
use crate::animations::animations::Directions;
use crate::scenegrap::scenenode::SceneNode;

pub struct Player {
    sprite: Rc<texture::Texture2D>,
    entity_base: Entity,
    animation: SpreadAnimation,
}

impl Player {
    pub async fn new(texture_holder: &mut TextureManager) -> Self {

        texture_holder.add_texture(
            EntityID::Player.to_string(),
            "Media/Textures/Slime1/Idle/Slime1_Idle_full.png").await;
        let position = Vec2::new(10.0, 10.0);
        let health = 100;
        let base = Entity::new(health, position);
        let texture_path = texture_holder.get_texture(&EntityID::Player.to_string());
        let fps = 1.0/2.0;
        let num_cols = 6;
        let num_rows = 4;
        let animation = SpreadAnimation::new(&texture_path.clone(), num_cols, num_rows, fps);
        Self {
            sprite: texture_path.clone(),
            entity_base: base,
            animation,
        }
    }
}

impl EntityLogic for Player {
    fn draw_current(&self) {
        let position = self.entity_base.get_position();
        let draw_rect = self.animation.get_draw_rect(); 
        texture::draw_texture_ex(
            &self.sprite.clone(),
            position.x,
            position.y,
            color::WHITE,
            texture::DrawTextureParams {
                source: Some(*draw_rect),
                ..Default::default()
            });
    }

    fn update_current(&mut self, _node: &Rc<RefCell<SceneNode>>, dt: f32) {
        let keys = input::get_keys_down();
        for key in keys.iter() {
            let speed = match key {
                input::KeyCode::A => {
                    self.animation.set_row_frame(Directions::Left);
                    Vec2::new(-5.0, 0.0)
                },
                input::KeyCode::D => {
                    self.animation.set_row_frame(Directions::Right);
                    Vec2::new(5.0, 0.0)
                },
                input::KeyCode::W => {
                    self.animation.set_row_frame(Directions::Up);
                    Vec2::new(0.0, -5.0)
                },
                input::KeyCode::S => {
                    self.animation.set_row_frame(Directions::Down);
                    Vec2::new(0.0, 5.0)
                },
                _ => {Vec2::new(0.0, 0.0)},
            };
            self.entity_base.set_movement(speed);
        }
        self.animation.update(dt);
    }

    fn get_position(&self) -> Vec2 {
        *self.entity_base.get_position()
    }
}

