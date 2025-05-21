use std::{cmp::{max, min}, fmt::Display, io::{stdout, Stdout}};

use crossterm::{cursor::MoveTo, style::Print, QueueableCommand};
use rand::Rng;

#[derive(Debug, Clone)]
pub struct Rect {
    x: usize,
    y: usize,
    width: usize,
    height: usize
}
impl Display for Rect {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut stdout = stdout();
        for w in 0..self.width {
            draw_char(&mut stdout, self.x + w, self.y, '#')?;
            draw_char(&mut stdout, self.x + w, self.y + self.height, '#')?;
        }
        for h in 0..self.height {
            draw_char(&mut stdout, self.x, self.y + h, '#')?;
            draw_char(&mut stdout, self.x + self.width, self.y + h, '#')?;
        }
        Ok(())
    }
}
impl Rect {
    pub fn new(x: usize, y: usize, width: usize, height: usize) -> Self {
        Self { x, y, width, height }
    }

    pub fn center(&self) -> (usize, usize) {
        (self.x + self.width / 2, self.y + self.height / 2)
    }

    pub fn intersects(&self, other: &Self) -> bool {
        self.x <= other.x + other.width
            && self.x + self.width >= other.x
            && self.y <= other.y + other.height
            && self.y + self.height >= other.y
    }
}

#[derive(Debug)]
pub struct BSPNode {
    rect: Rect,
    left: Option<Box<BSPNode>>,
    right: Option<Box<BSPNode>>,
    room: Option<Rect>
}
impl BSPNode {    
    const MINIMUM_HEIGHT: usize = 5;
    const MINIMUM_WIDTH: usize = 5;
    
    pub fn root(map: &Map) -> Self {
        Self { rect: Rect::new(0, 0, map.columns(), map.rows()), left: None, right: None, room: None }
    }

    fn new(rect: Rect) -> Self {
        Self { rect: rect, left: None, right: None, room: None }
    }

    pub fn is_leaf(&self) -> bool {
        self.left.is_none() && self.right.is_none()
    }

    fn split(&mut self) -> bool {
        let mut rng = rand::rng();     
        if self.left.is_some() || self.right.is_some() {
            return false;
        }         
        let is_horizontal = match self.rect.width.cmp(&self.rect.height) {
            std::cmp::Ordering::Less => false,
            std::cmp::Ordering::Equal => rng.random_bool(0.5),
            std::cmp::Ordering::Greater => true,
        };
        let (left, right) = if is_horizontal {
            if self.rect.width < Self::MINIMUM_WIDTH * 2 {
                return false;
            }            
            let min_range = self.rect.x + Self::MINIMUM_WIDTH;
            let max_range = self.rect.x + self.rect.width - Self::MINIMUM_WIDTH;
            if min_range == max_range {
                return false;
            }
            let split_at = rng.random_range(min_range..max_range);
            let left = Rect::new(
                self.rect.x, 
                self.rect.y, 
                split_at - self.rect.x, 
                self.rect.height
            );
            let right = Rect::new(
                split_at, 
                self.rect.y, 
                self.rect.x + self.rect.width - split_at, 
                self.rect.height
            );
            (left, right)
        } else {
            if self.rect.height < Self::MINIMUM_HEIGHT * 2 {
                return false;
            }
            let min_range = self.rect.y + Self::MINIMUM_HEIGHT;
            let max_range = self.rect.y + self.rect.height - Self::MINIMUM_HEIGHT;
            if min_range == max_range {
                return false;
            }
            let split_at = rng.random_range(min_range..max_range);
            let left = Rect::new(
                self.rect.x, 
                self.rect.y, 
                self.rect.width, 
                split_at - self.rect.y
            );
            let right = Rect::new(
                self.rect.x, 
                split_at, 
                self.rect.width, 
                self.rect.y + self.rect.height - split_at
            );
            (left, right)
        };
        let left_node = Box::new(BSPNode::new(left));
        let right_node = Box::new(BSPNode::new(right));
        self.left = Some(left_node);
        self.right = Some(right_node);        
        true
    }

    pub fn split_recursively(&mut self, depth: isize) {
        if depth <= 0 {
            return;
        }
        if self.split() {
            if let Some(left) = &mut self.left {
                left.split_recursively(depth - 1);
            }
            if let Some(right) = &mut self.right {
                right.split_recursively(depth - 1);
            }
        }
    }

    fn carve_room(&mut self, carved_rooms: &[Rect]) -> bool {
        let mut rng = rand::rng();
        if !self.is_leaf() {
            return false;
        }
        let margin_x = rng.random_range(1..Self::MINIMUM_WIDTH / 2);
        let margin_y = rng.random_range(1..Self::MINIMUM_HEIGHT / 2);                
        let rect = Rect::new(
            self.rect.x + margin_x,
            self.rect.y + margin_y, 
            self.rect.width - 2 * margin_x,
            self.rect.height - 2 * margin_y
        );
        if carved_rooms.iter().any(|r| r.intersects(&rect)) {
            return false;
        }
        self.room = Some(rect);
        true
    }

