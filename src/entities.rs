use crate::game::GameState;
use crate::utils::Coord;
use crate::utils::Dir;

pub trait Entity {
    fn update(&mut self, game_state: &GameState);
    fn position(&self) -> &Coord;
    fn icon(&self) -> char;
}

pub struct Player {
    _health: u8,
    position: Coord,
    missile_timer: u8,
}

pub struct Invader {
    direction: Dir,
    position: Coord,
}

pub struct Missile {
    position: Coord,
    direction: Dir,
}

impl Player {
    pub fn new(position: Coord) -> Self {
        Self {
            _health: 3,
            position,
            missile_timer: 0,
        }
    }
}

impl Entity for Player {
    fn update(&mut self, game_state: &GameState) {
        use crate::game::CtrlEvent;

        self.missile_timer = std::cmp::min(self.missile_timer + 1, 255);

        // Handle user inputs
        for event in game_state.events.iter() {
            match event {
                CtrlEvent::Left => {
                    if self.position.0 > 0 {
                        self.position.0 -= 1
                    }
                },
                CtrlEvent::Right => {
                    if self.position.0 < (game_state.map_dimensions.0 - 1) {
                        self.position.0 += 1
                    }
                },
                CtrlEvent::Shoot => {
                    if self.missile_timer > 5 {
                        self.missile_timer = 0;
                        // TODO Fire!
                    }
                },
            }
        }
    }

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
    fn update(&mut self, game_state: &GameState) {
        match self.direction {
            Dir::Down => {
                if self.position.0 < (game_state.map_dimensions.0 - self.position.0) {
                    // Closer to left edge
                    self.direction = Dir::Right;
                    self.position.0 += 1;
                } else {
                    // Closer to right edge
                    self.direction = Dir::Left;
                    self.position.0 -= 1;
                }
            }
            Dir::Left => {
                if self.position.0 == 0 {
                    self.direction = Dir::Down;
                    self.position.1 += 1;
                } else {
                    self.position.0 -= 1;
                }
            }
            Dir::Right => {
                if self.position.0 == (game_state.map_dimensions.0 - 1) {
                    self.direction = Dir::Down;
                    self.position.1 += 1;
                } else {
                    self.position.0 += 1;
                }
            }
            Dir::Up => {
                // TODO Log error.
                panic!("Invader with Dir::Up direction should not exist.")
            }
        }
    }

    fn position(&self) -> &Coord {
        &self.position
    }

    fn icon(&self) -> char {
        '@'
    }
}

impl Entity for Missile {
    fn update(&mut self, _game_state: &GameState) {
        match self.direction {
            Dir::Up => self.position.1 += 1,
            Dir::Down => self.position.1 -= 1,
            Dir::Left => self.position.0 -= 1,
            Dir::Right => self.position.0 += 1,
        }
    }

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
