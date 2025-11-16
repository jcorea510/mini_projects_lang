use macroquad::math::Vec2;
use core::fmt;
use std::rc::Rc;
use std::cell::RefCell;
use crate::scenegrap::scenenode::SceneNode;

pub struct Entity {
    health: i32,
    position: Vec2,
}

impl Entity {
    pub fn new(health: i32, position: Vec2) -> Self {
        Self { health, position}
    }
    pub fn take_damage(&mut self, damage: i32) {
        self.health -= damage;
        if self.health < 0 {
            self.health = 0;
        }
    }

    pub fn should_remove(&self) -> bool {
        self.health == 0
    }

    pub fn set_movement(&mut self, speed: Vec2) {
        self.position += speed;
    }

    pub fn get_position(&self) -> &Vec2 {
        &self.position
    }

}

pub trait EntityLogic {
    fn draw_current(&self) {
        
    }

    fn update_current(&mut self,_node: &Rc<RefCell<SceneNode>>, _dt: f32) {

    }

    fn get_position(&self) -> Vec2;
}

impl fmt::Debug for dyn EntityLogic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "A critical issue related to software arquitecture happend. It can not be identified")
    }
    
}
