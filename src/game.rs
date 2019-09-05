use crate::map::Map;
use crate::{Coord, Dir, Tile};
use failure::Error;
use std::clone::Clone;
use std::io::{stdout, Stdout, Write};
use std::thread;
use std::time;
use termion::cursor::Goto;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};

enum GameEvent {
    Quit,
}

pub struct Game<'a> {
    out: RawTerminal<Stdout>,
    input: &'a mut termion::AsyncReader,
    margins: (u16, u16),
    map: Map,
    player: Coord,
    entities: Vec<Coord>,
    frame_counter: usize,
}

impl<'a> Game<'a> {
    pub fn init(map: Map, input: &'a mut termion::AsyncReader) -> Result<Self, Error> {
        let out = stdout().into_raw_mode().unwrap();

        let margins = if let Ok((w, h)) = termion::terminal_size() {
            ((w - map.width as u16) / 2, (h - map.height as u16) / 2)
        } else {
            (0, 0)
        };

        let mut entities = Vec::new();
        let mut player = None;

        for y in 0..map.height {
            for x in 0..map.width {
                match map[(x, y)] {
                    Tile::Invader(_) | Tile::Missile(_) => entities.push((x, y)),
                    Tile::Player => {
                        if player.is_none() {
                            player = Some((x, y));
                        } else {
                            let msg =
                                format!("Too many players in map: {:?} and {:?}", player, (x, y));
                            return Err(failure::err_msg(msg));
                        }
                    }
                    _ => (),
                }
            }
        }

        let frame_counter = 0;

        if let Some(player) = player {
            Ok(Game {
                frame_counter,
                input,
                out,
                margins,
                map,
                player,
                entities,
            })
        } else {
            return Err(failure::err_msg("No player defined in map."));
        }
    }

    pub fn run(&mut self) -> Result<(), Error> {
        let mut run = true;

        while run {
            // Timer!
            let now = time::Instant::now();

            match self.handle_input() {
                Some(GameEvent::Quit) => run = false,
                _ => (),
            }

            // Process entities
            match self.process_entities() {
                Some(GameEvent::Quit) => run = false,
                _ => (),
            }

            // Draw frame
            self.draw();

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

    fn handle_input(&mut self) -> Option<GameEvent> {
        use std::io::Error;
        use termion::event::Event;

        let mut input_event = None;
        let events = self.input.events().collect::<Vec<Result<Event, Error>>>();

        for event in events {
            match event {
                Ok(event) => match event {
                    Event::Key(c) => match c {
                        Key::Char('q') => input_event = Some(GameEvent::Quit),
                        Key::Ctrl('c') => input_event = Some(GameEvent::Quit),
                        Key::Left => self.move_player(Dir::Left),
                        Key::Right => self.move_player(Dir::Right),
                        Key::Char(' ') => self.fire((self.player.0, self.player.1 - 1), Dir::Up),
                        _ => (),
                    },
                    _ => (),
                },
                Err(e) => error!("Stdin error: {}", e.to_string()),
            }
        }

        input_event
    }

    fn process_entities(&mut self) -> Option<GameEvent> {
        let entities: &mut Vec<Coord> = &mut self.entities;
        let map: &mut Map = &mut self.map;

        entities.retain(|e| map[e.clone()] != Tile::Empty);

        if entities.is_empty() {
            return Some(GameEvent::Quit);
        }

        let end = entities.len();

        for i in 0..end {
            let mut coord = entities[i];
            let tile = map[coord].clone();

            match tile {
                Tile::Missile(dir) => match dir {
                    Dir::Up => {
                        map[coord] = Tile::Empty;

                        if coord.1 == 0 {
                            continue;
                        }

                        coord.1 -= 1;

                        match map[coord] {
                            Tile::Invader(_) => map[coord] = Tile::Explosion,
                            _ => map[coord] = tile,
                        }

                        entities.push(coord)
                    }
                    Dir::Down => {
                        map[coord] = Tile::Empty;

                        if coord.1 == map.height {
                            continue;
                        }

                        coord.1 += 1;

                        if map[coord] == Tile::Player {
                            map[coord] = Tile::Explosion;
                            return Some(GameEvent::Quit)
                        } else {
                            map[coord] = tile;
                        }

                        entities.push(coord);
                    }
                    _ => (),
                },
                Tile::Invader(dir) => {
                    if self.frame_counter % 5 != 0 {
                        continue;
                    }

                    let dir = if dir == Dir::Down {
                        if coord.0 > map.width - 2 {
                            Dir::Left
                        } else {
                            Dir::Right
                        }
                    } else if coord.0 > map.width - 2 && map[(coord.0 - 1, coord.1)] == Tile::Empty
                        || coord.0 < 2 && map[(coord.0 + 1, coord.1)] == Tile::Empty
                    {
                        Dir::Down
                    } else {
                        dir
                    };

                    map[coord] = Tile::Empty;
                    match dir {
                        Dir::Down => {
                            if coord.1 < map.height - 1 {
                                coord.1 += 1;
                            }
                        }
                        Dir::Left => {
                            if coord.0 > 1 {
                                coord.0 -= 1;
                            }
                        }
                        Dir::Right => {
                            if coord.0 < map.width - 1 {
                                coord.0 += 1;
                            }
                        }
                        _ => (),
                    }

                    map[coord] = Tile::Invader(dir);
                    entities.push(coord);
                }
                Tile::Explosion => map[coord] = Tile::Empty,
                _ => (),
            }
        }

        None
    }

    fn draw(&mut self) {
        use termion::color;

        let mut cursor = self.margins.clone();
        let dimensions = (self.map.width, self.map.height);

        // Top border
        print!("{}{}+", termion::clear::All, Goto(cursor.0, cursor.1));

        for _ in 0..dimensions.0 {
            print!("-");
            cursor.0 += 1;
        }

        cursor.0 = self.margins.0;
        cursor.1 += 1;

        print!("+{}", Goto(cursor.0, cursor.1));

        // Contents
        for y in 0..dimensions.1 {
            print!("|");
            for x in 0..dimensions.0 {
                match self.map[(x, y)] {
                    Tile::Empty => print!(" "),
                    Tile::Explosion => {
                        print!("{}*{}", color::Fg(color::Red), color::Fg(color::Reset))
                    }
                    Tile::Player => print!("^"),
                    Tile::Missile(_) => print!("!"),
                    Tile::Invader(_) => print!("@"),
                }
            }
            print!("|{}", Goto(self.margins.0, self.margins.1 + y as u16 + 1));
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

    fn move_player(&mut self, direction: Dir) {
        // TODO Handle collision?
        assert!(self.map[self.player] == Tile::Player);

        self.map[self.player] = Tile::Empty;
        match direction {
            Dir::Left => {
                if self.player.0 > 0 {
                    self.player.0 -= 1;
                }
            }
            Dir::Right => {
                if self.player.0 < self.map.width - 1 {
                    self.player.0 += 1;
                }
            }
            _ => warn!("direction {:?} not supported for move_player.", direction),
        }
        self.map[self.player] = Tile::Player;
    }

    fn fire(&mut self, pos: Coord, dir: Dir) {
        if self.map[pos] == Tile::Empty {
            self.map[pos] = Tile::Missile(dir);
            self.entities.push(pos);
        }
    }
}
