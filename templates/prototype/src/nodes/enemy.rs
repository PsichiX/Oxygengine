use oxygengine::prelude::{
    intuicio::derive::*,
    intuicio::{core as intuicio_core, data as intuicio_data, prelude::*},
    *,
};

#[derive(IntuicioStruct, Debug)]
#[intuicio(name = "EnemyNode", module_name = "game")]
pub struct EnemyNode {
    #[intuicio(ignore)]
    pub speed: Scalar,
    #[intuicio(ignore)]
    pub animation_speed: Scalar,
    #[intuicio(ignore)]
    pub animation_phase: Scalar,
    #[intuicio(ignore)]
    pub sprite: ScriptedNodeEntity,
    #[intuicio(ignore)]
    pub spatial: ScriptedNodeEntity,
}

impl Default for EnemyNode {
    fn default() -> Self {
        Self {
            speed: 1.0,
            animation_speed: 1.0,
            animation_phase: 0.0,
            sprite: Default::default(),
            spatial: Default::default(),
        }
    }
}

impl EnemyNode {
    pub fn install(registry: &mut Registry) {
        registry.add_struct(Self::define_struct(registry));
        registry.add_function(Self::event_update__define_function(registry));
    }
}

#[intuicio_methods(module_name = "game")]
impl EnemyNode {
    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn event_update(
        this: &mut Self,
        transform: &mut HaTransform,
        universe: &Universe,
        entity: Entity,
    ) {
        let world = universe.world();
        let life_cycle = universe.expect_resource::<AppLifeCycle>();
        let hierarchy = universe.expect_resource::<Hierarchy>();
        let spatial = universe.expect_resource::<SpatialQueries>();
        let mut commands = universe.expect_resource_mut::<UniverseCommands>();
        let dt = life_cycle.delta_time_seconds();
        let player = ScriptedNodeEntity::find("player", &hierarchy);

        let position = transform.get_world_origin();
        let direction = player
            .node::<&HaTransform, _>(&world, |_, transform| transform.get_world_origin())
            .map(|origin| origin - position)
            .filter(|direction| direction.magnitude_squared() > 0.1)
            .unwrap_or_default();

        if let Some(dir) = direction.try_normalized() {
            transform.change_translation(|position| *position += dir * this.speed * dt);
            this.animation_phase += dt * this.animation_speed;
            this.sprite
                .with_mut::<SpriteNode, (), _>(&world, |node, _| {
                    if dir.x.abs() > 0.0 {
                        node.mirror_x = dir.x < 0.0;
                    }
                    node.region = SpriteNodeRegion::AnimationFrame {
                        frame: this.animation_phase as usize % 4,
                        cols: 4,
                        rows: 1,
                    };
                    Some(())
                });
        }

        if let Some(player) = player.get() {
            if spatial.collides_with(entity, player) {
                commands.schedule(DespawnEntity(player));
            }
        }
    }
}
