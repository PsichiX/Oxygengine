use crate::{
    resource::{NavMeshID, NavPathMode, NavQuery, NavVec3},
    Scalar,
};
use core::{
    ecs::{Component, Entity, NullStorage, VecStorage},
    id::ID,
};

pub type NavAgentID = ID<NavAgent>;

#[derive(Debug, Default, Copy, Clone)]
pub struct SimpleNavDriverTag;

impl Component for SimpleNavDriverTag {
    type Storage = NullStorage<Self>;
}

#[derive(Debug, Clone, Copy)]
pub enum NavAgentTarget {
    Point(NavVec3),
    Entity(Entity),
}

impl NavAgentTarget {
    pub fn is_point(&self) -> bool {
        match self {
            NavAgentTarget::Point(_) => true,
            _ => false,
        }
    }

    pub fn is_entity(&self) -> bool {
        match self {
            NavAgentTarget::Entity(_) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct NavAgentDestination {
    pub target: NavAgentTarget,
    pub query: NavQuery,
    pub mode: NavPathMode,
    pub mesh: NavMeshID,
}

#[derive(Debug, Clone)]
pub struct NavAgent {
    id: NavAgentID,
    pub position: NavVec3,
    pub direction: NavVec3,
    pub speed: Scalar,
    pub min_target_distance: Scalar,
    pub(crate) destination: Option<NavAgentDestination>,
    pub(crate) path: Option<Vec<NavVec3>>,
    pub(crate) dirty_path: bool,
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

    pub fn target(&self) -> Option<NavAgentTarget> {
        if let Some(destination) = &self.destination {
            Some(destination.target)
        } else {
            None
        }
    }

    pub fn destination(&self) -> Option<&NavAgentDestination> {
        if let Some(destination) = &self.destination {
            Some(destination)
        } else {
            None
        }
    }

    pub fn set_destination(
        &mut self,
        target: NavAgentTarget,
        query: NavQuery,
        mode: NavPathMode,
        mesh: NavMeshID,
    ) {
        self.destination = Some(NavAgentDestination {
            target,
            query,
            mode,
            mesh,
        });
        self.dirty_path = true;
    }

    pub fn clear_path(&mut self) {
        self.destination = None;
        self.dirty_path = false;
        self.path = None;
    }

    pub fn recalculate_path(&mut self) {
        self.dirty_path = true;
    }

    pub fn path(&self) -> Option<&[NavVec3]> {
        if let Some(path) = &self.path {
            Some(path)
        } else {
            None
        }
    }

    pub fn set_path(&mut self, path: Vec<NavVec3>) {
        self.path = Some(path);
        self.dirty_path = false;
    }
}
