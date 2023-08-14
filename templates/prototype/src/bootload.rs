use crate::nodes::{character::Character, grass::Grass, gui::GameUi, indicator::Indicator};
use oxygengine::prelude::{ecs::*, intuicio::prelude::*, *};

pub fn bootload<T>(app: T) -> T
where
    T: PrototypeApp,
{
    let mut registry = Registry::default();
    Scripting::install(&mut registry);
    registry.add_struct(NativeStructBuilder::new_uninitialized::<Entity>().build());
    registry.add_struct(NativeStructBuilder::new::<ScriptedNodesSpawns>().build());
    registry.add_struct(NativeStructBuilder::new::<ScriptedNodesSignals>().build());
    registry.add_struct(NativeStructBuilder::new::<InputController>().build());
    registry.add_struct(NativeStructBuilder::new::<Renderables>().build());
    registry.add_struct(NativeStructBuilder::new::<Camera>().build());
    registry.add_struct(NativeStructBuilder::new::<HaTransform>().build());
    ScriptedNodes::install(&mut registry);
    Grass::install(&mut registry);
    Character::install(&mut registry);
    Indicator::install(&mut registry);
    GameUi::install(&mut registry);

    app.view_size(512.0)
        .sprite_filtering(ImageFiltering::Nearest)
        .preload_asset("image://images/logo.json")
        .preload_asset("image://images/panel.json")
        .preload_asset("image://images/crab.json")
        .preload_asset("image://images/grass.json")
        .preload_asset("font://fonts/roboto.json")
        .preload_asset("audio://audio/pop.ogg")
        .scripting_registry(registry)
}
