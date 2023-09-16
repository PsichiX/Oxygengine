pub mod spatial;
pub mod sprite;
pub mod text;

use oxygengine_core::{
    ecs::{Entity, Universe},
    scripting::{
        intuicio::core::{registry::Registry, struct_type::NativeStructBuilder},
        ScriptFunctionReference,
    },
};
use oxygengine_ha_renderer::components::transform::HaTransform;
use oxygengine_nodes::ScriptedNodes;

pub const EVENT_PREPARE: &str = "event_prepare";
pub const EVENT_UPDATE: &str = "event_update";
pub const EVENT_DRAW: &str = "event_draw";
pub const EVENT_DRAW_GUI: &str = "event_draw_gui";

pub fn install(registry: &mut Registry) {
    registry.add_struct(
        NativeStructBuilder::new_named_uninitialized::<Universe>("Universe")
            .module_name("core")
            .build(),
    );
    registry.add_struct(
        NativeStructBuilder::new_named_uninitialized::<Entity>("Entity")
            .module_name("core")
            .build(),
    );
    registry.add_struct(
        NativeStructBuilder::new_named::<HaTransform>("HaTransform")
            .module_name("core")
            .build(),
    );
    sprite::SpriteNode::install(registry);
    text::TextNode::install(registry);
    spatial::SpatialNode::install(registry);
}

pub fn dispatch_events(universe: &Universe, names: &[&str]) {
    let nodes = &mut *universe.expect_resource_mut::<ScriptedNodes>();
    for name in names {
        nodes.dispatch::<&mut HaTransform>(
            universe,
            ScriptFunctionReference::parse(name).unwrap(),
            &[universe.into()],
        );
    }
}
