use macroquad::color::WHITE;
use macroquad::math::Vec2;
use macroquad::texture::{self, DrawTextureParams};

use crate::animations::animations::Directions;
use crate::ecs::components::{Entity, RenderLayer};
use crate::ecs::world::World;

impl World {
    /// System: handle player input and directly move the player.
    pub fn system_player_input(&mut self) {
        use macroquad::input::{self, KeyCode};

        let entities: Vec<Entity> = self.player_inputs.iter().copied().collect();
        for e in entities {
            let pos_opt = self.positions.get_mut(&e);
            let sprite_opt = self.sprites.get_mut(&e);
            if pos_opt.is_none() || sprite_opt.is_none() {
                continue;
            }
            let pos = pos_opt.unwrap();
            let sprite = sprite_opt.unwrap();

            let mut speed = Vec2::new(0.0, 0.0);
            let keys = input::get_keys_down();
            for key in keys.iter() {
                match key {
                    KeyCode::A => {
                        if let Some(anim) = sprite.animation.as_mut() {
                            anim.set_row_frame(Directions::Left);
                        }
                        speed.x -= 5.0;
                    }
                    KeyCode::D => {
                        if let Some(anim) = sprite.animation.as_mut() {
                            anim.set_row_frame(Directions::Right);
                        }
                        speed.x += 5.0;
                    }
                    KeyCode::W => {
                        if let Some(anim) = sprite.animation.as_mut() {
                            anim.set_row_frame(Directions::Up);
                        }
                        speed.y -= 5.0;
                    }
                    KeyCode::S => {
                        if let Some(anim) = sprite.animation.as_mut() {
                            anim.set_row_frame(Directions::Down);
                        }
                        speed.y += 5.0;
                    }
                    _ => {}
                }
            }

            pos.0 += speed;
        }
    }

    /// System: simple AI that chases the target entity.
    pub fn system_chase_ai(&mut self) {
        use std::cmp::Ordering;

        let ai_entries: Vec<(Entity, Entity)> = self
            .chase_ais
            .iter()
            .map(|(e, ai)| (*e, ai.target))
            .collect();

        for (e, target) in ai_entries {
            let target_pos_opt = self.positions.get(&target).cloned();
            if target_pos_opt.is_none() {
                continue;
            }
            let target_pos = target_pos_opt.unwrap().0;

            let pos_opt = self.positions.get_mut(&e);
            let sprite_opt = self.sprites.get_mut(&e);
            if pos_opt.is_none() || sprite_opt.is_none() {
                continue;
            }
            let pos = pos_opt.unwrap();
            let sprite = sprite_opt.unwrap();

            let mut speed_x = 0.0;
            let mut speed_y = 0.0;

            match target_pos.x.total_cmp(&pos.0.x) {
                Ordering::Less => {
                    if let Some(anim) = sprite.animation.as_mut() {
                        anim.set_row_frame(Directions::Left);
                    }
                    speed_x = -1.0;
                }
                Ordering::Greater => {
                    if let Some(anim) = sprite.animation.as_mut() {
                        anim.set_row_frame(Directions::Right);
                    }
                    speed_x = 1.0;
                }
                Ordering::Equal => {}
            }

            match target_pos.y.total_cmp(&pos.0.y) {
                Ordering::Less => {
                    if let Some(anim) = sprite.animation.as_mut() {
                        anim.set_row_frame(Directions::Up);
                    }
                    speed_y = -1.0;
                }
                Ordering::Greater => {
                    if let Some(anim) = sprite.animation.as_mut() {
                        anim.set_row_frame(Directions::Down);
                    }
                    speed_y = 1.0;
                }
                Ordering::Equal => {}
            }

            pos.0.x += speed_x;
            pos.0.y += speed_y;
        }
    }

    /// System: advance animations.
    pub fn system_animate(&mut self, dt: f32) {
        for sprite in self.sprites.values_mut() {
            if let Some(anim) = sprite.animation.as_mut() {
                anim.update(dt);
            }
        }
    }

    /// System: render everything in layer order for a top-down view.
    pub fn system_render(&self) {
        use RenderLayer::*;

        let layers = [Surface, SurfaceObject, Entity];

        for &layer in &layers {
            for (entity, sprite) in self.sprites.iter() {
                let _ = entity; // currently unused; keep for future logic (e.g., debug, selection)
                if sprite.layer != layer {
                    continue;
                }
                if let Some(pos) = self.positions.get(entity) {
                    let mut params = DrawTextureParams {
                        ..Default::default()
                    };
                    if let Some(anim) = sprite.animation.as_ref() {
                        params.source = Some(*anim.get_draw_rect());
                    }
                    texture::draw_texture_ex(
                        &sprite.texture,
                        pos.0.x,
                        pos.0.y,
                        WHITE,
                        params,
                    );
                }
            }
        }
    }
}
