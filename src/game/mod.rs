use crate::game::game_loop::GameLoop;
use crate::game::menu_loop::MenuLoop;
use crate::utils::*;
use std::thread;
use std::time;
use failure::Error;
use std::io::{Stdout, Write};
use termion::raw::RawTerminal;

mod game_loop;
mod menu_loop;

pub trait Loop<'a> {
    fn init(screen: Screen) -> Self;
    fn frame(&mut self, input: &mut termion::AsyncReader, out: &mut RawTerminal<Stdout>) -> Result<GameAction, Error>;
}

pub enum GameState {
    Menu,
    Running,
    Done,
}

pub enum GameAction {
    NewGame,
	EndGame,
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

        Self {
            game_loop,
            menu_loop,
            state,
            screen,
            out,
            input,
        }
    }

    pub fn run(&mut self) {
		loop {
			// Timer!
			let now = time::Instant::now();

			let action  = match self.state {
				GameState::Menu => self.menu_loop.frame(self.input, self.out),
				GameState::Running => self.game_loop.frame(self.input, self.out),
				GameState::Done => self.menu_loop.frame(self.input, self.out)
			}.expect("Encountered error: ");
			
			match action {
				GameAction::Continue => {
					self.state = GameState::Running;
				},
				GameAction::NewGame => {
					self.game_loop = GameLoop::init(self.screen.clone());
					self.state = GameState::Running;
				},
				GameAction::EndGame => {
					self.game_loop = GameLoop::init(self.screen.clone());
					self.state = GameState::Menu;
				},
				GameAction::Menu => self.state = GameState::Menu,
				GameAction::Quit => break,
			}

			// TODO support separate DEBUG mode?
			write!(self.out, "{}{:?}", termion::cursor::Goto(1, 1), now.elapsed());
			self.out.flush().unwrap();

			// Wait
			thread::sleep(time::Duration::from_millis(30) - now.elapsed());
		}
		

        write!(self.out, "{}", termion::cursor::Show).unwrap();
    }
}
