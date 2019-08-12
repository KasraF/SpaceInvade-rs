#[macro_use]
extern crate log;
extern crate termion;

use failure::Error;
use std::clone::Clone;
use std::io::{stdout, Write};
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
    Invader,
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
    fn new(width: usize, height: usize) -> Self {
        let size = width * height;
        let mut grid = Vec::with_capacity(size);

        for _ in 0..size {
            grid.push(Tile::Empty)
        }

        Map {
            width,
            height,
            grid,
        }
    }

    fn index(&self, x: usize, y: usize) -> usize {
        (y * self.width + x)
    }

    fn coord(&self, i: usize) -> Coord {
        // TODO Off by one error?
        let x = i % self.width;
        (x, (i - x) / self.width)
    }
}

impl Index<Coord> for Map {
    // TODO Should this return an Option?
    type Output = Tile;

    fn index(&self, (x, y): Coord) -> &Self::Output {
        &self.grid[self.index(x, y)]
    }
}

impl IndexMut<Coord> for Map {
    fn index_mut(&mut self, (x, y): Coord) -> &mut Self::Output {
        let i = self.index(x, y);
        &mut self.grid[i]
    }
}

impl Game {
    fn init(x: usize, y: usize) -> Self {
        let mut map = Map::new(x, y);

        let margins = if let Ok((w, h)) = termion::terminal_size() {
            ((w - x as u16) / 2, (h - y as u16) / 2)
        } else {
            (0, 0)
        };

        // Add player
        let player = (x / 2 + 1, y - 1);
        map[(player.0, player.1)] = Tile::Player;

        // Add invader
        let invader = (x / 2 + 1, 1);
        map[(invader.0, invader.1)] = Tile::Invader;

        Game {
            margins,
            map,
            player,
            entities: vec![invader],
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

                            if map[coord] == Tile::Invader {
                                map[coord] = Tile::Explosion;
                            } else {
                                map[coord] = tile;
                            }

                            entities.push(coord)
                        }
                        Dir::Down => {
                            map[coord] = Tile::Empty;

                            if coord.1 == map.height {
                                continue;
                            }

                            coord.1 += 1;

                            map[coord] = tile;
                            entities.push(coord);
                        }
                        _ => (),
                    },
                    Tile::Explosion => map[coord] = Tile::Empty,
                    _ => (),
                }
            }

            // Draw frame
            self.draw();
            print!("{}{:?}", Goto(1, 1), now.elapsed());
            stdout.flush().unwrap();

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
                    Tile::Invader => print!("@"),
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

    let mut game = Game::init(45, 15);
    let mut stdin = termion::async_stdin();
    game.run(&mut stdin)?;
    Ok(())
}
