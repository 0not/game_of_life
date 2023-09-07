use rand::Rng;
use rayon::prelude::*;
use std::collections::HashSet;

pub type Int = i32;
pub type Pos = (Int, Int);
pub type CellSetType = HashSet<Pos>;

pub struct CellSet(pub CellSetType);

impl std::ops::Deref for CellSet {
    type Target = CellSetType;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<CellSetType> for CellSet {
    fn from(set: CellSetType) -> Self {
        CellSet(set)
    }
}

// TODO: Add 'builder' pattern support and support for translating structures.
impl CellSet {
    #[allow(dead_code)]
    pub fn new() -> Self {
        CellSet(CellSetType::new())
    }

    #[allow(dead_code)]
    pub fn random(number: usize, extents: (Int, Int, Int, Int)) -> Self {
        let (x, y, width, height) = extents;
        let mut rng = rand::thread_rng();

        let mut cells = CellSetType::new();

        for _n in 0..number {
            let a = rng.gen_range(x..(x + width));
            let b = rng.gen_range(y..(y + height));
            cells.insert((a, b));
        }

        CellSet(cells)
    }

    #[allow(dead_code)]
    pub fn solid_rect(extents: (Int, Int, Int, Int)) -> Self {
        let (x, y, width, height) = extents;

        let mut cells = CellSetType::new();

        for ix in x..(x + width) {
            for iy in y..(y + height) {
                cells.insert((ix, iy));
            }
        }

        CellSet(cells)
    }

    /// Create a hollow rectangle. I don't know if there is a use case for this,
    /// since all internal cells will die from overpopulation anyway.
    #[allow(dead_code)]
    pub fn hollow_rect(wall_thick: Int, extents: (Int, Int, Int, Int)) -> Self {
        let (x, y, width, height) = extents;

        let mut cells = CellSetType::new();
        let mut cells_inner = CellSetType::new();

        for ix in x..(x + width) {
            for iy in y..(y + height) {
                cells.insert((ix, iy));
            }
        }

        for ix in (x + wall_thick)..(x + width - wall_thick) {
            for iy in (y + wall_thick)..(y + height - wall_thick) {
                cells_inner.insert((ix, iy));
            }
        }

        CellSet(cells.difference(&cells_inner).map(|c| *c).collect())
    }

    #[allow(dead_code)]
    pub fn blinker() -> Self {
        CellSet(CellSetType::from([(-1, 0), (0, 0), (1, 0)]))
    }

    #[allow(dead_code)]
    pub fn glider() -> Self {
        CellSet(CellSetType::from([
            (-1, 0),
            (0, 0),
            (1, 0),
            (1, -1),
            (0, -2),
        ]))
    }

    #[allow(dead_code)]
    fn pentadecathlon() -> Self {
        CellSet(CellSetType::from([
            (0, 0),
            (0, -1),
            (1, -1),
            (1, -2),
            (3, 0),
            (6, 0),
            (6, -1),
            (6, -2),
            (7, -1),
        ]))
    }

    pub fn get_neighbors_keys(&self, key: &Pos) -> CellSetType {
        let (x, y) = *key;
        CellSetType::from([
            (x - 1, y - 1),
            (x, y - 1),
            (x + 1, y - 1),
            (x - 1, y),
            (x + 1, y),
            (x - 1, y + 1),
            (x, y + 1),
            (x + 1, y + 1),
        ])
    }

    pub fn get_self_and_neighbors_keys(&self, key: &Pos) -> CellSetType {
        let mut keys = self.get_neighbors_keys(key);
        keys.insert(*key);
        keys
    }

    pub fn count_neighbors(&self, key: &Pos) -> u8 {
        let neighbors = self.get_neighbors_keys(key);

        // Count neighbors
        neighbors.iter().map(|k| self.0.contains(k) as u8).sum()
    }

    pub fn update_cells(&self) -> Self {
        // Create list of all cells to check
        let cells_to_check: CellSetType = self
            .par_iter()
            .map(|k| self.get_self_and_neighbors_keys(k))
            .flatten()
            .collect();

        // Loop over all live cells and their neighbors
        let next_live_cells: CellSetType = cells_to_check
            .par_iter()
            .map(|key| {
                let count = self.count_neighbors(&key);
                let alive = self.contains(&key);

                // The three rules of Game of Life
                return if alive && (count == 2 || count == 3) {
                    // Stay alive
                    Some(*key)
                } else if !alive && count == 3 {
                    // Come alive
                    Some(*key)
                } else {
                    None
                };
            })
            .into_par_iter()
            .flatten() // Remove Nones and "unwrap" Somes
            .collect();

        let next_cells = Self::from(next_live_cells);

        // Replace existing cells with next generation
        next_cells
    }
}
