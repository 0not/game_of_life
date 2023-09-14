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
        let cells_to_check = self
            .par_iter()
            .map(|k| self.get_self_and_neighbors_keys(k))
            .flatten();

        // Loop over all live cells and their neighbors
        let next_live_cells: CellSetType = cells_to_check
            .map(|key| {
                let count = self.count_neighbors(&key);
                let alive = self.contains(&key);

                // The three rules of Game of Life
                return if alive && (count == 2 || count == 3) {
                    // Stay alive
                    Some(key)
                } else if !alive && count == 3 {
                    // Come alive
                    Some(key)
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

pub struct CellSetBuilder {
    draft: CellSetType,
    committed: CellSetType,
}

impl CellSetBuilder {
    pub fn new() -> Self {
        Self {
            draft: CellSetType::new(),
            committed: CellSetType::new(),
        }
    }

    pub fn commit(mut self) -> Self {
        self.committed = self.committed.union(&self.draft).map(|c| *c).collect();
        self.draft = CellSetType::new();
        self
    }

    pub fn build(mut self) -> CellSet {
        self = self.commit();
        CellSet(self.committed)
    }

    pub fn translate(mut self, trans_vec: Pos) -> Self {
        let (x, y) = trans_vec;
        self.draft = self.draft.iter().map(|c| (c.0 + x, c.1 + y)).collect();
        self
    }

    // TODO: Add support for rotations
    #[allow(dead_code)]
    pub fn glider(mut self) -> Self {
        self = self.commit();

        self.draft = CellSetType::from([(-1, 0), (0, 0), (1, 0), (1, -1), (0, -2)]);
        self
    }

    #[allow(dead_code)]
    pub fn blinker(mut self) -> Self {
        self = self.commit();

        self.draft = CellSetType::from([(-1, 0), (0, 0), (1, 0)]);
        self
    }

    #[allow(dead_code)]
    fn pentadecathlon(mut self) -> Self {
        self = self.commit();

        self.draft = CellSetType::from([
            (0, 0),
            (0, -1),
            (1, -1),
            (1, -2),
            (3, 0),
            (6, 0),
            (6, -1),
            (6, -2),
            (7, -1),
        ]);
        self
    }

    #[allow(dead_code)]
    pub fn random(mut self, number: usize, dim: Pos) -> Self {
        let (width, height) = dim;
        let mut rng = rand::thread_rng();

        let mut cells = CellSetType::new();

        for _n in 0..number {
            let a = rng.gen_range(0..width);
            let b = rng.gen_range(0..height);
            cells.insert((a, b));
        }

        self = self.commit();

        self.draft = cells;
        self
    }

    #[allow(dead_code)]
    pub fn solid_rect(mut self, dim: Pos) -> Self {
        let (width, height) = dim;

        let mut cells = CellSetType::new();

        for ix in 0..width {
            for iy in 0..height {
                cells.insert((ix, iy));
            }
        }

        self = self.commit();

        self.draft = cells;
        self
    }

    /// Create a hollow rectangle. I don't know if there is a use case for this,
    /// since all internal cells will die from overpopulation anyway.
    #[allow(dead_code)]
    pub fn hollow_rect(mut self, wall_thick: Int, dim: Pos) -> Self {
        let (width, height) = dim;

        let mut cells = CellSetType::new();
        let mut cells_inner = CellSetType::new();

        for ix in 0..width {
            for iy in 0..height {
                cells.insert((ix, iy));
            }
        }

        for ix in wall_thick..(width - wall_thick) {
            for iy in (wall_thick)..(height - wall_thick) {
                cells_inner.insert((ix, iy));
            }
        }

        self = self.commit();

        self.draft = cells.difference(&cells_inner).map(|c| *c).collect();
        self
    }
}
