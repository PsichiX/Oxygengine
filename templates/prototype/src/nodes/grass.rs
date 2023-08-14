use oxygengine::prelude::{
    intuicio::derive::*,
    intuicio::{core as intuicio_core, data as intuicio_data, prelude::*},
    *,
};

#[derive(IntuicioStruct, Debug, Default)]
#[intuicio(name = "Grass", module_name = "game")]
pub struct Grass {
    #[intuicio(ignore)]
    pub image: String,
    #[intuicio(ignore)]
    pub size: Vec2,
    #[intuicio(ignore)]
    pub tiling: Vec2,
}

impl Grass {
    pub fn install(registry: &mut Registry) {
        registry.add_struct(Self::define_struct(registry));
        registry.add_function(Self::event_draw__define_function(registry));
    }

    pub fn new(image: impl ToString) -> Self {
        Self {
            image: image.to_string(),
            size: 100.0.into(),
            tiling: 1.0.into(),
        }
    }

    pub fn size(mut self, value: impl Into<Vec2>) -> Self {
        self.size = value.into();
        self
    }

    pub fn tiling(mut self, value: impl Into<Vec2>) -> Self {
        self.tiling = value.into();
        self
    }

    pub fn draw(&self, transform: &mut HaTransform, renderables: &mut Renderables) {
        renderables.draw(
            SpriteRenderable::new(&self.image)
                .position(transform.get_world_origin())
                .size(self.size)
                .tiling(self.tiling),
        );
    }
}

#[intuicio_methods(module_name = "game")]
impl Grass {
    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn event_draw(
        this: &mut Self,
        transform: &mut HaTransform,
        _dt: &Scalar,
        renderables: &mut Renderables,
    ) {
        this.draw(transform, renderables);
    }
}
