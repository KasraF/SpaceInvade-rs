#![feature(drain_filter)]
#![feature(async_closure)]

#[macro_use]
extern crate log;

mod map;
mod game;
mod entities;
mod utils;

use failure::Error;
use crate::game::Game;

fn main() -> Result<(), Error> {
    env_logger::init();

    // let args: Vec<String> = std::env::args().collect();
    let mut input = termion::async_stdin();
    
    let mut game = Game::init(&mut input);
    game.run()?;
    
    Ok(())
}
