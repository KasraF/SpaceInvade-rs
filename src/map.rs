use crate::{Coord, Dir, Tile};
use failure::Error;
use std::io::{BufRead, BufReader, Read};
use std::ops::{Index, IndexMut};

pub struct Map {
    pub width: usize,
    pub height: usize,
    pub grid: Vec<Tile>,
}

impl Map {
    pub fn load(reader: impl Read) -> Result<Self, Error> {
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

    pub fn new(width: usize, height: usize) -> Self {
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
