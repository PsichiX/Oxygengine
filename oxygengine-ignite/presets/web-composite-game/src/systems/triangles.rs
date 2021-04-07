use crate::components::triangles::Triangles;
use oxygengine::prelude::*;

#[derive(Default)]
pub struct TrianglesSystem {
    phase: Scalar,
}

impl<'s> System<'s> for TrianglesSystem {
    type SystemData = (
        ReadExpect<'s, AppLifeCycle>,
        ReadStorage<'s, Triangles>,
        WriteStorage<'s, CompositeRenderable>,
    );

    fn run(&mut self, (lifecycle, triangles, mut renderables): Self::SystemData) {
        self.phase += lifecycle.delta_time_seconds() * 0.5;
        for (triangle, mut renderable) in (&triangles, &mut renderables).join() {
            let radius = triangle.size * 0.5;
            let vertices = vec![
                ((0.0, 0.0).into(), (0.5, 0.5).into()),
                (hex_point_position(radius, 0), hex_point_tex_coord(1.5, 0)),
                (hex_point_position(radius, 1), hex_point_tex_coord(1.5, 1)),
                (hex_point_position(radius, 2), hex_point_tex_coord(1.5, 2)),
                (hex_point_position(radius, 3), hex_point_tex_coord(1.5, 3)),
                (hex_point_position(radius, 4), hex_point_tex_coord(1.5, 4)),
                (hex_point_position(radius, 5), hex_point_tex_coord(1.5, 5)),
            ];
            let faces = vec![
                TriangleFace::new(0, 1, 2),
                TriangleFace::new(0, 2, 3),
                TriangleFace::new(0, 3, 4),
                TriangleFace::new(0, 4, 5),
                TriangleFace::new(0, 5, 6),
                TriangleFace::new(0, 6, 1),
            ];
            renderable.0 = oxygengine::prelude::Triangles {
                image: triangle.image.clone().into(),
                color: Default::default(),
                vertices,
                faces,
            }
            .into();
        }
    }
}

fn hex_point_position(radius: Scalar, corner: usize) -> Vec2 {
    let angle = (60.0 * corner as Scalar - 30.0).to_radians();
    Vec2::new(radius * angle.cos(), radius * angle.sin())
}

fn hex_point_tex_coord(scale: Scalar, corner: usize) -> Vec2 {
    hex_point_position(scale * 0.5, corner) + Vec2::new(0.5, 0.5)
}
