use std::{io::{stdout, Write}, time::Duration, usize};

use crossterm::{cursor, event::{poll, read, Event, KeyCode}, execute, style, terminal, QueueableCommand};
use rand::Rng;

use crate::{components::{AggressionIntent, ArchetypeId, ArchetypeKey, Damage, Entity, Position, Strength}, world::{Table, World}};

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
                            let player_key = ArchetypeId::Player.key();
                            if let Some(player_table) = world.tables.get_mut(&player_key) {
                                for pos in &mut player_table.positions {
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
            } else if key.is_hostile {
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
                } else if key.is_hostile {
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
        let mut attackers: Vec<Entity> = vec![]; 
        let player_key = ArchetypeId::Player.key();
        let player_position = match world.tables.get(&player_key) {
            Some(player_table) => match player_table.positions.first() {
                Some(position) => position.clone(),
                None => return,
            },
            None => return,
        };
        for (key, table) in &mut world.tables {
            if !key.is_hostile {
                continue;
            }  
            for (attacker_idx, attacker_position) in table.positions.iter().enumerate() {
                if attacker_position != &player_position {
                    continue;
                }
                if let Some(ai) = table.aggression_intents.get_mut(attacker_idx) {
                    *ai = Some(AggressionIntent(0)); // Attacking the player
                    if let Some(attacker_id) = table.entities.get(attacker_idx) {
                        attackers.push(*attacker_id);
                    }                    
                }                                                        
            }
        }
        for attacker_id in attackers {
            if let Some(player_table) = world.tables.get_mut(&player_key) {
                if let Some(ai) = player_table.aggression_intents.first_mut() {
                    *ai = Some(AggressionIntent(attacker_id));
                }
            }
        }
    }

    pub fn run2(world: &mut World) {
        let player_key = ArchetypeKey {
            has_position: true,
            is_controllable: true,
            is_hostile: false,
            has_hp: true,
            has_strength: true,
            has_loot: false,
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
            if !key.has_position || !key.has_hp || !key.has_strength || !key.is_hostile {
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
        let mut to_damage: Vec<(Entity, Damage)> = vec![];
        let mut idx: usize;
        for (key, table) in &mut world.tables {
            if !key.has_strength { // Only entities with strength can inflict damage
                continue;
            }
            idx = 0;       
            for aggression_intent in &mut table.aggression_intents {
                if let Some(attack) = aggression_intent {
                    let strength = table.strengths.get(idx)
                        .expect("The given table should have strength");                     
                    to_damage.push((attack.0, Self::calculate_damage(strength)));
                    *aggression_intent = None;
                }
                idx += 1;
            } 
        }
        for (key, table) in &mut world.tables {
            if !key.has_hp { // Only entities with HP can receive damage
                continue;
            }
            for (attacked, damage) in &to_damage {
                if let Some(idx) = table.entities.iter().position(|entity| entity == attacked) {
                    let hp = table.hitpoints.get_mut(idx)
                        .expect("The given table should have HP");
                    hp.0 = hp.0.saturating_sub(damage.0);
                }
            }
        }
    }

    fn calculate_damage(str: &Strength) -> Damage {
        let mut rng = rand::rng();
        Damage(
            if rng.random_bool(0.05) {
                0
            } else {
                rng.random_range(1..=str.0)
            }
        )      
    }
}

pub struct DeathSystem;
impl DeathSystem {
    pub fn run(world: &mut World) {
        let mut to_remove: Vec<(ArchetypeKey, Entity)> = vec![];
        for (key, hp_table) in &world.tables {
            if !key.has_hp {
                continue;
            }            
            for (idx, hp) in hp_table.hitpoints.iter().enumerate() {
                if hp.0 == 0 {
                    to_remove.push((key.clone(), idx));
                }                
            }
        }
        if to_remove.is_empty() {
            return;
        }
        for (key, idx) in to_remove.iter().rev() {
            if let Some(hp_table) = world.tables.get_mut(key) {
                
                hp_table.hitpoints.remove(*idx);
                if key.has_strength {
                    hp_table.strengths.remove(*idx);
                }       
                hp_table.aggression_intents.remove(*idx);         
            }
        }
    }
}

type MovementCost = usize;
pub struct EnemyAISystem;
impl EnemyAISystem {    
    fn heuristic(position: &Position, destination: &Position) -> usize {
        destination.x.abs_diff(destination.x) + destination.y.abs_diff(destination.y)
    }

    fn add_valids_adjacent_squares(world: &World, current: &Position, destination: &Position, list: &mut Vec<(Position, Option<Position>, MovementCost)>) {
        if world.map.is_walkable(current.x.saturating_sub(1), current.y) {
            list.push((
                Position::new(current.x.saturating_sub(1), current.y),
                Some(current.clone()),
                Self::heuristic(current, destination)
            ));
        }
        if world.map.is_walkable(current.x + 1, current.y) {
            list.push((
                Position::new(current.x + 1, current.y),
                Some(current.clone()),
                Self::heuristic(current, destination)
            ));
        }
        if world.map.is_walkable(current.x, current.y.saturating_sub(1)) {
            list.push((
                Position::new(current.x, current.y.saturating_sub(1)),
                Some(current.clone()),
                Self::heuristic(current, destination)
            ));
        }
        if world.map.is_walkable(current.x, current.y + 1) {
            list.push((
                Position::new(current.x, current.y + 1),
                Some(current.clone()),
                Self::heuristic(current, destination)
            ));
        }
    }


    pub fn run(world: &mut World) {
        let player_key = ArchetypeId::Player.key();
        let player_pos = match world.tables.get(&player_key) {
            Some(table) => table.positions
                .first()
                .expect("Player should already exist")
                .clone(),
            None => return,
        };
        let enemy_key = ArchetypeId::Enemy.key();
        let enemy_table = match world.tables.get_mut(&enemy_key) {
            Some(table) => table,
            None => return,
        };        
        let mut open_list: Vec<(Position, Option<Position>, MovementCost)> = vec![];  
        let mut closed_list: Vec<Position>      
        for enemy_pos in &mut enemy_table.positions {            
            closed_list.push(enemy_pos.clone());            
            while !closed_list.contains(&player_pos) {
                let pos = match closed_list.last() {
                    Some(p) => p,
                    None => return,
                };
                Self::add_valids_adjacent_squares(world, pos, &player_pos, &mut open_list);
                let t = open_list
                    .iter()
                    .find_map(|(current, root, mov_cost)| {
                        match root {
                            Some(p) => Self::heuristic(current, p),
                            None => None
                        }
                    })
                let mut less_cost_path = open_list
                    .iter()
                    .filter_map(|(curr, node, mov_cost)| {
                        match node {
                            Some(pos) if pos != curr && world.map.is_walkable(curr.x, curr.y) => {
                                Some((
                                    curr.clone(), 
                                    Some(pos.clone()), 
                                    Self::heuristic(curr, &player_pos)
                                ))
                            },
                            _ => None,
                        }
                    })
                    .collect::<Vec<(Position, Option<Position>, MovementCost)>>();
                open_list.append(&mut less_cost_path); 
            }
            break;
        }
        dbg!(closed_list);
    }
    pub fn run2(world: &mut World) {
        let player_key = ArchetypeId::Player.key();
        let player_pos = match world.tables.get(&player_key) {
            Some(table) => table.positions
                .first()
                .expect("Player should already exist")
                .clone(),
            None => return,
        };
        let enemy_key = ArchetypeId::Enemy.key();
        let enemy_table = match world.tables.get_mut(&enemy_key) {
            Some(table) => table,
            None => return,
        };        
        let mut already_updated = true;
        for enemy_pos in &mut enemy_table.positions {            
            match enemy_pos.x.cmp(&player_pos.x) {
                std::cmp::Ordering::Less => enemy_pos.x = enemy_pos.x + 1,                
                std::cmp::Ordering::Equal => already_updated = false,
                std::cmp::Ordering::Greater => enemy_pos.x = enemy_pos.x - 1,
            }
            if !already_updated {
                match enemy_pos.y.cmp(&player_pos.y) {
                    std::cmp::Ordering::Less => enemy_pos.y = enemy_pos.y + 1,
                    std::cmp::Ordering::Equal => (),
                    std::cmp::Ordering::Greater => enemy_pos.y = enemy_pos.y - 1,
                }
            }
        }
    }
}