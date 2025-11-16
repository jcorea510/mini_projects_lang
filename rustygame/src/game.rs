use macroquad::prelude::*;
use crate::entities::{player, slimesimple};
use crate::scenegrap::scenenode::SceneNode;
use crate::scenegrap::layers::Layers;
use crate::scenegrap::background::Background;
use crate::resources::textures::TextureManager;
use std::rc::Rc;
use std::cell::RefCell;

pub struct Game {
    // scene_graph: Rc<RefCell<SceneNode>>,
    // world_map: Background,
    scene_layers: Vec<Layers>,
    _texture_holder: TextureManager,
}

impl Game {
    pub async fn new() -> Self {
        let mut texture_holder = TextureManager::default();

        let player = player::Player::new(&mut texture_holder).await;
        let player = Rc::new(
            RefCell::new(
                SceneNode::new(
                    Box::new(player))));

        let slime = slimesimple::SlimeSimple::new(&mut texture_holder).await;
        let slime = Rc::new(
            RefCell::new(
                SceneNode::new(
                    Box::new(slime))));


        let slime2 = slimesimple::SlimeSimple::new(&mut texture_holder).await;
        let slime2 = Rc::new(
            RefCell::new(
                SceneNode::new(
                    Box::new(slime2))));

        let slime3 = slimesimple::SlimeSimple::new(&mut texture_holder).await;
        let slime3 = Rc::new(
            RefCell::new(
                SceneNode::new(
                    Box::new(slime3))));

        SceneNode::add_child(&player ,&slime);
        SceneNode::add_child(&slime ,&slime2);
        SceneNode::add_child(&slime ,&slime3);

        let world_map = Background::new(&mut texture_holder).await;

        let scene_layers = vec![Layers::Background(world_map), Layers::Entities(player.clone())];
        Self {
            scene_layers,
            _texture_holder: texture_holder
        }
    }

    pub async fn run(&mut self) {
        loop {
            clear_background(BLACK);
            for layer in self.scene_layers.iter() {
                layer.update(get_frame_time());
                layer.draw();
            }
            draw_fps();
            next_frame().await
        }
    }
}

