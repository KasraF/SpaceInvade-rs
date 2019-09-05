#[macro_use]
extern crate log;

mod map;
mod game;
mod entities;
mod utils;

use failure::Error;
use crate::game::Game;



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

    // let args: Vec<String> = std::env::args().collect();
    let mut input = termion::async_stdin();
    
    let mut game = Game::init(&mut input);
    game.run()?;
    
    Ok(())
}
