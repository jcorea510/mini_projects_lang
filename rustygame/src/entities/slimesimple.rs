use crate::animations::animations::Directions;
use crate::animations::animations::SpreadAnimation;
use crate::entities::entity::Entity;
use crate::entities::entity::EntityLogic;
use crate::entities::identifier::EntityID;
use crate::resources::textures::TextureManager;
use crate::scenegrap::scenenode::SceneNode;
use macroquad::color;
use macroquad::math::Vec2;
use macroquad::texture;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::rc::Rc;

pub struct SlimeSimple {
    sprite: Rc<texture::Texture2D>,
    entity_base: Entity,
    animation: SpreadAnimation,
}

impl SlimeSimple {
    pub async fn new(texture_holder: &mut TextureManager) -> Self {
        texture_holder
            .add_texture(
                EntityID::SlimeSimple.to_string(),
                "Media/Textures/Slime2/Idle/Slime2_Idle_full.png",
            )
            .await;
        let texture_path = texture_holder.get_texture(&EntityID::SlimeSimple.to_string());
        let position = Vec2::new(10.0, 10.0);
        let health = 100;
        let base = Entity::new(health, position);
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

    pub fn get_movement_directions(&mut self, player_position: &Vec2) -> Vec2 {
        let entity_position = self.entity_base.get_position();
        let speed_x = match player_position.x.total_cmp(&entity_position.x) {
            Ordering::Less => {
                self.animation.set_row_frame(Directions::Left);
                -1.0
            }
            Ordering::Greater => {
                self.animation.set_row_frame(Directions::Right);
                1.0
            }
            Ordering::Equal => 0.0,
        };
        let speed_y = match player_position.y.total_cmp(&entity_position.y) {
            Ordering::Less => {
                self.animation.set_row_frame(Directions::Up);
                -1.0
            }
            Ordering::Greater => {
                self.animation.set_row_frame(Directions::Down);
                1.0
            }
            Ordering::Equal => 0.0,
        };
        Vec2::new(speed_x, speed_y)
    }
}

impl EntityLogic for SlimeSimple {
    fn update_current(&mut self, node: &Rc<RefCell<SceneNode>>, dt: f32) {
        let player_position = SceneNode::get_parent_position(node);
        let speed = self.get_movement_directions(&player_position);
        self.entity_base.set_movement(speed);
        self.animation.update(dt);
    }

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
            },
        );
    }

    fn get_position(&self) -> Vec2 {
        *self.entity_base.get_position()
    }
}
