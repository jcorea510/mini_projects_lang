use std::{default::Default};
use std::collections::HashMap;
use std::rc::Rc;

use macroquad::texture::Texture2D;

#[derive(Default)]
pub struct TextureManager {
    pub(crate) holder: HashMap<String, Rc<HashMap<String, Rc<Texture2D>>>>,
}

impl TextureManager {
    /// Register a group of textures for a given logical ID.
    /// The caller is responsible for loading the textures beforehand.
    pub fn add_textures(&mut self, id: String, textures: HashMap<String, Rc<Texture2D>>) {
        self.holder.insert(id, Rc::new(textures));
    }

    /// Get the full texture map for a given ID (e.g. all states for an entity type).
    pub fn get_textures(&self, id: &str) -> Option<Rc<HashMap<String, Rc<Texture2D>>>> {
        self.holder.get(id).cloned()
    }

    /// Get a single texture for a given ID and state.
    pub fn get_texture(&self, id: &str, state: &str) -> Option<Rc<Texture2D>> {
        self.holder
            .get(id)
            .and_then(|states| states.get(state).cloned())
    }
}
