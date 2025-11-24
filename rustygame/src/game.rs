use std::collections::HashMap;
use std::rc::Rc;

use macroquad::prelude::*;

use crate::animations::animations::Directions;
use crate::commands::{Action, Commands};
use crate::ecs::World;
use crate::ecs::components::State;
use crate::entities::identifier::EntityID;
use crate::resources::textures::TextureManager;
use crate::scenegrap::background::Background;

const PLAYER_SPEED: f32 = 5.0;

pub struct Game {
    background: Background,
    world: World,
    _texture_holder: TextureManager,
    commands: Commands,
}

impl Game {
    pub async fn new() -> Self {
        let mut texture_holder = TextureManager::default();

        // Set player textures according to state.
        let mut player_textures = HashMap::new();
        player_textures.insert(
            "Idle".to_string(),
            Rc::new(image_loader("Media/Textures/Player/Sword_Idle/Sword_Idle_full.png").await),
        );
        player_textures.insert(
            "Walk".to_string(),
            Rc::new(image_loader("Media/Textures/Player/Sword_Walk/Sword_Walk_full.png").await),
        );
        player_textures.insert(
            "Run".to_string(),
            Rc::new(image_loader("Media/Textures/Player/Sword_Run/Sword_Run_full.png").await),
        );
        player_textures.insert(
            "Attack".to_string(),
            Rc::new(image_loader("Media/Textures/Player/Sword_Attack/Sword_attack_full.png").await),
        );
        player_textures.insert(
            "Death".to_string(),
            Rc::new(image_loader("Media/Textures/Player/Sword_Death/Sword_Death_full.png").await),
        );
        
        // Set Slime1 textures according to state.
        let mut slime_simple_textures = HashMap::new();
        slime_simple_textures.insert(
            "Idle".to_string(),
            Rc::new(image_loader("Media/Textures/Slime1/Idle/Slime1_Idle_full.png").await),
        );
        slime_simple_textures.insert(
            "Walk".to_string(),
            Rc::new(image_loader("Media/Textures/Slime1/Walk/Slime1_Walk_full.png").await),
        );
        slime_simple_textures.insert(
            "Run".to_string(),
            Rc::new(image_loader("Media/Textures/Slime1/Run/Slime1_Run_full.png").await),
        );
        slime_simple_textures.insert(
            "Attack".to_string(),
            Rc::new(image_loader("Media/Textures/Slime1/Attack/Slime1_Attack_full.png").await),
        );
        slime_simple_textures.insert(
            "Death".to_string(),
            Rc::new(image_loader("Media/Textures/Slime1/Death/Slime1_Death_full.png").await),
        );

        // Set Slime2 textures according to state.
        let mut slime_cold_textures = HashMap::new();
        slime_cold_textures.insert(
            "Idle".to_string(),
            Rc::new(image_loader("Media/Textures/Slime2/Idle/Slime2_Idle_full.png").await),
        );
        slime_cold_textures.insert(
            "Walk".to_string(),
            Rc::new(image_loader("Media/Textures/Slime2/Walk/Slime2_Walk_full.png").await),
        );
        slime_cold_textures.insert(
            "Run".to_string(),
            Rc::new(image_loader("Media/Textures/Slime2/Run/Slime2_Run_full.png").await),
        );
        slime_cold_textures.insert(
            "Attack".to_string(),
            Rc::new(image_loader("Media/Textures/Slime2/Attack/Slime2_Attack_full.png").await),
        );
        slime_cold_textures.insert(
            "Death".to_string(),
            Rc::new(image_loader("Media/Textures/Slime2/Death/Slime2_Death_full.png").await),
        );

        // Set Slime3 textures according to state.
        let mut slime_fire_textures = HashMap::new();
        slime_fire_textures.insert(
            "Idle".to_string(),
            Rc::new(image_loader("Media/Textures/Slime3/Idle/Slime3_Idle_full.png").await),
        );
        slime_fire_textures.insert(
            "Walk".to_string(),
            Rc::new(image_loader("Media/Textures/Slime3/Walk/Slime3_Walk_full.png").await),
        );
        slime_fire_textures.insert(
            "Run".to_string(),
            Rc::new(image_loader("Media/Textures/Slime3/Run/Slime3_Run_full.png").await),
        );
        slime_fire_textures.insert(
            "Attack".to_string(),
            Rc::new(image_loader("Media/Textures/Slime3/Attack/Slime3_Attack_full.png").await),
        );
        slime_fire_textures.insert(
            "Death".to_string(),
            Rc::new(image_loader("Media/Textures/Slime3/Death/Slime3_Death_full.png").await),
        );
        
        // load textures per entity type
        texture_holder.add_textures(EntityID::Player.to_string(), player_textures);
        texture_holder.add_textures(EntityID::SlimeSimple.to_string(), slime_simple_textures);
        texture_holder.add_textures(EntityID::SlimeCold.to_string(), slime_cold_textures);
        texture_holder.add_textures(EntityID::SlimeFire.to_string(), slime_fire_textures);

        // Take shared texture maps for each entity.
        let player_textures = texture_holder
            .get_textures(&EntityID::Player.to_string())
            .expect("player textures should be loaded");
        let slime_simple = texture_holder
            .get_textures(&EntityID::SlimeSimple.to_string())
            .expect("slime textures should be loaded");
        let slime_cold_textures = texture_holder
            .get_textures(&EntityID::SlimeCold.to_string())
            .expect("slime textures should be loaded");
        let slime_fire_textures = texture_holder
            .get_textures(&EntityID::SlimeFire.to_string())
            .expect("slime textures should be loaded");

        // Build ECS world and spawn entities.
        let mut world = World::new(String::from("Chaserslimes"));
        let player_entity = world.spawn_player(player_textures.clone());

        // Three different simple chaser slimes
        let _slime1 = world.spawn_chaser_slime(slime_simple.clone(), player_entity);
        let _slime2 = world.spawn_chaser_slime_cold(slime_cold_textures.clone(), player_entity);
        let _slime3 = world.spawn_chaser_slime_fire(slime_fire_textures.clone(), player_entity);
        let _slime4 = world.spawn_chaser_slime(slime_simple.clone(), player_entity);
        let _slime5 = world.spawn_chaser_slime_cold(slime_cold_textures.clone(), player_entity);
        let _slime6 = world.spawn_chaser_slime_fire(slime_fire_textures.clone(), player_entity);
        let _slime7 = world.spawn_chaser_slime(slime_simple.clone(), player_entity);
        let _slime8 = world.spawn_chaser_slime_cold(slime_cold_textures.clone(), player_entity);
        let _slime9 = world.spawn_chaser_slime_fire(slime_fire_textures.clone(), player_entity);

        // Background (still drawn as a separate layer behind everything).
        let background = Background::new(&mut texture_holder).await;

        // Set up command system and handlers.
        let mut commands = Commands::default();

        // Movement commands operate on the world and use a helper to move players.
        commands.add_command(Action::MoveLeft, |world: &mut World| {
            world.move_players(Vec2::new(-PLAYER_SPEED, 0.0), Directions::Left);
        });
        commands.add_command(Action::MoveRight, |world: &mut World| {
            world.move_players(Vec2::new(PLAYER_SPEED, 0.0), Directions::Right);
        });
        commands.add_command(Action::MoveUp, |world: &mut World| {
            world.move_players(Vec2::new(0.0, -PLAYER_SPEED), Directions::Up);
        });
        commands.add_command(Action::MoveDown, |world: &mut World| {
            world.move_players(Vec2::new(0.0, PLAYER_SPEED), Directions::Down);
        });

        // Attack command: set player(s) into Attack state; state machine will return to Idle.
        commands.add_command(Action::Attack, |world: &mut World| {
            world.set_players_state(State::Attack);
        });

        Self {
            background,
            world,
            _texture_holder: texture_holder,
            commands,
        }
    }

    pub async fn run(&mut self) {
        loop {
            clear_background(BLACK);

            let dt = get_frame_time();

            // Background
            self.background.draw();

            // Input -> command queue -> execute on world.
            self.world.system_player_input(&mut self.commands);
            self.commands.process_queue(&mut self.world);

            // ECS systems: AI, animation, rendering.
            self.world.system_chase_ai();
            self.world.system_animate(dt);
            self.world.system_render();

            // After rendering, update state machines so transient states can return to Idle
            // without cutting animations too early in the same frame.
            self.world.system_state_machine(dt);

            draw_fps();
            next_frame().await
        }
    }
}

async fn image_loader(path: &str) -> Texture2D {
    let texture_opt = load_texture(path).await;
    match texture_opt {
        Ok(texture) => texture,
        Err(_) => {
            let image = Image::gen_image_color(640, 256, WHITE);
            Texture2D::from_image(&image)
        }
    }
}
