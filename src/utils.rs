#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Dir {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Copy, Clone)]
pub struct Coord (pub usize, pub usize);

impl Coord {
    pub fn area(&self) -> usize {
        self.0 * self.1
    }
}
