use std::collections::HashMap;

pub use navmesh::*;

/// ECS resource that holds and manages nav meshes.
#[derive(Debug, Default)]
pub struct NavMeshesRes(pub(crate) HashMap<NavMeshID, NavMesh>);

impl NavMeshesRes {
    /// Register new nav mesh.
    ///
    /// # Arguments
    /// * `mesh` - nav mesh object.
    ///
    /// # Returns
    /// Identifier of registered nav mesh.
    #[inline]
    pub fn register(&mut self, mesh: NavMesh) -> NavMeshID {
        let id = mesh.id();
        self.0.insert(id, mesh);
        id
    }

    /// Unregister nav mesh.
    ///
    /// # Arguments
    /// * `id` - nav mesh identifier.
    ///
    /// # Returns
    /// `Some` with nav mesh object if nav mesh with given identifier was found, `None` otherwise.
    #[inline]
    pub fn unregister(&mut self, id: NavMeshID) -> Option<NavMesh> {
        self.0.remove(&id)
    }

    /// Unregister all nav meshes.
    #[inline]
    pub fn unregister_all(&mut self) {
        self.0.clear()
    }

    /// Get nav meshes iterator.
    #[inline]
    pub fn meshes_iter(&self) -> impl Iterator<Item = &NavMesh> {
        self.0.values()
    }

    /// Find nav mesh by its identifier.
    ///
    /// # Arguments
    /// * `id` - nav mesh identifier.
    ///
    /// # Returns
    /// `Some` with nav mesh if exists or `None` otherwise.
    #[inline]
    pub fn find_mesh(&self, id: NavMeshID) -> Option<&NavMesh> {
        self.0.get(&id)
    }

    /// Find nav mesh by its identifier.
    ///
    /// # Arguments
    /// * `id` - nav mesh identifier.
    ///
    /// # Returns
    /// `Some` with mutable nav mesh if exists or `None` otherwise.
    #[inline]
    pub fn find_mesh_mut(&mut self, id: NavMeshID) -> Option<&mut NavMesh> {
        self.0.get_mut(&id)
    }

    /// Find closest point on nav meshes.
    ///
    /// # Arguments
    /// * `point` - query point.
    /// * `query` - query quality.
    ///
    /// # Returns
    /// `Some` with nav mesh identifier and point on nav mesh if found or `None` otherwise.
    pub fn closest_point(&self, point: NavVec3, query: NavQuery) -> Option<(NavMeshID, NavVec3)> {
        self.0
            .iter()
            .filter_map(|(id, mesh)| {
                mesh.closest_point(point, query)
                    .map(|p| (p, (p - point).sqr_magnitude(), *id))
            })
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(p, _, id)| (id, p))
    }
}
