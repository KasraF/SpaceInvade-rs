#![feature(drain_filter)]
#![feature(async_closure)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate failure;

mod map;
mod game;
mod entities;
mod utils;

use termion::raw::IntoRawMode;
use failure::Error;
use crate::game::Game;

fn main() -> Result<(), Error> {
    env_logger::init();

    // let args: Vec<String> = std::env::args().collect();
    let mut input = termion::async_stdin();
    let mut output = std::io::stdout().into_raw_mode()?;
    
    let mut game = Game::new(&mut input, &mut output);
    game.run();
    
    Ok(())
}
