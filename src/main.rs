#[macro_use]
extern crate log;

use failure::Error;
use termion::cursor::Goto;

struct Game {
    margins: (u16, u16),
    dimensions: (usize, usize),
    map: Vec<Tile>,
}

impl Game {
    fn init(x: usize, y: usize) -> Self {
        let size = x * y;
        let mut map = Vec::with_capacity(size);

        for _ in 0..size {
            map.push(Tile::Empty);
        }

        let margins = if let Ok((w, h)) = termion::terminal_size() {
            ((w - x as u16) / 2, (h - y as u16) / 2)
        } else {
            (0, 0)
        };

        map[size - x / 2 - 1] = Tile::Player;
        
        Game {
            margins,
            dimensions: (x, y),
            map,
        }
    }

    fn draw(&self) {
        let mut cursor = self.margins.clone();

        // Top border
        print!("{}{}+", termion::clear::All, Goto(cursor.0, cursor.1));
        for _ in 0..self.dimensions.0 {
            print!("-");
            cursor.0 += 1;
        }

        cursor.0 = self.margins.0;
        cursor.1 += 1;

        print!("+{}", Goto(cursor.0, cursor.1));

        // Contents
        for y in 0..self.dimensions.1 {
            print!("|");
            for x in 0..self.dimensions.0 {
                match self.map[y * self.dimensions.0 + x] {
                    Tile::Empty => {
                        print!(" ");
                    },
                    Tile::Player => {
                        print!("^");
                    }
                }
            }
            print!("|{}", Goto(self.margins.0, self.margins.1 + y as u16 + 1));
        }

        // Bottom border
        print!("+");
        for _ in 0..self.dimensions.0 {
            print!("-");
        }
        println!("+");
    }
}

enum Tile {
    Player,
    Empty,
}

fn main() -> Result<(), Error> {
    env_logger::init();

    info!("Hello, game!");

    let game = Game::init(45, 15);
    // let mut stdin = std::io::stdin();
    game.draw();

    Ok(())
}
