use serde::{Deserialize, Serialize};
use std::ops::{Index, IndexMut, Range};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Grid2d<T> {
    cells: Vec<T>,
    cols: usize,
    rows: usize,
}

impl<T> Grid2d<T>
where
    T: Clone,
{
    pub fn new(cols: usize, rows: usize, fill: T) -> Self {
        Self {
            cells: vec![fill; cols * rows],
            cols,
            rows,
        }
    }

    pub fn with_cells(cols: usize, cells: Vec<T>) -> Self {
        if cells.len() % cols != 0 {
            panic!(
                "cells does not fill grid with desired cols number. cells: {}, cols: {}",
                cells.len(),
                cols
            );
        }
        let rows = cells.len() / cols;
        Self { cells, cols, rows }
    }

    #[inline]
    pub fn cols(&self) -> usize {
        self.cols
    }

    #[inline]
    pub fn rows(&self) -> usize {
        self.rows
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.cols * self.rows
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.cols == 0 && self.rows == 0
    }

    #[inline]
    pub fn cells(&self) -> &[T] {
        &self.cells
    }

    #[inline]
    pub fn cells_mut(&mut self) -> &mut [T] {
        &mut self.cells
    }

    #[inline]
    pub fn cell(&self, col: usize, row: usize) -> Option<&T> {
        if col < self.cols && row < self.rows {
            Some(&self.cells[row * self.cols + col])
        } else {
            None
        }
    }

    #[inline]
    pub fn cell_mut(&mut self, col: usize, row: usize) -> Option<&mut T> {
        if col < self.cols && row < self.rows {
            Some(&mut self.cells[row * self.cols + col])
        } else {
            None
        }
    }

    #[inline]
    pub fn get(&self, col: usize, row: usize) -> Option<T> {
        if col < self.cols && row < self.rows {
            Some(self.cells[row * self.cols + col].clone())
        } else {
            None
        }
    }

    #[inline]
    pub fn set(&mut self, col: usize, row: usize, value: T) {
        if col < self.cols && row < self.rows {
            self.cells[row * self.cols + col] = value;
        }
    }

    pub fn get_col_cells(&self, index: usize) -> Option<Vec<T>> {
        if index < self.cols {
            let mut result = Vec::with_capacity(self.rows);
            for row in 0..self.rows {
                result.push(self.cells[row * self.cols + index].clone());
            }
            Some(result)
        } else {
            None
        }
    }

    pub fn get_row_cells(&self, index: usize) -> Option<Vec<T>> {
        if index < self.rows {
            let start = index * self.cols;
            let end = start + self.cols;
            Some(self.cells.as_slice()[start..end].to_owned())
        } else {
            None
        }
    }

    pub fn get_part(&self, mut range: Range<(usize, usize)>) -> Self {
        range.end.0 = range.end.0.min(self.cols);
        range.end.1 = range.end.1.min(self.rows);
        range.start.0 = range.start.0.min(range.end.0);
        range.start.1 = range.start.1.min(range.end.1);
        let cols = range.end.0 - range.start.0;
        let rows = range.end.1 - range.start.1;
        let mut result = Vec::with_capacity(cols * rows);
        for row in range.start.1..range.end.1 {
            for col in range.start.0..range.end.0 {
                result.push(self.cells[row * self.cols + col].clone());
            }
        }
        Self::with_cells(cols, result)
    }

    pub fn sample(&self, (col, row): (usize, usize), margin: usize) -> Self {
        let min = (col.max(margin) - margin, row.max(margin) - margin);
        let max = (col + margin, row + margin);
        self.get_part(min..max)
    }

    pub fn map<F, R>(&self, mut f: F) -> Grid2d<R>
    where
        F: FnMut(usize, usize, &T) -> R,
        R: Clone,
    {
        Grid2d::<R>::with_cells(
            self.cols,
            self.cells
                .iter()
                .enumerate()
                .map(|(i, v)| f(i % self.cols, i / self.cols, v))
                .collect(),
        )
    }
}

impl<T> Index<(usize, usize)> for Grid2d<T>
where
    T: Clone,
{
    type Output = T;

    fn index(&self, (col, row): (usize, usize)) -> &T {
        self.cell(col, row).unwrap()
    }
}

impl<T> Index<[usize; 2]> for Grid2d<T>
where
    T: Clone,
{
    type Output = T;

    fn index(&self, [col, row]: [usize; 2]) -> &T {
        self.cell(col, row).unwrap()
    }
}

impl<T> IndexMut<(usize, usize)> for Grid2d<T>
where
    T: Clone,
{
    fn index_mut(&mut self, (col, row): (usize, usize)) -> &mut T {
        self.cell_mut(col, row).unwrap()
    }
}

impl<T> IndexMut<[usize; 2]> for Grid2d<T>
where
    T: Clone,
{
    fn index_mut(&mut self, [col, row]: [usize; 2]) -> &mut T {
        self.cell_mut(col, row).unwrap()
    }
}
