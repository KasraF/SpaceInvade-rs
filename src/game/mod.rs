use crate::game::game_loop::*;
use std::io::{Stdout, Write};
use termion::raw::RawTerminal;

mod game_loop;
mod menu_loop;

pub enum GameState {
    Menu,
    Running,
    Done,
}

pub struct Game<'a> {
    // IO stuff
    out: &'a mut RawTerminal<Stdout>,
    input: &'a mut termion::AsyncReader,

    // Game state info
    frame_counter: u8,

    // Loops for game states
    game_loop: GameLoop,
    state: GameState,
}

impl<'a> Game<'a> {
    pub fn new(input: &'a mut termion::AsyncReader, out: &'a mut RawTerminal<Stdout>) -> Self {
        let game_loop = GameLoop::init();
        let state = GameState::Menu;
        let frame_counter = 0;

        Self {
            game_loop,
            state,
            frame_counter,
            out,
            input,
        }
    }

    pub fn run(&mut self) {
        use crate::utils::looped_inc;
        
        while let Ok(_) = self
            .game_loop
            .frame(self.input, self.out, self.frame_counter)
        {
            looped_inc(&mut self.frame_counter);
        }

        write!(self.out, "{}", termion::cursor::Show).unwrap();
    }
}
