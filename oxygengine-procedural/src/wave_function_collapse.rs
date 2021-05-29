use oxygengine_utils::{grid_2d::Grid2d, Scalar};
#[cfg(feature = "parallel")]
use rayon::prelude::*;
use std::{
    collections::{HashSet, VecDeque},
    iter::FromIterator,
};

const NEIGHBOR_COORD_DIRS: [Direction; 4] = [
    Direction::Left,
    Direction::Right,
    Direction::Top,
    Direction::Bottom,
];

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Debug, Clone, Copy)]
pub enum WaveFunctionCollapseError {
    /// pattern index
    FoundPatternWithZeroFrequency(usize),
    /// pattern index
    FoundEmptyPattern(usize),
    /// (col, row)
    SuperpositionCellHasNoPattern(usize, usize),
    FoundUncollapsedCell,
    FoundImpossibleInitialState,
    BuilderInProgress,
}

#[derive(Debug, Clone)]
pub enum WaveFunctionCollapseResult<T> {
    Incomplete,
    Collapsed(Grid2d<T>),
    Impossible,
}

#[derive(Debug, Default, Clone)]
pub struct WaveFunctionCollapseModel<T>
where
    T: Clone + Send + Sync + PartialEq,
{
    /// [(pattern, weight)]
    patterns: Vec<(Grid2d<T>, Scalar)>,
    /// [[(pattern, direction)]]
    neighbors: Vec<HashSet<(usize, Direction)>>,
}

impl<T> WaveFunctionCollapseModel<T>
where
    T: Clone + Send + Sync + PartialEq,
{
    pub fn from_patterns(
        patterns: Vec<(Grid2d<T>, usize)>,
    ) -> Result<Self, WaveFunctionCollapseError> {
        for (i, (p, f)) in patterns.iter().enumerate() {
            if *f == 0 {
                return Err(WaveFunctionCollapseError::FoundPatternWithZeroFrequency(i));
            } else if p.is_empty() {
                return Err(WaveFunctionCollapseError::FoundEmptyPattern(i));
            }
        }
        let total = patterns.iter().fold(0, |a, (_, f)| a + f) as Scalar;
        let mut unique = Vec::with_capacity(patterns.len());
        for (p, f) in patterns {
            if let Some((_, f2)) = unique.iter_mut().find(|(p2, _)| &p == p2) {
                *f2 += f;
            } else {
                unique.push((p, f));
            }
        }
        let patterns = unique
            .into_iter()
            .map(|(p, f)| (p, f as Scalar / total))
            .collect::<Vec<_>>();
        let mut neighbors = vec![HashSet::default(); patterns.len()];
        for (ai, (ap, _)) in patterns.iter().enumerate() {
            let (ac, ar) = ap.size();
            for (bi, (bp, _)) in patterns.iter().enumerate() {
                let (bc, br) = bp.size();
                if ar == br {
                    if bp.has_union_with(ap, 1, 0) {
                        neighbors[ai].insert((bi, Direction::Left));
                        neighbors[bi].insert((ai, Direction::Right));
                    }
                    if ap.has_union_with(bp, 1, 0) {
                        neighbors[ai].insert((bi, Direction::Right));
                        neighbors[bi].insert((ai, Direction::Left));
                    }
                }
                if ac == bc {
                    if bp.has_union_with(ap, 0, 1) {
                        neighbors[ai].insert((bi, Direction::Top));
                        neighbors[bi].insert((ai, Direction::Bottom));
                    }
                    if ap.has_union_with(bp, 0, 1) {
                        neighbors[ai].insert((bi, Direction::Bottom));
                        neighbors[bi].insert((ai, Direction::Top));
                    }
                }
            }
        }
        Ok(Self {
            patterns,
            neighbors,
        })
    }

    pub fn from_views(
        sample_size: (usize, usize),
        seamless: bool,
        views: Vec<Grid2d<Option<T>>>,
    ) -> Result<Self, WaveFunctionCollapseError> {
        let f = |w: Grid2d<&Option<T>>| {
            let items = w
                .iter()
                .filter_map(|c| c.as_ref().cloned())
                .collect::<Vec<_>>();
            if items.len() == w.len() {
                Some((Grid2d::with_cells(w.cols(), items), 1))
            } else {
                None
            }
        };
        let patterns = views
            .into_iter()
            .flat_map(|view| {
                if seamless {
                    view.windows_seamless(sample_size)
                        .filter_map(f)
                        .collect::<Vec<_>>()
                } else {
                    view.windows(sample_size).filter_map(f).collect::<Vec<_>>()
                }
            })
            .collect();
        Self::from_patterns(patterns)
    }

    /// [(pattern, weight)]
    pub fn patterns(&self) -> &[(Grid2d<T>, Scalar)] {
        &self.patterns
    }

    /// [[pattern, direction]]
    pub fn neighbors(&self) -> &[HashSet<(usize, Direction)>] {
        &self.neighbors
    }
}

