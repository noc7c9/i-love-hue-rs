#[derive(Debug)]
pub struct Grid<T> {
    width: usize,
    height: usize,
    cells: Vec<T>,
}

impl<T> Grid<T> {
    pub fn from_closure<F>(width: usize, height: usize, closure: F) -> Self
    where
        F: Fn(usize, usize) -> T,
    {
        let mut cells = Vec::with_capacity(width * height);
        for y in 0..height {
            for x in 0..width {
                cells.push(closure(x, y));
            }
        }

        Self {
            width,
            height,
            cells,
        }
    }

    pub fn iter(&self) -> Iter<T> {
        Iter::new(self)
    }

    pub fn dims(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    pub fn swap(&mut self, a: usize, b: usize) {
        self.cells.swap(a, b)
    }

    pub fn get(&self, idx: usize) -> &T {
        &self.cells[idx]
    }
}

pub struct Iter<'a, T> {
    grid: &'a Grid<T>,
    index_iter: std::ops::Range<usize>,
}

impl<'a, T> Iter<'a, T> {
    fn new(grid: &'a Grid<T>) -> Self {
        let size = grid.width * grid.height;
        Self {
            grid,
            index_iter: 0..size,
        }
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.index_iter.next().map(|i| &self.grid.cells[i])
    }
}
