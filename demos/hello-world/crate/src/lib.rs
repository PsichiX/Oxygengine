extern crate oxygengine;

use oxygengine::{
    backend::web::*,
    composite_renderer::{component::*, composite_renderer::*, math::*},
    prelude::*,
};
use wasm_bindgen::prelude::*;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

struct DebugSystem;

impl<'s> System<'s> for DebugSystem {
    type SystemData = ReadExpect<'s, PlatformCompositeRenderer>;

    fn run(&mut self, renderer: Self::SystemData) {
        console_log!("{:#?}", renderer.state().stats());
    }
}

#[wasm_bindgen]
pub fn run() -> Result<(), JsValue> {
    set_panic_hook();

    let mut app = App::build()
        .with_bundle(
            oxygengine::core::assets::bundle_installer,
            (WebFetchEngine::default(), |_| {}),
        )
        .with_bundle(
            oxygengine::composite_renderer::bundle_installer,
            WebCompositeRenderer::with_state("screen", State::new(Some(Color::black()))),
        )
        .with_system(DebugSystem, "debug", &[])
        .build();

    app.world_mut()
        .create_entity()
        .with(CompositeRenderable(Renderable::Rectangle(Rectangle {
            color: Color::blue(),
            rect: [100.0, 100.0, 500.0, 100.0].into(),
        })))
        .with(CompositeTransform::default())
        .build();

    app.world_mut()
        .create_entity()
        .with(CompositeRenderable(Renderable::Text(Text {
            color: Color::yellow(),
            font: "Verdana",
            align: TextAlign::Center,
            text: "Hello World!",
            position: [100.0 + 250.0, 100.0 + 50.0 + 12.0].into(),
            size: 24.0,
        })))
        .with(CompositeTransform::default())
        .build();

    app.world_mut()
        .create_entity()
        .with(CompositeRenderable(Renderable::Path(Path {
            color: Color::white(),
            elements: vec![
                PathElement::MoveTo([300.0, 300.0].into()),
                PathElement::LineTo([400.0, 300.0].into()),
                PathElement::QuadraticCurveTo([400.0, 400.0].into(), [300.0, 400.0].into()),
                PathElement::LineTo([300.0, 300.0].into()),
            ],
        })))
        .with(CompositeTransform::default())
        .with(CompositeRenderableStroke(5.0))
        .build();

    AppRunner::new(app).run::<PlatformAppRunner, _>()?;

    Ok(())
}

fn set_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}
