use macroquad::prelude::*;

use crate::ecs::World;
use crate::entities::identifier::EntityID;
use crate::resources::textures::TextureManager;
use crate::scenegrap::background::Background;

pub struct Game {
    background: Background,
    world: World,
    _texture_holder: TextureManager,
}

impl Game {
    pub async fn new() -> Self {
        let mut texture_holder = TextureManager::default();

        // Load textures for entities.
        texture_holder
            .add_texture(
                EntityID::Player.to_string(),
                "Media/Textures/Slime1/Idle/Slime1_Idle_full.png",
            )
            .await;

        texture_holder
            .add_texture(
                EntityID::SlimeSimple.to_string(),
                "Media/Textures/Slime2/Idle/Slime2_Idle_full.png",
            )
            .await;

        let player_texture = texture_holder
            .get_texture(&EntityID::Player.to_string())
            .clone();

        let slime_texture = texture_holder
            .get_texture(&EntityID::SlimeSimple.to_string())
            .clone();

        // Build ECS world and spawn entities.
        let mut world = World::new();
        let player_entity = world.spawn_player(player_texture);

        // Three simple chaser slimes.
        let _slime1 = world.spawn_chaser_slime(slime_texture.clone(), player_entity);
        let _slime2 = world.spawn_chaser_slime(slime_texture.clone(), player_entity);
        let _slime3 = world.spawn_chaser_slime(slime_texture.clone(), player_entity);

        // Background (still drawn as a separate layer behind everything).
        let background = Background::new(&mut texture_holder).await;

        Self {
            background,
            world,
            _texture_holder: texture_holder,
        }
    }

    pub async fn run(&mut self) {
        loop {
            clear_background(BLACK);

            let dt = get_frame_time();

            // Background
            self.background.draw();

            // ECS systems: input, AI, animation, rendering.
            self.world.system_player_input();
            self.world.system_chase_ai();
            self.world.system_animate(dt);
            self.world.system_render();

            draw_fps();
            next_frame().await
        }
    }
}
