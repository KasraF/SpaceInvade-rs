use crate::game::GameAction;
use crate::game::Loop;
use crate::utils::Screen;
use termion::event::Key;
use failure::Error;
use termion::input::TermRead;
use std::io::{Stdout, Write};
use termion::cursor::Goto;
use termion::raw::RawTerminal;

pub enum CtrlEvent {
	Up,
	Down,
	Select,
	Quit,
}

#[derive(PartialEq)]
pub enum MenuItem {
    NewGame,
    Continue,
    Quit,
}

impl MenuItem {
	pub fn next(&self) -> Self {
		match self {
			MenuItem::NewGame => Self::Continue,
			MenuItem::Continue => Self::Quit,
			MenuItem::Quit => Self::NewGame,
		}
	}

	pub fn previous(&self) -> Self {
		match self {
			MenuItem::NewGame => Self::Quit,
			MenuItem::Continue => Self::NewGame,
			MenuItem::Quit => Self::Continue,
		}
	}
}

pub struct MenuLoop {
    screen: Screen,
    selected: MenuItem,
}

impl Loop<'_> for MenuLoop {
    fn init(screen: Screen) -> Self {
        Self {
            screen,
            selected: MenuItem::NewGame,
        }
    }

    fn frame(
        &mut self,
        input: &mut termion::AsyncReader,
        out: &mut RawTerminal<Stdout>,
    ) -> Result<GameAction, Error> {
		self.draw(out)?;

		let mut events = Vec::new();
		self.handle_input(input, &mut events);

		let mut action = GameAction::Menu;
		
		for event in events {
			
			match event {
				CtrlEvent::Down => self.selected = self.selected.next(),
				CtrlEvent::Up => self.selected = self.selected.previous(),
				CtrlEvent::Select =>
					match self.selected {
						MenuItem::Continue => action = GameAction::Continue,
						MenuItem::NewGame => action = GameAction::NewGame,
						MenuItem::Quit => action = GameAction::Quit,
					},
				CtrlEvent::Quit => action = GameAction::Quit
			}

		}

		Ok(action)
    }
}

impl MenuLoop {
	fn handle_input(&mut self, input: &mut termion::AsyncReader, events: &mut Vec<CtrlEvent>) {
        // TODO Is there a better way to do this?
        use std::io::Error;
        use termion::event::Event;
		
        let input_events = input.events().collect::<Vec<Result<Event, Error>>>();

        for event in input_events {
            match event {
                Ok(event) => match event {
                    Event::Key(c) => match c {
                        Key::Char('q') | Key::Ctrl('c') => events.push(CtrlEvent::Quit),
                        Key::Up | Key::Left => events.push(CtrlEvent::Up),
                        Key::Down | Key::Right => events.push(CtrlEvent::Down),
                        Key::Char(' ') | Key::Char('\n') => events.push(CtrlEvent::Select),
                        _ => (),
                    },
                    _ => (),
                },
                Err(e) => error!("Stdin error: {}", e.to_string()),
            }
        }
    }
	
    fn draw(&self, out: &mut RawTerminal<Stdout>) -> Result<(), Error>{
		use std::fmt::Write;
		
        let margins = self.screen.margins();
        let margins = (margins.0 as u16, margins.1 as u16);
        let mut cursor = (margins.0 as u16, margins.1 as u16);
        let dimensions = self.screen.size();
		let mut buff = String::with_capacity(self.screen.frame_buffer_size());

        // Top border
        write!(&mut buff, "{}{}+", termion::clear::All, Goto(cursor.0, cursor.1))?;

        for _ in 0..dimensions.0 {
            write!(&mut buff, "-")?;
            cursor.0 += 1;
        }

        cursor.0 = margins.0 as u16;
        cursor.1 += 1;

        write!(&mut buff, "+{}", Goto(cursor.0, cursor.1))?;

        // Write the menu items
        // TODO This is stupid. Please redo!
        write!(&mut buff, "{}", Goto(margins.0, margins.1 + 2))?;

        if self.selected == MenuItem::Continue {
            write!(&mut buff, "\t> Continue{}", Goto(margins.0, margins.1 + 3))?;
        } else {
            write!(&mut buff, "\t  Continue{}", Goto(margins.0, margins.1 + 3))?;
        }

        if self.selected == MenuItem::NewGame {
            write!(&mut buff, "\t> New Game{}", Goto(margins.0, margins.1 + 4))?;
        } else {
            write!(&mut buff, "\t  New Game{}", Goto(margins.0, margins.1 + 4))?;
        }

        if self.selected == MenuItem::Quit {
            write!(&mut buff, "\t> Quit{}", Goto(margins.0, margins.1 + 5))?;
        } else {
            write!(&mut buff, "\t  Quit{}", Goto(margins.0, margins.1 + 5))?;
        }

        // Contents
        // for y in 6..dimensions.1 {
        //     write!(&mut buff, "|{}|{}", Goto(margins.0 + dimensions.0, margins.1 + y as u16 + 1), Goto(margins.0, margins.1 + y as u16 + 1));
        //     write!(&mut buff,"|{}",

        //     );
        // }

        // Bottom border
        write!(&mut buff, "+")?;
        for _ in 0..dimensions.0 {
            write!(&mut buff, "-")?;
        }
        write!(&mut buff, "+")?;

        write!(&mut buff, "{}", Goto(1, 1))?;

		write!(out, "{}", buff)?;
        out.flush().unwrap();

		Ok(())
    }
}
