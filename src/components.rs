use std::collections::HashMap;

pub type Entity = usize;

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct ArchetypeKey {
    pub has_position: bool,
    pub is_controllable: bool,
    pub is_hostile: bool,
    pub has_hp: bool,
    pub has_strength: bool,
    pub has_loot: bool
}

#[derive(PartialEq, Clone, Debug)]
pub struct Position {
    pub x: usize,
    pub y: usize
}
impl Position {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

#[derive(Debug)]
pub struct HP(pub usize);

#[derive(Debug)]
pub struct Strength(pub usize);

#[derive(Debug)]
pub struct AggressionIntent(pub Entity);

#[derive(Debug)]
pub struct Damage(pub usize);

#[derive(Debug)]
pub struct Loot(pub Vec<Entity>);

pub struct EntityTemplate {    
    pub default_hp: Option<HP>,
    pub default_strength: Option<Strength>,
    pub default_loot: Option<Loot>
}

pub enum ArchetypeId {
    Player,
    Enemy,
    Corpse
}
impl ArchetypeId {
    pub fn template(&self) -> Option<EntityTemplate> {
        match self {
            ArchetypeId::Player => Some(EntityTemplate {
                        default_hp: Some(HP(10)),
                        default_strength: Some(Strength(2)),
                        default_loot: None
                    }),
            ArchetypeId::Enemy => Some(EntityTemplate {
                        default_hp: Some(HP(6)),
                        default_strength: Some(Strength(1)),
                        default_loot: None
                    }),
            ArchetypeId::Corpse => Some(EntityTemplate {
                default_hp: None,
                default_strength: None,
                default_loot: Some(Loot(vec![0])),
            }),
                    }
    }

    pub fn key(&self) -> ArchetypeKey {
        match self {
            ArchetypeId::Player => ArchetypeKey {
                has_position: true,
                is_controllable: true,
                is_hostile: false,
                has_hp: true,
                has_strength: true,
                has_loot: false,
            },
            ArchetypeId::Enemy => ArchetypeKey {
                has_position: true,
                is_controllable: false,
                is_hostile: true,
                has_hp: true,
                has_strength: true,
                has_loot: false,
            },
            ArchetypeId::Corpse => ArchetypeKey {
                has_position: true,
                is_controllable: false,
                is_hostile: false,
                has_hp: false,
                has_strength: false,
                has_loot: true,
            },
        }
    }
}
