use crate::entities::{Entity, Invader, Missile, Player};
use crate::map::Map;
use crate::utils::{Coord, Dir};
use failure::Error;
use std::clone::Clone;
use std::io::{Stdout, Write};
use std::thread;
use std::time;
use termion::cursor::Goto;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::RawTerminal;

pub enum CtrlEvent {
    Left,
    Right,
    Shoot,
}

pub struct FrameState {
    pub events: Vec<CtrlEvent>,
    pub map_dimensions: Coord,
    pub frame: u8,
}

pub struct GameLoop {
    // Game Logic stuff
    is_running: bool,
    map_size: Coord,

    // Entities
    player: Player,
    invaders: Vec<Invader>,
    missiles: Vec<Missile>,
}

impl FrameState {
    fn new(dimensions: Coord) -> Self {
        Self {
            events: Vec::new(),
            map_dimensions: dimensions,
            frame: 0,
        }
    }
}

impl GameLoop {
    pub fn init() -> Self {
        // TODO move to level file
        let map_size = Coord(45, 15);
        let player_pos = Coord(map_size.0 / 2, map_size.1 - 1);

        let invader1 = Invader::new(Coord(2, 2), Dir::Right);
        let invader2 = Invader::new(Coord(3, 2), Dir::Right);
        let invader3 = Invader::new(Coord(4, 2), Dir::Right);
        let invader4 = Invader::new(Coord(5, 2), Dir::Right);

        let invader5 = Invader::new(Coord(7, 2), Dir::Right);
        let invader6 = Invader::new(Coord(8, 2), Dir::Right);
        let invader7 = Invader::new(Coord(9, 2), Dir::Right);
        let invader8 = Invader::new(Coord(10, 2), Dir::Right);

        let invader9 = Invader::new(Coord(12, 2), Dir::Right);
        let invader10 = Invader::new(Coord(13, 2), Dir::Right);
        let invader11 = Invader::new(Coord(14, 2), Dir::Right);
        let invader12 = Invader::new(Coord(15, 2), Dir::Right);

        let invader13 = Invader::new(Coord(17, 2), Dir::Right);
        let invader14 = Invader::new(Coord(18, 2), Dir::Right);
        let invader15 = Invader::new(Coord(19, 2), Dir::Right);
        let invader16 = Invader::new(Coord(20, 2), Dir::Right);

        let invaders = vec![
            invader1, invader2, invader3, invader4, invader5, invader6, invader7, invader8,
            invader9, invader10, invader11, invader12, invader13, invader14, invader15, invader16,
        ];

        Self {
            map_size,
            invaders,
            is_running: true,
            player: Player::new(player_pos),
            missiles: Vec::new(),
        }
    }

    pub fn frame(
        &mut self,
        input: &mut termion::AsyncReader,
        out: &mut RawTerminal<Stdout>,
        frame: u8,
    ) -> Result<(), Error> {
        let mut frame_state = FrameState::new(self.map_size);

        // Timer!
        let now = time::Instant::now();

        frame_state.frame = frame;
        frame_state.events.clear();

        self.handle_input(input, &mut frame_state.events);
        self.process_entities(&frame_state);
        let map = self.handle_collisions();
        self.draw(out, map)?;

        if self.invaders.is_empty() {
            self.is_running = false;
        }

        // TODO support separate DEBUG mode?
        write!(out, "{}{:?}", Goto(1, 1), now.elapsed())?;
        out.flush().unwrap();

        // Wait
        thread::sleep(time::Duration::from_millis(30) - now.elapsed());

        if self.is_running {
            Ok(())
        } else {
            Err(format_err!("Game over"))
        }
    }

    fn handle_input(&mut self, input: &mut termion::AsyncReader, events: &mut Vec<CtrlEvent>) {
        // TODO Is there a better way to do this?
        use std::io::Error;
        use termion::event::Event;

        let input_events = input.events().collect::<Vec<Result<Event, Error>>>();

        for event in input_events {
            match event {
                Ok(event) => match event {
                    Event::Key(c) => match c {
                        Key::Char('q') => self.is_running = false,
                        Key::Ctrl('c') => self.is_running = false,
                        Key::Left => events.push(CtrlEvent::Left),
                        Key::Right => events.push(CtrlEvent::Right),
                        Key::Char(' ') => events.push(CtrlEvent::Shoot),
                        _ => (),
                    },
                    _ => (),
                },
                Err(e) => error!("Stdin error: {}", e.to_string()),
            }
        }
    }

    fn process_entities(&mut self, frame_state: &FrameState) {
        // TODO This could be done in parallel.
        Self::update_missiles(&mut self.missiles, frame_state);
        let mut missiles = Self::update_invaders(&mut self.invaders, frame_state);
        let missile = Self::update_player(&mut self.player, frame_state);

        if let Some(missile) = missile {
            missiles.push(missile);
        }

        self.missiles.append(&mut missiles);
    }

