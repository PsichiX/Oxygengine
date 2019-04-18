extern crate oxygengine;

use oxygengine::{
    backend::web::*,
    composite_renderer::{
        component::*, composite_renderer::*, math::*, png_image_asset_protocol::*,
    },
    core::assets::{database::AssetsDatabase, protocols::prelude::*},
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

struct LoadingState;

impl State for LoadingState {
    fn on_enter(&mut self, world: &mut World) {
        world
            .write_resource::<AssetsDatabase>()
            .load("set://assets.txt")
            .expect("cannot load `assets.txt`");
    }

    fn on_process(&mut self, world: &mut World) -> StateChange {
        let assets = &world.read_resource::<AssetsDatabase>();
        if assets.is_ready() {
            StateChange::Swap(Box::new(MainState))
        } else {
            StateChange::None
        }
    }
}

struct MainState;

impl State for MainState {
    fn on_enter(&mut self, world: &mut World) {
        let text = {
            let assets = &world.read_resource::<AssetsDatabase>();
            assets
                .asset_by_path("txt://a.txt")
                .expect("`a.txt` is not loaded")
                .get::<TextAsset>()
                .expect("`a.txt` is not TextAsset")
                .get()
                .to_owned()
        };

        world
            .create_entity()
            .with(CompositeRenderable(Renderable::Rectangle(Rectangle {
                color: Color::blue(),
                rect: [100.0, 100.0, 500.0, 100.0].into(),
            })))
            .with(CompositeTransform::default())
            .build();

        world
            .create_entity()
            .with(CompositeRenderable(Renderable::Text(Text {
                color: Color::yellow(),
                font: "Verdana".into(),
                align: TextAlign::Center,
                text: text.into(),
                position: [100.0 + 250.0, 100.0 + 50.0 + 12.0].into(),
                size: 24.0,
            })))
            .with(CompositeTransform::default())
            .build();

        world
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

        world
            .create_entity()
            .with(CompositeRenderable(Renderable::Image(Image {
                image: "logo.png".into(),
                source: None,
                destination: None,
            })))
            .with(CompositeTransform::default())
            .with(CompositeRenderableStroke(5.0))
            .build();
    }
}

#[wasm_bindgen]
pub fn run() -> Result<(), JsValue> {
    set_panic_hook();

    let app = App::build()
        .with_bundle(
            oxygengine::core::assets::bundle_installer,
            (WebFetchEngine::default(), |assets| {
                assets.register(BinaryAssetProtocol);
                assets.register(TextAssetProtocol);
                assets.register(SetAssetProtocol);
                assets.register(PngImageAssetProtocol);
            }),
        )
        .with_bundle(
            oxygengine::composite_renderer::bundle_installer,
            WebCompositeRenderer::with_state("screen", RenderState::new(Some(Color::black()))),
        )
        .with_system(DebugSystem, "debug", &[])
        .build(LoadingState, WebAppTimer::default());

    AppRunner::new(app).run::<PlatformAppRunner, _>()?;

    Ok(())
}

fn set_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}
