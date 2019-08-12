#[macro_use]
extern crate log;

use failure::Error;
use std::clone::Clone;
use std::io::{stdout, BufRead, BufReader, Read, Write};
use std::ops::{Index, IndexMut};
use std::thread;
use std::time;
use termion::cursor::Goto;
use termion::event::{Event, Key};
use termion::input::TermRead;
use termion::raw::IntoRawMode;

type Coord = (usize, usize);

#[derive(Debug, PartialEq, Clone, Copy)]
enum Dir {
    Up,
    Down,
    Left,
    Right,
}

#[derive(PartialEq, Clone)]
enum Tile {
    Player,
    Missile(Dir),
    Invader(Dir),
    Explosion,
    Empty,
}

struct Game {
    margins: (u16, u16),
    map: Map,
    player: Coord,
    entities: Vec<Coord>,
}

struct Map {
    width: usize,
    height: usize,
    grid: Vec<Tile>,
}

impl Map {
    fn load(reader: impl Read) -> Result<Self, Error> {
        let reader = BufReader::new(reader);
        let mut lines = reader.lines();

        // First line should be formatted as "[width][arbitrary amount of whitespace][height]"
        let (width, height) = {
            let first_line = lines.next().expect("Empty map file.")?;
            let first_line: Vec<&str> = first_line.trim().split_whitespace().collect();
            let width: usize = first_line[0]
                .parse::<usize>()
                .expect("Failed to parse map width");
            let height: usize = first_line[1]
                .parse::<usize>()
                .expect("Failed to parse map height");

            (width, height)
        };
        let mut grid = Vec::with_capacity(width * height);
        let mut line_counter = 0;

        for line in lines {
            if let Err(e) = line {
                error!("Failed to parse map line: {}", e.to_string());
                continue;
            }

            let line = line?;

            if line.len() > width {
                let msg = format!(
                    "Map line longer than expected:\n{}\nExpected {}, found {} characters.",
                    line,
                    width,
                    line.len()
                );
                return Err(failure::err_msg(msg));
            }

            let mut counter = 0;
            for c in line.chars() {
                error!("Reading char '{}'", c);
                match c {
                    ' ' => grid.push(Tile::Empty),
                    '@' => grid.push(Tile::Invader(Dir::Down)),
                    '*' => grid.push(Tile::Explosion),
                    '^' => grid.push(Tile::Player),
                    '!' => grid.push(Tile::Missile(Dir::Down)),
                    _ => error!("Map tile not recognized: '{}'. Ignoring.", c),
                }
                counter += 1;
            }

            for _ in counter..width {
                grid.push(Tile::Empty);
            }

            line_counter += 1;
        }

        for _ in line_counter..height {
            for _ in 0..width {
                grid.push(Tile::Empty);
            }
        }

        Ok(Map {
            grid,
            width,
            height,
        })
    }

    fn new(width: usize, height: usize) -> Self {
        let size = width * height;
        let mut grid = Vec::with_capacity(size);

        for _ in 0..size {
            grid.push(Tile::Empty)
        }

        let mut map = Map {
            width,
            height,
            grid,
        };

        // Add player
        let player = (width / 2 + 1, height - 1);
        map[(player.0, player.1)] = Tile::Player;

        // Add invader
        let invader = (width / 2 + 1, 1);
        map[(invader.0, invader.1)] = Tile::Invader(Dir::Left);

        map
    }
}

impl Index<Coord> for Map {
    // TODO Should this return an Option?
    type Output = Tile;

    fn index(&self, (x, y): Coord) -> &Self::Output {
        &self.grid[y * self.width + x]
    }
}

impl IndexMut<Coord> for Map {
    fn index_mut(&mut self, (x, y): Coord) -> &mut Self::Output {
        let i = y * self.width + x;
        &mut self.grid[i]
    }
}

impl Game {
    fn init(map: Map) -> Result<Self, Error> {
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

        if let Some(player) = player {
            Ok(Game {
                margins,
                map,
                player,
                entities,
            })
        } else {
            return Err(failure::err_msg("No player defined in map."));
        }
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

    fn run(&mut self, stdin: &mut termion::AsyncReader) -> Result<(), Error> {
        let mut stdout = stdout().into_raw_mode().unwrap();
        let mut run = true;
        let mut frame_counter = 1;
        stdout.flush().unwrap();

        while run {
            // Timer!
            let now = time::Instant::now();

            // Handle any user inputs
            for event in stdin.events() {
                match event {
                    Ok(event) => match event {
                        Event::Key(c) => match c {
                            Key::Char('q') => run = false,
                            Key::Ctrl('c') => run = false,
                            Key::Left => self.move_player(Dir::Left),
                            Key::Right => self.move_player(Dir::Right),
                            Key::Char(' ') => {
                                self.fire((self.player.0, self.player.1 - 1), Dir::Up)
                            }
                            _ => (),
                        },
                        _ => (),
                    },
                    Err(e) => error!("Stdin error: {}", e.to_string()),
                }
            }

            // Process entities
            let entities: &mut Vec<Coord> = &mut self.entities;
            let map: &mut Map = &mut self.map;

            entities.retain(|e| map[e.clone()] != Tile::Empty);

            if entities.is_empty() {
                return Ok(());
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
                                run = false;
                            } else {
                                map[coord] = tile;
                            }

                            entities.push(coord);
                        }
                        _ => (),
                    },
                    Tile::Invader(dir) => {
                        if frame_counter % 5 != 0 {
                            continue;
                        }

                        let dir = if dir == Dir::Down {
                            if coord.0 > map.width - 2 {
                                Dir::Left
                            } else {
                                Dir::Right
                            }
                        } else if coord.0 > map.width - 2
                            && map[(coord.0 - 1, coord.1)] == Tile::Empty
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

            // Draw frame
            self.draw();
            print!("{}{:?}", Goto(1, 1), now.elapsed());
            stdout.flush().unwrap();

            frame_counter += 1;

            // Wait
            thread::sleep(time::Duration::from_millis(30) - now.elapsed());
        }
        write!(stdout, "{}", termion::cursor::Show).unwrap();

        Ok(())
    }

    fn draw(&self) {
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
    }
}

fn main() -> Result<(), Error> {
    env_logger::init();

    info!("Hello, game!");

    let args: Vec<String> = std::env::args().collect();

    let map = if args.len() > 1 {
        Map::load(std::fs::File::open(args[1].clone())?)?
    } else {
        Map::new(45, 15)
    };

    let mut game = Game::init(map)?;
    let mut stdin = termion::async_stdin();
    game.run(&mut stdin)?;
    Ok(())
}
