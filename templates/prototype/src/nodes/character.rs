use oxygengine::prelude::{
    intuicio::derive::*,
    intuicio::{core as intuicio_core, data as intuicio_data, prelude::*},
    *,
};

#[derive(IntuicioStruct, Debug, Default)]
#[intuicio(name = "Character", module_name = "game")]
pub struct Character {
    #[intuicio(ignore)]
    pub image: String,
    #[intuicio(ignore)]
    pub size: Vec2,
    #[intuicio(ignore)]
    pub mirror_horizontaly: bool,
    #[intuicio(ignore)]
    pub speed: Scalar,
    #[intuicio(ignore)]
    pub animation_speed: Scalar,
    #[intuicio(ignore)]
    animation_phase: Scalar,
    #[intuicio(ignore)]
    indicator: ScriptedNodeEntity,
}

impl Character {
    pub fn install(registry: &mut Registry) {
        registry.add_struct(Self::define_struct(registry));
        registry.add_function(Self::event_update__define_function(registry));
        registry.add_function(Self::event_draw__define_function(registry));
    }

    pub fn new(image: impl ToString, indicator: ScriptedNodeEntity) -> Self {
        Self {
            image: image.to_string(),
            size: 100.0.into(),
            mirror_horizontaly: false,
            speed: 100.0,
            animation_speed: 1.0,
            animation_phase: 0.0,
            indicator,
        }
    }

    pub fn size(mut self, value: impl Into<Vec2>) -> Self {
        self.size = value.into();
        self
    }

    pub fn speed(mut self, value: Scalar) -> Self {
        self.speed = value;
        self
    }

    pub fn animation_speed(mut self, value: Scalar) -> Self {
        self.animation_speed = value;
        self
    }

    pub fn update(
        &mut self,
        transform: &mut HaTransform,
        dt: Scalar,
        inputs: &InputController,
        signals: &mut ScriptedNodesSignals,
    ) {
        let direction =
            Vec2::from(inputs.mirror_multi_axis_or_default([
                ("move-right", "move-left"),
                ("move-down", "move-up"),
            ]));
        if let Some(dir) = direction.try_normalized() {
            transform.change_translation(|position| *position += dir * self.speed * dt);
            self.animation_phase += dt * self.animation_speed;
            if dir.x > 0.0 {
                self.mirror_horizontaly = false;
            } else if dir.x < 0.0 {
                self.mirror_horizontaly = true;
            }
        }

        if inputs.trigger_or_default("mouse-action").is_pressed() {
            if let Some(entity) = self.indicator.get() {
                signals.signal::<()>(ScriptedNodeSignal::new(
                    entity,
                    ScriptFunctionReference::parse("event_toggle_visibility").unwrap(),
                ));
            }
        }
    }

    pub fn draw(&self, transform: &mut HaTransform, renderables: &mut Renderables) {
        let scale = if self.mirror_horizontaly {
            vec3(-1.0, 1.0, 1.0)
        } else {
            vec3(1.0, 1.0, 1.0)
        };
        renderables.draw(
            SpriteRenderable::new(&self.image)
                .position(transform.get_world_origin())
                .size(self.size * transform.get_world_scale_lossy() * scale)
                .region_from_animation_frame(self.animation_phase as usize, 4, 1),
        );
    }
}

#[intuicio_methods(module_name = "game")]
impl Character {
    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn event_update(
        this: &mut Self,
        transform: &mut HaTransform,
        dt: &Scalar,
        inputs: &InputController,
        signals: &mut ScriptedNodesSignals,
    ) {
        this.update(transform, *dt, inputs, signals);
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