#[derive(Debug, Clone)]
struct Cell {
    patterns: HashSet<usize>,
    entropy: Scalar,
}

#[derive(Debug, Clone, Copy)]
enum BuilderPhase {
    /// current cell index
    Process(usize),
    Done,
    Error(WaveFunctionCollapseError),
}

#[derive(Clone)]
pub struct WaveFunctionCollapseSolverBuilder<T>
where
    T: Clone + Send + Sync + PartialEq,
{
    model: WaveFunctionCollapseModel<T>,
    superposition: [Grid2d<Cell>; 2],
    current: usize,
    phase: BuilderPhase,
    cells_per_step: usize,
}

impl<T> WaveFunctionCollapseSolverBuilder<T>
where
    T: Clone + Send + Sync + PartialEq,
{
    fn new(
        model: WaveFunctionCollapseModel<T>,
        superposition: Grid2d<Vec<T>>,
        cells_per_step: Option<usize>,
    ) -> Result<Self, WaveFunctionCollapseError> {
        let (cols, rows) = superposition.size();
        let cells = superposition
            .iter_view((0, 0)..(cols, rows))
            .map(|(col, row, cells)| {
                let patterns = cells
                    .iter()
                    .flat_map(|cell| {
                        model
                            .patterns()
                            .iter()
                            .enumerate()
                            .filter_map(|(index, (pattern, _))| {
                                let pattern_cell = pattern.cell(0, 0).unwrap();
                                if cell == pattern_cell {
                                    Some(index)
                                } else {
                                    None
                                }
                            })
                            .collect::<HashSet<_>>()
                    })
                    .collect::<HashSet<_>>();
                if patterns.is_empty() {
                    Err(WaveFunctionCollapseError::SuperpositionCellHasNoPattern(
                        col, row,
                    ))
                } else {
                    let entropy = calculate_entropy(&model, &patterns);
                    Ok(Cell { patterns, entropy })
                }
            })
            .collect::<Result<Vec<_>, _>>()?;
        let max_patterns = cells
            .iter()
            .map(|cell| cell.patterns.len())
            .max_by(|a, b| a.cmp(&b))
            .unwrap_or(1);
        let cells_per_step = if let Some(cells_per_step) = cells_per_step {
            cells_per_step
        } else if max_patterns > 0 {
            cells.len() / max_patterns
        } else {
            cells.len()
        }
        .max(1);
        let superposition = Grid2d::with_cells(cols, cells);
        Ok(Self {
            model,
            superposition: [superposition.clone(), superposition],
            current: 0,
            phase: BuilderPhase::Process(0),
            cells_per_step,
        })
    }

    /// true if has to continue (is not done and has no error)
    pub fn process(&mut self) -> bool {
        match self.phase {
            BuilderPhase::Done | BuilderPhase::Error(_) => false,
            BuilderPhase::Process(mut index) => {
                let mut remaining = self.cells_per_step;
                let mut reduced = false;
                let cols = self.source().cols();
                let rows = self.source().rows();
                let count = self.source().len();
                while index < count && remaining > 0 {
                    let col = index % cols;
                    let row = index / cols;
                    let patterns = &self.source().cell(col, row).unwrap().patterns;
                    let count = patterns.len();
                    match count {
                        0 | 1 => {
                            let cell = Cell {
                                patterns: patterns.clone(),
                                entropy: 0.0,
                            };
                            self.target().set(col, row, cell)
                        }
                        _ => {
                            let samples = [
                                self.source().cell((cols + col - 1) % cols, row).unwrap(),
                                self.source().cell((col + 1) % cols, row).unwrap(),
                                self.source().cell(col, (rows + row - 1) % rows).unwrap(),
                                self.source().cell(col, (row + 1) % rows).unwrap(),
                            ];
                            #[cfg(not(feature = "parallel"))]
                            let patterns = patterns.iter();
                            #[cfg(feature = "parallel")]
                            let patterns = patterns.par_iter();
                            let patterns = patterns
                                .filter(|index| {
                                    let neighbors = self.model.neighbors().get(**index).unwrap();
                                    if neighbors.is_empty() {
                                        return false;
                                    }
                                    NEIGHBOR_COORD_DIRS.iter().enumerate().all(|(i, d)| {
                                        samples[i].patterns.iter().any(|n| {
                                            neighbors.iter().any(|(neighbor, direction)| {
                                                direction == d && neighbor == n
                                            })
                                        })
                                    })
                                })
                                .cloned()
                                .collect::<HashSet<_>>();
                            if patterns.is_empty() {
                                self.phase = BuilderPhase::Error(
                                    WaveFunctionCollapseError::FoundImpossibleInitialState,
                                );
                                return false;
                            } else if patterns.len() < count {
                                reduced = true;
                            }
                            let entropy = calculate_entropy(&self.model, &patterns);
                            self.target().set(col, row, Cell { patterns, entropy });
                        }
                    }
                    index += 1;
                    remaining -= 1;
                }
                if index == count {
                    if reduced {
                        self.phase = BuilderPhase::Process(0);
                        self.current = (self.current + 1) % 2;
                        true
                    } else {
                        self.phase = BuilderPhase::Done;
                        false
                    }
                } else {
                    self.phase = BuilderPhase::Process(index);
                    true
                }
            }
        }
    }

    /// (current, max)
    pub fn progress(&self) -> (usize, usize) {
        let count = self.source().len();
        match self.phase {
            BuilderPhase::Done | BuilderPhase::Error(_) => (count, count),
            BuilderPhase::Process(index) => (index, count),
        }
    }

    pub fn build(self) -> Result<WaveFunctionCollapseSolver<T>, WaveFunctionCollapseError> {
        match self.phase {
            BuilderPhase::Error(error) => Err(error),
            BuilderPhase::Done => {
                let count = self.source().len();
                Ok(WaveFunctionCollapseSolver {
                    superposition: self.source().clone(),
                    model: self.model,
                    cached_progress: 0,
                    cached_open: VecDeque::with_capacity(count),
                    lately_updated: HashSet::with_capacity(count),
                })
            }
            BuilderPhase::Process(_) => Err(WaveFunctionCollapseError::BuilderInProgress),
        }
    }

    fn source(&self) -> &Grid2d<Cell> {
        &self.superposition[self.current]
    }

    fn target(&mut self) -> &mut Grid2d<Cell> {
        &mut self.superposition[(self.current + 1) % 2]
    }
}

