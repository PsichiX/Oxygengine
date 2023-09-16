use crate::resources::renderables::*;
use oxygengine_core::{
    ecs::{Entity, Universe},
    scripting::intuicio::{core as intuicio_core, data as intuicio_data, derive::*, prelude::*},
};
use oxygengine_ha_renderer::{components::transform::*, math::*};

#[derive(Debug, Default)]
pub enum SpriteNodeRegion {
    #[default]
    None,
    Rect(Rect),
    AnimationFrame {
        frame: usize,
        cols: usize,
        rows: usize,
    },
    TileCell {
        col: usize,
        row: usize,
        cols: usize,
        rows: usize,
    },
}

#[derive(IntuicioStruct, Debug)]
#[intuicio(name = "SpriteNode", module_name = "renderable")]
pub struct SpriteNode {
    #[intuicio(ignore)]
    pub image: String,
    #[intuicio(ignore)]
    pub tint: Rgba,
    #[intuicio(ignore)]
    pub tiling: Vec2,
    #[intuicio(ignore)]
    pub size: Vec2,
    #[intuicio(ignore)]
    pub mirror_x: bool,
    #[intuicio(ignore)]
    pub mirror_y: bool,
    #[intuicio(ignore)]
    pub region: SpriteNodeRegion,
}

impl Default for SpriteNode {
    fn default() -> Self {
        Self {
            image: Default::default(),
            tint: Rgba::white(),
            tiling: 1.0.into(),
            size: 1.0.into(),
            mirror_x: false,
            mirror_y: false,
            region: Default::default(),
        }
    }
}

impl SpriteNode {
    pub fn install(registry: &mut Registry) {
        registry.add_struct(SpriteNode::define_struct(registry));
        registry.add_function(SpriteNode::event_draw__define_function(registry));
    }
}

#[intuicio_methods(module_name = "renderable")]
impl SpriteNode {
    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn event_draw(
        this: &mut Self,
        transform: &mut HaTransform,
        universe: &Universe,
        _entity: Entity,
    ) {
        let mut renderables = universe.expect_resource_mut::<Renderables>();
        let scale = vec2(
            if this.mirror_x { -1.0 } else { 1.0 },
            if this.mirror_y { -1.0 } else { 1.0 },
        );
        let renderable = SpriteRenderable::new(&this.image)
            .tint(this.tint)
            .tiling(this.tiling)
            .position(transform.get_world_origin())
            .rotation(transform.get_world_rotation_lossy().eulers().yaw)
            .size(this.size * scale * transform.get_world_scale_lossy());
        let renderable = match &this.region {
            SpriteNodeRegion::None => renderable,
            SpriteNodeRegion::Rect(rect) => renderable.region(*rect),
            SpriteNodeRegion::AnimationFrame { frame, cols, rows } => {
                renderable.region_from_animation_frame(*frame, *cols, *rows)
            }
            SpriteNodeRegion::TileCell {
                col,
                row,
                cols,
                rows,
            } => renderable.region_from_tile_cell(*col, *row, *cols, *rows),
        };
        renderables.draw(renderable);
    }
}
