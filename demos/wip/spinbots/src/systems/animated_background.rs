use crate::components::animated_background::AnimatedBackground;
use oxygengine::prelude::*;

pub type AnimatedBackgroundSystemResources<'a> = (
    WorldRef,
    &'a AppLifeCycle,
    Comp<&'a mut AnimatedBackground>,
    Comp<&'a mut CompositeRenderable>,
);

pub fn animated_background_system(universe: &mut Universe) {
    let (world, lifecycle, ..) = universe.query_resources::<AnimatedBackgroundSystemResources>();

    let dt = lifecycle.delta_time_seconds();
    for (_, (background, renderable)) in world
        .query::<(&mut AnimatedBackground, &mut CompositeRenderable)>()
        .iter()
    {
        background.phase.x = (background.phase.x + background.speed.x * dt).fract();
        background.phase.y = (background.phase.y + background.speed.y * dt).fract();
        if background.phase.x < 0.0 {
            background.phase.x = 1.0 - background.phase.x;
        }
        if background.phase.y < 0.0 {
            background.phase.y = 1.0 - background.phase.y;
        }
        let w = background.cols as Scalar;
        let h = background.rows as Scalar;
        let ox = w * 0.5 + background.phase.x.fract() - 0.5;
        let oy = h * 0.5 + background.phase.y.fract() - 0.5;
        let mut commands = Vec::with_capacity(background.cols * background.rows);
        for row in 0..background.rows {
            for col in 0..background.cols {
                let alignment = Vec2::new(col as Scalar - ox, row as Scalar - oy);
                commands.push(Command::Draw(
                    Image::new("images/background.svg").align(-alignment).into(),
                ));
            }
        }
        renderable.0 = Renderable::Commands(commands);
    }
}
