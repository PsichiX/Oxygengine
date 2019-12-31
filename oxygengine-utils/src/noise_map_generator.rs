use crate::grid_2d::Grid2d;
use noise::{NoiseFn, OpenSimplex, Seedable};
use std::ops::Range;

#[derive(Clone)]
pub struct NoiseMapGenerator {
    seed: u32,
    chunk_size: usize,
    zoom: f64,
    generator: OpenSimplex,
}

impl NoiseMapGenerator {
    pub fn new(seed: u32, chunk_size: usize, zoom: f64) -> Self {
        Self {
            seed,
            chunk_size: chunk_size.max(1),
            zoom,
            generator: OpenSimplex::new().set_seed(seed),
        }
    }

    pub fn seed(&self) -> u32 {
        self.seed
    }

    pub fn chunk_size(&self) -> usize {
        self.chunk_size
    }

    pub fn zoom(&self) -> f64 {
        self.zoom
    }

    pub fn sample_raw(&self, x: f64, y: f64, z: f64) -> f64 {
        self.generator.get([x, y, z]) + 1.0 * 0.5
    }

    pub fn sample(
        &self,
        coord: (isize, isize),
        (col, row): (usize, usize),
        margin: usize,
        depth: f64,
    ) -> f64 {
        let col = col % (self.chunk_size + margin * 2);
        let row = row % (self.chunk_size + margin * 2);
        let x = self.zoom * (coord.0 as f64 * self.chunk_size as f64 + col as f64 - margin as f64)
            / self.chunk_size as f64;
        let y = self.zoom * (coord.1 as f64 * self.chunk_size as f64 + row as f64 - margin as f64)
            / self.chunk_size as f64;
        self.sample_raw(x, y, depth)
    }

    #[inline]
    pub fn build_chunk(&self, coord: (isize, isize), margin: usize) -> Grid2d<f64> {
        self.build_chunk_with_depth(coord, margin, 0.0)
    }

    pub fn build_chunk_with_depth(
        &self,
        coord: (isize, isize),
        margin: usize,
        depth: f64,
    ) -> Grid2d<f64> {
        let mut cells = Vec::with_capacity(self.chunk_size * self.chunk_size);
        let mcs = self.chunk_size + margin * 2;
        for row in 0..mcs {
            for col in 0..mcs {
                cells.push(self.sample(coord, (col, row), margin, depth));
            }
        }
        Grid2d::with_cells(mcs, cells)
    }

    #[inline]
    pub fn build_chunks(&self, coord: Range<(isize, isize)>) -> Grid2d<f64> {
        self.build_chunks_with_depth(coord, 0.0)
    }

    pub fn build_chunks_with_depth(&self, coord: Range<(isize, isize)>, depth: f64) -> Grid2d<f64> {
        let xs = (coord.end.0 - coord.start.0) as usize;
        let ys = (coord.end.1 - coord.start.1) as usize;
        let mut cells = Vec::with_capacity(self.chunk_size * self.chunk_size * xs * ys);
        for row in 0..(self.chunk_size * ys) {
            for col in 0..(self.chunk_size * xs) {
                let xc = coord.start.0 + (col / self.chunk_size) as isize;
                let yc = coord.start.1 + (row / self.chunk_size) as isize;
                cells.push(self.sample(
                    (xc, yc),
                    (col % self.chunk_size, row % self.chunk_size),
                    0,
                    depth,
                ));
            }
        }
        Grid2d::with_cells(self.chunk_size * xs, cells)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_ascii() {
        let gen = NoiseMapGenerator::new(0, 8, 4.0);

        fn print_chunk(chunk: Grid2d<f64>) {
            for row in 0..chunk.rows() {
                let cells = chunk
                    .get_row_cells(row)
                    .unwrap()
                    .into_iter()
                    .map(|f| if f >= 0.5 { '#' } else { ' ' })
                    .collect::<String>();
                println!("|{}|", cells);
            }
        }

        // let chunk = gen.build_chunks((-2, 0)..(3, 2));
        let chunk = gen.build_chunk((0, 0), 1);
        print_chunk(chunk.clone());
        println!();
        print_chunk(chunk.get_part((1, 1)..(9, 9)));
    }
}
