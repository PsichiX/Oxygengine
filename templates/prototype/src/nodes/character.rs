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
    pub position: Vec2,
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
}

impl Character {
    pub fn install(registry: &mut Registry) {
        registry.add_struct(Self::define_struct(registry));
        registry.add_function(Self::event_update__define_function(registry));
        registry.add_function(Self::event_draw__define_function(registry));
    }

    pub fn new(image: impl ToString) -> Self {
        Self {
            image: image.to_string(),
            position: Default::default(),
            size: 24.0.into(),
            mirror_horizontaly: false,
            speed: 24.0,
            animation_speed: 1.0,
            animation_phase: 0.0,
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

    pub fn update(&mut self, dt: Scalar, inputs: &InputController) {
        let direction =
            Vec2::from(inputs.mirror_multi_axis_or_default([
                ("move-right", "move-left"),
                ("move-down", "move-up"),
            ]));
        if let Some(dir) = direction.try_normalized() {
            self.position += dir * self.speed * dt;
            self.animation_phase += dt * self.animation_speed;
            if dir.x > 0.0 {
                self.mirror_horizontaly = false;
            } else if dir.x < 0.0 {
                self.mirror_horizontaly = true;
            }
        }
    }

    pub fn draw(&self, renderables: &mut Renderables) {
        let scale = if self.mirror_horizontaly {
            vec2(-1.0, 1.0)
        } else {
            vec2(1.0, 1.0)
        };
        renderables.draw(
            SpriteRenderable::new(&self.image)
                .position(self.position)
                .size(self.size * scale)
                .region_from_animation_frame(self.animation_phase as usize, 4, 1),
        );
    }
}

#[intuicio_methods(module_name = "game")]
impl Character {
    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn event_update(this: &mut Self, dt: &Scalar, inputs: &InputController) {
        this.update(*dt, inputs);
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn event_draw(this: &mut Self, renderables: &mut Renderables) {
        this.draw(renderables);
    }
}
