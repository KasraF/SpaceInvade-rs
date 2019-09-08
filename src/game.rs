use crate::entities::{Invader, Missile, Player, Entity};
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
        }
    }
}

impl<'a> Game<'a> {
    pub fn init(input: &'a mut termion::AsyncReader) -> Self {
        // TODO move to level file
        let map_size = Coord(45, 15);
        let player_pos = Coord(map_size.0 / 2, map_size.1 - 1);

        let invader = Invader::new(Coord(2, 2), Dir::Right);
        
        Game {
            out: stdout().into_raw_mode().unwrap(),
            input,
            map_size,
            is_running: false,
            frame_counter: 0,
            player: Player::new(player_pos),
            invaders: vec![invader],
            missiles: Vec::new(),
        }
    }

    pub fn run(&mut self) -> Result<(), Error> {
        let mut game_state = GameState::new(self.map_size);
        self.is_running = true;

        while self.is_running {
            // Timer!
            let now = time::Instant::now();

            game_state.events.clear();

            self.handle_input(&mut game_state.events);
            self.process_entities(&game_state);
            self.draw();

            // TODO support separate DEBUG mode?
            print!("{}{:?}", Goto(1, 1), now.elapsed());
            self.out.flush().unwrap();

            self.frame_counter = (self.frame_counter + 1) % 1024;

            // Wait
            thread::sleep(time::Duration::from_millis(300) - now.elapsed());
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
        self.player.update(game_state);

        for invader in &mut self.invaders {
            invader.update(game_state);
        }

        for missile in &mut self.missiles {
            missile.update(game_state);
        }
    }

    fn draw(&mut self) {
        let mut map = Map::new(self.map_size);

        // Fill the map
        // TODO This should not be handled here
        for invader in &self.invaders {
            let pos = invader.position();
            map[(pos.0, pos.1)] = invader.icon();
        }

        for missile in &self.missiles {
            let pos = missile.position();
            map[(pos.0, pos.1)] = missile.icon();
        }

        {
            let pos = &self.player.position();
            map[(pos.0, pos.1)] = self.player.icon();
        }
        
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
                print!("{}", map[(x, y)]);
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
