extern crate rustygame;
use crate::rustygame::game::Game;

#[macroquad::main("BasicShapes")]
async fn main() {
    let mut game = Game::new().await;
    game.run().await
}
