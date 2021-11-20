use crate::{
    components::material_instance::HaMaterialInstance, material::domains::gizmo::GizmoFactory,
};

#[derive(Debug, Default, Clone)]
pub struct Gizmos {
    pub factory: GizmoFactory,
    pub material: HaMaterialInstance,
}

impl Gizmos {
    pub fn new(material: HaMaterialInstance) -> Self {
        Self {
            factory: Default::default(),
            material,
        }
    }

    pub fn with_capacity(
        vertex_capacity: usize,
        index_capacity: usize,
        material: HaMaterialInstance,
    ) -> Self {
        Self {
            factory: GizmoFactory::with_capacity(vertex_capacity, index_capacity),
            material,
        }
    }
}
