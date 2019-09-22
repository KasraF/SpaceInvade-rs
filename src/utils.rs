#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Dir {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Tile {
    Invader(usize),
    Player,
    Missile(usize),
    Explosion,
    None,
}

#[derive(Copy, Clone)]
pub struct Coord (pub usize, pub usize);

impl Coord {
    pub fn area(&self) -> usize {
        self.0 * self.1
    }
}

pub fn looped_inc<T: num::Integer + num::Unsigned + num::Bounded + Copy>(num: &mut T) {
    *num = (*num + T::one()) % (T::max_value() - T::one());
}

pub fn capped_inc<T: num::Integer + num::Unsigned + num::Bounded + Copy>(num: &mut T) {
    *num = std::cmp::min(*num + T::one(), T::max_value() - T::one());
}
