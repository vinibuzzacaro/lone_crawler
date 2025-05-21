use std::time::Duration;

use crossterm::event::{poll, read, Event, KeyCode};

use crate::{components::Position, world::World};

pub struct InputSystem;
impl InputSystem {
    pub fn run(world: &mut World) {
        if let Ok(true) = poll(Duration::from_secs(0)) {
            match read() {
                Ok(Event::Key(event)) => {                                        
                    match event.code {
                        KeyCode::Esc => std::process::exit(0),
                        KeyCode::Char(c) => {                        
                            let possibilities = ['w', 'W', 'a', 'A', 's', 'S', 'd', 'D'];
                            if !possibilities.contains(&c) {
                                return;
                            }
                            for e in &world.controllables {
                                if let Some(position) = world.positions.get_mut(e) {                                                                        
                                    let new_position = match c {
                                        'w' | 'W' => Position::new(position.x, position.y.saturating_sub(1)),
                                        'a' | 'A' => Position::new(position.x.saturating_sub(1), position.y),
                                        's' | 'S' => Position::new(position.x, position.y.saturating_add(1)),
                                        'd' | 'D' => Position::new(position.x.saturating_add(1), position.y),
                                        _ => return 
                                    };
                                    dbg!(world.map.is_walkable(new_position.x, new_position.y));
                                    if world.map.is_walkable(new_position.x, new_position.y) {
                                        *position = new_position;
                                    }
                                }
                            }                            
                        },
                        _ => return
                    }
                },
                Err(e) => eprintln!("error: {e}"),
                _ => return
            }
        }
    }
}

pub struct RenderSystem;
impl RenderSystem {
    pub fn render(world: &World) {
        let player_position = world.controllables
            .iter()
            .find_map(|e| world.positions.get(e))
            .expect("Player should already be spawned");
        for y in 0..world.map.rows() {
            for x in 0..world.map.columns() {
                let ch_to_print = if player_position.x == x && player_position.y == y {
                    '@'                    
                } else {
                    let idx = world.map.xy_idx(x, y);
                    if let Some(ch) = world.map.get_tile(idx) {
                        ch
                    } else {
                        continue;
                    }        
                };
                print!("{ch_to_print}")
            }
            println!();
        } 
    }
}