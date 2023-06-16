use crate::components::blink::*;
use oxygengine::prelude::*;

const FREQUENCY: Scalar = 8.0;

pub type BlinkSystemResources<'a> = (
    WorldRef,
    &'a AppLifeCycle,
    Comp<&'a mut Blink>,
    Comp<&'a mut HaMaterialInstance>,
);

pub fn blink_system(universe: &mut Universe) {
    let (world, lifecycle, ..) = universe.query_resources::<BlinkSystemResources>();

    let dt = lifecycle.delta_time_seconds();

    for (_, (blink, material)) in world
        .query::<(&mut Blink, &mut HaMaterialInstance)>()
        .iter()
    {
        if blink.0 > 0.0 {
            blink.0 = (blink.0 - dt).max(0.0);
            let color = if ((blink.0 * FREQUENCY) as usize) % 2 == 1 {
                vec4(1.0, 1.0, 1.0, 1.0)
            } else {
                vec4(1.0, 1.0, 1.0, 0.0)
            };
            material
                .values
                .insert("blinkColor".to_owned(), MaterialValue::Vec4F(color));
        }
    }
}
