use oxygengine::prelude::*;

pub fn bootload<T>(app: T) -> T
where
    T: PrototypeApp,
{
    app.view_size(512.0)
        .sprite_filtering(ImageFiltering::Nearest)
        .preload_asset("image://images/logo.json")
        .preload_asset("image://images/panel.json")
        .preload_asset("image://images/crab.json")
        .preload_asset("font://fonts/roboto.json")
        .preload_asset("audio://audio/pop.ogg")
}
