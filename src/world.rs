use std::collections::{HashMap, HashSet};

use crate::{components::Position, map::Map, systems::InputSystem};

pub type Entity = usize;

pub struct World {
    next_entity: Entity,
    pub map: Map,
    pub entities: HashSet<Entity>,
    pub controllables: HashSet<Entity>,
    pub positions: HashMap<Entity, Position>
}
impl World {
    pub fn new(map: Map) -> Self {
        Self { next_entity: 0, map, entities: HashSet::new(), controllables: HashSet::new(), positions: HashMap::new() }
    }
    
    pub fn update(&mut self) {
        InputSystem::run(self);
    }

    pub fn initialize(&mut self) {
        self.spawn_player();
    }

    pub fn create_entity(&mut self) -> Entity {
        let e = self.next_entity;
        self.next_entity += 1;
        self.entities.insert(e);
        e
    }

    pub fn spawn_player(&mut self) {
        let e = self.create_entity();
        self.controllables.insert(e);
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
        self.positions.insert(e, position);
    }
}