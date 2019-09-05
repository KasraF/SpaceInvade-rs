use crate::utils::Coord;
use std::ops::{Index, IndexMut};

pub struct Map {
    pub dimensions: Coord,
    pub grid: Vec<char>,
    pub margins: (u16, u16),
}

impl Map {
    pub fn new(map_size: Coord) -> Self {

        let margins = if let Ok((w, h)) = termion::terminal_size() {
            ((w - map_size.0 as u16) / 2, (h - map_size.1 as u16) / 2)
        } else {
            (0, 0)
        };

        let size = map_size.area();
        let mut grid = Vec::with_capacity(size);

        for _ in 0..size {
            grid.push(' ');
        }
        
        Map {
            dimensions: map_size,
            margins,
            grid,
        }
    }

    pub fn width(&self) -> usize {
        self.dimensions.0
    }

    pub fn height(&self) -> usize {
        self.dimensions.1
    }
}

impl Index<(usize, usize)> for Map {
    // TODO Should this return an Option?
    type Output = char;

    fn index(&self, (x, y): (usize, usize)) -> &Self::Output {
        &self.grid[y * self.dimensions.0 + x]
    }
}

impl IndexMut<(usize, usize)> for Map {
    fn index_mut(&mut self, (x, y): (usize, usize)) -> &mut Self::Output {
        let i = y * self.dimensions.0 + x;
        &mut self.grid[i]
    }
}
