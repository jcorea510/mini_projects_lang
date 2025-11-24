use macroquad::color::WHITE;
use macroquad::math::Vec2;
use macroquad::texture::{self, DrawTextureParams};

use crate::animations::animations::Directions;
use crate::ecs::components::{Entity, RenderLayer, State};
use crate::ecs::world::World;
use crate::commands::{Action, Commands};

impl World {
    /// Helper: move all player-controlled entities by the given delta and set their facing direction.
    pub fn move_players(&mut self, delta: Vec2, direction: Directions) {
        // Snapshot player entities so we can safely iterate while mutating world data.
        let entities: Vec<Entity> = self.player_inputs.iter().copied().collect();

        for e in entities {
            // --- 1) Decide if this player is allowed to move by `delta` ---
            let can_move = {
                // Get current collision region of the player (as a circle).
                if let Some(player_region) = self.get_collision_region(&e) {
                    // Predict where the collision circle center would be after moving.
                    let new_x = player_region.x + delta.x;
                    let new_y = player_region.y + delta.y;
                    let r = player_region.r;

                    // Check against all chaser slimes.
                    let mut blocked = false;
                    for enemy in self.chase_ais.keys() {
                        if let Some(enemy_region) = self.get_collision_region(enemy) {
                            let dx = new_x - enemy_region.x;
                            let dy = new_y - enemy_region.y;
                            let dist_sq = dx * dx + dy * dy;
                            let rad_sum = r + enemy_region.r;

                            // Circle–circle collision test at the *future* position.
                            if dist_sq < rad_sum * rad_sum {
                                blocked = true;
                                break;
                            }
                        }
                    }

                    !blocked
                } else {
                    // If we can't compute a collision region (no sprite/animation), just allow movement.
                    true
                }
            };

            // --- 2) Apply animation + movement if allowed ---
            let pos_opt = self.positions.get_mut(&e);
            let sprite_opt = self.sprites.get_mut(&e);
            if let (Some(pos), Some(sprite)) = (pos_opt, sprite_opt) {
                if let Some(anim) = sprite.animations.get_mut(&sprite.state) {
                    anim.set_row_frame(direction);
                }

                // Only move if the future position is not colliding with any enemy.
                if can_move {
                    pos.0 += delta;
                    if delta.x != 0.0 || delta.y != 0.0 {
                        // Mark player as walking while there is movement input.
                        self.set_entity_state(&e, State::Walk);
                    }
                }
            }
        }
    }

    /// System: read player input and translate it into queued commands.
    pub fn system_player_input(&mut self, commands: &mut Commands) {
        use macroquad::input::{self, KeyCode};

        let keys = input::get_keys_down();
        for key in keys.iter() {
            match key {
                KeyCode::A => commands.enqueue(Action::MoveLeft),
                KeyCode::D => commands.enqueue(Action::MoveRight),
                KeyCode::W => commands.enqueue(Action::MoveUp),
                KeyCode::S => commands.enqueue(Action::MoveDown),
                KeyCode::Enter => commands.enqueue(Action::Attack),
                _ => {}
            }
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
            let is_there_collition = self.entity_collision(&e, &target);
            if is_there_collition {
                self.set_entity_state(&e, State::Attack);
            } else {
                // If no collision, let Walk/Idle be driven by movement + state machine.
            }

            let target_pos_opt = self.positions.get(&target).cloned();
            if target_pos_opt.is_none() {
                continue;
            }
            let target_pos = target_pos_opt.unwrap().0;
            
            let pos_opt = self.positions.get_mut(&e);
            let sprite_opt = self.sprites.get_mut(&e);

            let mut speed_x = 0.0;
            let mut speed_y = 0.0;
            if let (Some(pos), Some(sprite)) = (pos_opt, sprite_opt) {
                match target_pos.x.total_cmp(&pos.0.x) {
                    Ordering::Less => {
                        if let Some(anim) = sprite.animations.get_mut(&sprite.state) {
                            anim.set_row_frame(Directions::Left);
                        }
                        if !is_there_collition {
                            speed_x = -1.0;
                        }
                    }
                    Ordering::Greater => {
                        if let Some(anim) = sprite.animations.get_mut(&sprite.state) {
                            anim.set_row_frame(Directions::Right);
                        }
                        if !is_there_collition {
                            speed_x = 1.0;
                        }
                    }
                    Ordering::Equal => {}
                }

                match target_pos.y.total_cmp(&pos.0.y) {
                    Ordering::Less => {
                        if let Some(anim) = sprite.animations.get_mut(&sprite.state) {
                            anim.set_row_frame(Directions::Up);
                        }
                        if !is_there_collition {
                            speed_y = -1.0;
                        }
                    }
                    Ordering::Greater => {
                        if let Some(anim) = sprite.animations.get_mut(&sprite.state) {
                            anim.set_row_frame(Directions::Down);
                        }
                        if !is_there_collition {
                            speed_y = 1.0;
                        }
                    }
                    Ordering::Equal => {}
                } 
                pos.0.x += speed_x;
                pos.0.y += speed_y;
            }
            if speed_x != 0.0 || speed_y != 0.0 {
                self.set_entity_state(&e, State::Walk);
            }
        }
    }

    /// System: advance animations.
    pub fn system_animate(&mut self, dt: f32) {
        for sprite in self.sprites.values_mut() {
            if let Some(anim) = sprite.animations.get_mut(&sprite.state) {
                anim.update(dt);
            }
        }
    }

    /// System: update state machines (e.g. auto-return to Idle after a short time).
    pub fn system_state_machine(&mut self, dt: f32) {
        // Clone keys to avoid borrow conflicts while mutating values.
        let entities: Vec<Entity> = self.state_machines.keys().copied().collect();
        for e in entities {
            if let Some(sm) = self.state_machines.get_mut(&e) {
                sm.time_in_state += dt;
                let current = sm.state;

                // Do not auto-return for Idle or Death; only for transient states.
                let should_return = match current {
                    State::Idle | State::Death => false,
                    _ => sm.time_in_state >= sm.time_to_idle,
                };

                if should_return {
                    sm.state = State::Idle;
                    sm.time_in_state = 0.0;
                    self.set_sprite_state(&e, State::Idle);
                }
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
                    if let Some(anim) = sprite.animations.get(&sprite.state) {
                        let rect = anim.get_draw_rect();
                        params.source = Some(*rect);
                        // Treat `pos` as the center of the sprite, so different
                        // frame sizes across states stay visually aligned.
                        let draw_x = pos.0.x - rect.w / 2.0;
                        let draw_y = pos.0.y - rect.h / 2.0;
                        texture::draw_texture_ex(
                            &sprite.texture,
                            draw_x,
                            draw_y,
                            WHITE,
                            params,
                        );
                    } else {
                        // Fallback: no animation, draw with pos as top-left.
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
}
