use crate::entities::{Entity, Invader, Missile, Player};
use crate::map::Map;
use crate::utils::{Coord, Dir};
use failure::Error;
use std::clone::Clone;
use std::io::{stdout, Stdout, Write};
use std::thread;
use std::time;
use termion::cursor::Goto;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};

pub enum CtrlEvent {
    Left,
    Right,
    Shoot,
}

pub struct GameState {
    pub events: Vec<CtrlEvent>,
    pub map_dimensions: Coord,
    pub frame: u8,
}

pub struct Game<'a> {
    // IO stuff
    out: RawTerminal<Stdout>,
    input: &'a mut termion::AsyncReader,

    // Game Logic stuff
    is_running: bool,
    frame_counter: usize,
    map_size: Coord,

    // Entities
    player: Player,
    invaders: Vec<Invader>,
    missiles: Vec<Missile>,
}

impl GameState {
    fn new(dimensions: Coord) -> Self {
        Self {
            events: Vec::new(),
            map_dimensions: dimensions,
            frame: 0,
        }
    }
}

impl<'a> Game<'a> {
    pub fn init(input: &'a mut termion::AsyncReader) -> Self {
        // TODO move to level file
        let map_size = Coord(45, 15);
        let player_pos = Coord(map_size.0 / 2, map_size.1 - 1);

        let invader1 = Invader::new(Coord(2, 2), Dir::Right);
        let invader2 = Invader::new(Coord(3, 2), Dir::Right);
        let invader3 = Invader::new(Coord(4, 2), Dir::Right);

        Game {
            out: stdout().into_raw_mode().unwrap(),
            input,
            map_size,
            is_running: false,
            frame_counter: 0,
            player: Player::new(player_pos),
            invaders: vec![invader1, invader2, invader3],
            missiles: Vec::new(),
        }
    }

    pub fn run(&mut self) -> Result<(), Error> {
        let mut game_state = GameState::new(self.map_size);
        self.is_running = true;

        while self.is_running {
            // Timer!
            let now = time::Instant::now();

            game_state.frame = (((game_state.frame as u16) + 1) % 255) as u8;
            game_state.events.clear();

            self.handle_input(&mut game_state.events);
            self.process_entities(&game_state);
            let map = self.handle_collisions();
            self.draw(map);

            if self.invaders.is_empty() {
                self.is_running = false;                    
            }

            // TODO support separate DEBUG mode?
            print!("{}{:?}", Goto(1, 1), now.elapsed());
            self.out.flush().unwrap();

            self.frame_counter = (self.frame_counter + 1) % 1024;

            // Wait
            thread::sleep(time::Duration::from_millis(30) - now.elapsed());
        }

        write!(self.out, "{}", termion::cursor::Show).unwrap();

        Ok(())
    }

    fn handle_input(&mut self, events: &mut Vec<CtrlEvent>) {
        use std::io::Error;
        use termion::event::Event;

        let input_events = self.input.events().collect::<Vec<Result<Event, Error>>>();

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

    fn process_entities(&mut self, game_state: &GameState) {
        use crate::entities::Request;

        for missile in &mut self.missiles {
            missile.update(game_state);
        }

        if let Some(Request::FireMissile) = self.player.update(game_state) {
            let mut pos = self.player.position().clone();
            pos.1 -= 1;

            self.missiles.push(Missile::new(pos, Dir::Up));
        }

        for invader in &mut self.invaders {
            invader.update(game_state);
        }
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

        self.missiles.retain(|missile| match map[missile.position()] {
            Tile::Missile(_) => true,
            _ => false, 
        });

        self.invaders.retain(|invader| match map[invader.position()] {
            Tile::Invader(_) => true,
            _ => false, 
        });
        
        let pos = self.player.position();
        map[pos] = Tile::Player;

        map
    }

    fn draw(&mut self, map: Map<crate::utils::Tile>) {
        use crate::utils::Tile;
        
        let mut cursor = map.margins.clone();
        let dimensions = (map.width(), map.height());

        // Top border
        print!("{}{}+", termion::clear::All, Goto(cursor.0, cursor.1));

        for _ in 0..dimensions.0 {
            print!("-");
            cursor.0 += 1;
        }

        cursor.0 = map.margins.0;
        cursor.1 += 1;

        print!("+{}", Goto(cursor.0, cursor.1));

        // Contents
        for y in 0..dimensions.1 {
            print!("|");
            for x in 0..dimensions.0 {
                let icon = match map[(x, y)] {
                    Tile::Explosion => '*',
                    Tile::Invader(_) => '@',
                    Tile::Missile(_) => '!',
                    Tile::Player => '^',
                    Tile::None => ' ',
                };
                                     
                print!("{}", icon);
            }
            print!("|{}", Goto(map.margins.0, map.margins.1 + y as u16 + 1));
        }

        // Bottom border
        print!("+");
        for _ in 0..dimensions.0 {
            print!("-");
        }
        println!("+");

        print!("{}", Goto(1, 1));

        self.out.flush().unwrap();
    }
}
