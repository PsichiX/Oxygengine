#[cfg(feature = "parallel")]
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::ops::{Add, Div, Index, IndexMut, Mul, Neg, Range, Sub};

#[derive(Debug, Clone)]
pub enum Grid2dError {
    DifferentDimensions((usize, usize), (usize, usize)),
}

#[derive(Debug)]
pub struct Grid2dNeighborSample<T> {
    cells: [T; 9],
    cols: usize,
    rows: usize,
}

impl<T> Grid2dNeighborSample<T>
where
    T: Clone,
{
    #[inline]
    pub fn cols(&self) -> usize {
        self.cols
    }

    #[inline]
    pub fn rows(&self) -> usize {
        self.rows
    }

    #[inline]
    pub fn size(&self) -> (usize, usize) {
        (self.cols, self.rows)
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

    pub fn map<F, R>(&self, mut f: F) -> Grid2dNeighborSample<R>
    where
        F: FnMut(usize, usize, &T) -> R,
        R: Default + Copy,
    {
        let mut cells = [R::default(); 9];
        for (i, v) in self.cells.iter().enumerate() {
            cells[i] = f(i % self.cols, i / self.cols, v);
        }
        Grid2dNeighborSample::<R> {
            cells,
            cols: self.cols,
            rows: self.rows,
        }
    }

    pub fn with<F>(&mut self, mut f: F)
    where
        F: FnMut(usize, usize, &T) -> T,
    {
        for (i, v) in self.cells.iter_mut().enumerate() {
            *v = f(i % self.cols, i / self.cols, v);
        }
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.cells.iter()
    }

    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.cells.iter_mut()
    }
}

impl<T> Index<(usize, usize)> for Grid2dNeighborSample<T>
where
    T: Clone + Send + Sync,
{
    type Output = T;

    fn index(&self, (col, row): (usize, usize)) -> &T {
        self.cell(col, row).unwrap()
    }
}

impl<T> Index<[usize; 2]> for Grid2dNeighborSample<T>
where
    T: Clone + Send + Sync,
{
    type Output = T;

    fn index(&self, [col, row]: [usize; 2]) -> &T {
        self.cell(col, row).unwrap()
    }
}

impl<T> IndexMut<(usize, usize)> for Grid2dNeighborSample<T>
where
    T: Clone + Send + Sync,
{
    fn index_mut(&mut self, (col, row): (usize, usize)) -> &mut T {
        self.cell_mut(col, row).unwrap()
    }
}

impl<T> IndexMut<[usize; 2]> for Grid2dNeighborSample<T>
where
    T: Clone + Send + Sync,
{
    fn index_mut(&mut self, [col, row]: [usize; 2]) -> &mut T {
        self.cell_mut(col, row).unwrap()
    }
}

