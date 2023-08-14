use oxygengine::prelude::{
    intuicio::derive::*,
    intuicio::{core as intuicio_core, data as intuicio_data, prelude::*},
    *,
};

#[derive(IntuicioStruct, Debug, Default)]
#[intuicio(name = "Indicator", module_name = "game")]
pub struct Indicator {
    #[intuicio(ignore)]
    pub image: String,
    #[intuicio(ignore)]
    pub show: bool,
    #[intuicio(ignore)]
    pub size: Vec2,
    #[intuicio(ignore)]
    pub animation_speed: Scalar,
    #[intuicio(ignore)]
    animation_phase: Scalar,
}

impl Indicator {
    pub fn install(registry: &mut Registry) {
        registry.add_struct(Self::define_struct(registry));
        registry.add_function(Self::event_toggle_visibility__define_function(registry));
        registry.add_function(Self::event_update__define_function(registry));
        registry.add_function(Self::event_draw__define_function(registry));
    }

    pub fn new(image: impl ToString) -> Self {
        Self {
            image: image.to_string(),
            show: false,
            size: 100.0.into(),
            animation_speed: 1.0,
            animation_phase: 0.0,
        }
    }

    pub fn size(mut self, value: impl Into<Vec2>) -> Self {
        self.size = value.into();
        self
    }

    pub fn animation_speed(mut self, value: Scalar) -> Self {
        self.animation_speed = value;
        self
    }

    pub fn update(&mut self, dt: Scalar) {
        if self.show {
            self.animation_phase += dt * self.animation_speed;
        }
    }

    pub fn draw(&self, transform: &mut HaTransform, renderables: &mut Renderables) {
        if self.show {
            let scale = (self.animation_phase.sin() + 3.0) * 0.25;
            renderables.draw(
                SpriteRenderable::new(&self.image)
                    .position(transform.get_world_origin())
                    .size(self.size * transform.get_world_scale_lossy() * scale),
            );
        }
    }
}

#[intuicio_methods(module_name = "game")]
impl Indicator {
    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn event_toggle_visibility(this: &mut Self) {
        this.show = !this.show;
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn event_update(
        this: &mut Self,
        _transform: &mut HaTransform,
        dt: &Scalar,
        _inputs: &InputController,
        _signals: &mut ScriptedNodesSignals,
    ) {
        this.update(*dt);
    }

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
