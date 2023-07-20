use oxygengine::prelude::*;

#[derive(Debug)]
pub struct Character {
    pub image: String,
    pub position: Vec2,
    pub size: Vec2,
    pub speed: Scalar,
    pub animation_speed: Scalar,
    animation_phase: Scalar,
}

impl Character {
    pub fn new(image: impl ToString) -> Self {
        Self {
            image: image.to_string(),
            position: Default::default(),
            size: 24.0.into(),
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

    pub fn move_position(&mut self, dt: Scalar, direction: Vec2) {
        if let Some(dir) = direction.try_normalized() {
            self.position += dir * self.speed * dt;
        }
    }

    pub fn update_animation(&mut self, dt: Scalar) {
        self.animation_phase += dt * self.animation_speed;
    }

    pub fn draw(&self, renderables: &mut Renderables) {
        renderables.draw(
            SpriteRenderable::new(&self.image)
                .position(self.position)
                .size(self.size)
                .region_from_animation_frame(self.animation_phase as usize, 4, 1),
        );
    }
}