impl<I, T> From<(usize, I)> for Grid2dNeighborSample<T>
where
    I: Iterator<Item = T>,
    T: Default + Copy,
{
    fn from((cols, iter): (usize, I)) -> Self {
        let mut cells = [T::default(); 9];
        let mut index = 0;
        for v in iter.take(9) {
            cells[index] = v;
            index += 1;
        }
        Self {
            cells,
            cols,
            rows: index / cols,
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Grid2d<T> {
    cells: Vec<T>,
    cols: usize,
    rows: usize,
}

impl<T> Grid2d<T>
where
    T: Clone + Send + Sync,
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

    pub fn resize(&mut self, cols: usize, rows: usize, default: T) {
        if cols == self.cols && rows == self.rows {
            return;
        }
        self.cells.resize(cols * rows, default);
        self.cols = cols;
        self.rows = rows;
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
    pub fn size(&self) -> (usize, usize) {
        (self.cols, self.rows)
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

    pub fn copy_part(&self, mut range: Range<(usize, usize)>, result: &mut Self)
    where
        T: Default,
    {
        range.end.0 = range.end.0.min(self.cols);
        range.end.1 = range.end.1.min(self.rows);
        range.start.0 = range.start.0.min(range.end.0);
        range.start.1 = range.start.1.min(range.end.1);
        let cols = range.end.0 - range.start.0;
        let rows = range.end.1 - range.start.1;
        result.resize(cols, rows, T::default());
        for row in range.start.1..range.end.1 {
            for col in range.start.0..range.end.0 {
                result.set(col, row, self.cells[row * self.cols + col].clone());
            }
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

    pub fn get_view(&self, mut range: Range<(usize, usize)>) -> Grid2d<&T> {
        range.end.0 = range.end.0.min(self.cols);
        range.end.1 = range.end.1.min(self.rows);
        range.start.0 = range.start.0.min(range.end.0);
        range.start.1 = range.start.1.min(range.end.1);
        let cols = range.end.0 - range.start.0;
        let rows = range.end.1 - range.start.1;
        let mut result = Vec::with_capacity(cols * rows);
        for row in range.start.1..range.end.1 {
            for col in range.start.0..range.end.0 {
                result.push(&self.cells[row * self.cols + col]);
            }
        }
        Grid2d::with_cells(cols, result)
    }

    pub fn copy_sample(&self, (col, row): (usize, usize), margin: usize, result: &mut Self)
    where
        T: Default,
    {
        let min = (col.max(margin) - margin, row.max(margin) - margin);
        let max = (col + margin + 1, row + margin + 1);
        self.copy_part(min..max, result);
    }

    pub fn sample(&self, (col, row): (usize, usize), margin: usize) -> Self {
        let min = (col.max(margin) - margin, row.max(margin) - margin);
        let max = (col + margin + 1, row + margin + 1);
        self.get_part(min..max)
    }

    pub fn view_sample(&self, (col, row): (usize, usize), margin: usize) -> Grid2d<&T> {
        let min = (col.max(margin) - margin, row.max(margin) - margin);
        let max = (col + margin + 1, row + margin + 1);
        self.get_view(min..max)
    }

    pub fn sin_map<F, R>(&self, mut f: F) -> Grid2d<R>
    where
        F: FnMut(usize, usize, &T) -> R,
        R: Clone + Send + Sync,
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

    #[cfg(feature = "parallel")]
    pub fn par_map<F, R>(&self, f: F) -> Grid2d<R>
    where
        F: FnMut(usize, usize, &T) -> R,
        F: Clone + Send + Sync,
        R: Clone + Send + Sync,
    {
        let cols = self.cols;
        let cells = self
            .cells
            .par_iter()
            .enumerate()
            .map(|(i, v)| f.clone()(i % cols, i / cols, v))
            .collect();
        Grid2d::<R>::with_cells(cols, cells)
    }

    pub fn map<F, R>(&self, f: F) -> Grid2d<R>
    where
        F: FnMut(usize, usize, &T) -> R,
        F: Clone + Send + Sync,
        R: Clone + Send + Sync,
    {
        #[cfg(not(feature = "parallel"))]
        {
            self.sin_map(f)
        }
        #[cfg(feature = "parallel")]
        {
            self.par_map(f)
        }
    }

    pub fn sin_with<F>(&mut self, mut f: F)
    where
        F: FnMut(usize, usize, &T) -> T,
    {
        for (i, v) in self.cells.iter_mut().enumerate() {
            *v = f(i % self.cols, i / self.cols, v);
        }
    }

    #[cfg(feature = "parallel")]
    pub fn par_with<F>(&mut self, f: F)
    where
        F: FnMut(usize, usize, &T) -> T,
        F: Clone + Send + Sync,
    {
        let cols = self.cols;
        self.cells.par_iter_mut().enumerate().for_each(|(i, v)| {
            *v = f.clone()(i % cols, i / cols, v);
        });
    }

    pub fn with<F>(&mut self, f: F)
    where
        F: FnMut(usize, usize, &T) -> T,
        F: Clone + Send + Sync,
    {
        #[cfg(not(feature = "parallel"))]
        self.sin_with(f);
        #[cfg(feature = "parallel")]
        self.par_with(f);
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.cells.iter()
    }

    #[cfg(feature = "parallel")]
    #[inline]
    pub fn par_iter(&self) -> impl IndexedParallelIterator<Item = &T> {
        self.cells.par_iter()
    }

    pub fn iter_view(
        &self,
        mut range: Range<(usize, usize)>,
    ) -> impl Iterator<Item = (usize, usize, &T)> {
        range.end.0 = range.end.0.min(self.cols);
        range.end.1 = range.end.1.min(self.rows);
        range.start.0 = range.start.0.min(range.end.0);
        range.start.1 = range.start.1.min(range.end.1);
        let cols = range.end.0 - range.start.0;
        let rows = range.end.1 - range.start.1;
        (0..(cols * rows)).map(move |i| {
            let lc = i % cols;
            let lr = i / cols;
            let gc = range.start.0 + lc;
            let gr = range.start.1 + lr;
            (gc, gr, &self.cells[gr * self.cols + gc])
        })
    }

    pub fn iter_sample<'a>(
        &'a self,
        mut range: Range<(usize, usize)>,
        margin: usize,
    ) -> impl Iterator<Item = (usize, usize, Grid2d<&T>)> + 'a {
        range.end.0 = range.end.0.min(self.cols);
        range.end.1 = range.end.1.min(self.rows);
        range.start.0 = range.start.0.min(range.end.0);
        range.start.1 = range.start.1.min(range.end.1);
        let cols = range.end.0 - range.start.0;
        let rows = range.end.1 - range.start.1;
        (0..(cols * rows)).map(move |i| {
            let lc = i % cols;
            let lr = i / cols;
            let gc = range.start.0 + lc;
            let gr = range.start.1 + lr;
            (gc, gr, self.view_sample((gc, gr), margin))
        })
    }

    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.cells.iter_mut()
    }

    #[cfg(feature = "parallel")]
    #[inline]
    pub fn par_iter_mut(&mut self) -> impl IndexedParallelIterator<Item = &mut T> {
        self.cells.par_iter_mut()
    }

    pub fn iter_view_mut(
        &mut self,
        mut range: Range<(usize, usize)>,
    ) -> impl Iterator<Item = (usize, usize, &mut T)> {
        range.end.0 = range.end.0.min(self.cols);
        range.end.1 = range.end.1.min(self.rows);
        range.start.0 = range.start.0.min(range.end.0);
        range.start.1 = range.start.1.min(range.end.1);
        let cols = self.cols;
        self.cells.iter_mut().enumerate().filter_map(move |(i, v)| {
            let c = i % cols;
            let r = i / cols;
            if c >= range.start.0 && c < range.end.0 && r >= range.start.1 && r < range.end.1 {
                Some((c, r, v))
            } else {
                None
            }
        })
    }

    pub fn neighbor_sample(&self, (col, row): (usize, usize)) -> Grid2dNeighborSample<T>
    where
        T: Default + Copy,
    {
        let min = (col.max(1) - 1, row.max(1) - 1);
        let max = ((col + 2).min(self.cols), (row + 2).min(self.rows));
        let cols = max.0 - min.0;
        let rows = max.1 - min.1;
        let mut cells = [T::default(); 9];
        let mut index = 0;
        for row in min.1..max.1 {
            for col in min.0..max.0 {
                cells[index] = self.cells[row * self.cols + col];
                index += 1;
            }
        }
        Grid2dNeighborSample { cells, cols, rows }
    }

    pub fn into_inner(self) -> (usize, usize, Vec<T>) {
        (self.cols, self.rows, self.cells)
    }
}

impl<T> Index<(usize, usize)> for Grid2d<T>
where
    T: Clone + Send + Sync,
{
    type Output = T;

    fn index(&self, (col, row): (usize, usize)) -> &T {
        self.cell(col, row).unwrap()
    }
}

impl<T> Index<[usize; 2]> for Grid2d<T>
where
    T: Clone + Send + Sync,
{
    type Output = T;

    fn index(&self, [col, row]: [usize; 2]) -> &T {
        self.cell(col, row).unwrap()
    }
}

impl<T> IndexMut<(usize, usize)> for Grid2d<T>
where
    T: Clone + Send + Sync,
{
    fn index_mut(&mut self, (col, row): (usize, usize)) -> &mut T {
        self.cell_mut(col, row).unwrap()
    }
}

impl<T> IndexMut<[usize; 2]> for Grid2d<T>
where
    T: Clone + Send + Sync,
{
    fn index_mut(&mut self, [col, row]: [usize; 2]) -> &mut T {
        self.cell_mut(col, row).unwrap()
    }
}

impl<T> Add for &Grid2d<T>
where
    T: Clone + Send + Sync + Add<Output = T>,
{
    type Output = Result<Grid2d<T>, Grid2dError>;

    fn add(self, other: &Grid2d<T>) -> Self::Output {
        if self.cols() == other.cols() && self.rows() == other.rows() {
            let cells = self
                .cells
                .iter()
                .zip(other.cells.iter())
                .map(|(a, b)| a.clone() + b.clone())
                .collect::<Vec<T>>();
            Ok(Grid2d::with_cells(self.cols(), cells))
        } else {
            Err(Grid2dError::DifferentDimensions(
                (self.cols(), self.rows()),
                (other.cols(), other.rows()),
            ))
        }
    }
}

impl<T> Sub for &Grid2d<T>
where
    T: Clone + Send + Sync + Sub<Output = T>,
{
    type Output = Result<Grid2d<T>, Grid2dError>;

    fn sub(self, other: &Grid2d<T>) -> Self::Output {
        if self.cols() == other.cols() && self.rows() == other.rows() {
            let cells = self
                .cells
                .iter()
                .zip(other.cells.iter())
                .map(|(a, b)| a.clone() - b.clone())
                .collect::<Vec<T>>();
            Ok(Grid2d::with_cells(self.cols(), cells))
        } else {
            Err(Grid2dError::DifferentDimensions(
                (self.cols(), self.rows()),
                (other.cols(), other.rows()),
            ))
        }
    }
}

impl<T> Mul for &Grid2d<T>
where
    T: Clone + Send + Sync + Mul<Output = T>,
{
    type Output = Result<Grid2d<T>, Grid2dError>;

    fn mul(self, other: &Grid2d<T>) -> Self::Output {
        if self.cols() == other.cols() && self.rows() == other.rows() {
            let cells = self
                .cells
                .iter()
                .zip(other.cells.iter())
                .map(|(a, b)| a.clone() * b.clone())
                .collect::<Vec<T>>();
            Ok(Grid2d::with_cells(self.cols(), cells))
        } else {
            Err(Grid2dError::DifferentDimensions(
                (self.cols(), self.rows()),
                (other.cols(), other.rows()),
            ))
        }
    }
}

impl<T> Div for &Grid2d<T>
where
    T: Clone + Send + Sync + Div<Output = T>,
{
    type Output = Result<Grid2d<T>, Grid2dError>;

    fn div(self, other: &Grid2d<T>) -> Self::Output {
        if self.cols() == other.cols() && self.rows() == other.rows() {
            let cells = self
                .cells
                .iter()
                .zip(other.cells.iter())
                .map(|(a, b)| a.clone() / b.clone())
                .collect::<Vec<T>>();
            Ok(Grid2d::with_cells(self.cols(), cells))
        } else {
            Err(Grid2dError::DifferentDimensions(
                (self.cols(), self.rows()),
                (other.cols(), other.rows()),
            ))
        }
    }
}

impl<T> Neg for &Grid2d<T>
where
    T: Clone + Send + Sync + Neg<Output = T>,
{
    type Output = Grid2d<T>;

    fn neg(self) -> Self::Output {
        let cells = self.cells.iter().map(|v| -v.clone()).collect::<Vec<T>>();
        Grid2d::with_cells(self.cols(), cells)
    }
}

impl<T> IntoIterator for Grid2d<T>
where
    T: Clone + Send + Sync,
{
    type Item = T;
    type IntoIter = ::std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.cells.into_iter()
    }
}

impl<I, T> From<(usize, I)> for Grid2d<T>
where
    I: Iterator<Item = T>,
    T: Clone + Send + Sync,
{
    fn from((cols, iter): (usize, I)) -> Self {
        Self::with_cells(cols, iter.collect::<Vec<T>>())
    }
}

impl<T> Into<Vec<T>> for Grid2d<T>
where
    T: Clone + Send + Sync,
{
    fn into(self) -> Vec<T> {
        self.cells
    }
}

impl<T> Into<(usize, usize, Vec<T>)> for Grid2d<T>
where
    T: Clone + Send + Sync,
{
    fn into(self) -> (usize, usize, Vec<T>) {
        self.into_inner()
    }
}
