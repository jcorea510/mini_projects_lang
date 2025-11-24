use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use macroquad::color::WHITE;
use macroquad::texture::{draw_texture, load_texture, Texture2D};

use crate::resources::textures::TextureManager;

#[derive(PartialEq, Eq, Hash)]
pub enum BackgroundID {
    Jungle,
}

impl fmt::Display for BackgroundID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let id = match self {
            BackgroundID::Jungle => "Jungle",
        };
        write!(f, "{id}")
    }
}

pub struct Background {
    sprite: Rc<Texture2D>,
}

impl Background {
    pub async fn new(texture_holder: &mut TextureManager) -> Self {
        // Treat the background as a single-state texture group with state "Default".
        let mut textures = HashMap::new();
        let jungle = load_texture("Media/Textures/Jungle.png").await.unwrap();
        textures.insert("Default".to_string(), Rc::new(jungle));
        texture_holder.add_textures(BackgroundID::Jungle.to_string(), textures);

        let texture_map = texture_holder
            .get_textures(&BackgroundID::Jungle.to_string())
            .expect("background textures should be loaded");
        let sprite = texture_map
            .get("Default")
            .expect("background default texture should exist")
            .clone();

        Self { sprite }
    }

    pub fn draw(&self) {
        draw_texture(&self.sprite.clone(), 0.0, 0.0, WHITE);
    }
}
