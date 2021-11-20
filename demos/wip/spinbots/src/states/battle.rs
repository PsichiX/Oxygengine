use crate::{
    asset_protocols::{part::*, parts::*},
    components::spinbot::*,
    resources::parts_registry::*,
    ui::screens::battle::*,
    utils::{physics::*, spinbot::*},
};
use oxygengine::{
    prelude::{
        material::{BasicMaterial, MaterialHandle},
        *,
    },
    user_interface::raui::core::prelude::DataBinding,
};
use std::{convert::TryInto, f32::consts::TAU};

const SPINBOT_SCALE: Scalar = 0.25;

#[derive(Debug, Clone)]
pub struct PlayerConfig {
    pub owner: SpinBotOwner,
    pub username: String,
    pub driver: String,
    pub disk: String,
    pub layer: String,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            owner: Default::default(),
            username: "Tester".to_owned(),
            driver: "default-stamina".to_owned(),
            disk: "default-defense".to_owned(),
            layer: "default-attack".to_owned(),
        }
    }
}

#[derive(Debug)]
pub struct BattleState<const N: usize> {
    players: [PlayerConfig; N],
}

impl<const N: usize> Default for BattleState<N> {
    fn default() -> Self {
        Self {
            players: vec![PlayerConfig::default(); N].try_into().unwrap(),
        }
    }
}

impl<const N: usize> BattleState<N> {
    pub fn new(players: [PlayerConfig; N]) -> Self {
        Self { players }
    }
}

impl<const N: usize> State for BattleState<N> {
    fn on_enter(&mut self, universe: &mut Universe) {
        universe
            .expect_resource_mut::<PrefabManager>()
            .instantiate("battle-arena", universe)
            .expect("Could not instantiate battle arena!");
        let token = universe
            .expect_resource::<AppLifeCycle>()
            .current_state_token();
        let arena_radius = universe.expect_resource::<Arena>().shape.radius();
        let parts = universe.expect_resource::<PartsRegistry>();

        universe
            .world_mut()
            .spawn(build_arena_entity(arena_radius, 32, 0.0, 0.0, token).build());

        for (i, player) in self.players.iter().enumerate() {
            let driver = match parts.get(&player.driver, PartType::Driver) {
                Some(part) => part,
                None => continue,
            };
            let disk = match parts.get(&player.disk, PartType::Disk) {
                Some(part) => part,
                None => continue,
            };
            let layer = match parts.get(&player.layer, PartType::Layer) {
                Some(part) => part,
                None => continue,
            };
            let angle = TAU * (i as Scalar / self.players.len() as Scalar);
            let (y, x) = angle.sin_cos();
            let pos = Vec2::new(x, y) * arena_radius * 0.5;
            let vel = pos.right().normalized() * 0.0;
            universe.world_mut().spawn(
                build_spinbot_entity(
                    format!("player{}", i),
                    pos,
                    vel,
                    1000.0,
                    (&player.driver, driver),
                    (&player.disk, disk),
                    (&player.layer, layer),
                    token,
                )
                .build(),
            );
        }
    }

    fn on_process(&mut self, universe: &mut Universe) -> StateChange {
        let mut ui = universe.expect_resource_mut::<UserInterface>();

        let found = ui.all_signals_received().find_map(|(name, (id, msg))| {
            match msg.as_any().downcast_ref() {
                Some(UiBattleSignal::Mounted) => Some((name.to_owned(), id.to_owned())),
                _ => None,
            }
        });
        if let Some((name, id)) = found {
            if let Some(app) = ui.application_mut(&name) {
                let players = self
                    .players
                    .iter()
                    .map(|config| UiBattlePlayer {
                        name: config.username.to_owned(),
                        health: DataBinding::new_bound(1.0, app.change_notifier()),
                        power: DataBinding::new_bound(
                            SpinBotPower::Charge(0.0),
                            app.change_notifier(),
                        ),
                    })
                    .collect();
                app.send_message(&id, UiBattleSignal::Init(UiBattleState(players)));
            }
        }

        StateChange::None
    }
}

fn build_arena_entity(
    radius: Scalar,
    segments: usize,
    restitution_coefficient: Scalar,
    friction_coefficient: Scalar,
    token: StateToken,
) -> EntityBuilder {
    if segments < 3 {
        panic!("Segments count less than 3: {}", segments);
    }
    let points = (0..segments)
        .map(|i| {
            let angle = TAU * i as Scalar / segments as Scalar;
            let (y, x) = angle.sin_cos();
            Point::new(x * radius, y * radius)
        })
        .collect::<Vec<_>>();
    let indices = (0..segments)
        .map(|i| Point::new(i, (i + 1) % segments))
        .collect::<Vec<_>>();
    let mut result = EntityBuilder::new();
    result
        .add(RigidBody2d::new(
            RigidBodyDesc::new()
                .mass(0.0)
                .status(BodyStatus::Static)
                .gravity_enabled(false),
        ))
        .add(Collider2d::new(
            ColliderDesc::new(ShapeHandle::new(Polyline::new(points, Some(indices)))).material(
                MaterialHandle::new(BasicMaterial::new(
                    restitution_coefficient,
                    friction_coefficient,
                )),
            ),
        ))
        .add(Collider2dBody::Me)
        .add(NonPersistent(token));
    result
}

fn build_spinbot_entity(
    name: String,
    position: Vec2,
    linear_velocity: Vec2,
    angular_velocity: Scalar,
    driver: (&str, &PartAsset),
    disk: (&str, &PartAsset),
    layer: (&str, &PartAsset),
    token: StateToken,
) -> EntityBuilder {
    let SpinBotStats {
        radius,
        mass,
        friction_coefficient,
        restitution_coefficient,
        linear_damping,
        angular_damping,
    } = SpinBotStats::combine(&[driver.1.stats(), disk.1.stats(), layer.1.stats()]);
    let mut result = EntityBuilder::new();
    result
        .add(Tag("world".into()))
        .add(Name(name.into()))
        .add(build_spinbot_renderable(driver.0, disk.0, layer.0))
        .add(CompositeTransform::scale(SPINBOT_SCALE.into()))
        .add(RigidBody2d::new(
            RigidBodyDesc::new()
                .mass(mass)
                .gravity_enabled(false)
                .translation(Vector::new(position.x, position.y))
                .velocity(Velocity::new(
                    Vector::new(linear_velocity.x, linear_velocity.y),
                    angular_velocity,
                ))
                .linear_damping(linear_damping)
                .angular_damping(angular_damping),
        ))
        .add(Collider2d::new(
            ColliderDesc::new(ShapeHandle::new(Ball::new(radius)))
                .density(1.0)
                .material(MaterialHandle::new(BasicMaterial::new(
                    restitution_coefficient,
                    friction_coefficient,
                ))),
        ))
        .add(Collider2dBody::Me)
        .add(Physics2dSyncCompositeTransform)
        .add(NonPersistent(token));
    result
}

fn build_spinbot_renderable(driver: &str, disk: &str, layer: &str) -> CompositeRenderable {
    Renderable::Commands(vec![
        Command::Draw(
            Image::new_owned(format!("drivers/{}/image.svg", driver))
                .align(0.5.into())
                .into(),
        ),
        Command::Draw(
            Image::new_owned(format!("disks/{}/image.svg", disk))
                .align(0.5.into())
                .into(),
        ),
        Command::Draw(
            Image::new_owned(format!("layers/{}/image.svg", layer))
                .align(0.5.into())
                .into(),
        ),
        Command::Draw(Image::new("images/chip.svg").align(0.5.into()).into()),
    ])
    .into()
}
