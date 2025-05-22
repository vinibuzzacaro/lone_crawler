use std::{io::{stdout, Write}, time::Duration};

use crossterm::{cursor, event::{poll, read, Event, KeyCode}, style, terminal, QueueableCommand};
use rand::Rng;

use crate::{components::{AggressionIntent, Damage, Position}, game::TurnState, world::{ArchetypeKey, Entity, World}};

pub struct InputSystem;
impl InputSystem {
    pub fn run(world: &mut World) -> bool {
        if let Ok(true) = poll(Duration::from_secs(0)) {
            match read() {
                Ok(Event::Key(event)) => {
                    match event.code {
                        KeyCode::Esc => std::process::exit(0),
                        KeyCode::Char(c) => {
                            let possibilities = ['w', 'W', 'a', 'A', 's', 'S', 'd', 'D'];
                            if !possibilities.contains(&c) {
                                return false;
                            }
                            for (key, table) in &mut world.tables {
                                if !key.has_position || !key.is_controllable {
                                    continue;
                                }
                                for pos in &mut table.positions {
                                    let (x, y) = match c {
                                        'w' | 'W' => (pos.x, pos.y - 1),
                                        'a' | 'A' => (pos.x - 1, pos.y),
                                        's' | 'S' => (pos.x, pos.y + 1),
                                        'd' | 'D' => (pos.x + 1, pos.y),
                                        _ => return false
                                    };
                                    if world.map.is_walkable(x, y) {
                                        *pos = Position::new(x, y);
                                        return true
                                    }

                                }
                            }
                        },
                        _ => return false
                    }
                },
                Err(e) =>{
                    eprintln!("error: {e}");
                    std::process::exit(1);
                },
                _ => return false
            }
        }
        false
    }
}

pub struct RenderSystem;
impl RenderSystem {
    fn render_xy(x: usize, y: usize) -> (usize, usize) {
        (x, y + 1)
    }

    pub fn render(world: &World) -> std::io::Result<()> {
        let mut stdout = stdout();
        for (key, table) in &world.tables {
            if !key.has_hp {
                continue;
            }
            if key.is_controllable {
                if let Some(hp) = table.hitpoints.first() {
                    stdout
                        .queue(cursor::MoveTo(0, 0))?
                        .queue(style::Print(format!("hp: {}", hp.0)))?;
                }
            } else if key.is_enemy {
                if let Some(hp) = table.hitpoints.first() {
                    stdout  
                        .queue(cursor::MoveTo(10, 0))?
                        .queue(style::Print(format!("enemy hp: {}", hp.0)))?;
                }
            }
        }
        for x in 0..world.map.columns() {
            for y in 0..world.map.rows() {
                let idx = world.map.xy_idx(usize::from(x), usize::from(y));
                if let Some(ch) = world.map.get_tile(idx) {
                    let (x, y) = Self::render_xy(x, y);
                    stdout
                        .queue(cursor::MoveTo(x as u16, y as u16))?
                        .queue(style::Print(ch))?;                    
                }
            }            
        }
        for (key, table) in &world.tables {
            if !key.has_position {
                continue;
            }
            for pos in &table.positions {
                if key.is_controllable {
                    let (x, y) = Self::render_xy(pos.x, pos.y);
                    stdout
                        .queue(cursor::MoveTo(x as u16, y as u16))?
                        .queue(style::Print('@'))?;
                } else if key.is_enemy {
                    let (x, y) = Self::render_xy(pos.x, pos.y);
                    stdout
                        .queue(cursor::MoveTo(x as u16, y as u16))?
                        .queue(style::Print('g'))?;
                }
            }
            for aggro_intent in &table.aggression_intents {
                if let Some(aggro) = aggro_intent {
                    stdout  
                        .queue(cursor::MoveTo(0, 10))?
                        .queue(style::Print(format!("attacking {}!", aggro.0)))?;
                }
            }
        }        
        stdout.flush()        
    }
}

pub struct AggressionSystem;
impl AggressionSystem {
    pub fn run(world: &mut World) {
        let player_key = ArchetypeKey {
            has_position: true,
            is_controllable: true,
            is_enemy: false,
            has_hp: true,
            has_strength: true,
        };
        let player_position = match world.tables.get(&player_key) {
            Some(table) => match table.positions.first() {
                Some(pos) => pos.clone(),
                None => return,
            },
            None => return,
        };       
        let mut to_aggro: Vec<(Entity, Entity)> = vec![];
        for (key, enemy_table) in &mut world.tables {
            if !key.has_position || !key.has_hp || !key.has_strength || !key.is_enemy {
                continue;                
            }
            for (idx, enemy_position) in enemy_table.positions.iter().enumerate() {
                if &player_position == enemy_position {
                    enemy_table.aggression_intents[idx] = Some(AggressionIntent(0)); // Enemy attacks player 
                    let enemy = enemy_table.entities[idx];
                    to_aggro.push((enemy, 0)); // (Attacker, Defender)
                }                
            }
        }
        if let Some(player_table) = world.tables.get_mut(&player_key) {            
            for (enemy, player) in to_aggro {
                let player_idx = player_table.entities
                    .iter()                    
                    .position(|e| *e == player)
                    .expect("Player should already be spawned");
                player_table.aggression_intents[player_idx] = Some(AggressionIntent(enemy));
            } 
        }
    }
}

pub struct DamageSystem;
impl DamageSystem {
    pub fn run(world: &mut World) {
        let mut rng = rand::rng();
        let mut to_damage: Vec<(Entity, Damage)> = vec![];
        for (key, table) in &mut world.tables {
            if !key.has_strength {
                continue;
            }
            let mut idx = 0;
            for aggression_intent in &mut table.aggression_intents {                                
                let strength = table.strengths
                    .get(idx)
                    .expect("This archetype should have strength");
                let damage = Damage(rng.random_range(0..strength.0));
                if let Some(aggro) = aggression_intent {
                    to_damage.push((aggro.0, damage));
                    *aggression_intent = None;
                }
                idx += 1;
            }            
        }        
        for (entity, damage_received) in to_damage {                
            for (key, table) in &mut world.tables {
                if !key.has_hp {
                    continue;
                }
                if let Some(idx) = table.entities.iter().position(|e| *e == entity) {
                    if let Some(hp) = table.hitpoints.get_mut(idx) {
                        hp.0 = hp.0.saturating_sub(damage_received.0);
                    }
                }
            }
        }            
    }
}

pub struct DeathSystem;
impl DeathSystem {
    pub fn run(world: &mut World) {
        let mut to_remove: Vec<(ArchetypeKey, Entity)> = vec![];
        for (key, table) in &mut world.tables {
            if !key.has_hp {
                continue;
            }            
            for (idx, hp) in table.hitpoints.iter().enumerate() {
                if hp.0 == 0 {
                    to_remove.push((key.clone(), idx));
                }                
            }
        }
        for (key, idx) in to_remove.iter().rev() {
            if let Some(table) = world.tables.get_mut(key) {
                table.hitpoints.remove(*idx);
                if key.has_position {
                    table.positions.remove(*idx);
                }
                if key.has_strength {
                    table.strengths.remove(*idx);
                }       
                table.aggression_intents.remove(*idx);         
            }
        }
    }
}