use crate::{
    resource::{NavMesh, NavMeshID, NavMeshesRes, NavPathMode, NavQuery, NavVec3, ZERO_TRESHOLD},
    Scalar,
};
use core::{
    ecs::{Component, VecStorage},
    id::ID,
};

pub type NavAgentID = ID<NavAgent>;

#[derive(Debug, Clone)]
pub struct NavAgent {
    id: NavAgentID,
    pub position: NavVec3,
    pub direction: NavVec3,
    pub speed: Scalar,
    pub min_target_distance: Scalar,
    destination: Option<(NavVec3, NavQuery, NavPathMode, NavMeshID)>,
    path: Option<Vec<NavVec3>>,
    dirty_path: bool,
}

impl Component for NavAgent {
    type Storage = VecStorage<Self>;
}

impl Default for NavAgent {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl NavAgent {
    pub fn new(position: NavVec3) -> Self {
        Self::new_with_direction(position, Default::default())
    }

    pub fn new_with_direction(position: NavVec3, direction: NavVec3) -> Self {
        Self {
            id: Default::default(),
            position,
            direction: direction.normalize(),
            speed: 10.0,
            min_target_distance: 1.0,
            destination: None,
            path: None,
            dirty_path: false,
        }
    }

    pub fn id(&self) -> NavAgentID {
        self.id
    }

    pub fn destination(&self) -> Option<NavVec3> {
        if let Some((destination, _, _, _)) = &self.destination {
            Some(*destination)
        } else {
            None
        }
    }

    pub fn set_destination(
        &mut self,
        point: NavVec3,
        query: NavQuery,
        mode: NavPathMode,
        mesh: NavMeshID,
    ) {
        self.destination = Some((point, query, mode, mesh));
        self.dirty_path = true;
    }

    pub fn clear_path(&mut self) {
        self.destination = None;
        self.dirty_path = false;
        self.path = None;
    }

    pub fn path(&self) -> Option<&[NavVec3]> {
        if let Some(path) = &self.path {
            Some(path)
        } else {
            None
        }
    }

    pub fn destination_reached(&self) -> bool {
        if let Some((destination, _, _, _)) = &self.destination {
            (self.position - *destination).sqr_magnitude() < ZERO_TRESHOLD
        } else {
            true
        }
    }

    pub fn process(&mut self, meshes: &NavMeshesRes, delta_time: Scalar) {
        if self.dirty_path {
            self.dirty_path = false;
            if let Some((destination, query, mode, id)) = self.destination {
                if let Some(mesh) = meshes.0.get(&id) {
                    self.path = mesh.find_path(self.position, destination, query, mode);
                } else {
                    self.destination = None;
                }
            }
        }
        if delta_time < 0.0 {
            return;
        }
        if let Some(path) = &self.path {
            if let Some((target, _)) = NavMesh::path_target_point(
                path,
                self.position,
                self.speed.max(self.min_target_distance.max(0.0)) * delta_time,
            ) {
                let diff = target - self.position;
                let dir = diff.normalize();
                self.position =
                    self.position + dir * (self.speed.max(0.0) * delta_time).min(diff.magnitude());
                self.direction = diff.normalize();
            }
        }
    }
}
