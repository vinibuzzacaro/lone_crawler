use std::{thread, time::{Duration, Instant}};

use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

use crate::{map::{BSPNode, Map}, systems::RenderSystem, world::World};

pub struct Game {
    world: World
}
impl Drop for Game {
    fn drop(&mut self) {
        if let Err(e) = disable_raw_mode() {
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
        self.world.update();
    }

    fn render(&self) {
        RenderSystem::render(&self.world);
    }

    pub fn run(&mut self) {
        const TARGET_FPS: f32 = 8.0;
        let frame_duration = Duration::from_secs_f32(1.0 / TARGET_FPS);
        let mut frame_start: Instant;
        let mut elapsed: Duration;
        /*if let Err(e) = enable_raw_mode() {
            eprintln!("error: {e}");
            return;
        }*/
        loop {
            frame_start = Instant::now();
            self.render();
            self.update();            
            elapsed = frame_start.elapsed();
            if elapsed < frame_duration {
                thread::sleep(frame_duration - elapsed);
            }
        }        
    }
}