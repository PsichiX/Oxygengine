use crate::resources::{renderables::*, Transform2d};
use oxygengine_core::{
    ecs::{Entity, Universe},
    scripting::intuicio::{core as intuicio_core, data as intuicio_data, derive::*, prelude::*},
    Scalar,
};
use oxygengine_ha_renderer::{
    components::{text_instance::*, transform::*},
    math::*,
};

#[derive(IntuicioStruct, Debug, Default)]
#[intuicio(name = "TextNode", module_name = "renderable")]
pub struct TextNode {
    #[intuicio(ignore)]
    pub content: HaTextContent,
    #[intuicio(ignore)]
    pub font: String,
    #[intuicio(ignore)]
    pub size: Scalar,
    #[intuicio(ignore)]
    pub color: Rgba,
    #[intuicio(ignore)]
    pub alignment: Vec2,
    #[intuicio(ignore)]
    pub bounds_width: Option<Scalar>,
    #[intuicio(ignore)]
    pub bounds_height: Option<Scalar>,
    #[intuicio(ignore)]
    pub wrapping: HaTextWrapping,
}

impl TextNode {
    pub fn install(registry: &mut Registry) {
        registry.add_struct(TextNode::define_struct(registry));
        registry.add_function(TextNode::event_draw__define_function(registry));
    }
}

#[intuicio_methods(module_name = "renderable")]
impl TextNode {
    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn event_draw(
        this: &mut Self,
        transform: &mut HaTransform,
        universe: &Universe,
        _entity: Entity,
    ) {
        let mut renderables = universe.expect_resource_mut::<Renderables>();
        let renderable = TextRenderable {
            transform: Transform2d::default()
                .position(transform.get_world_origin())
                .rotation(transform.get_world_rotation_lossy().eulers().yaw),
            content: this.content.to_owned(),
            font: this.font.to_owned(),
            size: this.size,
            color: this.color,
            alignment: this.alignment,
            bounds_width: this.bounds_width,
            bounds_height: this.bounds_height,
            wrapping: this.wrapping.to_owned(),
        };
        renderables.draw(renderable);
    }
}
