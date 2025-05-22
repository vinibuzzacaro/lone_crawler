use crate::world::Entity;

#[derive(PartialEq, Clone)]
pub struct Position {
    pub x: usize,
    pub y: usize
}
impl Position {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

pub struct HP(pub usize);

pub struct Strength(pub usize);

pub struct AggressionIntent(pub Entity);

pub struct Damage(pub usize);