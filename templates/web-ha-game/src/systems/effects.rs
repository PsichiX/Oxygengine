use crate::{components::*, resources::effects::*};
use oxygengine::prelude::*;

type V = SurfaceVertexPT;

pub type EffectsSystemResources<'a> = (
    WorldRef,
    &'a AppLifeCycle,
    &'a mut Effects,
    &'a Board,
    &'a HaBoardSettings,
    Comp<&'a mut HaImmediateBatch<V>>,
    Comp<&'a BatchedAttacksTag>,
    Comp<&'a BatchedSecretsTag>,
);

pub fn effects_system(universe: &mut Universe) {
    let (world, lifecycle, mut effects, board, settings, ..) =
        universe.query_resources::<EffectsSystemResources>();

    if let Some((_, batch)) = world
        .query::<&mut HaImmediateBatch<V>>()
        .with::<&BatchedAttacksTag>()
        .iter()
        .next()
    {
        for location in effects.attacks() {
            batch_effect(batch, location, &board, &settings);
        }
    }

    if let Some((_, batch)) = world
        .query::<&mut HaImmediateBatch<V>>()
        .with::<&BatchedSecretsTag>()
        .iter()
        .next()
    {
        for (location, _) in effects.secrets() {
            batch_effect(batch, location, &board, &settings);
        }
    }

    effects.maintain(lifecycle.delta_time_seconds());
}

fn batch_effect(
    batch: &mut HaImmediateBatch<V>,
    location: Location,
    board: &Board,
    settings: &HaBoardSettings,
) {
    let Vec2 { x: w, y: h } = settings.cell_size();
    let Vec3 { x, y, .. } = board_location_to_world_position(location, board, settings);
    batch.factory.quad([
        V {
            position: Vec3::new(x, y, 0.0),
            texture_coord: Vec3::new(0.0, 0.0, 0.0),
        },
        V {
            position: Vec3::new(x + w, y, 0.0),
            texture_coord: Vec3::new(1.0, 0.0, 0.0),
        },
        V {
            position: Vec3::new(x + w, y + h, 0.0),
            texture_coord: Vec3::new(1.0, 1.0, 0.0),
        },
        V {
            position: Vec3::new(x, y + h, 0.0),
            texture_coord: Vec3::new(0.0, 1.0, 0.0),
        },
    ]);
}
