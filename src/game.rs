use std::{io::stdout, thread, time::{Duration, Instant}};

use crossterm::{cursor, execute, terminal::{disable_raw_mode, enable_raw_mode}};

use crate::{map::{BSPNode, Map}, systems::RenderSystem, world::World};

pub enum TurnState {
    Player,
    Enemy
}

pub struct Game {
    world: World
}
impl Drop for Game {
    fn drop(&mut self) {
        if let Err(e) = disable_raw_mode() {
            eprintln!("error: {e}");
        }
        if let Err(e) = execute!(stdout(), cursor::Hide) {
            eprintln!("error: {e}");
        }
    }
}
impl Game {
    pub fn new(width: usize, height: usize, depth: isize) -> Self {
        let mut map = Map::new(width, height);        
        BSPNode::create_dungeon(&mut map, depth);
        let mut world = World::new(map);
        world.initialize();
        Self { world }
    }

    fn update(&mut self) {
        self.world.update()
    }

    fn render(&self) -> std::io::Result<()> {
        RenderSystem::render(&self.world)
    }

    pub fn run(&mut self) -> std::io::Result<()> {
        const TARGET_FPS: f32 = 8.0;
        let frame_duration = Duration::from_secs_f32(1.0 / TARGET_FPS);
        let mut frame_start: Instant;
        let mut elapsed: Duration;
        enable_raw_mode()?;
        execute!(stdout(), cursor::Hide)?;
        loop {
            frame_start = Instant::now();
            self.render()?;
            self.update();
            elapsed = frame_start.elapsed();
            if elapsed < frame_duration {
                thread::sleep(frame_duration - elapsed);
            }
        }        
    }
}