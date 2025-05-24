use std::{collections::HashMap, vec};

use rand::{seq::IndexedRandom, Rng};

use crate::{components::{AggressionIntent, ArchetypeId, ArchetypeKey, Entity, Loot, Position, Strength, HP}, game::TurnState, map::Map, systems::{AggressionSystem, DamageSystem, DeathSystem, EnemyAISystem, InputSystem}};

#[derive(Debug)]
pub struct Table {
    pub key: ArchetypeKey,
    pub entities: Vec<Entity>,
    pub positions: Vec<Position>,
    pub hitpoints: Vec<HP>,
    pub aggression_intents: Vec<Option<AggressionIntent>>,
    pub strengths: Vec<Strength>,
    pub loot: Vec<Loot>
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
        //if InputSystem::run(self) {
            //AggressionSystem::run(self);
            //DamageSystem::run(self);
            //DeathSystem::run(self);
            EnemyAISystem::run(self);
        //}                       
    }

    pub fn initialize(&mut self) {
        self.spawn_player();
        self.spawn_enemy();
    }

    pub fn spawn_player(&mut self) -> Entity {
        self.spawn_from_archetype(ArchetypeId::Player)
            .expect("Player template should already exist")
    }

    pub fn spawn_enemy(&mut self) -> Entity {
        self.spawn_from_archetype(ArchetypeId::Enemy)
            .expect("Enemy template should already exist")       
    }

    pub fn spawn_from_archetype(&mut self, archetype_id: ArchetypeId) -> Option<Entity> {
        let template = match archetype_id.template() {
            Some(et) => et,
            None => return None,
        };
        let id = self.get_next_entity();
        let key = archetype_id.key();
        let table = self.tables.entry(key.clone()).or_insert_with(|| {
            Table {
                key: key.clone(),
                entities: vec![],
                positions: vec![],
                hitpoints: vec![],
                aggression_intents: vec![],
                strengths: vec![],
                loot: vec![]
            }
        });
        table.entities.push(id);
        if key.has_hp {
            if let Some(default_hp) = template.default_hp {
                table.hitpoints.push(default_hp);
            }
        }
        if key.has_strength {
            if let Some(default_strength) = template.default_strength {
                table.strengths.push(default_strength);
            }
        }
        if key.has_position {
            let positions: Vec<Position> = self.map.get_tiles()
            .iter()
            .enumerate()
            .filter_map(|(idx, ch)| {
                if *ch == '.' {
                    let (y, x) = self.map.idx_xy(idx);
                    Some(Position::new(x, y))
                } else {
                    None
                }
            })
            .collect();
            let mut rng = rand::rng();
            let position = match positions.choose(&mut rng) {
                Some(pos) => pos,
                None => positions.first().expect("Should exist a carved room already")
            };
            table.positions.push(position.clone());
        }
        if key.has_loot {
            if let Some(default_loot) = template.default_loot {
                table.loot.push(default_loot);
            }
        } 
        table.aggression_intents.push(None);                
        Some(id)
    }    
}