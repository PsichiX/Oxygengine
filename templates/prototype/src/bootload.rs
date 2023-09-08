use oxygengine::prelude::{intuicio::prelude::*, *};

pub fn bootload<T>(app: T) -> T
where
    T: PrototypeApp,
{
    let mut registry = Registry::default();
    Scripting::install(&mut registry);
    oxygengine::prototype::nodes::install(&mut registry);
    crate::nodes::install(&mut registry);

    app.view_size(512.0)
        .sprite_filtering(ImageFiltering::Nearest)
        .preload_asset("image://images/logo.json")
        .preload_asset("image://images/panel.json")
        .preload_asset("image://images/crab.json")
        .preload_asset("image://images/gopher.json")
        .preload_asset("image://images/grass.json")
        .preload_asset("font://fonts/roboto.json")
        .preload_asset("audio://audio/pop.ogg")
        .scripting_registry(registry)
}
