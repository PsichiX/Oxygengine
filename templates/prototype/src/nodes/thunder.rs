use oxygengine::prelude::{
    intuicio::derive::*,
    intuicio::{core as intuicio_core, data as intuicio_data, prelude::*},
    *,
};
use rand::{thread_rng, Rng};

#[derive(IntuicioStruct, Debug)]
#[intuicio(name = "ThunderNode", module_name = "game")]
pub struct ThunderNode {
    #[intuicio(ignore)]
    pub spread: Scalar,
    #[intuicio(ignore)]
    pub size: Scalar,
    #[intuicio(ignore)]
    pub temperature: Scalar,
    #[intuicio(ignore)]
    pub target: Vec2,
}

impl Default for ThunderNode {
    fn default() -> Self {
        Self {
            spread: 1.0,
            size: 1.0,
            temperature: 0.5,
            target: Default::default(),
        }
    }
}

impl ThunderNode {
    pub fn install(registry: &mut Registry) {
        registry.add_struct(Self::define_struct(registry));
        registry.add_function(Self::event_update__define_function(registry));
        registry.add_function(Self::event_draw__define_function(registry));
    }
}

#[intuicio_methods(module_name = "game")]
impl ThunderNode {
    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn event_update(
        this: &mut Self,
        _transform: &mut HaTransform,
        universe: &Universe,
        _entity: Entity,
    ) {
        let mut commands = universe.expect_resource_mut::<UniverseCommands>();
        let spatial = universe.expect_resource::<SpatialQueries>();
        let mut signals = universe.expect_resource_mut::<ScriptedNodesSignals>();

        for (entity, _) in spatial.contains(this.target) {
            commands.schedule(DespawnEntity(entity));
            signals.signal::<()>(
                ScriptedNodeSignal::parse(None, "signal_score_increased")
                    .unwrap()
                    .broadcast(),
            );
        }
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn event_draw(
        this: &mut Self,
        transform: &mut HaTransform,
        universe: &Universe,
        _entity: Entity,
    ) {
        let warm = Rgba::new_opaque(1.0, 1.0, 0.7);
        let cold = Rgba::new_opaque(0.9, 0.9, 1.0);
        let from = Vec2::from(transform.get_world_origin());
        let to = this.target;
        let forward = (to - from).normalized();
        let right = vec2(-forward.y, forward.x);
        let mut points = vec![from];
        let mut factor = 0.0;
        let mut rng = thread_rng();
        loop {
            factor += rng.gen_range(0.0..0.35);
            if factor >= 1.0 {
                break;
            }
            let offset = right * rng.gen_range(-1.0..1.0) * this.spread;
            points.push(Vec2::lerp(from, to, factor) + offset);
        }
        points.push(to);

        let mut renderables = universe.expect_resource_mut::<Renderables>();
        renderables.draw(ShapeRenderable::Lines {
            tint: Rgba::lerp(cold, warm, this.temperature),
            points,
            size: this.size,
        });
    }
}
