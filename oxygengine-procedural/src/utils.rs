use oxygengine_utils::grid_2d::Grid2d;
use std::ops::Range;

pub fn remap_in_ranges(value: f64, from: Range<f64>, to: Range<f64>) -> f64 {
    let f = (value - from.start) / (from.end - from.start);
    (to.end - to.start) * f + to.start
}

pub fn transfer<F>(field_prev: &Grid2d<f64>, field_next: &mut Grid2d<f64>, rate: f64, mut f: F)
where
    F: FnMut(usize, usize, f64) -> f64,
{
    field_next.with(|c, r, _| {
        let value = field_prev[(c, r)];
        let target = f(c, r, value);
        value + (target - value) * rate
    });
}

pub fn diffuse(field_prev: &Grid2d<f64>, field_next: &mut Grid2d<f64>) {
    field_next.with(|col, row, _| {
        let sample_coord = (if col == 0 { 0 } else { 1 }, if row == 0 { 0 } else { 1 });
        let values_sample = field_prev.sample((col, row), 1);
        let values_min = *values_sample
            .iter()
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        let values_max = *values_sample
            .iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        let values_diff = values_max - values_min;
        if values_diff >= 0.0 {
            let energy_sample = values_sample.map(|_, _, v| *v - values_min);
            let energy = energy_sample.iter().fold(0.0, |a, v| a + v);
            let amount = energy / values_sample.len() as f64;
            values_sample[sample_coord] - energy_sample[sample_coord] + amount
        } else {
            values_sample[sample_coord]
        }
    });
    // error correction.
    let before = field_prev.iter().fold(0.0, |a, v| a + v);
    let after = field_next.iter().fold(0.0, |a, v| a + v);
    let diff = (before - after) / field_prev.len() as f64;
    field_next.with(|_, _, value| *value + diff);
}

pub fn diffuse_with_barriers(
    barriers: &Grid2d<f64>,
    field_prev: &Grid2d<f64>,
    field_next: &mut Grid2d<f64>,
) {
    let levels = (barriers + field_prev).unwrap();
    field_next.with(|col, row, _| {
        let sample_coord = (if col == 0 { 0 } else { 1 }, if row == 0 { 0 } else { 1 });
        let barriers_sample = barriers.sample((col, row), 1);
        let values_sample = field_prev.sample((col, row), 1);
        let levels_sample = levels.sample((col, row), 1);
        let levels_min = *levels_sample
            .iter()
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        let levels_max = *levels_sample
            .iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        let capacity_sample = barriers_sample.map(|_, _, v| levels_max - v.max(levels_min));
        let capacity = capacity_sample.iter().fold(0.0, |a, v| a + v);
        if capacity >= 0.0 {
            let energy_sample = Grid2d::from((
                levels_sample.cols(),
                levels_sample
                    .iter()
                    .zip(barriers_sample.iter())
                    .map(|(v, b)| *v - b.max(levels_min)),
            ));
            let energy = energy_sample.iter().fold(0.0, |a, v| a + v);
            let amount = energy * capacity_sample[sample_coord] / capacity;
            values_sample[sample_coord] - energy_sample[sample_coord] + amount
        } else {
            values_sample[sample_coord]
        }
    });
    // error correction.
    let before = field_prev.iter().fold(0.0, |a, v| a + v);
    let after = field_next.iter().fold(0.0, |a, v| a + v);
    let diff = (before - after) / field_prev.len() as f64;
    field_next.with(|_, _, value| *value + diff);
}
