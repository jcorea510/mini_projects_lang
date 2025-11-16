use std::rc::Rc;
use std::fmt;
use macroquad::texture::{Texture2D, draw_texture};
use macroquad::color::WHITE;
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
        texture_holder
            .add_texture(
                BackgroundID::Jungle.to_string(),
                "Media/Textures/Jungle.png",
            )
            .await;
        let texture_path = texture_holder.get_texture(&BackgroundID::Jungle.to_string());
        Self {
            sprite: texture_path.clone(),
        }
    }
    pub fn draw(&self) {
        draw_texture(&self.sprite.clone(), 0.0, 0.0, WHITE);
    }
}
