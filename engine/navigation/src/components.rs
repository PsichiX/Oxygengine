use crate::resources::{NavMeshID, NavPathMode, NavQuery, NavVec3};
use core::{
    ecs::Entity,
    id::ID,
    prefab::{Prefab, PrefabComponent},
    Scalar,
};
use serde::{Deserialize, Serialize};

/// Nav agent identifier.
pub type NavAgentId = ID<NavAgent>;

/// Simple nav driver component tag to mark entity to use simple movement on nav mesh.
#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct SimpleNavDriverTag;

impl Prefab for SimpleNavDriverTag {}
impl PrefabComponent for SimpleNavDriverTag {}

/// Nav agent target.
#[derive(Debug, Clone, Copy)]
pub enum NavAgentTarget {
    /// Point in world space.
    Point(NavVec3),
    /// Entity to follow.
    Entity(Entity),
}

impl NavAgentTarget {
    pub fn is_point(&self) -> bool {
        matches!(self, NavAgentTarget::Point(_))
    }

    pub fn is_entity(&self) -> bool {
        matches!(self, NavAgentTarget::Entity(_))
    }
}

/// Nav agent destination descriptor.
#[derive(Debug, Clone)]
pub struct NavAgentDestination {
    /// Target.
    pub target: NavAgentTarget,
    /// Query quality.
    pub query: NavQuery,
    /// path finding quality.
    pub mode: NavPathMode,
    /// Nav mesh identifier that agent is moving on.
    pub mesh: NavMeshID,
}

/// Nav agent component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavAgent {
    id: NavAgentId,
    /// Current agent position in world space.
    pub position: NavVec3,
    /// Current agent normalized direction.
    pub direction: NavVec3,
    /// Current speed (units per second).
    pub speed: Scalar,
    /// Agent sphere radius (used in obstacle and agent avoidance).
    pub radius: Scalar,
    /// Mnimal distance to target (affects direction, tells how far look for point to go to in an
    /// instant).
    pub min_target_distance: Scalar,
    #[serde(skip)]
    pub(crate) destination: Option<NavAgentDestination>,
    #[serde(skip)]
    pub(crate) path: Option<Vec<NavVec3>>,
    #[serde(skip)]
    pub(crate) dirty_path: bool,
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
            radius: 1.0,
            min_target_distance: 1.0,
            destination: None,
            path: None,
            dirty_path: false,
        }
    }

    pub fn id(&self) -> NavAgentId {
        self.id
    }

    pub fn target(&self) -> Option<NavAgentTarget> {
        self.destination
            .as_ref()
            .map(|destination| destination.target)
    }

    pub fn destination(&self) -> Option<&NavAgentDestination> {
        self.destination.as_ref()
    }

    /// Sets destination to go to.
    ///
    /// # Arguments
    /// * `target` - target to go to.
    /// * `query` - query quality.
    /// * `mode` - path finding quality.
    /// * `mesh` - nav mesh to move on.
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

impl Prefab for NavAgent {}
impl PrefabComponent for NavAgent {}