    pub fn carve_all_rooms(&mut self) -> Vec<Rect> {
        let mut carved_rooms: Vec<Rect> = Vec::new();                
        self.traverse_pre_order_mut(&mut |node| {            
            if node.carve_room(&carved_rooms) {
                if let Some(room) = &node.room {
                    carved_rooms.push(room.clone());
                }                
            }            
        });
        carved_rooms
    }

    fn create_corridor(room1: &Rect, room2: &Rect, map: &mut Map) {
        let ((x1, y1), (x2, y2)) = (room1.center(), room2.center());
        for x in min(x1, x2)..=max(x1, x2) {
            map.set_tile(x, y1, '.');
        }
        for y in min(y1, y2)..=max(y1, y2) {
            map.set_tile(x2, y, '.');
        }
    }

    pub fn create_all_corridors(&mut self, map: &mut Map) {        
        if let (Some(left), Some(right)) = (&mut self.left, &mut self.right) {
            if let (Some(l), Some(r)) = (&left.room, &right.room) {
                Self::create_corridor(l, r, map);
            }
            left.create_all_corridors(map);
            right.create_all_corridors(map);            
        }
    }

    pub fn collect_rooms(&self, rooms: &mut Vec<Rect>) {
        if let Some(room) = &self.room {
            rooms.push(room.clone());
        }
        if let Some(left) = &self.left {
            left.collect_rooms(rooms);
        }
        if let Some(right) = &self.right {
            right.collect_rooms(rooms);
        }
    }

    fn connect_rooms_in_sequence(&self, map: &mut Map) {
        let mut rooms: Vec<Rect> = Vec::new();
        self.collect_rooms(&mut rooms);
        for i in 1..rooms.len() {
            Self::create_corridor(&rooms[i - 1], &rooms[i], map);
        }
    }

    pub fn traverse_pre_order<F: FnMut(&Self)>(&self, visit: &mut F) {
        visit(self);
        if let Some(left) = &self.left {
            left.traverse_pre_order(visit);
        }
        if let Some(right) = &self.right {
            right.traverse_pre_order(visit);
        }
    }

    pub fn traverse_pre_order_mut<F: FnMut(&mut Self)>(&mut self, visit: &mut F) {
        visit(self);
        if let Some(left) = &mut self.left {
            left.traverse_pre_order_mut(visit);
        }
        if let Some(right) = &mut self.right {
            right.traverse_pre_order_mut(visit);
        }
    }

    pub fn create_dungeon(map: &mut Map, depth: isize) {
        let mut root = Self::root(map);
        root.split_recursively(depth);        
        let carved_rooms = root.carve_all_rooms();        
        for room in carved_rooms {
            for y in (room.y + 1)..(room.y + room.height) {
                for x in (room.x + 1)..(room.x + room.width) {
                    map.set_tile(x, y, '.');
                }
            }
        }
        root.create_all_corridors(map);
        root.connect_rooms_in_sequence(map);
    }
}

#[derive(Debug)]
pub struct Map {
    tiles: Vec<char>,
    stride: usize
}
impl Map {    
    pub fn new(width: usize, height: usize) -> Self {
        Self { 
            tiles: vec!['#'; width * height], 
            stride: width 
        }
    }

    pub fn get_tile(&self, idx: usize) -> Option<char> {
        self.tiles.get(idx).cloned()
    }    

    pub fn set_tile(&mut self, x: usize, y: usize, tile: char) {
        let idx = self.xy_idx(x, y);
        if let Some(t) = self.tiles.get_mut(idx) {
            *t = tile;
        };
    }

    pub fn xy_idx(&self, x: usize, y: usize) -> usize {
        y * self.stride + x
    }

    pub fn idx_xy(&self, idx: usize) -> (usize, usize) {
        let row = idx / self.columns();
        let column = idx % self.columns();
        (row, column)
    }

    pub fn rows(&self) -> usize {
        self.tiles.len() / self.stride
    }

    pub fn columns(&self) -> usize {
        self.stride
    }

    pub fn is_walkable(&self, x: usize, y: usize) -> bool {
        let idx = self.xy_idx(x, y);
        dbg!(self.get_tile(idx));
        if let Some(ch) = self.get_tile(idx) {
            ch == '.'
        } else {
            false
        }
    }

    pub fn get_tiles(&self) -> &[char] {
        &self.tiles
    }
}

pub fn draw_char(stdout: &mut Stdout, x: usize, y: usize, symbol: char) -> std::fmt::Result {
    stdout
        .queue(MoveTo(x as u16, y as u16))
            .map_err(|_| std::fmt::Error)?
        .queue(Print(symbol))
            .map_err(|_| std::fmt::Error)?;
    Ok(())       
}