    fn update_missiles(missiles: &mut Vec<Missile>, frame_state: &FrameState) {
        missiles.drain_filter(|missile: &mut Missile| match missile.direction {
            Dir::Up => {
                if missile.position.1 > 0 {
                    missile.position.1 -= 1;
                    false
                } else {
                    true
                }
            }
            Dir::Down => {
                if missile.position.1 < frame_state.map_dimensions.1 {
                    missile.position.1 += 1;
                    false
                } else {
                    true
                }
            }
            Dir::Left => {
                missile.position.0 -= 1;
                false
            }
            Dir::Right => {
                missile.position.0 += 1;
                false
            }
        });
    }

    fn update_player(player: &mut Player, frame_state: &FrameState) -> Option<Missile> {
        player.missile_timer = std::cmp::min(player.missile_timer + 1, 254);

        let mut request = None;

        // Handle user inputs
        for event in frame_state.events.iter() {
            match event {
                CtrlEvent::Left => {
                    if player.position.0 > 0 {
                        player.position.0 -= 1
                    }
                }
                CtrlEvent::Right => {
                    if player.position.0 < (frame_state.map_dimensions.0 - 1) {
                        player.position.0 += 1
                    }
                }
                CtrlEvent::Shoot => {
                    if player.missile_timer > 5 {
                        player.missile_timer = 0;
                        let mut pos = player.position.clone();
                        pos.1 -= 1;
                        request = Some(Missile::new(pos, Dir::Up))
                    }
                }
            }
        }

        request
    }

    fn update_invaders(invaders: &mut Vec<Invader>, frame_state: &FrameState) -> Vec<Missile> {
        let rv = Vec::with_capacity(0);

        // TODO The invaders should fire back!!!

        if frame_state.frame % 5 == 0 {
            for invader in invaders {
                match invader.direction {
                    Dir::Down => {
                        if invader.position.0 < (frame_state.map_dimensions.0 - invader.position.0)
                        {
                            // Closer to left edge
                            invader.direction = Dir::Right;
                            invader.position.0 += 1;
                        } else {
                            // Closer to right edge
                            invader.direction = Dir::Left;
                            invader.position.0 -= 1;
                        }
                    }
                    Dir::Left => {
                        if invader.position.0 == 0 {
                            invader.direction = Dir::Down;
                            invader.position.1 += 1;
                        } else {
                            invader.position.0 -= 1;
                        }
                    }
                    Dir::Right => {
                        if invader.position.0 == (frame_state.map_dimensions.0 - 1) {
                            invader.direction = Dir::Down;
                            invader.position.1 += 1;
                        } else {
                            invader.position.0 += 1;
                        }
                    }
                    Dir::Up => {
                        // TODO Log error.
                        panic!("Invader with Dir::Up direction should not exist.")
                    }
                }
            }
        }

        rv
    }

    fn handle_collisions(&mut self) -> Map<crate::utils::Tile> {
        use crate::utils::Tile;

        let mut map = Map::<Tile>::new(self.map_size, Tile::None);

        for (index, missile) in self.missiles.iter().enumerate() {
            let pos = missile.position();
            map[pos] = Tile::Missile(index);
        }

        for (index, invader) in self.invaders.iter().enumerate() {
            let pos = invader.position();
            map[pos] = match map[pos] {
                Tile::None => Tile::Invader(index),
                _ => Tile::Explosion,
            }
        }

        self.missiles
            .retain(|missile| match map[missile.position()] {
                Tile::Missile(_) => true,
                _ => false,
            });

        self.invaders
            .retain(|invader| match map[invader.position()] {
                Tile::Invader(_) => true,
                _ => false,
            });

        let pos = self.player.position();
        map[pos] = Tile::Player;

        map
    }

    fn draw(
        &mut self,
        out: &mut RawTerminal<Stdout>,
        map: Map<crate::utils::Tile>,
    ) -> Result<(), Error> {
        use crate::utils::Tile;

        let mut cursor = map.margins.clone();
        let dimensions = (map.width(), map.height());

        // Top border
        write!(out, "{}{}+", termion::clear::All, Goto(cursor.0, cursor.1))?;

        for _ in 0..dimensions.0 {
            write!(out, "-")?;
            cursor.0 += 1;
        }

        cursor.0 = map.margins.0;
        cursor.1 += 1;

        write!(out, "+{}", Goto(cursor.0, cursor.1))?;

        // Contents
        for y in 0..dimensions.1 {
            write!(out, "|")?;
            for x in 0..dimensions.0 {
                let icon = match map[(x, y)] {
                    Tile::Explosion => "*",
                    Tile::Invader(_) => "@",
                    Tile::Missile(_) => "!",
                    Tile::Player => "^",
                    Tile::None => " ",
                };

                write!(out, "{}", icon)?;
            }
            write!(
                out,
                "|{}",
                Goto(map.margins.0, map.margins.1 + y as u16 + 1)
            )?;
        }

        // Bottom border
        write!(out, "+")?;
        for _ in 0..dimensions.0 {
            write!(out, "-")?;
        }
        println!("+");

        write!(out, "{}", Goto(1, 1))?;

        out.flush().unwrap();

        Ok(())
    }
}
