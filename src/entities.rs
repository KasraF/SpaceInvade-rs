use crate::utils::Coord;
use crate::utils::Dir;

pub trait Entity {
    fn position(&self) -> &Coord;
    fn icon(&self) -> char;
}

pub struct Player {
    pub position: Coord,
    pub missile_timer: u8,
}

pub struct Invader {
    pub direction: Dir,
    pub position: Coord,
}

pub struct Missile {
    pub position: Coord,
    pub direction: Dir,
}

impl Player {
    pub fn new(position: Coord) -> Self {
        Self {
            position,
            missile_timer: 0,
        }
    }
}

impl Entity for Player {
    fn position(&self) -> &Coord {
        &self.position
    }

    fn icon(&self) -> char {
        '^'
    }
}

impl Invader {
    pub fn new(position: Coord, direction: Dir) -> Self {
        Invader {
            position,
            direction,
        }
    }
}

impl Entity for Invader {
    fn position(&self) -> &Coord {
        &self.position
    }

    fn icon(&self) -> char {
        '@'
    }
}

impl Missile {
    pub fn new(position: Coord, direction: Dir) -> Self {
        Missile {
            position,
            direction,
        }
    }
}

impl Entity for Missile {
    fn position(&self) -> &Coord {
        &self.position
    }

    fn icon(&self) -> char {
        match self.direction {
            Dir::Up => '!',
            Dir::Down => ';',
            _ => '=',
        }
    }
}
