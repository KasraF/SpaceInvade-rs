use crate::game::game_loop::GameLoop;
use crate::game::menu_loop::MenuLoop;
use crate::utils::*;
use std::io::{Stdout, Write};
use termion::raw::RawTerminal;

mod game_loop;
mod menu_loop;

pub trait Loop<'a> {
    fn init(screen: Screen) -> Self;
    fn frame(&mut self, input: &mut termion::AsyncReader, out: &mut RawTerminal<Stdout>) -> Option<GameAction>;
}

pub enum GameState {
    Menu,
    Running,
    Done,
}

pub enum GameAction {
    NewGame,
    Continue,
    Menu,
    Quit,
}

pub struct Game<'a> {
    // IO stuff
    out: &'a mut RawTerminal<Stdout>,
    input: &'a mut termion::AsyncReader,

    // Game state info
    screen: Screen,
    frame_counter: u8,

    // Loops for game states
    game_loop: GameLoop,
    menu_loop: MenuLoop,
    state: GameState,
}

impl<'a> Game<'a> {
    pub fn new(input: &'a mut termion::AsyncReader, out: &'a mut RawTerminal<Stdout>) -> Self {
        let screen_size = Coord(45, 15);
        let term_size = termion::terminal_size();

        let margins = if let Ok((w, h)) = term_size {
			let x = (w as usize - screen_size.0) / 2;
			let y = (h as usize - screen_size.0) / 2;
            Coord(x, y)
        } else {
            Coord(0, 0)
        };

        let screen = Screen::new(margins, screen_size);
        let game_loop = GameLoop::init(screen.clone());
        let menu_loop = MenuLoop::init(screen.clone());
        let state = GameState::Menu;
        let frame_counter = 0;

        Self {
            game_loop,
            menu_loop,
            state,
            frame_counter,
            screen,
            out,
            input,
        }
    }

    pub fn run(&mut self) {
        while let Some(GameAction::Continue) = self
            .game_loop
            .frame(self.input, self.out)
        {
            looped_inc(&mut self.frame_counter);
        }

        write!(self.out, "{}", termion::cursor::Show).unwrap();
    }
}
