use crate::game::GameAction;
use crate::game::Loop;
use crate::utils::Screen;
// use crate::utils::Coord;
use std::io::{Stdout, Write};
use termion::cursor::Goto;
// use termion::event::Key;
// use termion::input::TermRead;
use termion::raw::RawTerminal;

#[derive(PartialEq)]
pub enum MenuItem {
    NewGame,
    Continue,
    Quit,
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
    ) -> Option<GameAction> {
        None
    }
}

impl MenuLoop {
    fn draw(&self, out: &mut RawTerminal<Stdout>) {
        let margins = self.screen.margins();
        let margins = (margins.0 as u16, margins.1 as u16);
        let mut cursor = (margins.0 as u16, margins.1 as u16);
        let dimensions = self.screen.size();

        // Top border
        write!(out, "{}{}+", termion::clear::All, Goto(cursor.0, cursor.1));

        for _ in 0..dimensions.0 {
            write!(out, "-");
            cursor.0 += 1;
        }

        cursor.0 = margins.0 as u16;
        cursor.1 += 1;

        write!(out, "+{}", Goto(cursor.0, cursor.1));

        // Write the menu items
        // TODO This is stupid. Please redo!
        write!(out, "{}", Goto(margins.0, margins.1 + 2));
        
        if self.selected == MenuItem::Continue {
            write!(out, "\t> Continue{}", Goto(margins.0, margins.1 + 3));
        } else {
            write!(out, "\t  Continue{}", Goto(margins.0, margins.1 + 3));
        }

        if self.selected == MenuItem::NewGame {
            write!(out, "\t> New Game{}", Goto(margins.0, margins.1 + 4));
        } else {
            write!(out, "\t  New Game{}", Goto(margins.0, margins.1 + 4));
        }

        if self.selected == MenuItem::Continue {
            write!(out, "\t> Quit{}", Goto(margins.0, margins.1 + 5));
        } else {
            write!(out, "\t  Quit{}", Goto(margins.0, margins.1 + 5));
        }

        // Contents
        // for y in 6..dimensions.1 {
        //     write!(out, "|{}|{}", Goto(margins.0 + dimensions.0, margins.1 + y as u16 + 1), Goto(margins.0, margins.1 + y as u16 + 1));
        //     write!(out,"|{}",
                
        //     );
        // }

        // Bottom border
        write!(out, "+");
        for _ in 0..dimensions.0 {
            write!(out, "-");
        }
        println!("+");

        write!(out, "{}", Goto(1, 1));

        out.flush().unwrap();
    }
}
