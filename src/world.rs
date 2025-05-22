use std::{collections::{HashMap, HashSet}, ops::AddAssign, vec};

use rand::Rng;

use crate::{components::{AggressionIntent, Position, Strength, HP}, game::TurnState, map::Map, systems::{AggressionSystem, DamageSystem, DeathSystem, InputSystem}};

pub type Entity = usize;

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct ArchetypeKey {
    pub has_position: bool,
    pub is_controllable: bool,
    pub is_enemy: bool,
    pub has_hp: bool,
    pub has_strength: bool    
}

pub struct Table {
    pub key: ArchetypeKey,
    pub entities: Vec<Entity>,
    pub positions: Vec<Position>,
    pub hitpoints: Vec<HP>,
    pub aggression_intents: Vec<Option<AggressionIntent>>,
    pub strengths: Vec<Strength>
}

pub struct World {
    next_entity: Entity,
    pub map: Map,
    pub tables: HashMap<ArchetypeKey, Table>,
    pub turn_state: TurnState
}
impl World {
    pub fn new(map: Map) -> Self {
        let mut rng = rand::rng();
        let turn_state = if rng.random_bool(0.5) { TurnState::Enemy } else { TurnState::Player };
        Self { next_entity: 0, map, tables: HashMap::new(), turn_state }
    }

    fn get_next_entity(&mut self) -> Entity {
        self.next_entity += 1;
        self.next_entity - 1
    }

    pub fn update(&mut self) {
        if InputSystem::run(self) {
            AggressionSystem::run(self);
            DamageSystem::run(self);
            DeathSystem::run(self);
        }                       
    }

    pub fn initialize(&mut self) {
        self.spawn_player();
        self.spawn_enemy();
    }

    pub fn spawn_player(&mut self) -> Entity {
        let key = ArchetypeKey { 
            is_controllable: true, 
            has_position: true,        
            is_enemy: false,
            has_hp: true,
            has_strength: true            
        };
        let id = self.get_next_entity();
        let table = self.tables.entry(key.clone())
            .or_insert_with(|| Table {
                key,
                entities: vec![],
                positions: vec![],
                hitpoints: vec![],
                aggression_intents: vec![],
                strengths: vec![]
            });            
        let position = self.map.get_tiles()
            .iter()
            .enumerate()
            .find_map(|(idx, ch)| {
                if *ch == '.' {
                    let (y, x) = self.map.idx_xy(idx);
                    Some(Position::new(x, y))
                } else {
                    None
                }
            })
            .expect("Should already exist a carved room");            
        let mut rng = rand::rng();
        let hp = HP(rng.random_range(0..10));
        table.entities.push(id);
        table.positions.push(position);
        table.hitpoints.push(hp);
        table.aggression_intents.push(None);
        table.strengths.push(Strength(rng.random_range(1..6)));
        id
    }

    pub fn spawn_enemy(&mut self) -> Entity {
        let key = ArchetypeKey {
            has_position: true,
            is_controllable: false,
            is_enemy: true,
            has_hp: true,
            has_strength: true            
        };
        let id = self.get_next_entity();
        let table = self.tables.entry(key.clone())
            .or_insert_with(|| Table { 
                key, 
                entities: vec![], 
                positions: vec![],
                hitpoints: vec![],
                aggression_intents: vec![],
                strengths: vec![]
            });
        let position = self.map.get_tiles()
            .iter()
            .enumerate()
            .find_map(|(idx, ch)| {
                if *ch == '.' {
                    let (y, x) = self.map.idx_xy(idx);
                    Some(Position::new(x + 2, y + 2))
                } else {
                    None
                }
            })
            .expect("Should already exist a carved room");  
        let mut rng = rand::rng();
        let hp = HP(rng.random_range(0..6));
        table.entities.push(id);
        table.positions.push(position);
        table.hitpoints.push(hp);
        table.aggression_intents.push(None);
        table.strengths.push(Strength(rng.random_range(1..3)));
        id        
    }
}