use crate::utils::Coord;
use std::ops::{Index, IndexMut};

pub struct Map<T: Copy> {
    pub dimensions: Coord,
    pub grid: Vec<T>,
}

impl<T: Copy> Map<T> {
    pub fn new(map_size: Coord, default: T) -> Self {

        let size = map_size.area();
        let mut grid = Vec::<T>::with_capacity(size);

        for _ in 0..size {
            grid.push(default);
        }
        
        Map {
            dimensions: map_size,
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

impl<T: Copy> Index<(usize, usize)> for Map<T> {
    // TODO Should this return an Option?
    type Output = T;

    fn index(&self, (x, y): (usize, usize)) -> &Self::Output {
        debug_assert!(x < self.dimensions.0);
        debug_assert!(y < self.dimensions.1);
            
        &self.grid[y * self.dimensions.0 + x]
    }
}

impl<T: Copy> IndexMut<(usize, usize)> for Map<T> {
    fn index_mut(&mut self, (x, y): (usize, usize)) -> &mut Self::Output {
        debug_assert!(x < self.dimensions.0);
        debug_assert!(y < self.dimensions.1);
        
        &mut self.grid[y * self.dimensions.0 + x]
    }
}

impl<T: Copy> Index<&Coord> for Map<T> {
    // TODO Should this return an Option?
    type Output = T;

    fn index(&self, pos: &Coord) -> &Self::Output {
        let x = pos.0;
        let y = pos.1;
        
        debug_assert!(x < self.dimensions.0);
        debug_assert!(y < self.dimensions.1);
            
        &self.grid[y * self.dimensions.0 + x]
    }
}

impl<T: Copy> IndexMut<&Coord> for Map<T> {
    fn index_mut(&mut self, pos: &Coord) -> &mut Self::Output {
        let x = pos.0;
        let y = pos.1;
        
        debug_assert!(x < self.dimensions.0);
        debug_assert!(y < self.dimensions.1);
        
        &mut self.grid[y * self.dimensions.0 + x]
    }
}
