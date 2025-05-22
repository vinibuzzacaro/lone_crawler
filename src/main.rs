mod map;
mod world;
mod components;
mod systems;
mod game;

use game::Game;

fn main() {
    let mut game = Game::new(80, 30, 4);
    if let Err(e) = game.run() {
        eprintln!("error: {e}");
        std::process::exit(1);
    };
}