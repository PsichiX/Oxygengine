use crate::nodes::character::Character;
use oxygengine::prelude::{intuicio::prelude::*, *};

pub fn bootload<T>(app: T) -> T
where
    T: PrototypeApp,
{
    let mut registry = Registry::default();
    registry.add_struct(NativeStructBuilder::new::<InputController>().build());
    registry.add_struct(NativeStructBuilder::new::<Renderables>().build());
    Scripting::install(&mut registry);
    ScriptedNodes::install(&mut registry);
    Character::install(&mut registry);

    app.view_size(512.0)
        .sprite_filtering(ImageFiltering::Nearest)
        .preload_asset("image://images/logo.json")
        .preload_asset("image://images/panel.json")
        .preload_asset("image://images/crab.json")
        .preload_asset("font://fonts/roboto.json")
        .preload_asset("audio://audio/pop.ogg")
        .scripting_registry(registry)
}