#[derive(Clone)]
pub struct WaveFunctionCollapseSolver<T>
where
    T: Clone + Send + Sync + PartialEq,
{
    model: WaveFunctionCollapseModel<T>,
    superposition: Grid2d<Cell>,
    cached_progress: usize,
    cached_open: VecDeque<(usize, usize)>,
    lately_updated: HashSet<(usize, usize)>,
}

impl<T> std::fmt::Debug for WaveFunctionCollapseSolver<T>
where
    T: Clone + Send + Sync + PartialEq + std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WaveFunctionCollapseSolver")
            .field("model", &self.model)
            .field("superposition", &self.superposition)
            .field("cached_progress", &self.cached_progress)
            .field("cached_open", &self.cached_open)
            .field("lately_updated", &self.lately_updated)
            .finish()
    }
}

impl<T> WaveFunctionCollapseSolver<T>
where
    T: Clone + Send + Sync + PartialEq,
{
    pub fn lately_updated(&self) -> &HashSet<(usize, usize)> {
        &self.lately_updated
    }

    pub fn lately_updated_uncollapsed_cells<V>(&self) -> Vec<(usize, usize, V)>
    where
        V: FromIterator<T>,
    {
        self.lately_updated
            .iter()
            .filter_map(|(col, row)| {
                self.superposition.cell(*col, *row).map(|cell| {
                    let items = cell
                        .patterns
                        .iter()
                        .map(|index| self.model.patterns()[*index].0.cell(0, 0).unwrap().clone())
                        .collect::<V>();
                    (*col, *row, items)
                })
            })
            .collect()
    }

    pub fn lately_updated_collapsed_cells(&self) -> Vec<(usize, usize, T)> {
        self.lately_updated
            .iter()
            .filter_map(|(col, row)| {
                if let Some(cell) = self.superposition.cell(*col, *row) {
                    if cell.patterns.len() == 1 {
                        let index = *cell.patterns.iter().next().unwrap();
                        let item = self.model.patterns()[index].0.cell(0, 0).unwrap().clone();
                        Some((*col, *row, item))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn build(
        model: WaveFunctionCollapseModel<T>,
        superposition: Grid2d<Vec<T>>,
        cells_per_step: Option<usize>,
    ) -> Result<WaveFunctionCollapseSolverBuilder<T>, WaveFunctionCollapseError> {
        WaveFunctionCollapseSolverBuilder::new(model, superposition, cells_per_step)
    }

    pub fn new(
        model: WaveFunctionCollapseModel<T>,
        superposition: Grid2d<Vec<T>>,
    ) -> Result<Self, WaveFunctionCollapseError> {
        let count = superposition.len();
        let mut builder = Self::build(model, superposition, Some(count))?;
        while builder.process() {}
        builder.build()
    }

    pub fn new_inspect<F>(
        model: WaveFunctionCollapseModel<T>,
        superposition: Grid2d<Vec<T>>,
        cells_per_step: Option<usize>,
        mut f: F,
    ) -> Result<Self, WaveFunctionCollapseError>
    where
        F: FnMut(usize, usize),
    {
        let mut builder = Self::build(model, superposition, cells_per_step)?;
        let (p, m) = builder.progress();
        f(p, m);
        while builder.process() {
            let (p, m) = builder.progress();
            f(p, m);
        }
        let (p, m) = builder.progress();
        f(p, m);
        builder.build()
    }

    pub fn collapse<R>(&mut self, gen_range: R) -> WaveFunctionCollapseResult<T>
    where
        R: FnMut(Scalar, Scalar) -> Scalar + Clone,
    {
        loop {
            match self.collapse_step(gen_range.clone()) {
                WaveFunctionCollapseResult::Incomplete => continue,
                result => return result,
            }
        }
    }

    pub fn collapse_with_tries<R>(
        &mut self,
        mut tries: usize,
        gen_range: R,
    ) -> WaveFunctionCollapseResult<T>
    where
        R: FnMut(Scalar, Scalar) -> Scalar + Clone,
    {
        while tries > 0 {
            match self.collapse(gen_range.clone()) {
                WaveFunctionCollapseResult::Impossible => {
                    tries -= 1;
                    continue;
                }
                result => return result,
            }
        }
        WaveFunctionCollapseResult::Impossible
    }

    pub fn collapse_inspect<R, F>(
        &mut self,
        gen_range: R,
        mut f: F,
    ) -> WaveFunctionCollapseResult<T>
    where
        F: FnMut(usize, usize, &Self),
        R: FnMut(Scalar, Scalar) -> Scalar + Clone,
    {
        loop {
            match self.collapse_step(gen_range.clone()) {
                WaveFunctionCollapseResult::Incomplete => {
                    let (p, m) = self.progress();
                    f(p, m, self);
                    continue;
                }
                result => return result,
            }
        }
    }

    pub fn collapse_inspect_with_tries<R, F>(
        &mut self,
        mut tries: usize,
        gen_range: R,
        mut f: F,
    ) -> WaveFunctionCollapseResult<T>
    where
        F: FnMut() -> Box<dyn FnMut(usize, usize, &Self)>,
        R: FnMut(Scalar, Scalar) -> Scalar + Clone,
    {
        while tries > 0 {
            match self.collapse_inspect(gen_range.clone(), f()) {
                WaveFunctionCollapseResult::Impossible => {
                    tries -= 1;
                    continue;
                }
                result => return result,
            }
        }
        WaveFunctionCollapseResult::Impossible
    }

    pub fn collapse_step<R>(&mut self, gen_range: R) -> WaveFunctionCollapseResult<T>
    where
        R: FnMut(Scalar, Scalar) -> Scalar,
    {
        let coord = if let Ok(coord) = self.get_uncollapsed_coord() {
            coord
        } else {
            return WaveFunctionCollapseResult::Impossible;
        };
        let (col, row) = if let Some(coord) = coord {
            coord
        } else if let Ok(collapsed) =
            Self::superposition_to_collapsed_world(&self.model, &self.superposition)
        {
            return WaveFunctionCollapseResult::Collapsed(collapsed);
        } else {
            return WaveFunctionCollapseResult::Impossible;
        };
        self.lately_updated.clear();
        if !self.collapse_cell(col, row, gen_range) {
            return WaveFunctionCollapseResult::Impossible;
        }
        self.lately_updated.insert((col, row));
        let (cols, rows) = self.superposition.size();
        self.cached_open.push_back(((col + cols - 1) % cols, row));
        self.cached_open.push_back(((col + 1) % cols, row));
        self.cached_open.push_back((col, (row + rows - 1) % rows));
        self.cached_open.push_back((col, (row + 1) % rows));
        while !self.cached_open.is_empty() {
            self.partially_reduce_superposition();
        }
        self.cached_progress =
            self.superposition
                .iter()
                .fold(0, |a, c| if c.patterns.len() == 1 { a + 1 } else { a });
        WaveFunctionCollapseResult::Incomplete
    }

    pub fn progress(&self) -> (usize, usize) {
        (self.cached_progress, self.superposition.len())
    }

    fn collapse_cell<R>(&mut self, col: usize, row: usize, mut gen_range: R) -> bool
    where
        R: FnMut(Scalar, Scalar) -> Scalar,
    {
        let patterns = self.model.patterns();
        let cell = self.superposition.cell(col, row).unwrap();
        let total = cell
            .patterns
            .iter()
            .fold(0.0, |accum, index| accum + patterns[*index].1);
        let mut selected = gen_range(0.0, total);
        for index in cell.patterns.iter() {
            let weight = patterns[*index].1;
            if selected <= weight {
                let mut patterns = HashSet::with_capacity(1);
                patterns.insert(*index);
                self.superposition.set(
                    col,
                    row,
                    Cell {
                        patterns,
                        entropy: 0.0,
                    },
                );
                return true;
            }
            selected -= weight;
        }
        false
    }

    fn get_uncollapsed_coord(&self) -> Result<Option<(usize, usize)>, ()> {
        if self
            .superposition
            .iter()
            .any(|cell| cell.patterns.is_empty())
        {
            return Err(());
        }
        let cols = self.superposition.cols();
        let result = {
            let mut result = None;
            for (index, cell) in self.superposition.iter().enumerate() {
                let col = index % cols;
                let row = index / cols;
                if cell.patterns.len() > 1 {
                    if let Some((_, _, entropy)) = result {
                        if cell.entropy < entropy {
                            result = Some((col, row, cell.entropy));
                        }
                    } else {
                        result = Some((col, row, cell.entropy));
                    }
                }
            }
            result
        };
        Ok(result.map(|(col, row, _)| (col, row)))
    }

    fn partially_reduce_superposition(&mut self) {
        if self.cached_open.is_empty() {
            return;
        }
        let (col, row) = self.cached_open.pop_front().unwrap();
        let (cols, rows) = self.superposition.size();
        let patterns = &self.superposition.cell(col, row).unwrap().patterns;
        let count = patterns.len();
        if count > 1 {
            let samples = [
                self.superposition
                    .cell((cols + col - 1) % cols, row)
                    .unwrap(),
                self.superposition.cell((col + 1) % cols, row).unwrap(),
                self.superposition
                    .cell(col, (rows + row - 1) % rows)
                    .unwrap(),
                self.superposition.cell(col, (row + 1) % rows).unwrap(),
            ];
            let neighbors = self.model.neighbors();
            #[cfg(not(feature = "parallel"))]
            let patterns = patterns.iter();
            #[cfg(feature = "parallel")]
            let patterns = patterns.par_iter();
            let patterns = patterns
                .filter(|index| {
                    let neighbors = neighbors.get(**index).unwrap();
                    if neighbors.is_empty() {
                        return false;
                    }
                    NEIGHBOR_COORD_DIRS.iter().enumerate().all(|(i, d)| {
                        samples[i].patterns.iter().any(|n| {
                            neighbors
                                .iter()
                                .any(|(neighbor, direction)| direction == d && neighbor == n)
                        })
                    })
                })
                .cloned()
                .collect::<HashSet<_>>();
            if patterns.len() < count {
                self.lately_updated.insert((col, row));
                let coord = ((col + cols - 1) % cols, row);
                if samples[0].patterns.len() > 1 && !self.cached_open.contains(&coord) {
                    self.cached_open.push_back(coord);
                }
                let coord = ((col + 1) % cols, row);
                if samples[1].patterns.len() > 1 && !self.cached_open.contains(&coord) {
                    self.cached_open.push_back(coord);
                }
                let coord = (col, (row + rows - 1) % rows);
                if samples[2].patterns.len() > 1 && !self.cached_open.contains(&coord) {
                    self.cached_open.push_back(coord);
                }
                let coord = (col, (row + 1) % rows);
                if samples[3].patterns.len() > 1 && !self.cached_open.contains(&coord) {
                    self.cached_open.push_back(coord);
                }
                let entropy = calculate_entropy(&self.model, &patterns);
                self.superposition.set(col, row, Cell { patterns, entropy });
            }
        }
    }

    pub fn get_uncollapsed_world(&self) -> Grid2d<Vec<T>> {
        let cols = self.superposition.cols();
        let cells = self
            .superposition
            .iter()
            .map(|cell| {
                cell.patterns
                    .iter()
                    .map(|index| self.model.patterns()[*index].0.cell(0, 0).unwrap().clone())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        Grid2d::with_cells(cols, cells)
    }

    fn superposition_to_collapsed_world(
        model: &WaveFunctionCollapseModel<T>,
        superposition: &Grid2d<Cell>,
    ) -> Result<Grid2d<T>, WaveFunctionCollapseError> {
        let cols = superposition.cols();
        let cells = superposition
            .iter()
            .map(|cell| {
                if cell.patterns.len() == 1 {
                    let index = cell.patterns.iter().next().unwrap();
                    Ok(model.patterns()[*index].0.cell(0, 0).unwrap().clone())
                } else {
                    Err(WaveFunctionCollapseError::FoundUncollapsedCell)
                }
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Grid2d::with_cells(cols, cells))
    }
}

fn calculate_entropy<T>(model: &WaveFunctionCollapseModel<T>, patterns: &HashSet<usize>) -> Scalar
where
    T: Clone + Send + Sync + PartialEq,
{
    if patterns.is_empty() {
        return 0.0;
    }
    let mut total_weight = 0.0;
    let mut total_weight_log = 0.0;
    for index in patterns {
        let weight = model.patterns()[*index].1;
        total_weight += weight;
        total_weight_log += weight * weight.log2();
    }
    total_weight.log2() - (total_weight_log / total_weight)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{thread_rng, Rng};
    use std::time::Instant;

    #[allow(dead_code)]
    fn parse_view(data: &str) -> Grid2d<Option<char>> {
        let lines = data
            .split(|c| c == '\n' || c == '\r')
            .filter(|l| !l.is_empty())
            .collect::<Vec<_>>();
        let cols = lines.iter().fold(0, |a, l| a.max(l.len()));
        let rows = lines.len();
        let mut result = Grid2d::new(cols, rows, None);
        for (row, line) in lines.into_iter().enumerate() {
            for (col, character) in line.chars().enumerate() {
                if !character.is_whitespace() {
                    result.set(col, row, Some(character));
                }
            }
        }
        print_view("= VIEW:", &result);
        result
    }

    #[allow(dead_code)]
    fn print_view(msg: &str, pattern: &Grid2d<Option<char>>) {
        println!("{}", msg);
        for row in 0..pattern.rows() {
            for cell in pattern.get_row_cells(row).unwrap() {
                if let Some(cell) = cell {
                    print!("{}", cell);
                } else {
                    print!(" ");
                }
            }
            println!();
        }
    }

    #[allow(dead_code)]
    fn print_collapsed_world(msg: &str, world: &Grid2d<char>) {
        println!("{}", msg);
        for row in 0..world.rows() {
            for cell in world.get_row_cells(row).unwrap() {
                print!("{}", cell);
            }
            println!();
        }
    }

    #[allow(dead_code)]
    fn print_uncollapsed_world(msg: &str, world: &Grid2d<Vec<char>>, uncertain: char) {
        println!("{}", msg);
        for row in 0..world.rows() {
            for cell in world.get_row_cells(row).unwrap() {
                if cell.len() == 1 {
                    print!("{}", cell[0]);
                } else {
                    print!("{}", uncertain);
                }
            }
            println!();
        }
    }

    #[allow(dead_code)]
    fn print_pattern(msg: &str, pattern: &Grid2d<char>) {
        println!("{}", msg);
        for row in 0..pattern.rows() {
            for cell in pattern.get_row_cells(row).unwrap() {
                print!("{}", cell);
            }
            println!();
        }
    }

    #[test]
    #[cfg(feature = "longrun")]
    fn test_general() {
        let view = parse_view(include_str!("../resources/view.txt"));
        let values = {
            let mut values = view.iter().filter_map(|c| c.clone()).collect::<Vec<_>>();
            values.sort();
            values.dedup();
            values
        };
        println!("= VALUES: {:?}", values);
        let model = WaveFunctionCollapseModel::from_views((3, 3), true, vec![view]).unwrap();
        let world = Grid2d::new(75, 75, values);
        let timer = Instant::now();
        let mut timer2 = Instant::now();
        let mut solver = WaveFunctionCollapseSolver::new_inspect(model, world, None, |p, m| {
            if timer2.elapsed().as_millis() >= 400 {
                timer2 = Instant::now();
                println!(
                    "= INITIALIZE: {} / {} ({}%)",
                    p,
                    m,
                    100.0 * p as Scalar / m as Scalar
                );
            }
        })
        .unwrap();
        println!("= INITIALIZED IN: {:?}", timer.elapsed());
        let timer = Instant::now();
        let mut timer2 = Instant::now();
        let mut rng = thread_rng();
        let mut max_changes = 0;
        let result = solver.collapse_inspect(
            move |f, t| rng.gen_range(f..t),
            |p, m, s| {
                max_changes = max_changes.max(s.lately_updated().len());
                if timer2.elapsed().as_millis() >= 400 {
                    timer2 = Instant::now();
                    println!();
                    println!();
                    print_uncollapsed_world(
                        "= UNCOLLAPSED WORLD:",
                        &s.get_uncollapsed_world(),
                        ' ',
                    );
                    println!(
                        "= PROGRESS: {} / {} ({}%)",
                        p,
                        m,
                        100.0 * p as Scalar / m as Scalar
                    )
                }
            },
        );
        match result {
            WaveFunctionCollapseResult::Collapsed(world) => {
                println!();
                println!();
                println!(
                    "= COLLAPSED IN: {:?} | MAX CHANGES: {}",
                    timer.elapsed(),
                    max_changes
                );
                print_collapsed_world("= COLLAPSED WORLD:", &world);
            }
            _ => panic!("= IMPOSSIBLE WORLD"),
        }
    }
}
