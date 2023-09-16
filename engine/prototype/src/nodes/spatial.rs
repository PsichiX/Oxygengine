use crate::resources::spatial_queries::*;
use oxygengine_core::{
    ecs::{Entity, Universe},
    scripting::intuicio::{core as intuicio_core, data as intuicio_data, derive::*, prelude::*},
};
use oxygengine_ha_renderer::{components::transform::*, math::*};
use oxygengine_nodes::ScriptedNodeEntity;

#[derive(IntuicioStruct, Debug, Default)]
#[intuicio(name = "SpatialNode", module_name = "spatial")]
pub struct SpatialNode {
    #[intuicio(ignore)]
    pub area: Rect,
    #[intuicio(ignore)]
    pub entity: ScriptedNodeEntity,
}

impl SpatialNode {
    pub fn install(registry: &mut Registry) {
        registry.add_struct(SpatialNode::define_struct(registry));
        registry.add_function(SpatialNode::event_prepare__define_function(registry));
    }
}

#[intuicio_methods(module_name = "renderable")]
impl SpatialNode {
    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn event_prepare(
        this: &mut Self,
        transform: &mut HaTransform,
        universe: &Universe,
        entity: Entity,
    ) {
        let mut spatial = universe.expect_resource_mut::<SpatialQueries>();
        let position = transform.get_world_origin();
        let mut area = this.area;
        area.x += position.x;
        area.y += position.y;
        spatial.add(this.entity.get().unwrap_or(entity), area);
    }
}
