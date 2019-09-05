#[macro_use]
extern crate log;

mod map;
mod game;

use failure::Error;
use crate::map::Map;
use crate::game::Game;

type Coord = (usize, usize);

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Dir {
    Up,
    Down,
    Left,
    Right,
}

#[derive(PartialEq, Clone)]
pub enum Tile {
    Player,
    Missile(Dir),
    Invader(Dir),
    Explosion,
    Empty,
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

    let mut input = termion::async_stdin();
    
    let mut game = Game::init(map, &mut input)?;
    game.run()?;
    Ok(())
}
