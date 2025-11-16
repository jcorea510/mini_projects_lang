use std::default::Default;
use std::collections::HashMap;
use macroquad::texture::{Texture2D, load_texture};
use std::rc::Rc;

#[derive(Default)]
pub struct TextureManager {
    holder: HashMap<String, Rc<Texture2D>>
}

impl TextureManager {
    pub async fn add_texture(&mut self, id: String, path: &str) {
        if let std::collections::hash_map::Entry::Vacant(e) = self.holder.entry(id) {
               let texture = load_texture(path).await.unwrap();
               e.insert(Rc::new(texture));
        }
    }

    pub fn get_texture(&self, id: &String) -> &Rc<Texture2D> {
        self.holder.get(id).unwrap()
    }
}
