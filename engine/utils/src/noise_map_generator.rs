use crate::{grid_2d::Grid2d, Scalar};
use noise::{NoiseFn, OpenSimplex};
use std::ops::Range;

#[derive(Clone)]
pub struct NoiseMapGenerator {
    seed: u32,
    chunk_size: usize,
    zoom: Scalar,
    generator: OpenSimplex,
}

impl NoiseMapGenerator {
    pub fn new(seed: u32, chunk_size: usize, zoom: Scalar) -> Self {
        Self {
            seed,
            chunk_size: chunk_size.max(1),
            zoom,
            generator: OpenSimplex::new(seed),
        }
    }

    pub fn seed(&self) -> u32 {
        self.seed
    }

    pub fn chunk_size(&self) -> usize {
        self.chunk_size
    }

    pub fn zoom(&self) -> Scalar {
        self.zoom
    }

    pub fn sample_raw(&self, x: Scalar, y: Scalar, z: Scalar) -> Scalar {
        self.generator.get([x as f64, y as f64, z as f64]) as Scalar + 1.0 * 0.5
    }

    pub fn sample(
        &self,
        coord: (isize, isize),
        (col, row): (usize, usize),
        margin: usize,
        depth: Scalar,
    ) -> Scalar {
        let col = col % (self.chunk_size + margin * 2);
        let row = row % (self.chunk_size + margin * 2);
        let x = self.zoom
            * (coord.0 as Scalar * self.chunk_size as Scalar + col as Scalar - margin as Scalar)
            / self.chunk_size as Scalar;
        let y = self.zoom
            * (coord.1 as Scalar * self.chunk_size as Scalar + row as Scalar - margin as Scalar)
            / self.chunk_size as Scalar;
        self.sample_raw(x, y, depth)
    }

    #[inline]
    pub fn build_chunk(&self, coord: (isize, isize), margin: usize) -> Grid2d<Scalar> {
        self.build_chunk_with_depth(coord, margin, 0.0)
    }

    pub fn build_chunk_with_depth(
        &self,
        coord: (isize, isize),
        margin: usize,
        depth: Scalar,
    ) -> Grid2d<Scalar> {
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
    pub fn build_chunks(&self, coord: Range<(isize, isize)>) -> Grid2d<Scalar> {
        self.build_chunks_with_depth(coord, 0.0)
    }

    pub fn build_chunks_with_depth(
        &self,
        coord: Range<(isize, isize)>,
        depth: Scalar,
    ) -> Grid2d<Scalar> {
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

        fn print_chunk(chunk: Grid2d<Scalar>) {
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
