use crate::scenegrap::scenenode::SceneNode;
use crate::scenegrap::background::Background;
use std::rc::Rc;
use std::cell::RefCell;

pub enum Layers {
    Background(Background),
    // GroundObjects,
    Entities(Rc<RefCell<SceneNode>>),
}

impl Layers {
    pub fn draw(&self) {
        match self {
            Layers::Background(inner) => inner.draw(),
            Layers::Entities(inner) => SceneNode::draw(inner),
        };
    }

    pub fn update(&self, dt: f32) {
        if let Layers::Entities(inner) = self { SceneNode::update(inner, dt) }
    }
}

