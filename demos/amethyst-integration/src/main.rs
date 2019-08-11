extern crate oxygengine_navigation as nav;

use amethyst::{
    core::transform::TransformBundle,
    input::{InputBundle, StringBindings},
    prelude::*,
    renderer::{
        plugins::{RenderDebugLines, RenderToWindow},
        types::DefaultBackend,
        RenderingBundle,
    },
    utils::application_root_dir,
};
use nav::prelude::*;
use systems::{CommandAgentsSystem, NavDriverSystem, RenderSystem};

mod components;
mod state;
mod systems;

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;

    let resources = app_root.join("resources");
    let display_config = resources.join("display_config.ron");

    let game_data = GameDataBuilder::default()
        .with_bundle(TransformBundle::new())?
        .with_bundle(InputBundle::<StringBindings>::new())?
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config)
                        .with_clear([0.34, 0.36, 0.52, 1.0]),
                )
                .with_plugin(RenderDebugLines::default()),
        )?
        .with(NavAgentMaintainSystem::default(), "nav-agent-maintain", &[])
        .with(CommandAgentsSystem::default(), "command-agents", &[])
        .with(NavDriverSystem, "nav-driver", &[])
        .with(RenderSystem, "render", &[]);

    let mut game = Application::new(resources, state::MyState, game_data)?;
    game.run();

    Ok(())
}
