use navmesh::*;
use std::collections::HashMap;

/// ECS resource that holds and manages nav grids.
#[derive(Debug, Default)]
pub struct NavGrids(pub(crate) HashMap<NavGridID, NavGrid>);

impl NavGrids {
    /// Register new nav grid.
    ///
    /// # Arguments
    /// * `grid` - nav grid object.
    ///
    /// # Returns
    /// Identifier of registered nav grid.
    #[inline]
    pub fn register(&mut self, grid: NavGrid) -> NavGridID {
        let id = grid.id();
        self.0.insert(id, grid);
        id
    }

    /// Unregister nav grid.
    ///
    /// # Arguments
    /// * `id` - nav grid identifier.
    ///
    /// # Returns
    /// `Some` with nav grid object if nav grid with given identifier was found, `None` otherwise.
    #[inline]
    pub fn unregister(&mut self, id: NavGridID) -> Option<NavGrid> {
        self.0.remove(&id)
    }

    /// Unregister all nav grids.
    #[inline]
    pub fn unregister_all(&mut self) {
        self.0.clear()
    }

    /// Get nav grids iterator.
    #[inline]
    pub fn grids_iter(&self) -> impl Iterator<Item = &NavGrid> {
        self.0.values()
    }

    /// Find nav grid by its identifier.
    ///
    /// # Arguments
    /// * `id` - nav grid identifier.
    ///
    /// # Returns
    /// `Some` with nav grid if exists or `None` otherwise.
    #[inline]
    pub fn find_grid(&self, id: NavGridID) -> Option<&NavGrid> {
        self.0.get(&id)
    }

    /// Find nav grid by its identifier.
    ///
    /// # Arguments
    /// * `id` - nav grid identifier.
    ///
    /// # Returns
    /// `Some` with mutable nav grid if exists or `None` otherwise.
    #[inline]
    pub fn find_grid_mut(&mut self, id: NavGridID) -> Option<&mut NavGrid> {
        self.0.get_mut(&id)
    }
}
